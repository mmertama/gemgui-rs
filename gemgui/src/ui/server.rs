
use futures::stream::SplitSink;
use warp::Filter;
use warp::ws::Message;
use warp::ws::WebSocket;
use warp::ws::Ws;

use futures::stream::StreamExt;

use core::fmt;
use std::sync::Arc;
use std::sync::Mutex;

use futures::SinkExt;

use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;

use tokio::sync::mpsc::Sender as SubscriptionSender;

use crate::Filemap;
use crate::JSMessageTx;
use crate::JSType;
use crate::ui::BATCH_BEGIN;
use crate::ui::BATCH_END;
use crate::ui::utils::get_extension_from_filename;
use crate::ui_data::ROOT_ID;

type MessageBuffer = Arc<Mutex<Vec<Message>>>;

pub struct WSServer {
    filemap: Arc<Mutex<Filemap>>,
    port: u16,
    client_tx: BroadcastSender<Message>,
    buffer: MessageBuffer,
    subscription_sender: SubscriptionSender<String>
}


impl fmt::Debug for WSServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = self.buffer.lock().unwrap();
        f.debug_struct("Server")
        .field("port", &self.port)
        .field("message queue size", &buf.len())
        .finish()
    }
}

#[derive(Clone)]
pub (crate) struct MsgTx {
    tx: BroadcastSender<Message>,
} 

impl MsgTx {
    pub (crate) fn send(&self, msg: String) {
            if self.tx.receiver_count() > 0 {
                let tx = self.tx.clone();
                tokio::spawn(MsgTx::do_send(tx, msg));
            }
        }

        
    pub (crate) fn send_bin(&self, bin: Vec<u8>) {
        if self.tx.receiver_count() > 0 {
            let tx = self.tx.clone();
            tokio::spawn(MsgTx::do_send_bin(tx, bin));
        }
    }
}
   

// sends message from element to socket server
impl MsgTx {
    async fn do_send(tx: BroadcastSender<Message>, msg: String) {
        tx.send(Message::text(msg)).unwrap_or_else(|e|{eprintln!("Fatal {}", e); 0});
    }
    
    async fn do_send_bin(tx: BroadcastSender<Message>, msg: Vec<u8>) {
        tx.send(Message::binary(msg)).unwrap_or_else(|e|{eprintln!("Fatal {}", e); 0});
    }  
}

// receive message from element

static ENTERED: &str = "entered";

async fn wait_early_messages(msg_queue: MessageBuffer, mut rx: BroadcastReceiver<Message>) {
    let mut entered = false;
    while ! entered  {
        let msg = rx.recv().await;
        match msg {
            Ok(msg) => {
                let mut queue = msg_queue.lock().unwrap();
                if msg.is_text() && msg.to_str().unwrap() == ENTERED {
                    entered = true;
                }
                if !entered {
                    queue.push(msg);
                }
            }
            Err(e) => {
                eprintln!("Cannot read {}", e);
                break;
            }
        }
    }
}

fn write_to_buffer(msg: Message, buffer: &MessageBuffer) {
    let mut buf = buffer.lock().unwrap();
    buf.push(msg);
}

pub(super) fn new(filemap: Arc<Mutex<Filemap>>, port: u16, subscription_sender: SubscriptionSender<String>) -> WSServer {
    let (client_tx, buffer_rx) = tokio::sync::broadcast::channel(64);
    // we need a buffer where to copy message before  ws is open
    let buffer = Arc::new(Mutex::new(Vec::new()));
    // messages are listed in own loop, that let us also handle messages from user
    tokio::spawn(wait_early_messages(buffer.clone(), buffer_rx));

    WSServer {
        filemap,
        port,
        client_tx,
        buffer,
        subscription_sender,
    }
}


impl WSServer {

    pub (crate) fn sender(&self) ->  MsgTx {
        MsgTx{tx: self.client_tx.clone()}
    }

    // Mutex protected data accessor, we cannot use await in iteration (for) loop
    // as then mutex is locked and async-send cannot happen
    // Just clone data from index 
    fn take_as_msg( buffer: &MessageBuffer) -> Vec<JSType> {
        let mut buf = buffer.lock().unwrap();
        let vec = buf.iter().map(|msg| {
            let s = msg.to_str().unwrap();
            serde_json::from_str(s).unwrap()
        }).collect(); // sigh copying values back and forth due types
        buf.clear();
        vec    
    }

    async fn send_buffered(sender: &mut SplitSink<WebSocket, Message>, buffer: &MessageBuffer) { 
        let msg_buffer = Self::take_as_msg(buffer);
        let msg = JSMessageTx {
            element: ROOT_ID,
            _type: "batch",
            batches: Some(&msg_buffer),
            ..Default::default()
        };
        let json = serde_json::to_string(&msg).unwrap();
        sender.send(Message::text(json)).await.unwrap_or_else(|e| eprintln!("Cannot send {}", e));
    }

    //TODO refactor this call, messy and lot of ARCs...
    async fn handle_ws_client(websocket: WebSocket,
        client_tx: BroadcastSender<Message>,
        buffer: MessageBuffer,
        subscription_sender: SubscriptionSender<String>) {
        // receiver - this server, from websocket client
        // sender - diff clients connected to this server
        let (mut sender, mut receiver) = websocket.split();

        // flush buffer before entering a loop
        client_tx.send(Message::text(ENTERED)).unwrap();
        Self::send_buffered(&mut sender, &buffer).await;

        let mut do_buffer = false; // at start we buffer

        let mut client_rx = client_tx.subscribe();

        loop {
            tokio::select! {
                Some(ws_msg) = receiver.next() => {
                    match ws_msg {
                        Ok(msg) => {
                            if msg.is_text() {
                                let txt = String::from(msg.to_str().unwrap());
                                subscription_sender.send(txt).await.unwrap();
                            } else if msg.is_close() {
                                // tell ui.rs to leave the loop - Json constant...
                                let close = String::from("{\"type\": \"close_request\"}"); 
                                subscription_sender.send(close).await.unwrap();
                                break;   
                            } else {
                                eprintln!("Unexpected message type");
                            }
                        },
                        Err(e) => {
                            eprintln!("error reading message on websocket: {}", e);
                        }
                    };   
                },
                cl_msg = client_rx.recv() => {
                     match cl_msg {
                        Ok(msg) => {
                            if msg.is_text() && msg.to_str().unwrap() == BATCH_BEGIN  {
                                do_buffer = true;
                            } else if msg.is_text() && msg.to_str().unwrap() == BATCH_END  {
                                do_buffer = true;
                                Self::send_buffered(&mut sender, &buffer).await;
                            } else if do_buffer {
                                write_to_buffer(msg, &buffer);    
                            } else {
                                sender.send(msg).await.unwrap_or_else(|e| eprintln!("Cannot send {}", e));
                            }
                        },
                        Err(e) => {
                            eprintln!("error reading message from element: {}", e);
                        }
                    };   
                },   
            }
        }
    }

    /// Run 
    pub fn run<F>(&self, on_start: F) -> Option<tokio::task::JoinHandle<()>>
    where F: FnOnce(u16) -> bool {

        let fm = self.filemap.clone();
        
        // Sigh there is not compile time warning while writing, this
        // but this is quite fragile, bad things happens if
        // name is not in fm - should be rewritten
        // how to add keys to paths?
        let  get_routes = 
        warp::get()
        .and(warp::path::tail()
        .map(move |path: warp::path::Tail|  {
            let name = path.as_str();
            let file_map = fm.lock().unwrap();
            assert!(file_map.contains_key(name), "Request not found: {:#?}", name);

            let mime = Self::file_to_mime(name).unwrap_or("octect-stream");
            
            warp::reply::with_header(file_map[name].clone(), "content-type", mime)
            
        }));

        let buffer = self.buffer.clone();
        let client_tx = self.client_tx.clone();
        let subscription_sender = self.subscription_sender.clone();
        let ws_routes = warp::ws()
        .and(warp::path("gemgui"))
        .map( move |ws: Ws| {
            //let clients = clients.clone();
            // And then our closure will be called when it completes...
            let buffer = buffer.clone();
            let client_tx = client_tx.clone();
            let subscription_sender = subscription_sender.clone();
            ws.on_upgrade( move |websocket: WebSocket| {
                Self::handle_ws_client(websocket, client_tx, buffer, subscription_sender)
            })
        });

        let all_routes = ws_routes
        .or(get_routes);
     
        let fut_srv = tokio::spawn(
            warp::serve(all_routes)
            .run(([127, 0, 0, 1], self.port)));

        // Start browser Ui after server is spawned
        if !on_start(self.port) {
            eprintln!("Start failed, exit");
            return None; // early end
        }
        
        Some(fut_srv)

    }

    fn file_to_mime(filename: &str) -> Option<&str>{
        let ext = get_extension_from_filename(filename)?;
        let ext = ext.to_ascii_lowercase();
        let ext = ext.as_str();

        static MAP: phf::Map<&'static str, &'static str> = phf::phf_map! {
            "html" => "text/html;charset=utf-8",
            "css" => "text/css;charset=utf-8",
            "js" => "text/javascript;charset=utf-8",
            "txt" => "text/txt;charset=utf-8",
            "ico" => "image/x-icon",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml"
        };

        match MAP.get(ext) {
            Some(v) => Some(*v),
            None => None,
        }
    }

} 



