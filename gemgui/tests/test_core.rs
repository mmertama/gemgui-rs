use gemgui::graphics::color::{name_to_rgb, style_to_name};

use gemgui::ui_ref::UiRef;
use gemgui::ui::{Target, Ui};

use std::path::Path;
use std::time::Instant;
use tokio::time::Duration;

use std::panic;


use serial_test::serial;

use crate::tests::setup;

#[path="./tests.rs"]
mod tests;


#[tokio::test]
#[serial]
async fn test_on_start() { // These on_xxxx tests that there is no deadlock as element do locking
     let mut ui = setup();
     static  mut VISITED: bool = false;
     ui.on_start(|ui| {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.run().await.unwrap();
    println!("ui: {:#?}", ui);
    unsafe{assert!(VISITED)};
}

#[tokio::test]
#[serial]
async fn test_on_start_async() { // These on_xxxx tests that there is no deadlock as element do locking
    let mut ui = setup();
    static  mut VISITED: bool = false;
    ui.on_start_async(|ui| async move {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.run().await.unwrap();
    unsafe{assert!(VISITED)};
}
 
#[tokio::test]
#[serial]
async fn test_on_error() { // These on_xxxx tests that there is no deadlock as element do locking
    let mut ui = setup();
    static  mut VISITED: bool = false;
    ui.on_error(|ui, _| {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.eval("blaa");
    ui.run().await.unwrap();
    unsafe{assert!(VISITED)};
}

#[tokio::test]
#[serial]
async fn test_on_error_async() { // These on_xxxx tests that there is no deadlock as element do locking
    let mut ui = setup();
    static  mut VISITED: bool = false;
    ui.on_error_async(|ui, _| async move {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.eval("blaa");
    ui.run().await.unwrap();
    unsafe{assert!(VISITED)};
}

#[tokio::test]
#[serial]
async fn test_on_after() { // These on_xxxx tests that there is no deadlock as element do locking
    let mut ui = setup();
    static  mut VISITED: bool = false;
    ui.after(Duration::from_millis(10), |ui, _| {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.run().await.unwrap();
    unsafe{assert!(VISITED)};
}

#[tokio::test]
#[serial]
async fn test_on_after_async() { // These on_xxxx tests that there is no deadlock as element do locking
    let mut ui = setup();
    static  mut VISITED: bool = false;
    ui.after_async(Duration::from_millis(10),|ui, _| async move {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.run().await.unwrap();
    unsafe{assert!(VISITED)};
}


#[tokio::test]
#[serial]
async fn test_on_duration() { // These on_xxxx tests that there is no deadlock as element do locking
    let mut ui = setup();
    static  mut VISITED: bool = false;
    ui.periodic(Duration::from_millis(10), |ui, _| {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.run().await.unwrap();
    unsafe{assert!(VISITED)};
}

#[tokio::test]
#[serial]
async fn test_on_duration_async() { // These on_xxxx tests that there is no deadlock as element do locking
    let mut ui = setup();
    static  mut VISITED: bool = false;
    ui.periodic_async(Duration::from_millis(10),|ui, _| async move {
        unsafe {VISITED = true};
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        ui.exit();
    });
    ui.run().await.unwrap();
    unsafe{assert!(VISITED)};
}



#[tokio::test]
#[serial]
async fn test_timer_id() {
    let mut ui = setup();
    static mut TID : gemgui::ui::TimerId = 99;
    let ttid = ui.after(Duration::from_secs(1), |ui: UiRef, id| { 
        unsafe {TID = id;}
        ui.exit();
     });
    ui.run().await.unwrap();
    unsafe {assert_eq!(ttid, TID, "we are testing addition with {} and {}", ttid, TID)};
}


#[tokio::test]
#[serial]
async fn test_timer_id_async() {
    let mut ui = setup();
    static mut TID : gemgui::ui::TimerId = 99;
    let ttid = ui.after_async(Duration::from_secs(1), |ui: UiRef, id| async move { 
        unsafe {TID = id;} // we KNOW that after wont exit current block
        ui.exit();
     }); // << -- boxed!
    ui.run().await.unwrap();
    unsafe {assert_eq!(ttid, TID,"we are testing addition with {} and {}", ttid, TID)};
}

#[tokio::test]
#[serial]
async fn test_counter() {
    let mut ui = setup();
    static mut COUNTER: i32 = 0;
    ui.periodic(Duration::from_millis(5), |ui: UiRef, tid| { 
        unsafe {
            COUNTER += 1; // we KNOW that after wont exit current block
            if COUNTER == 100  {
                ui.exit();
                ui.cancel_timer(tid).unwrap(); // exit tells UI to stop, hence timer may still loop
            }
        }
     });
    let start = Instant::now(); 
    ui.run().await.unwrap();
    let duration = start.elapsed();
    unsafe{assert_eq!(COUNTER, 100)};
    assert!(duration >= Duration::from_millis(500), "duration expected be at least 500ms, it is {}ms", duration.as_millis());
}


#[tokio::test]
#[serial]
async fn test_counter_async() {
    let mut ui = setup();
    static mut COUNTER: i32 = 0;
    ui.periodic_async(Duration::from_millis(5), |ui: UiRef, tid| async move { 
        unsafe {
            COUNTER += 1; // we KNOW that after wont exit current block
            if COUNTER == 100  {
                ui.exit();
                ui.cancel_timer(tid).unwrap();  // This actually wont help as with async - there can be already more requests issued
            }
        }
     });
    let start = Instant::now(); 
    ui.run().await.unwrap();
    let duration = start.elapsed();
    unsafe{assert!(COUNTER >= 100)}; // see comment above and test_counter()
    assert!(duration >= Duration::from_millis(500), "duration expected be at least 500ms, it is {}ms", duration.as_millis());
}


 
#[tokio::test]
#[serial]
async fn test_html() {
    let mut ui = setup();
    let cat = gemgui::Value::new(String::from("cat"));
    let assign =  {
        let cat = cat.clone();
        |ui_ref: UiRef, _| async move { 
        let content = ui_ref.element("content");
        let txt = content.html().await.unwrap();
        cat.assign(txt);
        ui_ref.exit();
        }
    };
    ui.after_async(Duration::from_secs(1), assign);
    ui.run().await.unwrap();
    assert_eq!("Lorem ipsum, vino veritas", cat.cloned().as_str().trim());
}

 
#[tokio::test]
#[serial]
async fn test_timer_cancel() {
    let mut ui = setup();
    let ttid = ui.after(Duration::from_secs(1), |_, _| { 
        panic!("Wrong timer");
     });
     ui.on_start(move |ui: UiRef| { 
        ui.cancel_timer(ttid).unwrap();
        ui.exit();
     });
    ui.run().await.unwrap();
}

 
#[tokio::test]
#[serial]
async fn test_timer_cancel_other() {
    let mut ui = setup();
    let ttid = ui.after(Duration::from_secs(1), |_, _| { 
        panic!("Wrong timer");
     });
     ui.after(Duration::from_millis(10), move |ui, _| { 
        ui.cancel_timer(ttid).unwrap();
        ui.exit();
     });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_periodic_cancel() {
    let mut ui = setup();
    let ttid = ui.periodic(Duration::from_micros(1), |_, _| { 
        panic!("Wrong timer");
     });
     ui.on_start(move |ui: UiRef| {
        println!("on_start!"); 
        ui.cancel_timer(ttid).unwrap();
        ui.exit();
     });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_periodic_cancel_async() {
    let mut ui = setup();
    let ttid = ui.periodic_async(Duration::from_micros(1), |_, _| async move { 
        panic!("Wrong timer");
     });
     ui.on_start(move |ui: UiRef| {
        println!("on_start!"); 
        ui.cancel_timer(ttid).unwrap();
        ui.exit();
     });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_periodic_cancel_other() {
    let mut ui = setup();
    static mut COUNTER : i32 = 0;
    let ttid = ui.periodic(Duration::from_millis(1), move |_, _| { 
        unsafe {
            COUNTER += 1;
            assert!(COUNTER <= 15, "Wrong timer - {} >= 15 - too much off", &COUNTER);
        }
     });
     ui.after(Duration::from_millis(9), move |ui, _| { 
        ui.cancel_timer(ttid).unwrap();
     });
     ui.after(Duration::from_millis(100), move |ui, _| { 
        ui.exit();
     });
    ui.run().await.unwrap();
    unsafe {assert!(COUNTER > 5 && COUNTER < 15) };
}

#[tokio::test]
#[serial]
async fn test_nested_events() {
    let mut ui = setup();
     ui.on_start(move |ui: UiRef| { 
        ui.after(Duration::from_millis(500), |ui, _| {
            let comp = ui.element("content"); 
            comp.subscribe("test_event", |ui: UiRef, _| { // actually monitors mutations of content
                ui.exit();
            });
         });
         ui.after(Duration::from_secs(1), |ui, _| {
            let comp = ui.element("content"); 
            comp.set_style("color", "green");
         }); 
     });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_unsubscribe() {
    let mut ui = setup();
    let comp = ui.element("content"); 
    comp.subscribe("test_event", |_, _| panic!("No here!")); // actually monitors mutations of content
    ui.after(Duration::from_secs(1), |ui, _| ui.exit());
    ui.after(Duration::from_millis(500), |ui, _| {
        let comp = ui.element("content"); 
        comp.set_style("color", "green");
     });  
    ui.on_start(move |ui: UiRef| ui.element("content").unsubscribe("test_event"));
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_nested_events_async() {
    let mut ui = setup();
     ui.on_start(move |ui: UiRef| { 
        ui.after_async(Duration::from_millis(500), |ui, _| async move {
            let comp = ui.element("content"); 
            comp.subscribe_async("test_event", |ui: UiRef, _| async move { // actually monitors mutations of content
                ui.exit();
            });
         });
         ui.after(Duration::from_secs(1), |ui, _| {
            let comp = ui.element("content"); 
            comp.set_style("color", "green");
         }); 
     });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_subscribe() {
    let mut ui = setup();
    let comp = ui.element("content"); 
    comp.subscribe_properties("test_event", |ui, event| {
        let prop = event.property_str("class").unwrap();
        assert_eq!(prop, "some_class");
        ui.exit();
    }, &vec!("class")); // actually monitors mutations of content
    ui.on_start(|ui| ui.element("content").set_style("color", "green"));  
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_set_style() {
    let mut ui = setup();
    let comp = ui.element("content"); 
    comp.set_style("color", "cyan");
    ui.after_async(Duration::from_millis(200), |ui,_| async move {
        let comp = ui.element("content"); 
        let styles = comp.styles(&["color"]).await.unwrap();
        assert!(styles.contains_key("color"));
        println!("I get {}", styles["color"]);
        assert_eq!(name_to_rgb(&style_to_name(&styles["color"]).unwrap()).unwrap(), name_to_rgb("cyan").unwrap());
        ui.exit();
    });
    ui.run().await.unwrap();
}

/*
    Not working - C++ test was faulty - TODO if needed
#[tokio::test]
#[serial]
async fn test_remove_style() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let el = ui.element("some_tests_header");
        let styles = el.styles(&["color"]).await.unwrap();
        assert!(styles.contains_key("color"));
        assert_eq!(name_to_rgb(style_to_name(&styles["color"]).unwrap()).unwrap(), name_to_rgb("red").unwrap());
        el.remove_style("color");
        let styles = el.styles(&["color"]).await.unwrap();
        assert!(styles.contains_key("color"));
        assert_ne!(name_to_rgb(style_to_name(&styles["color"]).unwrap()).unwrap(), name_to_rgb("red").unwrap());
        ui.exit();
    });
    ui.run().await.unwrap();
}
*/

#[tokio::test]
#[serial]
async fn test_attributes() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let el = ui.element("some_tests_header");
        let attrs = el.attributes().await.unwrap();
        assert!(attrs.contains_key("name"));
        assert_eq!(attrs["name"], "some_name");
        el.set_attribute("name", "boing");
        let attrs = el.attributes().await.unwrap();
        assert!(attrs.contains_key("name"));
        assert_eq!(attrs["name"], "boing");
        el.remove_attribute("name");
        let attrs = el.attributes().await.unwrap();
        assert!(!attrs.contains_key("name"));
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_children() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let el = ui.element("another_content");
        let children = el.children().await.unwrap();
        assert_eq!(children.len(), 2);
        children.iter().find(|&e| {e.id() == "paramount"}).unwrap();
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_remove() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let p = ui.element("paramount");
        p.remove();
        let el = ui.element("another_content");
        let children = el.children().await.unwrap();
        assert_eq!(children.len(), 1);
        assert_ne!(children[0].id(), "paramount");
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_remove_parent() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let el = ui.element("another_content");
        el.remove();
        let children = el.children().await;
        assert!(children.is_err());
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_type() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let el = ui.element("another_content");
        let element_type = el.element_type().await.unwrap();
        assert_eq!(element_type, "div");
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_rect() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let el = ui.element("another_content");
        let rect = el.rect::<f32>().await.unwrap();
        // not sure if these value works on 4K, 8K displays
        assert!(rect.x() > 0. && rect.x() < 1500.);
        assert!(rect.y() > 0. && rect.y() < 1500.);
        assert!(rect.width() > 0. && rect.width() < 1500.);
        assert!(rect.height() > 0. && rect.height() < 1500.);
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_root() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let el = ui.root();
        let children = el.children().await.unwrap();
        children.iter().find(|&e| {e.id() == "some_tests_header"}).unwrap();
        children.iter().find(|&e| {e.id() == "content"}).unwrap();
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_by_class() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let children = ui.by_class("some_class").await.unwrap();
        children.iter().find(|&e| {e.id() == "content"}).unwrap();
        children.iter().find(|&e| {e.id() == "another_content"}).unwrap();
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_by_class_none() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let children = ui.by_class("no_class").await.unwrap();
        assert!(children.is_empty());
        ui.exit();
    });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_by_name() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let children = ui.by_name("some_name").await.unwrap();
        children.iter().find(|&e| {e.id() == "some_tests_header"}).unwrap();
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_by_name_none() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let children = ui.by_name("no_name").await.unwrap();
        assert!(children.is_empty());
        ui.exit();
    });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_eval() {
    let mut ui = setup();
    ui.eval("document.write('<h3 id=\\\"foo\\\">Bar</h3>')");
    ui.on_start_async(|ui| async move {
        assert_eq!(ui.element("foo").html().await.unwrap(), "Bar");
        ui.exit();
    });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_on_error_string() {
    let mut ui = setup();
    static mut HAS_ERROR: bool = false;
    ui.on_error(|_, err| {
        unsafe {HAS_ERROR = true}
        assert!(err.contains("fii_foo"));
    });
    ui.on_start(|ui| {
        ui.eval(r"fii_foo();");
    });
    ui.after(Duration::from_millis(700), |ui, _| {
        ui.exit();
    });
    ui.run().await.unwrap();
    unsafe {assert!(HAS_ERROR)}
}

/* 

Wont reload

#[tokio::test]
#[serial]
async fn test_on_reload() {
    let mut ui = setup();
    static mut RELOADED: bool = false;
    ui.on_reload(|_| {
        unsafe {RELOADED = true}
    });
    ui.on_start(|ui| {
        ui.eval(r"window.location.reload(true);");
    });
    ui.after(Duration::from_millis(2000), |ui, _| {
        ui.exit();
    });
    ui.run().await.unwrap();
    unsafe {assert!(RELOADED)}
}
*/

#[tokio::test]
#[serial]
async fn test_set_logging() {
    let mut ui = setup();
    ui.on_error(|_, err| {
        panic!("failed {}", err);
    });
    ui.set_logging(true);
    ui.set_logging(false);
    ui.on_start(|ui| {
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_debug() {
    let mut ui = setup();
    ui.on_error(|_, err| {
        panic!("failed {}", err);
    });
    ui.debug("miao");
    ui.on_start(|ui| {
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_alert() {
    let mut ui = setup();
    ui.on_error(|_, err| {
        panic!("failed {}", err);
    });
    ui.alert("wof");
    ui.on_start(|ui| {
        ui.exit();
    });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_open() {
    let mut ui = setup();
    ui.on_error(|_, err| {
        panic!("failed {}", err);
    });
    ui.open("https://www.google.com", Target::Blank);
    ui.on_start(|ui| {
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_device_pixel_ratio() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let px = ui.device_pixel_ratio().await.unwrap();
        assert!(px == 1.0 || px == 2.0, "value is {}", px); 
        ui.exit();
    });
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_batch() {
    let mut ui = setup();
    static mut VISITED : bool = false;
    static mut BATCHED : bool = false;
    static mut VISITS : i32 = 0;
    ui.on_start(|ui| {
        unsafe{VISITS +=1};
        ui.batch_begin();
        ui.exit();
        unsafe{assert!(!BATCHED)};
        unsafe {BATCHED = true}
    });
    ui.after(Duration::from_millis(50), |_, _| {
        unsafe{VISITS +=1};
        unsafe{assert!(!VISITED)};
        unsafe{assert!(BATCHED)};
        unsafe {VISITED = true} // this tries to prove that if batch ongoing, exit is captured until end
       
    });
    ui.after(Duration::from_millis(1000), |ui, _| {
        unsafe{VISITS +=1};
        unsafe{assert!(BATCHED)};
        unsafe{assert!(VISITED)}
        ui.batch_end();
        
    });
    ui.run().await.unwrap();
    unsafe{assert!(VISITS == 3, "{VISITS} != 3")};
    unsafe{assert!(BATCHED)};
    unsafe{assert!(VISITED)}
}

#[tokio::test]
#[serial]
async fn test_resource() {
    let mut ui = setup();
    ui.on_start(|ui| {
        let html = ui.resource("tests.html").unwrap();
        let html = std::str::from_utf8(&html).unwrap();
        assert!(html.starts_with("<!doctype html>"));
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn add_element() {
    let mut ui = setup();
    ui.on_start_async(move |ui| async move {
        let el = ui.add_element("P", &ui.root()).unwrap();
        assert!(ui.exists(el.id()).await.unwrap());
        let id = el.id().clone();
        assert!(id != "");
        assert_eq!(ui.element(&id).element_type().await.unwrap(), "p");    
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn add_element_with_id() {
    let mut ui = setup();
    ui.on_start_async(move |ui| async move {
        let el = ui.add_element_with_id("some_random_id", "br", &ui.root()).unwrap();
        assert_eq!(el.id(), "some_random_id");
        assert!(ui.exists(el.id()).await.unwrap());
        assert_eq!(ui.element("some_random_id").element_type().await.unwrap(), "br");    
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn add_resource() {
    let ui = setup();
    let path = Path::new("./tests/ext/walruses.jpeg");
    assert_eq!("walruses.jpeg", ui.add_resource(&path).unwrap());
    assert!(ui.resource("walruses.jpeg").is_some());
    // add again is possible - name changes as follows
    assert_eq!("walruses.1.jpeg", ui.add_resource(&path).unwrap());
    assert!(ui.resource("walruses.1.jpeg").is_some());
}

#[tokio::test]
#[serial]
async fn no_exists() {
    let mut ui = setup();
    ui.on_start_async(move |ui| async move {
        assert!(!ui.exists("some_random_id").await.unwrap());
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn exists() {
    let mut ui = setup();
    ui.on_start_async(move |ui| async move {
        let has_it = ui.exists("some_tests_header").await.unwrap();
        assert!(has_it);
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn dynamic_exits() {
    let mut ui = setup();
    ui.set_logging(true);
    ui.on_start_async(move |ui| async move {
        assert!(!ui.exists("some_random_id").await.unwrap());
        let el = ui.add_element_with_id("some_random_id", "br", &ui.root()).unwrap();
        assert_eq!(el.id(), "some_random_id");
        assert!(ui.exists(el.id()).await.unwrap());
        tokio::time::sleep(Duration::from_millis(1000)).await;
        let has_it = ui.exists("some_random_id").await.unwrap();
        assert!(has_it);
        ui.exit();
    });
    ui.run().await.unwrap();
}

