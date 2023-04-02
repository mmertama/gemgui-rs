use std::collections::HashMap;

use crate::ui_data::UiDataRef;
use crate::{element::Element, ui_data::UiData};

pub (crate) type Properties = HashMap<String, String>;

/// See [Ui](`crate::element::MouseEvent::MouseMove`)
pub static MOUSE_MOVE: &str = "mousemove";
/// See [Ui](`crate::element::MouseEvent::MouseUp`)
pub static MOUSE_UP: &str = "mouseup";
/// See [Ui](`crate::element::MouseEvent::MouseDown`)
pub static MOUSE_DOWN: &str = "mousedown";
/// See [Ui](`crate::element::MouseEvent::MouseClick`)
pub static MOUSE_CLICK: &str = "click";
/// See [Ui](`crate::element::MouseEvent::MouseDblClick`)
pub static MOUSE_DBLCLICK: &str = "dblclick";

/// Key up
pub static KEY_UP: &str = "keyup";
/// Key pressed
pub static KEY_PRESS: &str = "keypress";
/// Key down
pub static KEY_DOWN: &str = "keydown";
/// Scroll
pub static SCROLL: &str = "scroll";
/// Generic click
pub static CLICK: &str = "click";
/// Generic change
pub static CHANGE: &str = "change";
/// Generic select
pub static SELECT: &str = "select";
/// On focus 
pub static FOCUS: &str = "focus";
/// Off focus
pub static BLUR: &str = "blur";
/// In Focus
pub static FOCUS_IN: &str = "focusin";
/// Focus Out
pub static FOCUS_OUT: &str = "focusout";


/// Ui Event
#[derive(Clone)]
pub struct Event {
    ui: UiDataRef,
    source: String,
    properties:  Properties,
}


impl Event {
    
    pub (crate) fn new(ui: UiDataRef, source: String, properties: Properties) -> Event {
        Event{ui, source, properties}
    }

    /// Element emit the event
    /// 
    /// # Return
    /// 
    /// Element
    pub fn element(&self) -> Element {
        UiData::element(&self.ui, &self.source)
    }
    
    /// Element property
    /// 
    /// # Arguments 
    /// 
    /// 'key' - property name
    /// 
    /// # Return
    /// 
    /// Property value 
    pub fn property_str(&self, key: &str) -> Option<&str> {
        if ! self.properties.contains_key(key) {
            return None;
        }
        Some(&self.properties[key])
    }

    /*

    Consider enable / test (there is some issue with this) upon need

    pub fn property<T>(&self, key: &str) -> Option<T>
    where T: for<'a> Deserialize<'a> {
        let string = self.property_str(key);
        if string.is_none() {
            return None;
        }       

        match serde_json::from_str(&string.unwrap()) {
            Ok(js) => Some(js),
            Err(_) => None,
        }
    }
   */ 
}
