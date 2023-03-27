

use gemgui::ui::Ui;
use gemgui::ui_ref::UiRef;
use gemgui::{self, GemGuiError, Value, event};
use gemgui::dialogs;
use home;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn amain(ui: UiRef) {
    ui.element("open_file").subscribe_async(event::CLICK, |ui, _| async {
      let result = dialogs::open_file(ui, &home::home_dir().expect(
        "Cannot find home"),
        &[("Text", vec!("*.txt", "*.text"))]).await;
      match result {
        Ok(file_name) => {println!("file_name {}", file_name.to_string_lossy());},
        Err(e) => eprintln!("On file open {e}"),
      };
    });
    ui.element("open_files").subscribe(event::CLICK, |ui, _| {});
    ui.element("open_dir").subscribe(event::CLICK, |ui, _| {});
    ui.element("save_file").subscribe(event::CLICK, |ui, _| {});
    ui.element("exit").subscribe(event::CLICK, |ui, _| ui.exit());
}

fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    gemgui::window_application(fm,
       "dialogs.html",
        gemgui::next_free_port(30000u16),
        |ui| async {amain(ui).await},
        "Dialogs", 800, 650, &[], 0)
 
    }
    

