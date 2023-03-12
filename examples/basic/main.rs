
use gemgui::event::{self};
use gemgui::graphics::bitmap::Bitmap;
use gemgui::graphics::canvas::{Canvas, DrawNotify};
use gemgui::graphics::color as Color;
use gemgui::{self, GemGuiError};
use gemgui::ui::{Gui, Ui};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));


#[tokio::main]
async fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    let mut ui = Gui::new(fm, "hello.html", gemgui::next_free_port(30000u16)).unwrap();
    ui.set_logging(true);
    let canvas = Canvas::new(&ui.element("canvas"));
    let bmp = Bitmap::rect(100, 100, Color::CYAN);
    canvas.draw_bitmap_at(99, 10, &bmp);
    canvas.on_draw( move |ui| {
      let canvas = Canvas::new(&ui.element("canvas"));
      canvas.draw_bitmap_at(200, 300, &bmp);
      canvas.on_draw(|_|{
        println!("Do Nothing");
      }, DrawNotify::NoKick);
    }, DrawNotify::NoKick);
    ui.on_start_async(|_| async move {println!("on start")});
    ui.element("exit_button").subscribe(event::CLICK, |ui, _| ui.exit());
    ui.run().await
}

