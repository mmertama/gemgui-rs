#![warn(missing_docs)]

//! # UI Library
//! 
//! ## User interface
//! 
//! Use Rust for engine and HTML/CSS/... for UI and store them into an own folder
//! in project.
//! 
//! The HTML file has to contain line:
//! ```html
//! <script type="text/javascript" src="gemgui.js"></script>
//! ```
//!
//! ## Resources
//! 
//! UI data files can be packed along the binary as resources. The packing
//! is done in application's `build.rs` file. 
//! See [pack](crate::respack::pack)  
//! 
//! 
//! # Main
//! 
//! There are few ways to initiate main
//! 1) Use [tokio](https://docs.rs/tokio/latest/tokio/attr.main.html) 
//! 
//! ```no_run
//! # use gemgui::GemGuiError;
//! # use gemgui::Result;
//! # use gemgui::ui::{Gui, Ui};
//! # const RESOURCES: &[(&'static str, &'static str)] = &[]; 
//! ##[tokio::main]
//! async fn main() -> Result<()> { 
//!    let fm = gemgui::filemap_from(RESOURCES);
//!    let mut ui = Gui::new(fm, "hello.html", 12345)?;
//!    //...
//!    ui.run().await
//! }
//!```
//! 2) Use [application](application)
//! 
//! ```no_run
//! # use gemgui::GemGuiError;
//! # use gemgui::Result;
//! # use gemgui::ui_ref::UiRef;
//! # const RESOURCES: &[(&'static str, &'static str)] = &[]; 
//! fn main() -> Result<()> { 
//!     let fm = gemgui::filemap_from(RESOURCES);
//!     gemgui::application(fm, "hello.html", 12345,
//!     |ui| async {my_app(ui).await})
//! }
//! 
//! async fn my_app(ui: UiRef) {
//!    //...     
//! } 
//!```
//! 3) [window application](window_application)
//! 
//! Window application uses Python webview, it to can be installed using Pip,
//! see [PyPi](https://pypi.org/project/pywebview/0.5/)
//! 
//! ```no_run
//! # use gemgui::GemGuiError;
//! # use gemgui::Result;
//! # use gemgui::ui_ref::UiRef;
//! # use gemgui::window::Menu;
//! # const RESOURCES: &[(&'static str, &'static str)] = &[]; 
//! fn main() -> Result<()> { 
//!     let fm = gemgui::filemap_from(RESOURCES);
//!     let file_menu = Menu::new().
//!     add_item("Exit", "do_exit");
//! 
//!     let about_menu = Menu::new().
//!     add_item("About", "do_about");  
//! 
//!     let menu = Menu::new().
//!     add_sub_menu("File", file_menu).
//!     add_sub_menu("About", about_menu);
//!     gemgui::window_application(fm, "hello.html", 12345,
//!     |ui| async {my_app(ui).await}, "My App", 500, 500, &[], 0, menu)
//! }
//! 
//! async fn my_app(ui: UiRef) {
//!    //...     
//! } 
//!```
//! ##  Callbacks
//! 
//! For each callback (where applicable) there is both sync and async variants.
//! Sync callbacks are executed in the UI thread and async callbacks are spawned
//! in a tokio task. Sync callbacks are FnMut and async callbacks are 
//! FnOnce (despite the name, they can be called multiple times within certain
//! limits not applied here). 
//! 
//! shortly:
//! 
//! Async functions are more permissive and they can be used to call async
//! functions, but synchronous functions are lighter.
//! 
//! ## Queries
//! 
//! A Query function is any function that requests information from the UI.
//! Queries are async functions that return a Result. Since they are async
//! functions, they has to be called in async contexts, see [Callbacks](#Callbacks).
//! Furthermore a query can be done after (or in) on_start callback as the UI
//! has to be ready to receive query requests. Premature query will lead to a panic
//! error.
//! 
//! ## Examples
//!  
//! See [Repository examples](https://github.com/mmertama/gemgui-rs/tree/main/examples)
//!      

/// Ui
pub mod ui;
/// Element
pub mod element;
/// Event
pub mod event;
/// Ui reference
pub mod ui_ref;
/// Graphics
pub mod graphics;
/// Dialogs and furnitures
pub mod window;

/// Resource pack for build.rs
pub mod respack;

mod msgsender;
mod ui_data;



use std::error::Error;
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;

use futures::Future;
use ui::Gui;
use ui_ref::UiRef;
use window::Menu;

use crate::ui::Ui;

/// Resource file map
pub type Filemap = HashMap<String, Vec<u8>>;

type JSType = serde_json::Value;
type JSMap = serde_json::Map<String, JSType>;

/// Result alias
pub type Result<T> = core::result::Result<T, GemGuiError>;


/// Rectangle
#[derive(Copy, Clone, Debug)]
pub struct Rect<T>
where T: FromStr + Copy { // so it can be parsed
    /// x coordinate
    x: T,
    /// y coordinate
    y: T,
    /// width
    width: T,
    /// height
    height: T,
}

impl<T> Rect<T>
where T: FromStr + Copy {
    /// New rect
    pub fn new(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect{x, y, width, height}
    }

    /// x
    pub fn x(&self) -> T {
        self.x
    }

    /// y
    pub fn y(&self) -> T {
        self.y
    }

    /// x
    pub fn width(&self) -> T {
        self.width
    }

    /// x
    pub fn height(&self) -> T {
        self.height
    }
}

/// Helper to move values to/from callback
/// 
/// #Example
/// ```
/// # use gemgui::ui_ref::UiRef;
/// # use gemgui::ui::Ui;
/// # use gemgui::Result; 
/// # use gemgui::ui::Gui;
/// # async fn example_1() -> Result<()> {
/// # let path = std::path::Path::new("tests/assets");
/// # let fm = gemgui::filemap_from_dir(&path)?;
/// # let mut ui = gemgui::ui::Gui::new(fm, "example.html", gemgui::next_free_port(30000u16))?;
///   let value = gemgui::Value::new(String::from(""));
///   let assign = {
///     let value = value.clone();
///        |ui_ref: UiRef| async move { 
///        let txt = ui_ref.element("content").html().await.unwrap();
///         value.assign(txt);
///         }
///  };
///  ui.on_start_async(assign);
///  /// later on...   
///  assert_eq!("Lorem ipsum, vino veritas", value.cloned().as_str().trim());
///  # Ok(())}
/// ```
#[derive(Clone)]
pub struct Value<T> 
where  T: Clone  {
    value: Arc<Mutex<T>>,
}

impl<T> Value<T>
where T: Clone {
    /// set a new value
    /// 
    /// # Arguments
    /// 
    /// `value` - value
    pub fn new(value : T) -> Value<T>
    where T: Clone {
        Value{value: Arc::new(Mutex::new(value))}
    }

    /// assign a new value
    /// 
    /// # Arguments
    /// 
    /// `value` - value
    pub fn assign(&self, new: T) {
        let mut l = self.value.lock().unwrap();
        *l = new;
    }

    /// Clone of the stored value
    /// 
    /// # Return
    /// 
    /// `value` - value
    pub fn cloned(&self) -> T {
        let l = self.value.lock().unwrap();
        l.clone()
    }
}




/*
#[macro_export]
/// Read filemap from resources
macro_rules! filemap_from_resources {
    () => {
        {
        let mut filemap = gemgui::Filemap::new();
        for resource in RESOURCES {
            let res = base64::decode(resource.1).unwrap();
            let key =  resource.0.to_string();
            if filemap.contains_key(&key) {
                eprintln!("Warning: {:#?} already in resources", &key);
            }
            filemap.insert(key, res);
        }
        filemap
        }
        };
}
*/

/// Read  a filemap from applied resources
/// 
/// # Arguments
/// 
/// `resource_map`- Defines static resources
/// 
/// # Return
/// Filemap
pub fn filemap_from(resource_map: &[(&'static str, &'static str)]) -> Filemap {
    let mut filemap = Filemap::new();
    for resource in resource_map {
        let res = base64::decode(resource.1).unwrap();
        let key =  resource.0.to_string();
        if filemap.contains_key(&key) {
            eprintln!("Warning: {:#?} already in resources", &key);
        }
        filemap.insert(key, res);
    }
    filemap
    }

 
#[derive(serde::Serialize, Debug, Default)]
struct JSMessageTx<'a> {
    element: &'a str,
    #[serde(rename = "type")]
    _type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msgid: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<&'a Vec<JSType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    throttle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    html_element: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_params: Option<&'a Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attribute: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logging: Option<bool>,  
    #[serde(skip_serializing_if = "Option::is_none")]
    debug: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alert: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    open: Option<&'a JSMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    batches: Option<&'a Vec<JSType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    add: Option<bool>,  
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rect: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clip: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pos: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commands: Option<&'a Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extension_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extension_call:  Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extension_params: Option<&'a JSMap>,
   
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct JSMessageRx {
    #[serde(rename = "type")]
    _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<JSMap>,
}


pub (crate) fn value_to_string_list(value: JSType) -> Option<Vec<String>> {
    if ! value.is_array() {
        eprintln!("Values is not an array {value}");
        return None;
       }
    let array = value.as_array().unwrap();
    let mut result = Vec::new();
    for v in array.iter()  {
        if v.is_string() {
           result.push(String::from(v.as_str().unwrap())); 
        } else {
            eprintln!("item not understood ... todo {v}");
            return None;
        }
    }   
    Some(result)
}

pub (crate) fn value_to_string_map(value: JSType) -> Option<HashMap<String, String>> {
   if ! value.is_object() {
    eprintln!("Values is not an object {value}");
    return None;
   }
   let obj = value.as_object().unwrap();
   let mut result = HashMap::new();
   for v in obj.iter() {
        if v.1.is_string() {
            result.insert( v.0.clone(), String::from(v.1.as_str().unwrap()));
        } else if v.1.is_boolean() || v.1.is_number() { // this  actually sucks a bit - we make strings TODO - create pub types for values
            result.insert( v.0.clone(), format!("{}", v.1));
        } else {
            eprintln!("item not understood ... todo {}", v.1);
            return None;
        }
   }
   Some(result)
}

pub (crate) fn value_to_string(value: JSType) -> Option<String> {
    if ! value.is_string() {
        eprintln!("Values is not an string {value}");
        return None;
       }
    Some(String::from(value.as_str().unwrap()))   
}

/// Error type
#[derive(Debug, Clone)]
pub enum GemGuiError {
    /// Error string
    Err(String),
} 

impl Error for GemGuiError {}

impl fmt::Display for GemGuiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Err(e) => write!(f, "GemGui error: {e}"),
        }
    }
}


impl From<std::io::Error> for GemGuiError {
    fn from(err: std::io::Error) -> GemGuiError {
        Self::Err(err.to_string())
    }
}

impl GemGuiError {

    fn new(err: &str) -> GemGuiError {
        GemGuiError::Err(err.to_string())
    }

    fn error<T, Str>(err: Str) -> Result<T>
    where Str: Into<String> {
        let err = err.into();
        Err(GemGuiError::new(&err))
    }
}

  
/// Create a filemap from a directory content
/// 
/// # Arguments
/// 
/// `path` - directory name
/// 
/// # Return
/// 
/// Filemap
/// 
pub fn filemap_from_dir<DirName>(path: DirName) -> std::io::Result<Filemap>
 where DirName: AsRef<Path>{
    let dirname = path.as_ref().to_str().unwrap();
    let dir = std::fs::read_dir(dirname).unwrap_or_else(|e| panic!("Cannot read {}/{}: {}",
    std::env::current_dir().unwrap().to_str().unwrap(), dirname, e));
    let mut filemap = Filemap::new();
    for entry in dir {
        let file = entry?;
        if file.file_type()?.is_file() {
            let contents = std::fs::read(file.path())?;
            let name = file.file_name().into_string().unwrap(); 
            filemap.insert(name, contents);  
        }
    }
    Ok(filemap)
}

/// Current version
/// 
/// # Return
/// 
/// Version tuple 
pub fn version() -> (u32, u32, u32) {
    const MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
    const MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
    const PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
    (MAJOR.parse::<u32>().unwrap(),  MINOR.parse::<u32>().unwrap(), PATCH.parse::<u32>().unwrap())
}

/// Find a next free port
/// 
/// Todo: Maybe not the best way to do this as free and use wont be
/// atomic and hence use of port may fail
/// 
/// # Arguments
/// 
/// `port`- a port where search is started
/// 
/// # Return
/// 
/// a free port
pub fn next_free_port(port: u16) -> u16 {
    let mut next_port = port;
    while ! port_scanner::local_port_available(next_port) {
        next_port  += 1;
    }
    next_port   
}

/// Wait a port to be free
/// 
/// Todo: Maybe not the best way to do this as free and use wont be
/// atomic and hence use of port may fail
/// 
/// # Arguments
/// 
/// `port`- a port where search is started
/// `timeout`
/// 
/// # Return
/// 
/// is free
pub fn wait_free_port(port: u16, max_wait: Duration) -> bool {
    let mut elapsed =Duration::ZERO;
    while ! port_scanner::local_port_available(port) {
        let sleep = std::time::Duration::from_secs(1);
        std::thread::sleep(sleep);
        elapsed += sleep;
        if elapsed >= max_wait {
            return false
        }
    }
    true
}


    fn create_application<CB, Fut, Create>(filemap: Filemap, index_html: &str, port: u16, application_cb: CB, mut on_create: Create)  -> Result<()> 
    where CB: FnMut(UiRef)-> Fut + Send + Clone + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        Create: FnMut(&mut Gui) {
        debug_assert!(filemap.contains_key(index_html));    
        let result: Arc<Mutex<Option<GemGuiError>>> = Arc::new(Mutex::new(None));
            tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let ui  = Gui::new(filemap, index_html, port);
                if ui.is_err() {
                    let mut r = result.lock().unwrap();
                    *r = ui.err();
                    return;
                }
                let mut ui = ui.unwrap();
                
                on_create(&mut ui);
                
                ui.on_start_async(application_cb);
                let rr = ui.run().await;
                if rr.is_err() {
                    let mut r = result.lock().unwrap();
                    *r = rr.err();
                }    
            });
            let r = result.lock().unwrap();
            match r.clone() {
                None => Ok(()),
                Some(e) => Err(e), 
            }
        }


/// Convenience to create an default UI application
/// 
/// # Arguments
/// 
/// `filemap`- resources
/// 
/// `index_html`- UI document
/// 
/// `port` - port used to connect in this application session
/// 
/// `application_cb`: Callback called when UI is ready
/// 
/// # Return
/// 
/// Application exit result
/// 
pub fn application<CB, Fut>(filemap: Filemap, index_html: &str, port: u16, application_cb: CB)  -> Result<()> 
where CB: FnMut(UiRef)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static {
        create_application(filemap, index_html, port, application_cb, |_|{})
    }


/// Convenience to create a windowed UI application 
/// 
/// # Arguments
/// 
/// `filemap`- resources
/// 
/// `index_html`- UI document
/// 
/// `port` - port used to connect in this application session
/// 
/// `application_cb`: Callback called when UI is ready
/// 
/// `title`- window title
/// 
/// `width` - window width
///
/// `height` - window height
/// 
/// `parameters` - list of key - value pairs to be passed to UI backend. 
///  
/// `flags` - bit flags be passed to UI backend
///  
/// * NORESIZE
/// * FULLSCREEN
/// * HIDDEN
/// * FRAMELESS
/// * MINIMIZED
/// * ONTOP
/// * CONFIRMCLOSE
/// * TEXTSELECT
/// * EASYDRAG
/// * TRANSPARENT
/// 
/// # Return
/// 
/// Application exit result
/// 
/// # Example
/// 
/// ```no_run
/// # use gemgui::GemGuiError;
/// # use gemgui::Result;
/// # use gemgui::ui::{Gui, Ui};
/// # use gemgui::ui_ref::UiRef;
/// # const RESOURCES: &[(&'static str, &'static str)] = &[]; 
/// fn main() -> Result<()> {
///    let fm = gemgui::filemap_from(RESOURCES);
///    gemgui::window_application(fm,
///       "index.html",
///        gemgui::next_free_port(30000u16),
///        |ui| async {my_main(ui).await},
///        "My application",
///        900,
///        500,
///        &[("debug", "True")],
///        0,
///        None)
///    }
/// 
/// async fn my_main(ui: UiRef) {
///     // ...
/// }
/// ```
/// 
#[allow(clippy::too_many_arguments)]
pub fn window_application<CB, Fut, OptionalMenu>(
        filemap: Filemap,
        index_html: &str,
        port: u16,
        application_cb: CB,
        title: &str,
        width:u32,
        height: u32,
        parameters: &[(&str, &str)],
        flags: u32,
        menu: OptionalMenu)  -> Result<()> 
    where CB: FnMut(UiRef)-> Fut + Send + Clone + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        OptionalMenu: Into<Option<Menu>> {
            let menu = menu.into();
            match menu {
                Some(menu) => {
                    create_application(filemap, index_html, port, application_cb,  move |ui| {
                        let menu = menu.clone();
                        ui.set_python_gui(title, width, height, parameters, flags, menu);})
                },
                None => create_application(filemap, index_html, port, application_cb,  move |ui| {
                    ui.set_python_gui(title, width, height, parameters, flags, None);})
            }       
        }


/// Default error function
/// 
/// Default function for [ui.on_error]
/// Shows an error message and exits.
/// 
pub fn default_error(ui: UiRef, err_msg: String) {
    eprint!("Exit on error: ");
    let json = serde_json::from_str::<HashMap<String, String>>(&err_msg);
    match json {
        Ok(json) => {
            eprintln!("Error: {}\nElement: {}\nTrace: {}\n", json["error"], json["element"], json["trace"]);
        },
        _ => eprintln!("{err_msg}")
    }
   ui.exit(); // todo! with error code
}
