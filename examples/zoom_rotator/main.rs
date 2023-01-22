
use std::time::{Instant, Duration};

use gemgui::graphics::bitmap_transform::BitmapTransform;
use gemgui::graphics::canvas::{Canvas, DrawNotify};
use gemgui::graphics::bitmap::Bitmap;
use gemgui::ui::Ui;
use gemgui::ui_ref::UiRef;
use gemgui::{self, GemGuiError, Value};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn amain(ui: UiRef) {
  let image = ui.resource("widgets.jpeg").unwrap();
  ui.set_logging(true);
  let bitmap = Bitmap::from_image_bytes(&image).unwrap();
  let mut transformer = BitmapTransform::new(&bitmap);

  let mut t:f64 = 0.0; 

  let canvas = ui.element("canvas");
  let rect = canvas.rect::<u32>().await.unwrap();
  let ww = rect.width();
  let wh = rect.height();

  let mut count = 0u32; 
  let mut int = Instant::now();
  let fps_element = ui.element("fps");

  let scale_value = Value::new(1.0);
  let scaled_value = scale_value.clone();
  let enlarge_value = Value::new(true);

  ui.periodic(Duration::from_millis(100), move |_, _| {
    let mut scale = scale_value.cloned();
    let mut enlarge = enlarge_value.cloned();
    if scale > 2.0 {
      enlarge = false;
      enlarge_value.assign(enlarge);
    }
    if scale < 0.5 {
      enlarge = true;
      enlarge_value.assign(enlarge);
    }    
    if enlarge {
        scale += 0.1;
      } else {
        scale -= 0.1;
      }
      scale_value.assign(scale);     
  });

  let canvas = Canvas::from(canvas);
  canvas.on_draw(move |ui| {
    let a = (t * 0.20).sin() * std::f64::consts::PI / 0.5;
    t += 0.2;
    transformer.restore(); // read initial 
    transformer.rotate(a); //  rotate  a angles
    transformer.resize(ww, wh).unwrap(); // scale to window size
    transformer.center(ww, wh).unwrap(); // center in
    let canvas = Canvas::from(ui.element("canvas"));

    let scale = scaled_value.cloned();
  
    transformer.scale(scale, scale).unwrap();
    transformer.center(ww, wh).unwrap();

    canvas.draw_bitmap(&transformer);

    if count == 50 {
       let elapsed = int.elapsed().as_millis();
       let fps = (1000 * 50) as f32 / elapsed as f32;
       fps_element.set_html(&fps.to_string());
       count = 0;
       int = Instant::now();
    }

    count += 1;
    
  }, DrawNotify::Kick);

}

fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    gemgui::application(fm,
       "index.html",
        gemgui::next_free_port(30000u16),
        |ui| async {amain(ui).await})
    }
    

