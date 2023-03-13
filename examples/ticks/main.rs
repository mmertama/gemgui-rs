
use gemgui::{self, GemGuiError};
use gemgui::ui::{Gui, Ui};
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[tokio::main]
async fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    let mut ui = Gui::new(fm, "index.html", gemgui::next_free_port(30000u16)).unwrap();
    let counter0 = ui.element("counter0");
    let counter1 = ui.element("counter1");
    let header = ui.element("header");
    let mut count0 = 0;
    let mut count1 = 0;
 
    ui.after(Duration::from_millis(10000), move |_,_| {
      header.set_html("time force");
    });
 
    ui.periodic(Duration::from_millis(1000), move |_,_| {
      count0 += 1;
      counter0.set_html(&format!("{count0}"));
    });
 
    ui.periodic(Duration::from_millis(100), move |uu, _| {
      count1 += 1;
      counter1.set_html(&format!("{count1}"));
        if count1 == 100  {
            uu.exit();
        }
    });
    let (ver, maj, min) = gemgui::version();
    ui.element("ver").set_html(&format!("{ver}.{maj}.{min}"));
    ui.run().await
}

