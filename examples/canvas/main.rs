use gemgui::graphics::canvas::Canvas;
use gemgui::graphics::color as Color;
use gemgui::graphics::context::{Context2D, TextAlign};
use gemgui::ui::Ui;
use gemgui::ui_ref::UiRef;
use gemgui::{self, GemGuiError, Rect};

use std::f32::consts::PI;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn amain(ui: UiRef) {
    let canvas = Canvas::from(ui.element("canvas"));
    canvas.draw_image("image", 200, 200, Some( Rect::new(130, 130, 50, 50)));
    canvas.draw_image_rect("image", Rect::new(0, 200, 50, 50), None);
    let mut ctx = Context2D::new();
    
      
    // house
    ctx.line_width(10.)
    .stroke_rect(&Rect::new(75., 140., 150., 110.))
    .fill_rect(&Rect::new(130., 190., 40., 60.))
    .begin_path()
    .move_to(50., 140.)
    .line_to(150., 60.)
    .line_to(250., 140.)
    .close_path()
    .stroke();


    // tilted ellipsoid
    ctx.begin_path()
      .stroke_color(Color::RED)
      .fill_color(Color::YELLOW)
      .ellipse(400., 100., 50., 75., PI / 4., 2. * PI, 0.0)
      .close_path()
      .stroke()
      .fill();
    
     
      // green curve
   ctx.begin_path()
    .stroke_color(Color::GREEN)
    .move_to(500., 500.)
    .bezier_curve_to(100., 520., 550., 550., 70., 580.)
    .stroke(); 

    // purple D
  ctx.line_width(12.)
    .fill_style("#0000EE00")
    .stroke_style("purple")
    .begin_path()
    .move_to(50., 20.)
    .quadratic_curve_to(230., 30., 50., 100.)
    .close_path()
    .stroke()
    .fill();
    
    canvas.draw_context(&ctx);
   
    ctx.clear();


    let x = 300.;
    let y = 300.;
    

    //  "petals"
    let pof = PI / 10.;
    let mut i = 0.0f32;
    loop {
        ctx.begin_path()
        .move_to(x, y)
        .fill_color(Color::rgb(0xFF, 0x8, 0x2))
        .line_to(x + (i + pof).sin() * 100., y + (i + pof).cos() * 100.)
        .arc_to(x + i.sin() * 150., y + i.cos() * 150., x + (i - pof).sin() * 100., y + (i - pof).cos() * 100., pof * 100.)
        .close_path()
        .stroke()
        .fill();
        i += PI / 8f32;
        if i >= 2.0 *  PI {
          break;
        }
    }

    // "pistil"
    ctx.begin_path()
    .line_width(1.)
    .stroke_color(Color::BLUE)
    .fill_color(Color::from_hex("#BB000055").unwrap())
    .arc(x, y, 50., 0., 2. * PI)
    .fill();

    canvas.draw_context(&ctx);

    ctx.clear();
    

    //  alpha arc
    ctx.fill_color(Color::from_hex("#A0CC00F3").unwrap())
    .begin_path()
    .arc(300., 475., 50., 0., 1.5 * PI)
    .close_path()
    .fill();

     
    // text stuff
    ctx.fill_style("lime")
    .save()
    .stroke_style("maroon")
    .fill_style("grey")
    .text_align(TextAlign::Center)
    .font("50px serif")
    .stroke_text("Solenoidal serenity", 250., 510.)
    .fill_text("Solenoidal serenity", 252., 512.)
    .restore()
    .font("20px monospace")
    .fill_text("gemgui", 400., 51.);
    
    canvas.draw_context(&ctx);
    

  }


fn main() -> Result<(), GemGuiError> {
    let fm = gemgui::filemap_from(RESOURCES);
    gemgui::application(fm,
       "index.html",
        gemgui::next_free_port(30000u16),
        |ui| async {amain(ui).await})
    }
    

