pub(crate) mod server;
mod utils;

use crate::Menu;
use crate::Result;

use core::fmt;
use std::sync::Arc;
use std::sync::Mutex;

use std::process::Command;

use std::time::Duration;

use futures::Future;
use tokio::sync::mpsc;
use tokio::sync::watch;

/// TimerId to identify a timer. Unique on Ui lifetime.
pub type TimerId = u32;

/// Subscription callback type.
pub type SubscribeCallback = dyn FnMut(UiRef, Event) + Send + 'static;

/// Time callback type.
pub type TimerCallback = dyn FnMut(UiRef, TimerId) + Send + 'static;

//pub type AsyncTimerCallback = dyn FnMut(UiRef, TimerId) ->
// (dyn std::future::Future<Output = ()> + Send + 'static) + Send + 'static;

use crate::Filemap;
use crate::GemGuiError;
use crate::JSMessageRx;

use crate::default_error;
use crate::event::Event;
use crate::event::Properties;
use crate::element::Element;

use crate::ui_data::UiData;
use crate::ui_data::UiDataRef;
use crate::ui_data::State;
use crate::ui_ref::UiRef;


pub (crate) type ChannelReceiver<T> = tokio::sync::mpsc::Receiver<T>;
pub (crate) type ChannelSender<T> = tokio::sync::mpsc::Sender<T>;

// internal resources that are added with external
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

// batch commands
pub (crate) static BATCH_BEGIN: &str = "batch_begin";
pub (crate) static BATCH_END: &str = "batch_end";  
pub (crate) static CLOSE_REQUEST: &str = "close_request";


// ##known issues:## 
// * Sync callbacks are FnMut, but are limited with Send + 'static
// ** It would be nice to get rid of those: Send to let use Rc + RefCell etc non-Send stuff,
// 'static to ui's lifetime would even let use ref's. However async_to_fn internal functions seems 
// to make that tricky and probably async should be be wrapped in sync handlers.  (that also
// has a extra task spawn so probably a real design flaw)
// Next versions
// * Port reserve
// ** Maybe not the best way to do this as free and use wont be
// atomic and hence use of port may fail



/// Target enum
 pub enum Target {
    /// Blank Opens the linked document in a new window or tab
    Blank,
    /// Same Opens the linked document in the same frame as it was clicked (this is default). 
    Same,
    /// Parent Opens the linked document in the parent frame.
    Parent,
    /// Top Opens the linked document in the full body of the window.
    Top,
    /// Framename Opens the linked document in the named iframe
    FrameName(String),	
}

/// Py Ui Flags See [Gui::set_python_gui]
pub mod py_ui_flags {
    /// See pywebview documentation 
    pub const NORESIZE : u32 = 0x1;
     /// See pywebview documentation 
    pub const FULLSCREEN : u32 = 0x2;
    /// See pywebview documentation 
    pub const HIDDEN : u32 = 0x4;
    /// See pywebview documentation 
    pub const FRAMELESS : u32 = 0x8;
    /// See pywebview documentation 
    pub const MINIMIZED : u32 = 0x10;
    /// See pywebview documentation 
    pub const ONTOP : u32 = 0x20;
    /// See pywebview documentation 
    pub const CONFIRMCLOSE : u32 = 0x40;
    /// See pywebview documentation 
    pub const TEXTSELECT : u32 = 0x80;
    /// See pywebview documentation 
    pub const EASYDRAG : u32 = 0x100;
    /// See pywebview documentation 
    pub const TRANSPARENT : u32 = 0x200;
}


impl Target {
    pub (crate) fn value(&self) -> &str {
        match self {
            Self::Blank => "_blank",
            Self::Same => "_self",	
            Self::Parent => "_parent",	
            Self::Top => "_top",	
            Self::FrameName(value) => value,	
        }
    }
}

pub (crate) mod private {
    use crate::ui_data::UiDataRef;
    pub trait UserInterface {    
        fn ui(&self) -> &UiDataRef;
    }
}

/// UI interface
pub trait Ui : private::UserInterface {

    /// Root element
    /// HTML really does not have a root element, but this helps to refer &lt;body&gt; level
    fn root(&self) -> Element {
        UiData::root(self.ui())
    }

    /// Execute Javascript on UI environment
    /// 
    /// # Arguments
    /// 
    /// `eval`- Javascript code executed in UI document context. 
    fn eval(&self, eval: &str) {
        UiData::eval(self.ui(), eval)
    }

    /// Start a batch
    /// Batches let collect multiple commands as singe request. That may speed up their processing and 
    /// avoid UI flickering on complex UI modification sequences.
    fn batch_begin(&self) {
        UiData::batch_begin(self.ui())
    }

    /// End a batch
    /// The collected batch is immediately sent to UI.
    fn batch_end(&self) {
        UiData::batch_end(self.ui())
    }

    /// Set logging
    /// 
    /// # Arguments
    /// 
    /// `logging` - true starts logging, false stops UI logging
    fn set_logging(&self, logging: bool) {
        UiData::set_logging(self.ui(), logging)
    }
 
    /// Send a message to UI that is bounced back as a log
    /// 
    /// # Arguments
    /// 
    /// `msg` - message
    fn debug(&self, msg: &str) {
        UiData::debug(self.ui(), msg)
    }
    
         
    /// Show a alert dialog with a message
    /// 
    /// # Arguments
    /// 
    /// `msg` - message
    fn alert(&self, msg: &str) {
        UiData::alert(self.ui(), msg)
    }
    
    /// Open an page on UI
    /// 
    /// Note If target page wont contain gemgui.rs the page cannot be accessed.
    /// 
    /// # Arguments
    /// 
    /// `url` - message
    /// 
    /// `target` - target where url is opened
    fn open(&self, url: &str, target: Target) {
        UiData::open(self.ui(), url, target)
    }

    /// Get an application resource
    /// 
    /// # Arguments
    /// 
    /// ´resource_name´ - name of the resource
    fn resource(&self, resource_name: &str) -> Option<Box<[u8]>> {
        UiData::resource(self.ui(), resource_name)
    }

    /// Add a file content as an application resources
    ///
    /// # Arguments
    /// 
    /// `path` - path to file
    /// 
    /// # Return
    /// 
    /// On success name of the resource 
    fn add_resource<PathStr>(&self, path: PathStr) -> Result<String>
    where PathStr: AsRef<std::path::Path> {
        UiData::add_file(self.ui(), path)
    }

    /// Exit event loop
    fn exit(&self) {
        UiData::exit(self.ui());
    }

    /// Instantiate an element
    /// It is expected that element is defined in HTML or added by `add_element`, see [UiRef::add_element_async] or [UiRef::add_element_async] how to create a non-exiting
    /// element. Please note that this function always success event there is no such element as this creates a light weight
    /// Rust side struct. You have to call some query function to get error or call some function to get an error callback. 
    /// 
    /// # Arguments
    /// 
    /// `id` - refer to HTML id
    /// 
    fn element(&self, id: &str) -> Element {
        UiData::element(self.ui(), id)
    }

    /// Cancel timer
    /// 
    /// # Arguments
    /// 
    /// `id` - Timer id.
    fn cancel_timer(&self, id: TimerId) -> Result<()> {
        UiData::cancel_timer(self.ui(), id)
    }

    /// One shot timer
    /// 
    /// # Arguments
    /// 
    /// `Duration` - timer timeouts
    /// 
    /// `callback` - Callback function called on timeout
    ///     
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// 
    /// `TimerId` - id of request timer
    ///     
    /// # Return
    /// 
    /// TimerId
    fn after<CB>(&self, after: Duration, callback: CB) -> TimerId
    where CB: FnMut(UiRef, TimerId) + Send + 'static {
        UiData::after(self.ui(), after, callback)
    }


    /// One shot timer
    /// 
    /// See [after](Self::after)
    fn after_async<CB, Fut>(&self, after: Duration, async_func: CB) -> TimerId
    where CB: FnOnce(UiRef, TimerId)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()>  + Send + 'static {
        UiData::after_async(self.ui(), after, async_func)
    }
   
    /// Periodic timer
    /// 
    /// See [periodic](Self::periodic)
    fn periodic<CB>(&self, period: Duration, callback: CB) -> TimerId
    where CB: FnMut(UiRef, TimerId) + Send + 'static {
        UiData::periodic(self.ui(), period, callback)   
    }

    /// Periodic timer
    /// 
    /// # Arguments
    /// 
    /// `Duration` - timer timeouts
    /// 
    /// `callback` - Callback function called on timeout
    /// 
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// 
    /// `TimerId` - id of request timer
    ///  
    /// # Return
    /// 
    /// TimerId
    fn periodic_async<CB, Fut>(&self, period: Duration, async_func: CB) -> TimerId
    where CB: FnOnce(UiRef, TimerId)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()>  + Send + 'static {
        UiData::periodic_async(self.ui(), period, async_func)
    }

}


/// Ui instance

pub struct Gui  {
    ui: UiDataRef,
    index_html : String,
    subscription_receiver: ChannelReceiver<String>,
    timer_receiver:  ChannelReceiver<TimerId>,
    start_cmd: Option<(String, Vec<String>)>,
    server: server::WSServer,
    on_start_cb: Option<Box<dyn FnMut(UiRef)>>,
    on_start_notifee: watch::Sender<State>,
    on_reload_cb: Option<Box<dyn FnMut(UiRef)>>,
    on_error_cb: Option<Box<dyn FnMut(UiRef, String)>>,
}

impl fmt::Debug for Gui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let ui = self.ui.lock().unwrap();
        let empty: (String, Vec<String>) = ("".to_string(), vec!());
        let (cmd, params) = if self.start_cmd.is_some() {self.start_cmd.as_ref().unwrap()} else {&empty};
        f.debug_struct("Gui")
         .field("ui", &ui)
         .field("server", &self.server)
         .field("index_html", &self.index_html)
         .field("start_cmd", &cmd)
         .field("start_params", &params)
         .finish()
    }
}


impl private::UserInterface for Gui {
    fn ui(&self) -> &UiDataRef {
        &self.ui
    }
}


impl Ui for Gui { }


impl Gui {
    /// Create a UI

    pub fn new(user_map : Filemap, index_html: &str, port: u16) -> Result<Self> {
        if ! port_scanner::local_port_available(port) {
            return GemGuiError::error(format!("Port {port} is not available"));
        }
        let mut filemap = user_map;
        for resource in RESOURCES {
            let res = base64::decode(resource.1).unwrap();
            let key =  resource.0.to_string();
            if filemap.contains_key(&key) {
                eprintln!("Warning: {:#?} already in resources", &key);
            }
            filemap.insert(key, res);
        }
        
        if ! filemap.contains_key(index_html) {
            return GemGuiError::error(format!("Error {index_html}, not found"));
        }

        let filemap = Arc::new(Mutex::new(filemap));
  
        let (subscription_sender, subscription_receiver) = mpsc::channel(32);
        let (timer_sender, timer_receiver) = mpsc::channel(32);
        let server = server::new(filemap.clone(), port, subscription_sender.clone());

        let(start_notifee, start_notify) = watch::channel(State::Init);

        let ui = UiData::new(filemap,
             server.sender(),
            timer_sender,
            start_notify,
            subscription_sender,
    
    );

       let start_cmd = None;

        Ok(Gui{
            ui: Arc::new(Mutex::new(ui)),
            index_html: index_html.to_string(),
            subscription_receiver,
            timer_receiver,
            start_cmd,
            server,
            on_start_cb: None,
            on_start_notifee: start_notifee,
            on_reload_cb: None,
            on_error_cb: Some(Box::new(|ui, err_msg| {default_error(ui, err_msg)})),
        })
    }

    /// URL of Ui
    pub fn address(&self) -> String {
        // currently only localhost - but remote UI in future easily possible!
        format!("http://127.0.0.1:{}/{}", self.server.port(), self.index_html)
    }

    async fn run_process(cmd: (String, Vec<String>)) -> Result<bool> {
   
        let output = Command::new(&cmd.0)
            .args(&cmd.1)
            .spawn();

        // we wait a moment so try_wait knows if spawn has really succeed
        tokio::time::sleep(Duration::from_millis(1500)).await;

        match output {
            Ok(mut child) => {
                match child.try_wait() {
                    Ok(status) => match status {
                        None => Ok(true),
                        Some(err) => {
                            if err.code().unwrap_or(0) != 0 {
                                eprintln!("Spawned process {} not running {err}", cmd.0);
                                Ok(false)
                            } else {
                                Ok(true)    // OSX uses 'open' app to spawn browser, hence it may have ended, we just rely on error code
                            }
                        },
                    },
                    Err(err) => GemGuiError::error(format!("Spawn process failed: {err}")),
                }
            }, // here we get handle to spawned UI - not used now as exit is done nicely
            Err(e) => {
                if cmd.1.is_empty() {
                    GemGuiError::error(format!("Error while spawning call:'{}' error:{} - URL is missing!", cmd.0, e))
                } else {
                    GemGuiError::error(format!("Error while spawning call:'{}' params:'{:#?}' error:{}", cmd.0, cmd.1, e))
                }
            }
        } 
    }


    /// Overrides UI application command line. The default is a OS specific call to system default browser.
    /// 
    /// 
    /// # Arguments
    /// 
    /// `ui_cmd` - command line call
    /// 
    /// `ui_cmp_params`- list of parameters
    pub fn set_gui_command_line<Str: Into<String> + Clone>(&mut self, cmd: &str, params: &[Str]) {
        let params = params.iter().map(move |v| v.clone().into()).collect();
        self.start_cmd = if !cmd.is_empty() {Some((cmd.to_string(), params))} else {None};
    }


    /// Set python (pywebview) UI
    /// 
    /// # Arguments
    /// `title` - window title
    /// `width` - window width
    /// `height` - window height
    /// `python_parameters` - parameters passed to pywebview e.g "{"debug", true}"
    /// `flags` - See [py_ui_flags](py_ui_flags)
    pub fn set_python_gui<OptionalMenu>(&mut self,
        title: &str,
        width:u32,
        height: u32,
        python_parameters: &[(&str, &str)],
        flags: u32,
        menu: OptionalMenu) -> bool 
        where OptionalMenu: Into<Option<Menu>>{
            let py = utils::python3();
            if py.is_none() {
                return false;
            }

            let mut py_pa = Vec::new();

            //if let Some(python_parameters) = python_parameters {
                for (k, v) in python_parameters.iter() {
                    py_pa.push(format!("{k}={v}"));
                }
            //}

            let py_src = RESOURCES.iter().find(|r| r.0 == "pyclient.py").unwrap().1;
            let py_src = base64::decode(py_src).unwrap();
            let py_src = String::from_utf8_lossy(&py_src);

            let mut params = vec!(
                "-c".to_string(),
                format!("{py_src}"),
                format!("--gempyre-url={}", self.address()),
                format!("--gempyre-width={width}"),
                format!("--gempyre-height={height}"),
                format!("--gempyre-title={title}"),
                format!("--gempyre-extra={}", py_pa.join(";")), 
                format!("--gempyre-flags={flags}"));

            let menu = menu.into();
            if menu.is_some() {
                params.push(format!("--gempyre-menu={}", menu.unwrap().to_string()));
            }    

            let path = py.unwrap().to_str().unwrap().to_string();

            self.set_gui_command_line(&path, &params);
            true
        }

    fn default_start_cmd(&self) -> Result<(String, Vec<String>)> {
        let start_cmd = utils::html_file_launch_cmd();
        if start_cmd.is_none() {
            return GemGuiError::error("Cannot find a default application");
        }
        let mut start_cmd = start_cmd.unwrap();
        start_cmd.1.push(self.address());
        Ok(start_cmd)
    }


    /// Start event loop
    pub async fn run(&mut self) -> Result<()> {
        static DEFAULT_ERROR: &str = "Cannot fallback to default";

        let default_cmd = self.default_start_cmd();

        let cmd = match &self.start_cmd {
            Some(v) => v.clone(),
            None => default_cmd.clone().expect(DEFAULT_ERROR),
        };

        let on_start = move |_| async move {
            let success = match Self::run_process(cmd.clone()).await {
                Ok(success) => {
                    if ! success {
                        let default_cmd = default_cmd.expect(DEFAULT_ERROR);
                        if cmd.0 != default_cmd.0.clone() {
                            let default_ok = Self::run_process(default_cmd).await.unwrap_or_else(|e| panic!("{e}"));
                            eprintln!("Requested UI failed, falling back to default: {default_ok}");
                        }
                    }
                true
                },
                Err(err) => panic!("{err}"),
            };
            success
        };

        //let on_start = move |_| {Self::run_process(cmd)};


        let server_wait = self.start_server(on_start).await;
        if server_wait.is_none() {
            return GemGuiError::error("Starting server failed");
        }
        let server_wait = server_wait.unwrap();
        tokio::pin!(server_wait); // see https://tokio.rs/tokio/tutorial/select

        // wait here
        loop {
            tokio::select! {
                // wait server close
                _ =  &mut server_wait => {  
                    break;
                },
                 //  wait WS
                Some(msg) = self.subscription_receiver.recv() => {
                match serde_json::from_str::<JSMessageRx>(&msg) {
                        Ok(m) => {
                            match m._type.as_str()  {
                                "keepalive" => {
                                    // println!("keep alive");
                                },
                                "uiready" => UiData::entered(&self.ui),
                                "start_request" => self.start_handler(),
                                "close_request"  => {  // whaaat CLOSE_REQUEST cannot be used!
                                    self.exit(); // send exit to all windows - then go
                                    break; },  
                                "event" => self.event_handler(m),
                                "query" => self.query_handler(&msg),
                                "error" => self.error_handler(&msg),
                                "extension_response" => self.extension_response_handler(&msg),
                                "extensionready" => println!("Extension ready"),
                                _ => panic!("Handler not implemented for {}", m._type)
                            }
                        }
                        Err(e) => {
                            eprintln!("Invalid response {e}");
                        }
                    }
                },
                // wait timer 
                Some(timer_msg) = self.timer_receiver.recv() => {
                    self.timer_handler(timer_msg)
                },
            
            }
        }
        Ok(())
    }

    fn error_handler(&mut self, msg: &str) {
          match &mut self.on_error_cb {
            Some(f) => f(UiRef::new(self.ui.clone()), msg.to_string()),
            None => (),
        }
        eprintln!("Ui Error {msg:#?}")
    }

    fn timer_handler(&self, timer_id: u32) {
        let handler = self.get_timer_callback(&timer_id);
        if handler.is_none() {
            eprintln!("Handler not found for {timer_id}");
            return;
        }
        let rc = handler.unwrap();
        let mut fun = rc.lock().unwrap();
        fun(UiRef::new(self.ui.clone()), timer_id); 
    }

    fn event_handler(&self, msg: JSMessageRx) {
        let event_name = &msg.event.unwrap();
        let element = msg.element.unwrap();
        let handler = self.get_subscribe_callback(&element, event_name);
        if handler.is_none() {
            eprintln!("Handler not found at {} for {}", &element, event_name);
            return;
        } 
        let mut prop = Properties::new();
        for (k, v) in msg.properties.unwrap().iter() {
            let key = k.clone();
            if v.is_string() {
                prop.insert(key, v.as_str().unwrap().to_string());
            } else {
                prop.insert(key, v.to_string());
            }        
        }
        let rc = handler.unwrap();
        let mut fun = rc.lock().unwrap();
        fun(UiRef::new(self.ui.clone()), Event::new(self.ui.clone(), element, prop));
    }

    fn query_handler(&mut self, raw: &str) {
        let mut js: serde_json::Value = serde_json::from_str(raw).unwrap();
        let query_value = String::from(js["query_value"].as_str().unwrap()); // otherwise we cannot take later as mutable
        let query_id = js["query_id"].as_str().unwrap();
        let tx = UiData::get_query_sender(&mut self.ui, query_id);
        match tx {
            Some(r) => {
                let value = js[query_value].take();
                r.send(value).unwrap_or_else(|e| {panic!("Cannot send query: {e}")});
            },
            None =>  {
                eprintln!("No query listener for {query_id}");
            }
        };
    }

    fn extension_response_handler(&mut self, raw: &str) {
        let mut js: serde_json::Value = serde_json::from_str(raw).unwrap();
        let extension_call = String::from(js["extension_call"].as_str().unwrap()); // otherwise we cannot take later as mutable
        let extension_id = js["extension_id"].as_str().unwrap();
        let tx = UiData::get_query_sender(&mut self.ui, extension_id);
        match tx {
            Some(r) => {
                let value = js[extension_call].take();
                r.send(value).unwrap_or_else(|e| {panic!("Cannot send extension: {e}")});
            },
            None =>  {
                eprintln!("No extension listener for {extension_id}");
            }
        };
       
    }


    fn start_handler(&mut self) {
        if ! UiData::is_started(&self.ui) {
            UiData::set_started(&self.ui);
            match &mut self.on_start_cb {
                Some(cb) => cb(UiRef::new(self.ui.clone())),
                None => (),
            };
            self.on_start_notifee.send(State::Running).unwrap_or_else(|_| panic!("Cannot set ready"));
        }
    }

    fn get_subscribe_callback(&self, id: &str, event_name: &str) -> Option<Arc<Mutex<SubscribeCallback>>> {
        let ui = self.ui.lock().unwrap();
        let map = ui.elements.get(id)?;
        let value = map.get(event_name)?;
        Some(value.clone())
    }

    fn get_timer_callback(&self, id: &TimerId) -> Option<Arc<Mutex<TimerCallback>>> {
        let ui = self.ui.lock().unwrap();
        let val = ui.timers.get(id)?;
        Some(val.0.clone())
    }

    async fn start_server<F, Fut>(&self, on_start: F) -> Option<tokio::task::JoinHandle<()>>
    where F: FnOnce(u16) -> Fut + Send + 'static,
        Fut: Future<Output = bool> + Send + 'static {
        self.server.run(on_start).await
    }

    

    /// Set callback called when UI is ready
    /// 
    /// # Arguments
    /// 
    /// `callback` - Callback function to be called when UI is ready and data can be accessed.
    /// 
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    pub fn on_start<CB>(&mut self, callback: CB)
    where CB: FnMut(UiRef) + Send + Clone + 'static {
        self.on_start_cb = Some(Box::new(callback));
    }

    /// Set callback called when UI is ready
    /// 
    /// See [on_start](Self::on_start)
    pub fn on_start_async<CB, Fut>(&mut self, callback: CB) 
    where CB: FnOnce(UiRef)-> Fut + Send + Clone + 'static,
        Fut: Future<Output = ()>  + Send + 'static {
        let cb = UiData::as_sync_monad(callback);
        self.on_start_cb = Some(Box::new(cb));
    }

    /// Set callback called when UI is reloaded
    /// 
    /// # Arguments
    /// 
    /// `callback` - Callback function to handle UI reload.
    ///    
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI 
    pub fn on_reload<CB>(&mut self, callback: CB)
    where CB: FnMut(UiRef) + Send + 'static {
        self.on_reload_cb = Some(Box::new(callback));
    }

    /// Set callback called when UI is reloaded
    /// 
    /// See [on_reload](Self::on_reload)
    pub fn on_reload_async<CB, Fut>(&mut self, callback: CB) 
    where CB: FnOnce(UiRef)-> Fut + Send + Clone + 'static,
        Fut: Future<Output = ()>  + Send + 'static {
        let cb = UiData::as_sync_monad(callback);
        self.on_reload_cb = Some(Box::new(cb));
    }

    /// Set callback called on UI error
    /// 
    /// Default [default_error] print error and do exit.
    /// 
    /// # Arguments
    /// 
    /// `callback` - Callback function to handle UI error.
    ///   
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// 
    /// `String` - error string
    /// 
    pub fn on_error<CB>(&mut self, callback: CB)
    where CB: FnMut(UiRef, String) + Send + 'static {
        self.on_error_cb = Some(Box::new(callback));
    }

    /// Set callback called on UI error
    /// 
    /// See [on_error](Self::on_error)
    pub fn on_error_async<CB, Fut>(&mut self, callback: CB) 
    where CB: FnOnce(UiRef, String)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()>  + Send + 'static {
        self.on_error(UiData::as_sync_fn(callback))
    }

    
    


}

