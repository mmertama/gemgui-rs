
use gemgui::{event::{CLICK, CHANGE, Event}, ui_ref::UiRef, GemGuiError, graphics::{bitmap_transform::BitmapTransform, canvas::Canvas, bitmap::Bitmap}, ui::Ui};
include!(concat!(env!("OUT_DIR"), "/generated.rs"));


fn radio_buttons(ui: &UiRef) {
  ui.element("title_color_red").subscribe(CHANGE, |ui, _| {
    ui.element("some_header").set_style("color", "red");
  });
  
  ui.element("title_color_green").subscribe(CHANGE, |ui, _|{
    ui.element("some_header").set_style("color", "green");
  });  
}

fn button(ui: &UiRef) {
  let text = ui.element("content");
  let button = ui.element("startbutton");
  button.set_html("Hello...?"); 
  button.subscribe(CLICK, move |_,_| {
      text.set_html("Hello World!");
    });
}

// this is async as calls async functions 
async fn slider(ui: &UiRef) {
  let image = ui.resource("widgets.jpeg").unwrap();
  let image = Bitmap::from_image_bytes(&image).unwrap();
  let rotated = Canvas::from(ui.element("rotated"));
  let rect = rotated.rect::<u32>().await.unwrap();
  let mut image = BitmapTransform::new(&image);
  image.resize_in(rect.width(), rect.height()).unwrap();
  image.center(rect.width(), rect.height()).unwrap();
  image.store(); // fix to this
  rotated.draw_bitmap(&image);
  ui.element("rotation").subscribe_properties(CHANGE, move |ui, event: Event| {
    let angle_value = event.property_str("value").unwrap();
    let value = angle_value.parse::<f64>().unwrap();
    ui.element("angle_text").set_html(angle_value);
    let radians = value.to_radians();
    image.restore(); // angle from base line - not cumulated
    image.rotate(radians);
    rotated.draw_bitmap(&image);
  }, &["value"]);
}

fn check_box(ui: &UiRef) {
  // we are using _async here  as callback contains async functions
  ui.element("image_2x").subscribe_async(CHANGE, |ui, event | async move {
    let logo = ui.element("rust_logo");
    let attr = logo.attributes().await.unwrap();
    let w = &attr["width"];
    let h = &attr["height"];
    let w = w.parse::<i32>().unwrap();
    let h = h.parse::<i32>().unwrap();
    let is_cheked = event.element().values().await.unwrap()["checked"] == "true";
    if is_cheked {
        logo.set_attribute("width", &format!("{}", w * 2));
        logo.set_attribute("height", &format!("{}", h * 2));
    } else {
      logo.set_attribute("width", &format!("{}", w / 2));
      logo.set_attribute("height", &format!("{}", h / 2));
    }
  });
}

fn list(ui: &UiRef) {
  let lang = ui.element("lang");
  lang.subscribe_properties_async(CHANGE, |ui, event| async move {
    let values = event.element().values().await.unwrap();
    let text = format!("{} --> {}", event.property_str("selectedIndex").unwrap_or("??"), values["value"]);
    ui.element("lang_value").set_html(&text);
  }, &["selectedIndex"]);
}

async fn async_main(ui: UiRef) { 
    ui.set_logging(true);
    button(&ui);
    radio_buttons(&ui);
    check_box(&ui);
    slider(&ui).await;
    list(&ui);
    list_box(&ui).await;
}

async fn list_box(ui: &UiRef) {
  let items = vec!("Neptunium", "Plutonium", "Americium", "Curium", "Berkelium", "Californium", "Einsteinium",
  "Fermium", "Mendelevium", "Nobelium", "Lawrencium", "Rutherfordium", "Dubnium", "Seaborgium", "Bohrium", "Hassium",
  "Meitnerium", "Darmstadtium", "Roentgenium", "Copernicium", "Nihonium", "Flerovium", "Moscovium", "Livermorium",
  "Tennessine", "Oganesson");
  let list_box = ui.element("ss_elem_list");
  //list_box.subscribe(CLICK, move |_,_| println!("on"));
  for item in items.iter() {
    let id = format!("ss_elem_{}", &item[0..3]); // take 
    let list_item= ui.add_element_with_id_async(&id, "li", &list_box).await.unwrap();
    assert_eq!(list_item.id(), &id);
    assert!(ui.exists(&id).await.unwrap());
    list_item.set_attribute("role", "option");
    list_item.set_html(item);
    let html = item.to_string();
    list_item.subscribe(CLICK, move |ui,_| ui.element("ss_elem").set_html(&html));
  }
}


fn main() -> Result<(), GemGuiError> {
  let fm = gemgui::filemap_from(RESOURCES);
  gemgui::application(fm,
     "widgets.html",
      gemgui::next_free_port(30000u16),
      |ui| async {async_main(ui).await})
  }
