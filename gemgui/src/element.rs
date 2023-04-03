

use futures::Future;

use crate::GemGuiError;
use crate::Result;
use crate::Rect;
use crate::event::Event;
use crate::event::MOUSE_CLICK;
use crate::event::MOUSE_DBLCLICK;
use crate::event::MOUSE_DOWN;
use crate::event::MOUSE_MOVE;
use crate::event::MOUSE_UP;
use crate::event::Properties;
use crate::msgsender::MsgSender;
use crate::ui_data::UiData;
use crate::ui_data::UiDataRef;
use crate::ui_ref::UiRef;
use crate::value_to_string;

use super::JSMessageTx;
use super::JSType;

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use std::sync::Arc;
use std::sync::Mutex; 

/// Elements values
pub type Values = HashMap<String, String>;

/// Elements collection
pub type Elements = Vec<Element>;

/// Convenience for a properties parameter to express no properties applied.
pub static NONE_PROPERTIES: Option<&[&str]> = None;


/// HTML Element representation
pub struct Element {
    id: String,
    tx: MsgSender,
    ui: UiDataRef,
}

/// Mouse Event types 
pub enum MouseEvent{
    /// mouse button up
    MouseUp,
    /// Mouse button down
    MouseDown,
    /// Mouse move
    MouseMove,
    /// Mouse click (up and down)
    MouseClick,
    /// Mouse double click (Mouse up and down twice)
    MouseDblClick,
}

impl MouseEvent {
    fn as_str(&self) -> &str {
        match self {
            Self::MouseUp  => MOUSE_UP,
            Self::MouseDown  => MOUSE_DOWN,
            Self::MouseMove  => MOUSE_MOVE,
            Self::MouseClick =>  MOUSE_CLICK,
            Self::MouseDblClick =>  MOUSE_DBLCLICK,
        }
    }
} 

impl Clone for Element {    
    fn clone(&self) -> Self {
        let tx = self.tx.clone();
        Element {
            id: self.id.clone(),
            tx,
            ui: self.ui.clone()}
    }
}

impl Element {


    /// Element id
    /// 
    /// # Return
    /// 
    /// Id string
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Unsubscribe event
    /// 
    /// # Arguments
    /// 
    /// `name` - Event name 
    pub fn unsubscribe(&self, name: &str) {
        UiData::remove_subscription(&self.ui, &self.id, name);   
    }


    /// Subscribe event
    /// 
    /// # Arguments
    /// 
    /// `name` - Event name
    /// 
    /// `callback` - Function called on event
    /// 
    /// `properties` - Optional list of properties that are expected to receive on callback. Note that you have to list
    /// all callbacks that you want to receive
    /// 
    /// `throttle` - Minimum time in between consecutive events. If a new event emit in shorter time, it is ignored
    /// 
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// `Event` - Event information
    /// 
    pub fn subscribe_throttled<CB, Str>(&self, name: &str, callback: CB, properties: Option<&[Str]>, throttle: Duration)
    where
        Str: Into<String> + std::clone::Clone,
        CB: FnMut(UiRef, Event) + Send + 'static {
        let callback = Box::new(callback);
        let properties =  match properties {
            Some(p) => p.iter().cloned().map(|v| { JSType::String(v.into())}).collect(),
            None =>  vec!(JSType::from("")),
        };
        UiData::add_subscription(&self.ui, &self.id, name, callback);

        if name != "created" && self.id() != crate::window::MENU_ELEMENT { // created are not subscribed - come when created. MENU_ELEMENT is not HTML
            let throttle = throttle.as_millis().to_string();   
            let msg =  JSMessageTx {
                element: self.id(),
                _type: "event",
                event: Some(name),
                properties: Some(&properties),
                throttle: Some(&throttle),
                ..Default::default()
            };
            self.send(msg);
        }
    }

    /// Subscribe event
    /// 
    /// # Arguments
    /// 
    /// `name` - Event name
    /// 
    /// `callback` - Function called on event
    /// 
    /// `properties` - list of properties that are expected to receive on callback. Note that you have to list
    /// all callbacks that you want to receive
    /// 
    /// 
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// 
    /// `Event` - Event information
    /// 
    pub fn subscribe_properties<CB, Str>(&self, name: &str, callback: CB, properties: &[Str])
    where  Str: Into<String> + std::clone::Clone,
        CB: FnMut(UiRef, Event) + Send + 'static {
            self.subscribe_throttled(name, callback, Some(properties), Duration::from_micros(0u64));

    }
    
    /// Subscribe event
    /// 
    /// # Arguments
    /// 
    /// `name` - Event name
    /// 
    /// `callback` - Function called on event
    /// 
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// 
    /// `Event` - Event information
    pub fn subscribe<CB>(&self, name: &str, callback: CB) 
    where CB: FnMut(UiRef, Event) + Send + 'static {    
        self.subscribe_throttled(name, callback, NONE_PROPERTIES, Duration::from_millis(0u64))
    }

    /// See [subscribe](Self::subscribe)
    pub fn subscribe_async<CB, Fut>(&self, name: &str, async_func: CB)
     where CB: FnOnce(UiRef, Event)-> Fut + Send + Clone + 'static,
     Fut: Future<Output =  ()> + Send +  'static {
        self.subscribe_throttled(name, UiData::as_sync_fn(async_func), NONE_PROPERTIES, Duration::from_millis(0u64))
    }

     /// See [subscribe_properties](Self::subscribe_properties)
    pub fn subscribe_properties_async<CB, Str, Fut>(&self, name: &str, async_func: CB,  properties: &[Str])
    where CB: FnOnce(UiRef, Event)-> Fut + Send + Clone + 'static,
    Fut: Future<Output = ()> + Send +  'static ,
        Str: Into<String> + std::clone::Clone {
       self.subscribe_throttled(name, UiData::as_sync_fn(async_func), Some(properties), Duration::from_millis(0u64))
   }

    /// See [subscribe_throttled](Self::subscribe_throttled)
    pub fn subscribe_throttled_async<CB, Str, Fut>(&self, name: &str, async_func: CB,  properties: &[Str], throttle: Duration)
    where CB: FnOnce(UiRef, Event)-> Fut + Send + Clone + 'static,
    Fut: Future<Output =  ()> + Send +  'static ,
        Str: Into<String> + std::clone::Clone {
       self.subscribe_throttled(name, UiData::as_sync_fn(async_func), Some(properties), throttle)
   }

   /// Subscribe mouse events
   /// 
   /// A Convenience class to subscribe mouse events
   /// 
   /// basically a:
   /// ```no_run 
   /// # use std::time::Duration;
   /// # use futures::Future;
   /// # use crate::gemgui::event::Event;
   /// # use crate::gemgui::ui_ref::UiRef;
   /// # use crate::gemgui::event::MOUSE_CLICK;
   /// # struct Element__ {};
   /// # impl Element__ {
   /// # pub fn subscribe_throttled<CB, Str>(&self, name: &str, callback: CB, properties: Option<&[Str]>, throttle: Duration)
   /// # where
   /// # Str: Into<String> + std::clone::Clone,
   /// # CB: FnMut(UiRef, Event) + Send + 'static {}
   /// # fn foo(&self) { 
   /// # let callback = |_: crate::gemgui::ui_ref::UiRef, _: Event| {};
   /// let properties = vec!("clientX", "clientY");
   /// self.subscribe_throttled(MOUSE_CLICK, callback, Some(&properties), Duration::from_millis(10));
   /// # }
   /// # }
   /// ```
   ///
   /// # Arguments
   /// 
   /// `mouse` - Mouse event 
   /// 
   /// `event` - Callback on mouse event
   /// 
   /// # Callback
   /// 
   /// `UiRef`- Reference to UI
   /// 
   /// `Event` - Event information
   pub fn subscribe_mouse<CB>(&self, mouse: MouseEvent, callback: CB) 
   where CB: FnMut(UiRef, Event) + Send + 'static {
            let properties = vec!("clientX", "clientY");
            /*
            
            Next version suggestion
            possible change is have callback that directly return x y

            let m_cb = move |ui: UiRef, ev: Event| {
                let x = ev.property_str("clientX").unwrap().parse::<i32>().unwrap();
                let y = ev.property_str("clientY").unwrap().parse::<i32>().unwrap();
                callback(ui, x, y, ev.element());
            };*/
            self.subscribe_throttled(mouse.as_str(), callback, Some(&properties), Duration::from_millis(10));
   }

   /// See [subscribe_mouse](Self::subscribe_mouse)
   pub fn subscribe_mouse_async<CB, Fut>(&self, mouse: MouseEvent, async_func: CB)
    where CB: FnOnce(UiRef, Event)-> Fut + Send + Clone + 'static,
    Fut: Future<Output =  ()> + Send +  'static  {
    let properties = vec!("clientX", "clientY");  
    self.subscribe_throttled(mouse.as_str(), UiData::as_sync_fn(async_func), Some(&properties), Duration::from_millis(0u64))
    }

    /// Get HTML content of the element
    /// 
    /// # Return
    /// 
    /// HTML content
    pub async fn html(&self) -> Result<String> {
    let result = self.query("innerHTML", &vec![]).await;
    match result {
        Ok(value) => Ok(String::from(value.as_str().unwrap())),
        Err(e) => Err(e),
    }
}
    

    /// Set inner HTML of the element
    /// 
    /// # Arguments
    /// 
    /// `html`- HTML applied to the element
    /// 
    pub fn set_html(&self, html: &str) {
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "html",
            html: Some(html),
            ..Default::default()
        };
        self.send(msg);
    }

    /// Set element styles
    /// 
    /// # Arguments
    /// 
    /// `style`- Style name
    /// 
    /// `value`- Style value 
    pub fn set_style(&self, style: &str, value: &str) {
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "set_style",
            style: Some(style),
            value: Some(value),
            ..Default::default()
        };
        self.send(msg);
    }


    /// Get element styles
    /// 
    /// # Arguments
    /// 
    /// `filter` -  get only styles that are lists
    /// 
    /// # Return
    /// 
    /// Style - value pairs
    pub async fn styles<Str: AsRef<str>>(&self, filter: &[Str]) -> Result<Values> {
        let plist = filter.iter().map(|s|{String::from(s.as_ref())}).collect();
        let result = self.query("styles", &plist).await;
        match result {
            Ok(value) => {
                match crate::value_to_string_map(value) {
                    Some(v) => {Ok(v)},
                    None => GemGuiError::error("Bad value"),
                }
            },
            Err(e) => Err(e),
        }
    }

    
    /*
    Not working - C++ test was faulty - TODO if needed
    pub fn remove_style<Str>(&self, style: Str) 
    where Str: Into<String> {
        let msg =  JSMessageTx {
            element: self.id.clone(),
            _type: String::from("remove_style"),
            style: Some(style.into()),
            ..Default::default()
        };
        self.send(msg);
    }
    */
    
    /// Apply an attribute
    /// 
    /// For attributes that has no value
    /// 
    /// # Arguments
    /// 
    /// `attr` - attribute name
    pub fn set_attribute_on(&self, attr: &str) {
        self.set_attribute(attr, "");
    }

    /// Set an attribute values
    /// 
    /// 
    /// # Arguments
    /// 
    /// `attr` - attribute name.
    /// 
    /// `value` - attribute value.
    pub fn set_attribute(&self, attr: &str, value: &str) {
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "set_attribute",
            attribute: Some(attr),
            value: Some(value),
            ..Default::default()
        };
        self.send(msg);
    }

    /// Get element attributes
    /// 
    /// 
    /// # Return
    /// 
    /// Attribute - value pairs
    pub async fn attributes(&self) -> Result<Values> {
        let result = self.query("attributes", &vec![]).await;
        match result {
            Ok(value) => {
                match crate::value_to_string_map(value) {
                    Some(v) => {println!("attributes {:#?}", &v); Ok(v)},
                    None => GemGuiError::error("Bad value"),
                }
            },
            Err(e) => Err(e),
        }
    }

    /// Remove an attribute
    /// 
    /// # Arguments
    /// `attr` - attribute name
    pub fn remove_attribute(&self, attr: &str) {
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "remove_attribute",
            attribute: Some(attr),
            ..Default::default()
        };
        self.send(msg);
    }

    /// Get element values
    /// 
    /// # Return
    /// 
    /// name - value pairs
    pub async fn values(&self) -> Result<Values> {
        let result = self.query("value", &vec![]).await;
        match result {
            Ok(value) => {
                match crate::value_to_string_map(value) {
                    Some(v) => {Ok(v)},
                    None => GemGuiError::error("Bad value"),
                }
            },
            Err(e) => Err(e),
        }
    }

    /// Get element child elements
    /// 
    /// # Return
    /// 
    /// children list
    pub async fn children(&self) -> Result<Elements> {
        let result = self.query("children", &vec![]).await;
        match result {
            Ok(value) => UiData::elements_from_values(&self.ui, value, &self.tx),
            Err(e) => Err(e),
        }
    }

    /*
    
    To test etc. enable for next version upon need

        pub async fn tag_name(&self, tag_name: &str) -> Result<Elements, gemguiError> {
        let result = self.query("tag_name", &vec!("tag_name")).await;
        match result {
            Ok(value) => UiData::elements_from_values(&self.ui, value, &self.tx),
            Err(e) => Err(e),
        }
    }
    
    */
    

    /// Remove this element
    pub fn remove(&self) {
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "remove",
            remove: Some(self.id()),
            ..Default::default()
        };
        self.send(msg);
    }

    /// Get element type
    /// 
    /// # Return
    /// 
    /// Element's type tag.
    pub async fn element_type(&self) -> Result<String> {
        let result = self.query("element_type", &vec![]).await;
        match result {
            Ok(tag) => {
                match value_to_string(tag) {
                    Some(v) => Ok(v),
                    None => GemGuiError::error("Bad value"),
                }
            },
            Err(e) => Err(e),
        }
    }



    /// Get element UI rectangle
    /// 
    /// # Return
    /// 
    /// Element's UI rectangle as requested type 
    pub async fn rect<T>(&self) -> Result<Rect<T>>
    where T: FromStr + Clone + Copy {
        let result = self.query("bounding_rect", &vec![]).await;
        let err = |e: &str| GemGuiError::error(format!("Bad value {e}"));
        match result {
            Ok(value) => {
                match crate::value_to_string_map(value) {
                    Some(v) => {
                        if ! (v.contains_key("x") && v.contains_key("y") && v.contains_key("width") && v.contains_key("height")) {
                            return err("N/A");
                        }
                        let x = &v["x"];
                        // values can be fractions, but that is not ok if request is non-float
                        let x = x.split_once('.').unwrap_or((x, "")).0;
                        let x = match x.parse::<T>() {Ok(v) => v, Err(_) => return err(&v["x"])};

                        let y = &v["y"];
                        let y= y.split_once('.').unwrap_or((y, "")).0;
                        let y = match y.parse::<T>() {Ok(v) => v, Err(_) => return err(&v["y"])};

                        let width = &v["width"];
                        let width = width.split_once('.').unwrap_or((width, "")).0;
                        let width = match width.parse::<T>() {Ok(v) => v, Err(_) => return err(&v["width"])};

                        let height = &v["height"];
                        let height = height.split_once('.').unwrap_or((height, "")).0;
                        let height = match height.parse::<T>() {Ok(v) => v, Err(_) => return err(&v["height"])};
                        Ok(Rect {x, y, width, height})
                    },
                    None => err("invalid"),
                }
            },
            Err(e) => Err(e),
        }
    }

    /// UiRef of the element
    /// 
    /// # Return
    /// 
    /// UiRef
    pub fn ui(&self) -> UiRef {
        UiRef::new(self.ui.clone())
    }

    pub (crate) fn call(&self, name: &str, properties: Properties) {
        UiData::call_subscription(&self.ui, &self.id, name, properties);
    }

    pub(crate) fn create(&self, html_element: &str, parent: &Element) {
        let msg =  JSMessageTx {
            element: parent.id(),
            _type: "create",
            new_id: Some(self.id()),
            html_element: Some(html_element),
            ..Default::default()
        };
        self.
        send(msg);
    }
    
    pub(crate) fn construct(id: String, tx: MsgSender, ui: Arc<Mutex<UiData>>) -> Element {
        Element  {
            id,
            tx,
            ui
        }
    }

    pub (crate) fn send_bin(&self, msg: Vec<u8>) {
        self.tx.send_bin(msg);
    }

    pub (crate) fn send(&self, msg: JSMessageTx) {
        let json = serde_json::to_string(&msg).unwrap();
        self.tx.send(json); 
    }

    async fn query(&self, name: &str, query_params: &Vec<String>) -> Result<JSType> {
        UiRef::do_query(&self.ui, &self.id, name, query_params).await
    }

}
