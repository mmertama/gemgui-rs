
use gemgui::event::CLICK;
use gemgui::{self, GemGuiError};
use gemgui::ui::{Gui, Ui};
use std::env;


#[tokio::main]
async fn main() -> Result<(), GemGuiError> { 
    let mut path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    path.push("examples/hello_load/gui");
    if ! path.is_dir() {
        eprintln!("Error: path {path:#?} not found");
        std::process::exit(1);
    }
    let fm = gemgui::filemap_from_dir(&path).unwrap();
    let mut ui = Gui::new(fm, "hello.html", 30000u16).unwrap();
    let text = ui.element("content");
    let button = ui.element("startbutton");
    button.subscribe_async(CLICK, |_, _| async move {
        let html = text.html().await.unwrap();
        let lmth = html.chars().rev().collect::<String>();
        text.set_html(&lmth);
      });     
    ui.run().await
}
