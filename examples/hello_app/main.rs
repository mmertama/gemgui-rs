use gemgui::{self, GemGuiError, ui_ref::UiRef, event::CLICK, ui::Ui};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn on_start(ui: UiRef) {
  let text = ui.element("content");
    let button = ui.element("startbutton");
    button.set_html("Hello...?"); 
    button.subscribe(CLICK, move |_, _| {
        text.set_html("Hello World!");
      });     
} 

fn main() -> Result<(), GemGuiError> { 
    let fm = gemgui::filemap_from(RESOURCES);
    gemgui::application(fm,
       "hello.html",
        gemgui::next_free_port(30000u16),
        |ui| async {on_start(ui).await}
      )
}
