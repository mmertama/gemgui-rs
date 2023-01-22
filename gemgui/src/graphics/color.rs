use regex::Regex;

/// Red
pub static RED: Pixel =         0xFF0000FF;
/// Green
pub static GREEN: Pixel =       0xFF00FF00;
/// also Green
pub static LIME: Pixel =        GREEN;
/// Blue
pub static BLUE: Pixel =        0xFFFF0000;

/// Yellow
pub static YELLOW: Pixel =      0xFF00FFFF;
/// Magenta
pub static MAGENTA: Pixel =     0xFFFF00FF;

#[allow(dead_code)] // It is used in tests --- but it still gives a warning
/// also Magenta 
static FUCHSIA: Pixel =         MAGENTA;
/// Cyan
pub static CYAN: Pixel =        0xFFFFFF00;
/// also Cyan
pub static AQUA: Pixel =        CYAN;

/// Black
pub static BLACK: Pixel =       0xFF000000;
/// White
pub static WHITE: Pixel =       0xFFFFFFFF;

/// Transparent
pub static TRANSPARENT: Pixel = 0x00000000;

/// Pixel type
pub type Pixel = u32;

/// Translate HTML style rgb name to HTML name
/// 
/// # Arguments
/// 
/// `rgb_string` in rgba(0, 0, 0, 0) or  rgb(0, 0, 0) format - alpha is ignored.
/// 
/// # Return
/// 
/// Color name
pub fn style_to_name(rgb_string: &str) -> Option<String> {
    let re = Regex::new(r"rgba?\((\d+),\s*(\d+),\s*(\d+)(,\s*(\d+))?\)").unwrap();
    let (r, g, b) = match re.captures(rgb_string) {
        Some(cap) => {(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str(), cap.get(3).unwrap().as_str())},
        None => return None,
    };
    pixel_to_name(rgb(
        r.parse::<u8>().unwrap(),
        g.parse::<u8>().unwrap(),
        b.parse::<u8>().unwrap()))
}

/// Translate Pixel name to HTML name
/// 
/// # Arguments
/// 
/// `Pixel` - alpha is ignored.
/// 
/// # Return
/// 
/// Color name
pub fn pixel_to_name(pixel: Pixel) -> Option<String> {
    let value = color_name::Color::name([
        r(pixel),
        g(pixel),
        b(pixel)]);
    if value == "404" {
        return None;
    }
   Some(value.to_lowercase())
}

/// Try to find a name to a color
/// 
/// # Arguments
/// 
/// `name` - color name.
/// 
/// # Return
/// Pixel
pub fn name_to_rgb(name: &str) -> Option<Pixel> {
    let s = name.into();
    match color_name::Color::val().by_string(s) {
        Ok(c) => Some(rgb(c[0], c[1], c[2])),
        Err(_) => None,
    }

}

/// Color from an hexadecimal string
/// 
/// # Arguments
/// 
/// `name` - #AARRGGBB or #RRGGBB.
/// 
/// # Return
/// 
/// Pixel
pub fn from_hex(string: &str) -> Option<Pixel> {
    let re = Regex::new(r"#([0-9a-fA-F][0-9a-fA-F])([0-9a-fA-F][0-9a-fA-F])([0-9a-fA-F][0-9a-fA-F])([0-9a-fA-F][0-9a-fA-F])?").unwrap();
    let cap =  re.captures(string)?;
    if cap.get(4).is_some() {
        Some(rgba(
            u8::from_str_radix(cap.get(2).unwrap().as_str(), 16).unwrap(),
            u8::from_str_radix(cap.get(3).unwrap().as_str(), 16).unwrap(),
            u8::from_str_radix(cap.get(4).unwrap().as_str(), 16).unwrap(),
            u8::from_str_radix(cap.get(1).unwrap().as_str(), 16).unwrap()))
    }  else {
        Some(rgb(
            u8::from_str_radix(cap.get(1).unwrap().as_str(), 16).unwrap(),
            u8::from_str_radix(cap.get(2).unwrap().as_str(), 16).unwrap(),
            u8::from_str_radix(cap.get(3).unwrap().as_str(), 16).unwrap()))
    }
}

/// Color from components
/// 
/// # Arguments
/// 
/// `r` - red
/// 
/// `g` - green
/// 
/// `b` - blue
/// 
/// `a` - alpha
/// 
/// # Return 
/// 
/// Pixel
pub fn rgba_clamped(r: Pixel, g: Pixel, b: Pixel, a: Pixel) -> Pixel {
    (0xFF & r) | ((0xFF & g) << 8) | ((0xFF & b) << 16) | ((0xFF & a) << 24)
}

/// Color from components
/// 
/// # Arguments
/// 
/// `r` - red
/// 
/// `g` - green
/// 
/// `b` - blue
/// 
/// # Return 
/// 
/// Pixel
pub fn rgb_clamped(r: Pixel, g: Pixel, b: Pixel) -> Pixel {
    (0xFF & r) | ((0xFF & g) << 8) | ((0xFF & b) << 16) | (0xFF << 24)
}

/// Color from components
/// 
/// # Arguments
/// 
/// `r` - red
/// 
/// `g` - green
/// 
/// `b` - blue
/// 
/// `a` - alpha
/// 
/// # Return 
/// 
/// Pixel
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Pixel {
    (r as Pixel) | ((g as Pixel) << 8) | ((b as Pixel) << 16) | ((a as Pixel)  << 24)
}

/// Color from components
/// 
/// # Arguments
/// 
/// `r` - red
/// 
/// `g` - green
/// 
/// `b` - blue
/// 
/// # Return 
/// 
/// Pixel
pub fn rgb(r: u8, g: u8, b: u8) -> Pixel {
    (r as Pixel) | ((g as Pixel)  << 8) | ((b as Pixel) << 16) | (0xFF << 24)
}

/// Red Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Red byte
pub fn r(pixel: Pixel) -> u8 {
    (pixel & 0xFF) as u8
}

/// Green Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Green byte
pub fn g(pixel: Pixel) -> u8 {
    ((pixel & 0xFF00) >> 8) as u8
}

/// Blue Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Blue byte
pub fn b(pixel: Pixel) -> u8 {
    ((pixel & 0xFF0000) >> 16) as u8
}

/// Alpha Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Alpha byte
pub fn a(pixel: Pixel) -> u8 {
    ((pixel & 0xFF000000) >> 24) as u8
}

/// Red Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Red Pixel
pub fn red(pixel: Pixel) -> Pixel {
    pixel & 0xFF
}

/// Green Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Green Pixel
pub fn green(pixel: Pixel) -> Pixel {
    (pixel & 0xFF00) >> 8
}

/// Blue Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Blue Pixel
pub fn blue(pixel: Pixel) -> Pixel {
    (pixel & 0xFF0000) >> 16
}

/// Alpha Component
/// 
/// # Arguments
/// 
/// `pixel` 
/// 
/// # Return
/// 
/// Alpha Pixel
pub fn alpha(pixel: Pixel) -> Pixel {
    (pixel & 0xFF000000) >> 24
}

/// Color String representation
/// 
/// # Arguments
/// 
/// `pixel`
/// 
/// # Return
/// 
/// String in #RRGGBBAA or if Alpha is 0xFF then #RRGGBB format
pub fn to_string(pixel: Pixel) -> String {
    if alpha(pixel) == 0xFF {
        format!("#{:02X}{:02X}{:02X}", r(pixel), g(pixel), b(pixel))
    } else {
        to_string_alpha(pixel)
    }
}

/// Color String representation
/// 
/// # Arguments
/// 
/// `pixel`
/// 
/// # Return
/// 
/// String in #RRGGBBAA
pub fn to_string_alpha(pixel: Pixel) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", r(pixel), g(pixel), b(pixel), alpha(pixel))
}

/// Pixel from tuple
/// 
/// # Arguments
/// 
/// `tuple` - (r, g, b)
/// 
/// # Return
/// 
/// Pixel
pub fn rgb_from_tuple(tuple: (u8, u8, u8)) -> Pixel {
    rgb(tuple.0, tuple.1, tuple.2)
}


/// Pixel from tuple
/// 
/// # Arguments
/// 
/// `tuple` - (r, g, b, a)
/// 
/// # Return
/// 
/// Pixel
pub fn rgba_from_tuple(tuple: (u8, u8, u8, u8)) -> Pixel {
    rgba(tuple.0, tuple.1, tuple.2, tuple.3)
}

#[test]
fn test_rgb() {
    let col1 = rgb(0x33, 0x44, 0x55);
    assert_eq!(to_string(col1), "#334455");
    assert_eq!(to_string_alpha(col1), "#334455FF");

    let col2 = rgb(0xAA, 0xBB, 0xCC);
    assert_eq!(to_string(col2), "#AABBCC");
    assert_eq!(to_string_alpha(col2), "#AABBCCFF");

    let col3 = rgb(0, 0, 0xCC);
    assert_eq!(to_string(col3), "#0000CC");
    assert_eq!(to_string_alpha(col3), "#0000CCFF");

    let col4 = rgb(0, 0x3, 0xCC);
    assert_eq!(to_string(col4), "#0003CC");
    assert_eq!(to_string_alpha(col4), "#0003CCFF");

    let col5 = rgba(0x11, 0x22, 0x33, 0xCC);
    assert_eq!(to_string(col5), "#112233CC");
    assert_eq!(to_string_alpha(col5), "#112233CC");

    let col6 = r(col5);
    assert_eq!(col6, 0x11);

    let col7 = g(col5);
    assert_eq!(col7, 0x22);

    let col8 = b(col5);
    assert_eq!(col8, 0x33);

    let col9 = alpha(col5);
    assert_eq!(col9, 0xCC);

    let col10 = rgb_clamped(0x1200, 0x203, 0x3CC);
    assert_eq!(to_string(col10), "#0003CC");
    assert_eq!(to_string_alpha(col10), "#0003CCFF");

    let col11 = rgba_clamped(0x411, 0x222, 0x433, 0x3CC);
    assert_eq!(to_string(col11), "#112233CC");
    assert_eq!(to_string_alpha(col11), "#112233CC");

}

#[cfg(test)] 
mod tests {

    use std::collections::HashMap;

    use super::*;    

    #[test]
    fn test_colors() {
        assert_eq!(RED, rgb(0xFF, 0, 0));
        assert_eq!(RED, rgba(0xFF, 0, 0, 0xFF));
        assert_eq!(GREEN, rgb(0, 0xFF, 0));
        assert_eq!(BLUE, rgb(0, 0, 0xFF));
        assert_eq!(WHITE, rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(BLACK, rgb(0, 0, 0));
    }

    #[test]
    fn test_from_hex_string() {
        assert_eq!(from_hex("#Aa12bB01").unwrap(), rgba(0x12, 0xBB, 0x01, 0xAA), "#Aa12bB01 --> {} != {}", to_string(from_hex("#Aa12bB01").unwrap()),  to_string(rgba(0x12, 0xBB, 0x01, 0xAA)));
        assert_eq!(from_hex("#12bB01").unwrap(), rgb(0x12, 0xBB, 0x01), "12bB01 --> {} != {}", to_string(from_hex("#12bB01").unwrap()),  to_string(rgb(0x12, 0xBB, 0x01)));
    }

    #[test]
    fn test_style_to_name() {
        let mut cols= HashMap::new();
        cols.insert("indianred", "rgb(205, 92, 92)");
        cols.insert("khaki", "rgb(240, 230, 140)");
        cols.insert("lightsalmon", "rgb(255, 160, 122)");
        cols.insert("gold", "rgb(255, 215, 0)");
        cols.insert("fuchsia", "rgb(255, 0, 255)");
        cols.insert("darkorchid", "rgb(153, 50, 204)");
        cols.insert("undefined", "rgb(0, 99, 204)");
        
            
        for (name, col) in cols.iter() {
            let c1 = style_to_name(col).unwrap_or("undefined".to_string());
            assert_eq!(c1, *name, "On {} --> {}",  name, col);
        }

        macro_rules! to_style {
            ($col: ident) => {
                format!("rgb({},{},{})", r($col), g($col), b($col))
            };
        }
        
        assert_eq!(style_to_name(&to_style!(RED)).unwrap(), "red");
        assert_eq!(style_to_name(&to_style!(GREEN)).unwrap(), "lime");
        assert_eq!(style_to_name(&to_style!(BLUE)).unwrap(), "blue");
        assert_eq!(style_to_name(&to_style!(CYAN)).unwrap(), "aqua");
        assert_eq!(style_to_name(&to_style!(MAGENTA)).unwrap(), "fuchsia");
        assert_eq!(style_to_name(&to_style!(YELLOW)).unwrap(), "yellow");
        assert_eq!(style_to_name(&to_style!(BLACK)).unwrap(), "black");
        assert_eq!(style_to_name(&to_style!(WHITE)).unwrap(), "white");


    }

    #[test]
    fn test_rgb_to_name() {

        let mut cols= HashMap::new();
        cols.insert("indianred", "#CD5C5C");
        cols.insert("darkred", "#8B0000");
        cols.insert("deeppink", "#FF1493");
        cols.insert("rebeccapurple", "#663399");
        cols.insert("teal", "#008080");
        cols.insert("darkturquoise", "#00CED1");
        cols.insert("sandybrown", "#F4A460");
        
            
        for (name, col) in cols.iter() {
            let c = from_hex(col).unwrap();
            let c1 = pixel_to_name(c).unwrap_or("undefined".to_string());
            assert_eq!(c1, *name, "On {} --> {}", name, col);
        }
        
        assert_eq!(pixel_to_name(RED).unwrap(), "red");
        assert_eq!(pixel_to_name(GREEN).unwrap(), "lime");
        assert_eq!(pixel_to_name(LIME).unwrap(), "lime");
        assert_eq!(pixel_to_name(BLUE).unwrap(), "blue");
        assert_eq!(pixel_to_name(CYAN).unwrap(), "aqua");
        assert_eq!(pixel_to_name(AQUA).unwrap(), "aqua");
        assert_eq!(pixel_to_name(MAGENTA).unwrap(), "fuchsia");
        assert_eq!(pixel_to_name(FUCHSIA).unwrap(), "fuchsia");
        assert_eq!(pixel_to_name(YELLOW).unwrap(), "yellow");
        assert_eq!(pixel_to_name(YELLOW).unwrap(), "yellow");
        assert_eq!(pixel_to_name(BLACK).unwrap(), "black");
        assert_eq!(pixel_to_name(WHITE).unwrap(), "white");
        
    }
}
    
    
