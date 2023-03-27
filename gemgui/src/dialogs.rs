use core::fmt;
use std::path::{Path, PathBuf};

use crate::{ui_ref::UiRef, GemGuiError, JSMap, JSType, ui_data::UiData, JSMessageTx, ui::private::UserInterface};

enum DialogType {
    OpenFile,
    OpenFiles,
    OpenDir,
    SaveFiles,
}

impl fmt::Display for DialogType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DialogType::OpenFile => write!(f, "openFile"),
            DialogType::OpenFiles => write!(f, "openFiles"),
            DialogType::OpenDir => write!(f, "openDir"),
            DialogType::SaveFiles => write!(f, "saveFile"),
        }
    }
}

enum DialogValue {
    FileName(String),
    FileNames(Vec<String>),
}

pub async fn open_file(ui: UiRef, dir: &Path, filters: &[(&str, std::vec::Vec<&str>)]) -> Result<PathBuf, GemGuiError>  {
    let mut ft = JSMap::new();
    for (name, exts) in filters.iter() {
        let mut ext_vec = Vec::new();
        for ext in exts.iter() {
            ext_vec.push(serde_json::json!(ext));
        }
        ft.insert(name.to_string(), serde_json::json!(ext_vec));
    }
    let mut properties = JSMap::new();
    properties.insert("dir".to_string(), JSType::from(dir.to_string_lossy()));
    properties.insert("filters".to_string(), serde_json::json!(ft));
    let file_name = dialog(ui, DialogType::OpenFile, properties).await?;
    if let DialogValue::FileName(file_name) = file_name {
        let path = Path::new(&file_name);
        return Ok(PathBuf::from(path));
    }
    return Err(GemGuiError::Err(format!("Invalid type")));

}

async fn dialog(ui: UiRef, dialog_type: DialogType, dialog_params: JSMap) ->  Result<DialogValue, GemGuiError>  {
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
                _ => Ok(DialogValue::FileName(value.to_string()))
            }        
        },
        Err(e) => Err(GemGuiError::Err(format!("Extension error {e}")))
    } 
}
