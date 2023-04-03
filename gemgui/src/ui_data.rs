
use crate::Filemap;
use crate::GemGuiError;
use crate::JSMap;
use crate::JSMessageRx;
use crate::JSType;
use crate::element::Element;
use crate::element::Elements;
use crate::event::Properties;
use crate::ui::BATCH_BEGIN;
use crate::ui::BATCH_END;
use crate::ui::SubscribeCallback;
use crate::event::Event;
use crate::msgsender::MsgSender;
use crate::ui::Target;
use crate::ui::TimerCallback;
use crate::ui::TimerId;
use crate::ui::ChannelSender;
use crate::JSMessageTx;
use crate::ui::server::ENTERED;
use crate::ui_ref::UiRef;
use crate::Result;


use futures::Future;
use rand::Rng;
use serde_json::Value;
use tokio::sync::watch;
use core::fmt;
use std::collections::HashMap;
use std::path;

use std::sync::Arc;
use std::sync::Mutex;
use tokio::time;
use std::time::Duration;

use tokio::sync::oneshot;
type SubscriptionSender = tokio::sync::mpsc::Sender<String>;

pub (crate) type Timers = HashMap<TimerId,(Arc<Mutex<TimerCallback>>, oneshot::Sender<u32>)>;

// we put them in Rc, to be able to borrow it without keep Element locked
// It is too easy to do deadlocks by having Elements locked while callback
// is called. 
pub (crate) type ElementMap = HashMap<String, HashMap<String, Arc<Mutex<SubscribeCallback>>>>;

#[doc(hidden)] // to let trait be access, protected with sealed pattern
pub type UiDataRef = Arc<Mutex<UiData>>; 

pub (crate) static ROOT_ID: &str = "";

type QuerySender  = tokio::sync::oneshot::Sender<serde_json::Value>;
type QueryReceiver  = tokio::sync::oneshot::Receiver<serde_json::Value>;

type Queries = HashMap<String, QuerySender>;
 
//needed? #[derive(PartialEq)]
pub (crate) enum State {
    Init,
    Running,
    _Closed,
}


#[doc(hidden)] // to let trait access UiDataRef, protected with sealed pattern
pub struct UiData {
    tx: MsgSender,
    pub (crate) elements: ElementMap,
    pub (crate) timers: Timers,
    pub (crate) timer_ids: TimerId,
    pub (crate) timer_sender: ChannelSender<TimerId>,
    started: bool,
    queries: Queries,
    on_start_notify: watch::Receiver<State>,
    filemap: Arc<Mutex<Filemap>>,
    subscription_sender: SubscriptionSender,
}

impl fmt::Debug for UiData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fm = self.filemap.lock().unwrap();
        f.debug_struct("UiData")
             .field("filemap", &fm.keys())
             .field("started", &self.started)
             .field("queries", &self.queries.keys())
             .field("elements", &self.elements.keys())
             .field("timers", &self.timers.keys())
             .finish()
    }
}


impl UiData {

    pub (crate) fn new(filemap: Arc<Mutex<Filemap>>,
        tx: MsgSender,
        timer_sender: ChannelSender<TimerId>,
        on_start_notify: watch::Receiver<State>,
        subscription_sender: SubscriptionSender) -> Self {
        UiData {
            tx,
            elements: HashMap::new(),
            timers: HashMap::new(),
            timer_ids: 1000,
            timer_sender,
            queries: HashMap::new(),
            started: false,
            on_start_notify,
            filemap,
            subscription_sender,
        }
    }

    pub (crate) fn resource(ui: &UiDataRef, resource_name: &str) -> Option<Box<[u8]>> {
        let ui = ui.lock().unwrap();
        let fm = ui.filemap.lock().unwrap();
        if ! fm.contains_key(resource_name) {
            return None;
        }
        let v = fm[resource_name].clone();
        Some(v.try_into().unwrap())
    }

    pub (crate) fn add_file<PathStr>(ui: &UiDataRef, path: PathStr) -> Result<String> 
    where PathStr: AsRef<path::Path>{
        let content = std::fs::read(&path)?;
        let path = path.as_ref();
        let  basename = path.file_stem().unwrap().to_str().unwrap().to_string();
        let  ext = path.extension().unwrap().to_str().unwrap();
        let ui = ui.lock().unwrap();
        let mut fm = ui.filemap.lock().unwrap();
        let mut count = 1;
        let mut name = format!("{basename}.{ext}");
        loop  {
            if ! fm.contains_key(&name) {
                break;
            }
            name = format!("{basename}.{count}.{ext}");
            count += 1;
        }
        fm.insert(name.to_string(), content);
        Ok(name)
    }
     
    pub (crate) fn entered(ui: &UiDataRef) {
        let ui = ui.lock().unwrap();
        ui.tx.send(ENTERED.to_string());
    }

    pub (crate) fn batch_begin(ui: &UiDataRef) {
        let ui = ui.lock().unwrap();
        ui.tx.send(BATCH_BEGIN.to_string());
    }

    pub (crate) fn batch_end(ui: &UiDataRef) {
        let ui = ui.lock().unwrap();
        ui.tx.send(BATCH_END.to_string());
    }
 
    pub (crate) fn exit(ui: &UiDataRef) {
        let ui = ui.lock().unwrap();
        let msg =  JSMessageTx {
            element: ROOT_ID,
            _type: "close_request",
            ..Default::default()
        };
        let json = serde_json::to_string(&msg).unwrap();
        ui.tx.send(json); 
    }

    pub (crate) fn eval(ui: &UiDataRef, eval: &str) {
        let ui = ui.lock().unwrap();
        let msg =  JSMessageTx {
            element: ROOT_ID,
            _type: "eval",
            eval: Some(eval),
            ..Default::default()
        };
        let json = serde_json::to_string(&msg).unwrap();
        ui.tx.send(json); 
    }

    pub (crate) fn set_logging(ui: &UiDataRef, logging: bool) {
        let ui = ui.lock().unwrap();
        let msg =  JSMessageTx {
            element: ROOT_ID,
            _type: "logging",
            logging: Some(logging),
            ..Default::default()
        };
        let json = serde_json::to_string(&msg).unwrap();
        ui.tx.send(json);
    }
    
    pub (crate) fn debug(ui: &UiDataRef, msg: &str) {
        let ui = ui.lock().unwrap();
        let msg =  JSMessageTx {
            element: ROOT_ID,
            _type: "debug",
            debug: Some(msg),
            ..Default::default()
        };
        let json = serde_json::to_string(&msg).unwrap();
        ui.tx.send(json);
    }
    
    pub (crate) fn alert(ui: &UiDataRef, msg: &str) {
        let ui = ui.lock().unwrap();
        let msg =  JSMessageTx {
            element: ROOT_ID,
            _type: "alert",
            alert: Some(msg),
            ..Default::default()
        };
        let json = serde_json::to_string(&msg).unwrap();
        ui.tx.send(json);
    }
    

    pub (crate) fn open(ui: &UiDataRef, url: &str, target: Target) {
        let ui = ui.lock().unwrap();
        let mut map = JSMap::new();
         map.insert("url".to_string(), JSType::from(url));
         map.insert("view".to_string(), JSType::from(target.value()));
        let msg =  JSMessageTx {
            element: ROOT_ID,
            _type: "open",
            open: Some(&map),
            ..Default::default()
        };
        let json = serde_json::to_string(&msg).unwrap();
        ui.tx.send(json);
    }

    pub (crate) fn sender(ui: &UiDataRef) -> MsgSender {
        let ui = ui.lock().unwrap();
        ui.tx.clone()
    }

    pub (crate) fn new_query(ui: &UiDataRef) -> (String, QueryReceiver) {
        assert!(Self::is_started(ui), "Queries are not allowed until UI has started!");
        let mut ui = ui.lock().unwrap();
        let id = ui.random_query_id();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        ui.queries.insert(id.clone(), sender);
        (id, receiver)
    }


    pub (crate) fn get_query_sender(ui: &mut UiDataRef, id: &str) -> Option<QuerySender> {
        let mut ui = ui.lock().unwrap();
        ui.queries.remove(id)
    }

    fn append_timer<CB>(ui: &UiDataRef, callback: CB, sender: oneshot::Sender<u32>) -> u32
    where CB: FnMut(UiRef, TimerId) + Send + 'static  {
        let mut ui = ui.lock().unwrap();
        ui.timer_ids += 1;
        let id = ui.timer_ids;
        assert!(! ui.timers.contains_key(&id)); 
        ui.timers.insert(id, (Arc::new(Mutex::new(callback)), sender));
        id  
    }

    
    async fn call_after(mut on_start: watch::Receiver<State>, tx: ChannelSender<TimerId>, id: TimerId, after: Duration, receiver: oneshot::Receiver<TimerId>) {
        on_start.changed().await.unwrap();
        let sleep  = time::sleep(after);
        tokio::pin!(sleep);
        tokio::select! {
            // wait timer elapses 
            () = &mut sleep => {
                tx.send(id).await.unwrap();
            },
            // cancel timer
            _ = receiver => {}
        };
    }

    async fn call_periodic(mut on_start: watch::Receiver<State>, tx: ChannelSender<TimerId>, id: TimerId, after: Duration, mut receiver: oneshot::Receiver<TimerId>) {
        on_start.changed().await.unwrap();
        let mut tick  = time::interval(after);
        let mut first = false; // tokio peridioc elapses 1st immediatedly, gemgui dont
        loop  {
            let wait = tick.tick();
            tokio::pin!(wait);
            tokio::select! {
                // wait timer elapses 
                _ = &mut wait => {
                    if first {
                        tx.send(id).await.unwrap();
                    } else {
                        first = true;
                    }
                },
                // cancel timer
                _ = &mut receiver => {
                    return;
                }
            };
        }
    }

    fn timer_channel(ui: &UiDataRef) -> ChannelSender<u32> {
        let ui = ui.lock().unwrap();
        ui.timer_sender.clone()
    }

    fn start_notify(ui: &UiDataRef) ->  watch::Receiver<State> {
        let ui = ui.lock().unwrap();
        ui.on_start_notify.clone()
    }

    /// Apply a callback to a timer
    pub (crate) fn after<CB>(ui: &UiDataRef, after: Duration, callback: CB) -> TimerId
    where CB: FnMut(UiRef, TimerId) + Send + 'static {
        let tx = Self::timer_channel(ui);
        let (sender, receiver) = oneshot::channel::<u32>();
        let id = Self::append_timer(ui, callback, sender);
        let on_start = Self::start_notify(ui);
        tokio::spawn(Self::call_after(on_start, tx, id, after, receiver));
        id
    }

    pub (crate) fn after_async<CB, Fut>(ui: &UiDataRef, after: Duration, async_func: CB) -> TimerId
    where CB: FnOnce(UiRef, TimerId)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()>  + Send + 'static {
        Self::after(ui, after, Self::as_sync_fn(async_func))
    }

    pub (crate) fn periodic<CB>(ui: &UiDataRef, period: Duration, callback: CB) -> TimerId
    where CB: FnMut(UiRef, TimerId) + Send + 'static {
        let tx = Self::timer_channel(ui);
        let (sender, receiver) = oneshot::channel::<u32>();
        let id = Self::append_timer(ui, callback, sender);
        let on_start = Self::start_notify(ui);
        tokio::spawn(Self::call_periodic(on_start, tx, id, period, receiver));
        id
    }

    pub (crate) fn periodic_async<CB, Fut>(ui: &UiDataRef, period: Duration, async_func: CB) -> TimerId
    where CB: FnOnce(UiRef, TimerId)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()>  + Send + 'static {
        Self::periodic(ui, period, Self::as_sync_fn(async_func))
    }

    pub (crate) fn insert_element(ui: &UiDataRef, id: &str) {
        let mut ui = ui.lock().unwrap();
        if ! ui.elements.contains_key(id) {
            ui.elements.insert(String::from(id), HashMap::new());
        }
    }

    pub (crate) fn set_started(ui: &UiDataRef) {
        let mut ui = ui.lock().unwrap();
        debug_assert!(!ui.started);
        ui.started = true;
    }

    pub (crate) fn is_started(ui: &UiDataRef) -> bool {
        let ui = ui.lock().unwrap();
        ui.started
    }

    pub (crate) fn element<Str : Into<String>>(ui: &UiDataRef, id: Str) -> Element {
        let key = id.into();
        assert_ne!(key, ROOT_ID);
        Self::insert_element(ui, &key);
        let ui = ui.clone();
        Element::construct(key, Self::sender(&ui), ui)
    }

    pub(crate) fn add_subscription<CB, Str>(ui_ref: &UiDataRef, id: &str, name: Str, callback: CB) 
    where CB: FnMut(UiRef, Event) + Send + 'static,
      Str: Into<String>{
        let mut ui = ui_ref.lock().unwrap();
        let handler_map = ui.elements.get_mut(id).unwrap();    
        handler_map.insert(name.into(), Arc::new(Mutex::new(callback)));
    }

    pub(crate) fn remove_subscription(ui_ref: &UiDataRef, id: &str, name: &str) {
        let mut ui = ui_ref.lock().unwrap();
        let handler_map = ui.elements.get_mut(id).unwrap();    
        handler_map.remove(name);
    }

    pub (crate) fn call_subscription(ui_ref: &UiDataRef, id: &str, name: &str, properties: Properties) {
        let js_properties: JSMap  = properties.iter().map(|(k, v)| {(k.clone(), JSType::from(v.clone()))}).collect();
        let msg = JSMessageRx {
            _type: "event".to_string(),
            element: Some(id.to_string()),
            event: Some(name.to_string()),
            properties: Some(js_properties),

        };
        let ui = ui_ref.lock().unwrap();
        let s = serde_json::to_string(&msg).unwrap();
        let sender = ui.subscription_sender.clone();
        tokio::task::spawn(async move {
            sender.send(s).await.unwrap();
        });
    }

    pub (crate) fn root(ui: &UiDataRef) -> Element {
        Self::insert_element(ui, ROOT_ID);
        let ui = ui.clone();
        Element::construct(ROOT_ID.to_string(), Self::sender(&ui), ui)
    }
 
    
    pub (crate) fn send(ui: &UiDataRef, msg: JSMessageTx) {
        let json = serde_json::to_string(&msg).unwrap();
        let ui = ui.lock().unwrap();
        ui.tx.send(json); 
    }

    pub (crate) fn elements_from_values(ui: &UiDataRef, value: Value, tx: &MsgSender) -> Result<Elements> {
        match &mut crate::value_to_string_list(value) {
            Some(v)  => {
                let mut elements = Vec::new();
                for val in v.iter() {
                    elements.push(Element::construct(val.to_string(), tx.clone(), ui.clone()));        
                }
                Ok(elements)
            },
            None => GemGuiError::error("Bad value"),
        }
    }

    pub (crate) fn random(prefix: &str) -> String {
        let mut name = String::from(prefix) + "__";
        let a = b'a';
        let z = b'z';
        for _i in 0..8 {
            let ord = rand::thread_rng().gen_range(a..z) as char;
            name.push(ord);
        }
        name
    }

    fn random_query_id(&self) -> String {
        loop {
            let name = Self::random("query");
            if ! self.queries.contains_key(&name)  {
                return name;
            }
        }
    }

    pub fn  random_element_id(ui: &UiDataRef) -> String {
        loop {
            let name = Self::random("element");
            let ui = ui.lock().unwrap();
            if ! ui.elements.contains_key(&name)  {
                return name;
            }
        }
    }

    pub (crate) fn cancel_timer(ui: &UiDataRef, id: TimerId) -> Result<()> {
        let mut ui = ui.lock().unwrap();
        let val = ui.timers.remove(&id);
        match val {
            Some(val) => {
                let tx = val.1;
                tx.send(id).unwrap_or(()); // ignore error, actually we should only ignore
                                                   //  if cancel is in on_start. TODO if we know the state   
                Ok(())                                   
            },
            None => {
                GemGuiError::error(format!("Warning timer {id} not found"))
            }
        }
    }

    pub (crate) fn as_sync_fn<CB, P, Fut>(async_func: CB) -> impl FnMut(UiRef, P) + Send  + Clone
    where
    CB: FnOnce(UiRef, P)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static,
    P: Send + 'static,
    {
        let af = std::sync::Arc::new(tokio::sync::Mutex::new(async_func));
        move |ui_ref: UiRef, param: P| {
            let af = af.clone();
            tokio::spawn(async move {
                let fun = af.lock().await.clone();
                fun(ui_ref, param).await;
            });
        }
    }

    // this may be only one needed if tuples would be used as params...
    pub (crate) fn as_sync_monad<CB, P, Fut>(async_func: CB) -> impl FnMut(P) + Send + Clone
    where
    CB: FnOnce(P)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()> + Send,
    P: Send + 'static
    {
        let af = std::sync::Arc::new(tokio::sync::Mutex::new(async_func));
        move |param: P| {
            let af = af.clone();
            tokio::spawn(async move {
                let fun = af.lock().await.clone();
                fun(param).await;
            });
        }
    }

}
