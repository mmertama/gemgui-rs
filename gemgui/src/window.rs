use core::fmt;
use std::path::{Path, PathBuf};

use crate::event::Event;
use crate::ui::Ui;
use crate::{ui_ref::UiRef, GemGuiError, JSMap, JSType, ui_data::UiData, JSMessageTx, ui::private::UserInterface};

use futures::Future;


/// Application menu
/// Use Builder pattern to create menu contents
#[derive(Clone)]
pub struct Menu {
    items: Vec<JSType>,
} 

impl Default for Menu {
    fn default() -> Self {
        Self::new()
         }
    }

pub (crate) static MENU_ELEMENT: &str = "app menu";
static MENU_EVENT: &str = "menu_event";

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct MenuItems {
    #[serde(rename = "type")]
    _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub_menu: Option<Vec<JSType>>,
}

impl Menu {

    
    /// Create an application menu
    /// 
    /// # Arguments
    /// 
    /// `ui` - UiRef
    /// 
    /// # Return
    /// 
    /// Menu 
    /// 
    pub fn new() -> Menu {
        Menu {items: Vec::new()}
    }

    /// Add a sepator in menu
    /// 
    /// # Return
    /// 
    /// Menu 
    /// 
    pub fn add_separator(mut self) -> Menu{
        let item = MenuItems {
            _type: "separator".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_value(item).unwrap();
        self.items.push(json);
        self
    }

    /// Add a menu item
    /// 
    /// # Arguments
    /// 
    /// `title` - Menu item name
    /// 
    /// `action_id` - An identifier get as menu event properties 
    /// 
    /// # Return
    /// 
    /// Menu 
    /// 
    pub fn add_item(mut self, title: &str, action_id: &str) -> Menu {
        let item = MenuItems {
            _type: "action".to_string(),
            title: Some(title.to_string()),
            action_id: Some(action_id.to_string()),
            ..Default::default()
        };
        let json = serde_json::to_value(item).unwrap();
        self.items.push(json);
        self
    }

    /// Add a sub menu
    /// 
    /// # Arguments
    /// 
    /// `title` - Sub menu name
    /// 
    /// `menu` - A sub menu 
    /// 
    /// # Return
    /// 
    /// Menu 
    /// 
    pub fn add_sub_menu(mut self, title: &str, menu: Menu) -> Menu {
        let item = MenuItems {
            _type: "sub_menu".to_string(),
            title: Some(title.to_string()),
            sub_menu: Some(menu.items),
            ..Default::default()
        };
        let json = serde_json::to_value(item).unwrap();
        self.items.push(json);
        self
    }

    /// Subscribe menu events
    /// 
    /// # Arguments
    /// 
    ///  `ui` - Ui reference
    /// 
    /// `callback` - callback on menu event
    /// 
    /// 
    pub fn subscribe<CB>(ui: &UiRef, callback: CB) 
    where CB: FnMut(UiRef, &str) + Send + Clone + 'static {
        let element_cb = move |ui: UiRef, event: Event| {
                let mut callback = callback.clone();
                let id = event.property_str("menu_id").expect("Invalid event");
                callback(ui, id)
            };    
            ui.element(MENU_ELEMENT).subscribe(MENU_EVENT, element_cb)
    }

    /// See [subscribe](Self::subscribe)
    pub fn subscribe_async<CB, Fut>(ui: &UiRef, callback: CB)
     where CB: FnOnce(UiRef, &str)-> Fut + Send + Clone + 'static,
     Fut: Future<Output =  ()> + Send +  'static {
            let element_cb = |ui: UiRef, event: Event| async move {
            let id = event.property_str("menu_id").expect("Invalid event");
            callback(ui, id).await
        };
        ui.element(MENU_ELEMENT).subscribe_async(MENU_EVENT, element_cb)
    }


   #[allow(clippy::inherent_to_string)]
   pub (crate) fn to_string(&self) -> String {
        serde_json::to_string(&self.items).unwrap()
   } 

}


enum DialogType {
    OpenFile,
    OpenFiles,
    OpenDir,
    SaveFile,
}

impl fmt::Display for DialogType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DialogType::OpenFile => write!(f, "openFile"),
            DialogType::OpenFiles => write!(f, "openFiles"),
            DialogType::OpenDir => write!(f, "openDir"),
            DialogType::SaveFile => write!(f, "saveFile"),
        }
    }
}

enum DialogValue {
    FileName(String),
    FileNames(Vec<String>),
}

fn make_filters(filters: &[(&str, std::vec::Vec<&str>)]) -> JSMap {
    let mut ft = JSMap::new();
    for (name, exts) in filters.iter() {
        let mut ext_vec = Vec::new();
        for ext in exts.iter() {
            ext_vec.push(serde_json::json!(ext));
        }
        ft.insert(name.to_string(), serde_json::json!(ext_vec));
    }
    ft
}

    /// Open file dialog
    /// 
    /// # Arguments
    /// 
    /// `ui` - UiRef
    /// 
    /// `dir` - Initial view directory
    /// 
    /// `filters` - Filters for dialog, 
    ///  List of tuples having a filter name and filters
    ///  e.g. ["Text", vec!("*.txt", "*.text")]
    /// 
    /// # Return
    /// 
    /// Path to selected file
    /// 
pub async fn open_file(ui: &UiRef, dir: &Path, filters: &[(&str, std::vec::Vec<&str>)]) -> Result<PathBuf, GemGuiError>  {
    let ft = make_filters(filters);
    let mut properties = JSMap::new();
    properties.insert("dir".to_string(), JSType::from(dir.to_string_lossy()));
    properties.insert("filters".to_string(), serde_json::json!(ft));
    let file_name = dialog(ui, DialogType::OpenFile, properties).await?;
    if let DialogValue::FileName(file_name) = file_name {
        let path = Path::new(&file_name);
        return Ok(path.to_path_buf());
    }
    GemGuiError::error("Invalid type".to_string())
}


    /// Open files dialog
    /// 
    /// # Arguments
    /// 
    /// `ui` - UiRef
    /// 
    /// `dir` - Initial view directory
    /// 
    /// `filters` - Filters for dialog, 
    ///  List of tuples having a filter name and filters
    ///  e.g. ["Text", vec!("*.txt", "*.text")]
    /// 
    /// # Return
    /// 
    /// Vector of paths to selected files
    /// 
pub async fn open_files(ui: &UiRef, dir: &Path, filters: &[(&str, std::vec::Vec<&str>)]) -> Result<Vec<PathBuf>, GemGuiError>  {
    let ft = make_filters(filters);
    let mut properties = JSMap::new();
    properties.insert("dir".to_string(), JSType::from(dir.to_string_lossy()));
    properties.insert("filters".to_string(), serde_json::json!(ft));
    let file_name = dialog(ui, DialogType::OpenFiles, properties).await?;
    if let DialogValue::FileNames(file_names) = file_name {
        let mut paths = Vec::new();
        for fname in file_names.iter() {
            let path = Path::new(&fname).to_path_buf();
            paths.push(path);
        }
        
        return Ok(paths);
    }
    GemGuiError::error("Invalid type".to_string())
}


    /// Open directory dialog
    /// 
    /// # Arguments
    /// 
    /// `ui` - UiRef
    /// 
    /// `dir` - Initial view directory
    /// 
    /// # Return
    /// 
    /// Path to selected directory
    /// 
pub async fn open_dir(ui: &UiRef, dir: &Path) -> Result<PathBuf, GemGuiError>  {
    let mut properties = JSMap::new();
    properties.insert("dir".to_string(), JSType::from(dir.to_string_lossy()));
    let file_name = dialog(ui, DialogType::OpenDir, properties).await?;
    if let DialogValue::FileName(file_name) = file_name {
        let path = Path::new(&file_name);
        return Ok(path.to_path_buf());
    }
    GemGuiError::error("Invalid type".to_string())
}


    /// Open save dialog
    /// 
    /// # Arguments
    /// 
    /// `ui` - UiRef
    /// 
    /// `dir` - Initial view directory
    /// 
    /// `filters` - Filters for dialog, 
    ///  List of tuples having a filter name and filters
    ///  e.g. ["Text", vec!("*.txt", "*.text")]
    /// 
    /// # Return
    /// 
    /// Path to selected or created file 
    /// 
pub async fn save_file(ui: &UiRef, dir: &Path, filters: &[(&str, std::vec::Vec<&str>)]) -> Result<PathBuf, GemGuiError>  {
    let ft = make_filters(filters);
    let mut properties = JSMap::new();
    properties.insert("dir".to_string(), JSType::from(dir.to_string_lossy()));
    properties.insert("filters".to_string(), serde_json::json!(ft));
    let file_name = dialog(ui, DialogType::SaveFile, properties).await?;
    if let DialogValue::FileName(file_name) = file_name {
        let path = Path::new(&file_name);
        return Ok(path.to_path_buf());
    }
    GemGuiError::error("Invalid type".to_string())

}

async fn dialog(ui: &UiRef, dialog_type: DialogType, dialog_params: JSMap) ->  Result<DialogValue, GemGuiError>  {
    let (id, receiver) = UiData::new_query(ui.ui());
    let extension_call = dialog_type.to_string();
    let msg =  JSMessageTx {
        _type: "extension",
        extension_id: Some(&id),
        extension_call: Some(&extension_call),
        extension_params: Some(&dialog_params),
        ..Default::default()
    };

    UiData::send(ui.ui(), msg);

    // spawn an syncrnous wait and wait that async
    let value = tokio::task::spawn_blocking(move || {
        receiver.blocking_recv()
    }).await.unwrap_or_else(|e| {panic!("Extension spawn blocking {e:#?}")});

    match value {
        Ok(value) => {
            match dialog_type {
                DialogType::OpenFiles => {
                    match crate::value_to_string_list(value) {
                    Some(v)  => Ok(DialogValue::FileNames(v)),
                    None => GemGuiError::error("Bad value"),
                    }
                },
                _ => Ok(DialogValue::FileName(value.as_str().expect("Not a string").to_string()))
            }        
        },
        Err(e) => GemGuiError::error(format!("Extension error {e}"))
    } 
}
