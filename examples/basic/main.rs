
use gemgui::{self, GemGuiError, event};
use gemgui::ui::{Gui, Ui};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));


#[tokio::main]
async fn main() -> Result<(), GemGuiError> { 
    let fm = gemgui::filemap_from(RESOURCES);
    let mut ui = Gui::new(fm, "hello.html", gemgui::next_free_port(30000u16)).unwrap();
    let text = ui.element("content");
    let button = ui.element("startbutton");
    ui.on_start_async(|_| async move {println!("on start")});
    button.set_html("Hello...?"); 
    button.subscribe(event::CLICK, move |_, _| {
        text.set_html("Hello World!");
      });     
    ui.run().await
}
