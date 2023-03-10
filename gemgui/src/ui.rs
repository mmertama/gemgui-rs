pub(crate) mod server;
mod utils;

use crate::Result;

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

// todo! 
// Documents
// Some example snippets
// Widgets example
// Coverage
// Perf test
// Lint
// Error types (see RUST API Guide)

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
    /// HTML really does not have a root element, but this helps to refer <body> level
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
    /// ??resource_name?? - name of the resource
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
    /// It is expected that element is defined in HTML or added by `add_element`, see `add_element` how to create a non-exiting
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
            return Err(GemGuiError::Err(format!("Port {} is not available", port)));
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
            return Err(GemGuiError::Err(format!("Error {}, not found", index_html)));
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

        let start_cmd = utils::html_file_launch_cmd();

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

    fn run_process(cmd: (String, Vec<String>), index_html: String, port: u16) -> bool {
        let cmd_line : String = format!("http://127.0.0.1:{}/{}",
            port,
            index_html);
       
        let output = Command::new(&cmd.0)
            .args(&cmd.1)
            .arg(&cmd_line)
            .spawn();

        match output {
            Ok(_child) => {true}, // here we get handle to spawned UI - not used now as exit is done nicely
            Err(e) => {
                if cmd.1.is_empty() {
                    eprintln!("Error while spawning call:'{}' target:'{}' error:{}", cmd.0, cmd_line, e);
                } else {
                    eprintln!("Error while spawning call:'{}' params:'{:#?}' target:'{}' error:{}", cmd.0, cmd.1, cmd_line, e);
                }
                false
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


    /// Start event loop
    pub async fn run(&mut self) -> Result<()> {
        let cmd = match &self.start_cmd {
            Some(v) => v.clone(),
            None => return GemGuiError::error("Cannot find a default application"),
        };

        let index_html = self.index_html.clone(); // clone to closure
        let on_start = move |port| {Self::run_process(cmd, index_html, port)};
        let server_wait = self.start_server(on_start);
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
                                "keepalive" => (), 
                                "uiready" => self.ready_handler(),
                                "close_request"  => break,    
                                "event" => self.event_handler(m),
                                "query" => self.query_handler(&msg),
                                "error" => self.error_handler(&msg),
                                _ => panic!("Handler not implemented for {}", m._type)
                            }
                        }
                        Err(e) => {
                            eprintln!("Invalid response {}", e);
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
        eprintln!("Ui Error {:#?}", msg)
    }

    fn timer_handler(&self, timer_id: u32) {
        let handler = self.get_timer_callback(&timer_id);
        if handler.is_none() {
            eprintln!("Handler not found for {}", timer_id);
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
                r.send(value).unwrap_or_else(|e| {panic!("Cannot send query: {}", e)});
            },
            None =>  {
                eprintln!("No query listener for {}", query_id);
            }
        };
    }

    fn ready_handler(&mut self) {
        match &mut self.on_start_cb {
            Some(cb) => cb(UiRef::new(self.ui.clone())),
            None => (),
        };
        UiData::set_started(&self.ui);
        self.on_start_notifee.send(State::Running).unwrap_or_else(|_| panic!("Cannot set ready"));
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

    fn start_server<F>(&self, on_start: F) -> Option<tokio::task::JoinHandle<()>>
    where F: FnOnce(u16) -> bool {
        self.server.run(on_start)
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

