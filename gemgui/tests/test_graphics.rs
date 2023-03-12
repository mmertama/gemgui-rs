use std::{time::Duration};

use serial_test::serial;
use crate::tests::setup;

use gemgui::{graphics::{canvas::{Canvas, DrawNotify}, context::{TextBaseLine, Context2D, TextAlign}, color as Color, bitmap::Bitmap}, Rect, ui_ref::UiRef, ui::Ui};


#[path="./tests.rs"]
mod tests;

#[allow(unused)]
fn timeout(time: u64) -> tokio::task::JoinHandle<()> {
    let backtrace = std::backtrace::Backtrace::force_capture();
    tokio::task::spawn(async move {
        tokio::time::sleep(Duration::from_millis(time)).await;
        eprintln!("Timeout - backtrace {}", backtrace);
        std::process::exit(62);    
    })
}


#[tokio::test]
#[serial]
async fn test_canvas_from() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let canvas = Canvas::from(ui.element("canvas"));
        assert_eq!(canvas.element().element_type().await.unwrap(), "canvas");
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_canvas_deref() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let canvas = Canvas::from(ui.element("canvas"));
        assert_eq!(canvas.element_type().await.unwrap(), "canvas");
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_canvas_new() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let canvas = Canvas::new(&ui.element("canvas"));
        assert_eq!(canvas.element_type().await.unwrap(), "canvas");
        ui.exit();
    });
    ui.run().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_canvas_on_paint() {
    let mut ui = setup();
    //let t = timeout(100);
    let canvas = Canvas::new(&ui.element("canvas"));
    canvas.on_draw(|ui| {
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        //assert!(interval.as_micros() > 0);
        ui.exit();
    }, DrawNotify::Kick);
    ui.run().await.unwrap();
   //t.abort();
}

 
#[tokio::test]
#[serial]
async fn test_canvas_on_paint_async() {
    let mut ui = setup();
    //let t = timeout(100);
    let canvas = Canvas::new(&ui.element("canvas"));
    canvas.on_draw_async(|ui| async move {
        let comp = ui.element("content"); 
        assert_ne!(comp.id(), ui.root().id());
        //assert!(interval.as_micros() > 0);
        ui.exit();
    }, DrawNotify::Kick);
    ui.run().await.unwrap();
    //t.abort();
}

#[tokio::test]
#[serial]
async fn test_context() {
    let mut ui = setup();
    let image_id = "some_image";
    let rect = Rect::new(0.0, 0.0, 100., 100.);
    let x = 10.0;
    let y = 45.0;
    let mut ctx = Context2D::new();
    ctx.stroke_rect(&rect) 
    .clear_rect(&rect)
    .fill_rect(&rect)  
    .fill_text("foo", x, y)
    .stroke_text("bar", x, y)
    .arc(x, y, 2., 1., 0.)
    .ellipse(x, y, 1., 2., 3., 4., 5.)
    .begin_path()
    .close_path() 
    .line_to(x, y)
    .move_to(x, y)
    .bezier_curve_to(0., 1., 2., 3., 4., 5.)
    .quadratic_curve_to(0., 1., 2., 3.)
    .arc_to(0., 1., 2., 3., 4.)
    .rect(&rect)
    .stroke() 
    .fill() 
    .fill_style("solid")
    .fill_color(Color::RED)
    .stroke_style("line")
    .stroke_color(Color::BLUE) 
    .line_width(1.0)
    .font("serif")
    .text_align(TextAlign::Left) 
    .save()
    .restore()
    .rotate(1.)
    .translate(x, y)
    .scale(x, y)
    .draw_image(image_id, x, y)
    .draw_image_rect(image_id, &rect) 
    .draw_image_clip(image_id, &rect, &rect)
    .text_baseline(TextBaseLine::Bottom);
    let canvas = Canvas::new(&ui.element("canvas"));
    canvas.draw_context(&ctx);
    canvas.on_draw(|ui| ui.exit(), DrawNotify::Kick);
    ui.run().await.unwrap();
}


#[tokio::test]
#[serial]
async fn test_draw_image_attached() {
    let mut ui = setup();
    ui.on_start(|ui|{
        let canvas = Canvas::new(&ui.element("canvas"));
        // his id is in HTML
        canvas.draw_image("hidden_image", 0, 0, None);
        ui.exit();
    });
    ui.run().await.unwrap();
}

 
#[tokio::test]
#[serial]
async fn test_image_external() {
    let mut ui = setup();
    ui.on_start_async(|ui| async move {
        let canvas = Canvas::new(&ui.element("canvas"));
        canvas.add_image("https://picsum.photos/200/300", |ui: UiRef, image: String| {
            let canvas = Canvas::new(&ui.element("canvas"));
            canvas.draw_image(&image, 0, 0, None);
            ui.exit();
        }).unwrap();
    });
    ui.run().await.unwrap();
}



#[tokio::test]
#[serial]
async fn test_image_added() {
    let mut ui = setup();
    let t = timeout(3000);
    ui.on_start_async(|ui| async move {
        let canvas = Canvas::new(&ui.element("canvas"));
        let res = ui.add_resource("./tests/ext/walruses.jpeg").unwrap();
        canvas.add_image(&res, |ui: UiRef, image: String| {
            let canvas = Canvas::new(&ui.element("canvas"));
            canvas.draw_image(&image, 0, 0, None);
            ui.exit();
        }).unwrap();    
    });
    ui.run().await.unwrap();
    t.abort();
}



#[tokio::test]
#[serial]
async fn test_image_resources() {
    let mut ui = setup();
    let t = timeout(3000);
    ui.on_start(|ui| {
        let canvas = Canvas::new(&ui.element("canvas"));
        canvas.add_image("/widgets.jpeg", |ui: UiRef, image: String| {
            let canvas = Canvas::new(&ui.element("canvas"));
            canvas.draw_image(&image, 0, 0, None);
            ui.exit();
        }).unwrap();    
    });
    ui.run().await.unwrap();
    t.abort();
}


async fn bitmap_test(x: i32, y: i32, w: u32, h: u32) {
    let mut ui = setup();
   // let no_call = x > 500 || x + (w as i32) <= 0 || y > 500 || y + (h as i32) <= 0;
    let t = timeout(5000);
   // if no_call {
   //     ui.after(Duration::from_secs(3), |ui, _| ui.exit());
   // }
    let canvas = Canvas::new(&ui.element("canvas"));
    let bmp = Bitmap::rect(w, h, Color::CYAN);
    canvas.on_draw(move |ui| {
   //     assert!(!no_call, "bitmap x:{} y:{} {}x{}", x, y, w, h);
        ui.exit();
    }, DrawNotify::NoKick);
    canvas.draw_bitmap_at(x, y, &bmp);
    ui.run().await.unwrap();
    t.abort();
}
 
#[tokio::test]
#[serial]
async fn test_bitmap_draw1() {
    bitmap_test(0, 0, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw2() {
    bitmap_test(0, 0, 500, 500).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw3() {
    bitmap_test(0, 0, 1000, 1000).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw4() {
    bitmap_test(-10, -10, 100, 100).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw5() {
    bitmap_test(450, 450, 100, 100).await
}


#[tokio::test]
#[serial]
async fn test_bitmap_draw6() {
    bitmap_test(500, 500, 100, 100).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw7() {
    bitmap_test(510, 510, 100, 100).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw8() {
    bitmap_test(10, 510, 100, 100).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw9() {
    bitmap_test(510, 10, 100, 100).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw10() {
    bitmap_test(-10, -10, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw11() {
    bitmap_test(-1, -10, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw12() {
    bitmap_test(-10, -1, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw13() {
    bitmap_test(-5, -5, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw14() {
    bitmap_test(5, -5, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw15() {
    bitmap_test(-5, 5, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw16() {
    bitmap_test(500, 500, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw17() {
    bitmap_test(-5, 500, 10, 10).await
}

#[tokio::test]
#[serial]
async fn test_bitmap_draw18() {
    bitmap_test(500, 5, 10, 10).await
}
