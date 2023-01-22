use std::fmt;

use crate::{graphics::{color as Color}, Rect};

/// Context holding draw commands
#[derive(Clone)]
pub struct Context2D {
    commands: Vec<String>,
}

/// Floating point type
pub type Float = f32;

/// TextAlignment
pub enum TextAlign {
     /// Left align data, left justify text.
    Left,
    /// Center align data, center justify text.
    Center,
    /// Right align data, right justify text.
    Right,
    /// Double justify text.
    Justify,
    /// If used, text is aligned around a specific character.
    Char	   
}

impl fmt::Display for TextAlign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Left => write!(f, "left"),
            Self::Center => write!(f, "center"),
            Self::Right => write!(f, "right"),
            Self::Justify => write!(f, "justify"),
            Self::Char => write!(f, "char"),
        }
    }
}

/// Text baseline
pub enum TextBaseLine {
    /// Default. The text baseline is the normal alphabetic baseline.
    Alphabetic,	   
    /// The text baseline is the top of the em square.
    Top,	
    /// The text baseline is the hanging baseline.
    Hanging,
    /// The text baseline is the middle of the em square.
    Middle,	       
    /// The text baseline is the ideographic baseline.
    Ideographic, 	
    /// The text baseline is the bottom of the bounding box.
    Bottom,       
    }

impl fmt::Display for TextBaseLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Alphabetic => write!(f, "alphabetic"),
            Self::Top => write!(f, "top"),
            Self::Hanging => write!(f, "hanging"),
            Self::Middle => write!(f, "middle"),
            Self::Ideographic => write!(f, "ideographic"),
            Self::Bottom => write!(f, "bottom"),
        }
    }
}

macro_rules! push {
    ( $self:ident, $($x:expr ),* ) => {
        {
            $(
                $self.commands.push($x.to_string());
            )*
            return $self
        }
    };
}

impl Default for Context2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Context2D {

    /// Create a new command context
    /// 
    /// # Return
    /// Context2D
    pub fn new() -> Context2D {
        Context2D {commands: Vec::new(),}
    }

    /// Reset this command context
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn stroke_rect<'a>(&'a mut self, rect: &Rect<Float>) -> &'a mut Context2D {
        push!(self, "strokeRect", rect.x, rect.y, rect.width, rect.height);
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn clear_rect<'a>(&'a mut self, rect: &Rect<Float>) -> &'a mut  Context2D {
        push!(self, "clearRect", rect.x, rect.y, rect.width, rect.height);
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn fill_rect<'a>(&'a mut self, rect: &Rect<Float>) -> &'a mut  Context2D {
        push!(self, "fillRect", rect.x, rect.y, rect.width, rect.height);
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn fill_text<'a>(&'a mut self, text: &str, x: Float, y: Float) -> &'a mut  Context2D  {
        push!(self, "fillText", text, x, y);
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn stroke_text<'a>(&'a mut self, text: &str, x: Float, y: Float) -> &'a mut  Context2D {
        push!(self, "strokeText", text, x, y);
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn arc(&'_ mut self, x: Float, y: Float, r: Float, start_angle: Float, end_angle: Float) -> &'_ mut  Context2D {
        push!(self, "arc", x, y, r, start_angle, end_angle)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(&'_ mut self, x: Float, y: Float, radius_x: Float, radius_y: Float, rotation: Float, start_angle: Float, end_angle: Float) -> &'_ mut  Context2D {
        push!(self, "ellipse", x, y, radius_x, radius_y, rotation, start_angle, end_angle)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn begin_path(&'_ mut self) -> &'_ mut  Context2D {
        push!(self, "beginPath")
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn close_path(&'_ mut self) -> &'_ mut  Context2D {
        push!(self, "closePath")
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn line_to(&'_ mut self, x: Float, y: Float) -> &'_ mut  Context2D {
        push!(self, "lineTo", x, y)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn move_to(&'_ mut self, x: Float, y: Float) -> &'_ mut  Context2D {
        push!(self, "moveTo", x, y);
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn bezier_curve_to(&'_ mut self, cp1x: Float, cp1y: Float, cp2x: Float, cp2y: Float, x: Float, y: Float) -> &'_ mut  Context2D {
        push!(self, "bezierCurveTo", cp1x, cp1y, cp2x, cp2y, x,  y)
    } 

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn quadratic_curve_to(&'_ mut self, cpx: Float, cpy: Float, x: Float, y: Float) -> &'_ mut  Context2D {
        push!(self, "quadraticCurveTo", cpx, cpy, x, y)
    }
    
    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn arc_to(&'_ mut self, x1: Float, y1: Float, x2: Float, y2: Float, radius: Float) -> &'_ mut  Context2D {
        push!(self, "arcTo", x1, y1, x2, y2, radius)
    }
   
    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn rect(&'_ mut self, rect: &Rect<Float>) -> &'_ mut  Context2D {
        push!(self, "rect", rect.x, rect.y, rect.width, rect.height)
    } 
    
    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn stroke(&'_ mut self) -> &'_ mut  Context2D {
        push!(self, "stroke")
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn fill(&'_ mut self) -> &'_ mut  Context2D {
        push!(self, "fill")
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn fill_style(&'_ mut self, style: &str) -> &'_ mut  Context2D {
        push!(self, "fillStyle", style)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn fill_color(&'_ mut self, color: Color::Pixel) -> &'_ mut  Context2D {
         push!(self, "fillStyle", Color::to_string(color));
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn stroke_style(&'_ mut self, style: &str) -> &'_ mut  Context2D  {
        push!(self, "strokeStyle", style)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn stroke_color(&'_ mut self, color: Color::Pixel) -> &'_ mut  Context2D {
        push!(self, "strokeStyle", Color::to_string(color))
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn line_width(&'_ mut self, width: Float) -> &'_ mut  Context2D {
        push!(self, "lineWidth", width)
    }
    
    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn font(&'_ mut self, style: &str) -> &'_ mut  Context2D {
        push!(self, "font", style)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn text_align(&'_ mut self, align: TextAlign) -> &'_ mut  Context2D {
        push!(self, "textAlign", align.to_string())
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn save(&'_ mut self) -> &'_ mut  Context2D {
        push!(self, "save")
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn restore(&'_ mut self) -> &'_ mut  Context2D {
        push!(self, "restore")
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn rotate(&'_ mut self, angle: Float) -> &'_ mut  Context2D {
        push!(self, "rotate", angle)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn translate(&'_ mut self, x: Float, y: Float) -> &'_ mut  Context2D {
        push!(self, "translate", x, y)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn scale(&'_ mut self, x: Float, y: Float) -> &'_ mut  Context2D {
        push!(self, "scale", x, y)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn draw_image(&'_ mut self, id: &str, x: Float, y: Float) -> &'_ mut  Context2D  {
        push!(self, "drawImage", id, x, y)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn draw_image_rect(&'_ mut self, id: &str, rect: &Rect<Float>) -> &'_ mut  Context2D {
        push!(self, "drawImageRect", id, rect.x, rect.y, rect.width, rect.height)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn draw_image_clip(&'_ mut self, id: &str, clip: &Rect<Float>, rect: &Rect<Float>) -> &'_ mut  Context2D {
        push!(self, "drawImageClip", id, clip.x, clip.y, clip.width, clip.height, rect.x, rect.y, rect.width, rect.height)
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub fn text_baseline(&'_ mut self, base_line: TextBaseLine) -> &'_ mut  Context2D
    {
        push!(self, "textBaseline", base_line.to_string())
    }

    /// <https://www.w3schools.com/graphics/canvas_reference.asp>
    pub (crate) fn composed(&self) -> &Vec<String> {
        &self.commands
    }

   
}

#[cfg(test)]
mod tests {

    use super::*;    

    #[test]
    fn test_data() { 
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
        let content = ctx.composed();
        let count = content.iter().filter(|s| s.contains("font")).count();
        assert_eq!(count, 1);
        let count = content.iter().filter(|s| s.contains("scale")).count();
        assert_eq!(count, 1);
        let count = content.iter().filter(|s| s.contains("drawImage")).count();
        assert_eq!(count, 3);

    }


    }