use gemgui::element::MouseEvent;
use gemgui::event::Event;
use gemgui::graphics::bitmap_transform::BitmapTransform;
use gemgui::graphics::canvas::Canvas;
use gemgui::graphics::bitmap::{Bitmap, BitmapData};
use gemgui::graphics::color as Color;
use gemgui::ui::Ui;
use gemgui::ui_ref::UiRef;
use gemgui::{self, GemGuiError, Rect};


include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn amain(ui: UiRef) {
  let image = ui.resource("alien.png").unwrap();
  let bitmap = Bitmap::from_image_bytes(&image).unwrap();

  let mut transformer = BitmapTransform::new(&bitmap);

  let source = Canvas::from(ui.element("source"));
  let source_rect = source.rect::<f64>().await.unwrap();
  let source_x = source_rect.x();
  let source_y = source_rect.y();

  transformer.resize(source_rect.width() as u32, source_rect.height() as u32).unwrap(); //  transform to canvas size
  source.draw_bitmap(&transformer); // draw on screen
  let source_bitmap = Bitmap::from(&transformer);
  let backup = source_bitmap.clone(); // we can move bitmap into only one callback, therefore we copy

  transformer.restore(); // set back to original  size

  transformer.scale(2.0, 2.0).unwrap(); //  scale x2

  let target = ui.element("target");
  let target_rect = target.rect::<f64>().await.unwrap();
  let width = target_rect.width() as u32;
  let height = target_rect.height() as u32;

  let width_ratio = (transformer.width() - width) as f64  / width as f64;
  let height_ratio = (transformer.height() - height) as f64  / height as f64;

  let rect_width_ratio = (bitmap.width() as f64 * 2.0) /  target_rect.width();
  let rect_height_ratio = (bitmap.height() as f64 * 2.0) /  target_rect.height();
  let rect_bitmap = Bitmap::rect(
    (source_rect.width()  / rect_width_ratio) as u32,
    (source_rect.height() / rect_height_ratio) as u32,
    Color::rgba(0xFF, 0xFF, 0xFF, 0x4F));

  // just a local function to avoid copy-paste snippet to get mouse x,y 
  let mouse_pos =  move |event: Event| -> Option<(f64, f64)> {
    let x = event.property_str("clientX").unwrap().parse::<i32>().unwrap() as f64 - source_x;
    let y = event.property_str("clientY").unwrap().parse::<i32>().unwrap()  as f64 - source_y;
    if x >= 0.0  && y >= 0.0 && x < source_rect.width() && y < source_rect.height()  {
      Some((x,  y))
    } else {
      None
    }
  };

  source.subscribe_mouse(MouseEvent::MouseMove, move  |ui, event| {
    let target = Canvas::from(ui.element("target"));
    let pos = mouse_pos(event);
    if pos.is_none() {
      return;
    }
    let (x, y) = pos.unwrap();
    let x = (width_ratio * x) as u32;
    let y = (height_ratio * y) as u32;
    let clipped = Bitmap::clip(&Rect::new(x, y, width, height), &transformer);
    if let Ok(clipped) = clipped {
      target.draw_bitmap(&clipped);
    }
  });


  source.subscribe_mouse(MouseEvent::MouseDown, move |ui, event| {
    let source = Canvas::from(ui.element("source"));
    let pos = mouse_pos(event);
    if pos.is_none() {
      return;
    }
    let (x, y) = pos.unwrap();
    let x  = (x  - (rect_bitmap.width() as f64 / 2.0)).round() as i32;
    let y  = (y  - (rect_bitmap.height() as f64 / 2.0)).round() as i32;
    assert!(x < source_bitmap.width() as i32);
    assert!(y < source_bitmap.height() as i32);
    let mut merged = source_bitmap.clone();
    merged.merge(x, y, &rect_bitmap);
    source.draw_bitmap(&merged);
  });

  source.subscribe_mouse(MouseEvent::MouseUp, move |ui, _| {
    let source = Canvas::from(ui.element("source"));
    source.draw_bitmap(&backup); // draw on screen
  });

}

fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    gemgui::window_application(fm,
       "index.html",
        gemgui::next_free_port(30000u16),
        |ui| async {amain(ui).await},
        "Pick",
        900,
        500,
        &[],
        0)
    }
    

