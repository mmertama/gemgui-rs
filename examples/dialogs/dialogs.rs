

use gemgui::element::Element;
use gemgui::ui::Ui;
use gemgui::ui_ref::UiRef;
use gemgui::{self, GemGuiError, event};
use gemgui::dialogs;
use home;
use std::fs;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn remove_files(ui: &UiRef) {

  let files = ui.by_class("file_entry").await;

  match files {
    Ok(files) => files.iter().for_each(|e| e.remove()),
    _=>()
  }

}

async fn amain(ui: UiRef) {

    // pick a files and show its content
    ui.element("open_file").subscribe_async(event::CLICK, |ui, _| async move {

      let result = dialogs::open_file(&ui,
        &home::home_dir().expect("Cannot find home"),
        &[("Text", vec!("*.txt", "*.text"))]).await;

      match result {

        Ok(file_name) => {

          let contents = fs::read_to_string(&file_name).expect(&format!("Cannot open {}", &file_name.to_string_lossy()));
          ui.element("file_content").set_html(&contents)
        },

        Err(e) => eprintln!("On file open {e}"),
      };

    });

    // pick files and show selection
    ui.element("open_files").subscribe_async(event::CLICK, |ui, _| async move {

        let result = dialogs::open_files(
          &ui, &home::home_dir().expect("Cannot find home"), &[]).await;

        remove_files(&ui).await;

        let ul = ui.element("files");

        match result {

          Ok(file_names) => file_names.iter().for_each(move |e| {
            let name = e.to_string_lossy().to_string();
            ui.add_element("li", &ul, move |ui, el: Element| {
              el.set_attribute("class", "file_entry"); // set the class so they can be removed easy
              el.set_html(&name);}).expect("Cannot create element");
            }),
            Err(e) => eprintln!("On files open {e}"),
          };  

    });

    // open dir and show its listing
    ui.element("open_dir").subscribe_async(event::CLICK, |ui, _| async move {

      let result = dialogs::open_dir(
        &ui, &home::home_dir().expect("Cannot find home")).await;

      remove_files(&ui).await;

      let ul = ui.element("files");
      
      match result {

        Ok(file_name) => {
          
          let contents = fs::read_dir(&file_name).expect(&format!("Cannot open {}", &file_name.to_string_lossy()));

          contents.for_each(|e|{
            if let Ok(entry) = e {
              let entry_name = entry.file_name().to_string_lossy().to_string();
              ui.add_element("li", &ul, move |ui, el: Element| {
                el.set_attribute("class", "file_entry"); // set the class so they can be removed easy
                el.set_html(&entry_name);
              }).expect("Cannot create li for file");
              
            }
          });
        },

        Err(e) => eprintln!("On dir open {e}"),
      };
    });
  

    // Save text from text Area
    ui.element("save_file").subscribe_async(event::CLICK, |ui, _| async move {
      
      let result = dialogs::save_file(&ui,
        &home::home_dir().expect("Cannot find home"),
        &[("Text", vec!("*.txt", "*.text"))]).await;

      let text = ui.element("text").html().await.expect("Cannot read content from text");  

      match result {
        Ok(file_name) => {
          fs::write(&file_name, text).expect(&format!("Cannot write to {}", &file_name.to_string_lossy()));
        },

        Err(e) => eprintln!("On file save {e}"),
      };
    });
    ui.element("exit").subscribe(event::CLICK, |ui, _| ui.exit());
}

fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    gemgui::window_application(fm,
       "dialogs.html",
        gemgui::next_free_port(30000u16),
        |ui| async {amain(ui).await},
        "Dialogs", 800, 650, &[("debug", "True")], 0
      )
    }
    

