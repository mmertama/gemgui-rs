

use gemgui::element::Element;
use gemgui::ui::Ui;
use gemgui::ui_ref::UiRef;
use gemgui::{self, GemGuiError, event};
use gemgui::window::{self, Menu};
use std::fs;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn remove_files(ui: &UiRef) {

  let files = ui.by_class("file_entry").await;

  if let Ok(files) = files {
     files.iter().for_each(|e| e.remove())
    }

}

async fn amain(ui: UiRef) {

  // subscribe and handle menu events
    Menu::subscribe(&ui, |ui, id| {
      if id == "do_exit" {
        eprintln!("App exit");
        ui.exit();
      }
      if id == "do_about" {
        ui.alert("This is a menu about!");
      }
    });

    // pick a files and show its content
    ui.element("open_file").subscribe_async(event::CLICK, |ui, _| async move {

      let result = window::open_file(&ui,
        &home::home_dir().expect("Cannot find home"),
        &[("Text", vec!("*.txt", "*.text"))]).await;

      match result {

        Ok(file_name) => {

          let contents = fs::read_to_string(&file_name).unwrap_or_else(|_| panic!("Cannot open {}", &file_name.to_string_lossy()));
          ui.element("file_content").set_html(&contents)
        },

        Err(e) => eprintln!("On file open {e}"),
      };

    });

    // pick files and show selection
    ui.element("open_files").subscribe_async(event::CLICK, |ui, _| async move {

        let result = window::open_files(
          &ui, &home::home_dir().expect("Cannot find home"), &[]).await;

        remove_files(&ui).await;

        let ul = ui.element("files");

        match result {

          Ok(file_names) => file_names.iter().for_each(move |e| {
            let name = e.to_string_lossy().to_string();
            ui.add_element("li", &ul, move |_, el: Element| {
              el.set_attribute("class", "file_entry"); // set the class so they can be removed easy
              el.set_html(&name);}).expect("Cannot create element");
            }),
            Err(e) => eprintln!("On files open {e}"),
          };  

    });

    // open dir and show its listing
    ui.element("open_dir").subscribe_async(event::CLICK, |ui, _| async move {

      let result = window::open_dir(
        &ui, &home::home_dir().expect("Cannot find home")).await;

      remove_files(&ui).await;

      let ul = ui.element("files");
      
      match result {

        Ok(file_name) => {
          
          let contents = fs::read_dir(&file_name).unwrap_or_else(|_| panic!("Cannot open {}", &file_name.to_string_lossy()));

          contents.for_each(|e|{
            if let Ok(entry) = e {
              let entry_name = entry.file_name().to_string_lossy().to_string();
              ui.add_element("li", &ul, move |_, el: Element| {
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
      
      let result = window::save_file(&ui,
        &home::home_dir().expect("Cannot find home"),
        &[("Text", vec!("*.txt", "*.text"))]).await;

      let text = ui.element("text").html().await.expect("Cannot read content from text");  

      match result {
        Ok(file_name) => {
          fs::write(&file_name, text).unwrap_or_else(|_| panic!("Cannot write to {}", &file_name.to_string_lossy()));
        },

        Err(e) => eprintln!("On file save {e}"),
      };
    });
    ui.element("exit").subscribe(event::CLICK, |ui, _| ui.exit());
}

fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    
    let file_menu = Menu::new().
      add_item("Exit", "do_exit");

      let about_menu = Menu::new().
      add_item("About", "do_about");  
    
    let menu = Menu::new().
      add_sub_menu("File", file_menu).
      add_sub_menu("About", about_menu);

    gemgui::window_application(fm,
       "dialogs.html",
        gemgui::next_free_port(30000u16),
        |ui| async {amain(ui).await},
        "Dialogs", 800, 650, &[("debug", "True")], 0,
        menu
      )
    }
    

