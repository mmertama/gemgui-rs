use std::{io::Cursor, path::Path};

use crate::{graphics::color as Color, GemGuiError, Result, Rect};
use image::{io::Reader, DynamicImage, RgbImage, RgbaImage, Pixel, ImageBuffer, Rgba};

use super::color::{rgba, rgb};

pub (crate) static BITMAP_MAX: u32 = 16000;

pub (crate) type Bytes = Vec<Color::Pixel>;

/// Trait for bitmap data
pub trait BitmapData {
    ///  Iterate over pixels 
    fn iter(&self) -> std::slice::Iter<'_, Color::Pixel>;
    /// Bitmap width
    fn width(&self) -> u32;
    /// Bitmap height
    fn height(&self) -> u32;
    /// Get a single pixel
    fn get(&self, x: u32, y: u32) -> Color::Pixel;
    /// get a many pixels
    fn slice(&self, x: u32, width: u32, y: u32) -> &[Color::Pixel];
}

/// Byte owning bitmap
#[derive(Clone)]
pub struct Bitmap {
    width: u32,
    height: u32,
    bytes: Bytes,
}

///Byte borrowing bitmap
#[derive(Clone)]
pub struct BitmapRef<'a> {
    width: u32,
    height: u32,
    bytes: &'a Bytes,
}

impl Bitmap {
    /// Allocate a new bitmap
    ///  
    /// The bitmap bytes are transparent
    /// 
    /// # Arguments
    /// 
    /// * `width` - bitmap width
    /// * `height` - bitmap height
    /// 
    /// # Return
    /// 
    /// Bitmap 
    pub fn new(width: u32, height: u32) -> Bitmap {
        assert!(width < BITMAP_MAX && height < BITMAP_MAX);
        let bytes= Self::alloc((width * height) as i32); 
        Bitmap {
            width,
            height,
            bytes,
        }
    } 

    fn alloc(sz: i32) -> Bytes {
        let sz = sz as usize;
        let mut data: Vec<u32> = Vec::with_capacity(sz);
        data.resize(sz, Color::TRANSPARENT);
        data
    }

    fn from_rgba(image: &RgbaImage) -> Bitmap {
        let mut data = Self::alloc((image.width() * image.height()) as i32); 
        let mut ii = 0;
        for y in 0..image.height() {
            for x in 0..image.width() {
                    let pixel = image.get_pixel(x, y).channels();
                    data[ii]= rgba(pixel[0], pixel[1], pixel[2], pixel[3]);
                    ii += 1; 
            }
        }
        Bitmap { 
            width: image.width(),
            height: image.height(),
            bytes: data,
        }
    }

    fn from_rgb(image: &RgbImage) -> Bitmap {
        let mut data = Self::alloc((image.width() * image.height()) as i32); 
        let mut ii = 0;

        for y in 0..image.height() {
            for x in 0..image.width() {
                    let pixel = image.get_pixel(x, y).channels();         
                    data[ii] = rgb(pixel[0], pixel[1], pixel[2]);
                    ii += 1; 
            }
        }

        Bitmap {
            width: image.width(),
            height: image.height(),
            bytes: data,
        }
    }

    ///  New bitmap from bytes
    /// 
    /// # Arguments
    /// 
    /// `width` - bitmap width
    /// 
    /// `height` - bitmap height
    /// 
    /// `bytes` - it is assumed that width * height equals to bytes.len()
    /// 
    /// # Return
    /// 
    /// Bitmap  
    pub fn from_bytes(width: u32, height: u32, bytes: Bytes) -> Bitmap {
        assert!(width < BITMAP_MAX && height < BITMAP_MAX);
        assert!(width * height == bytes.len() as u32, "{} {} != {}", width, height, bytes.len());
        Bitmap {
            width,
            height,
            bytes,
        }
    }

    fn from_image(image: DynamicImage) -> Result<Bitmap> {
        let result = image.as_rgba8();
        match result {
            Some(im) => Ok(Self::from_rgba(im)),
            None => {
                let result = image.as_rgb8();
                match result {
                    Some(im) => Ok(Self::from_rgb(im)),
                    None => Err(GemGuiError::Err("Bad image".to_string())),
                }
            }
        }
    }

    ///  New bitmap from file
    ///  
    /// # Arguments
    /// 
    /// `filename` - a path to image file, various formats are supported
    /// 
    /// # Return
    /// 
    /// Bitmap  
    pub fn from_image_file(filename: &str) -> Result<Bitmap> {
        match Reader::open(filename) {
            Ok(reader) => {
                match reader.decode() {
                    Ok(image)  => Self::from_image(image),
                    Err(e) => Err(GemGuiError::Err(format!("Bad format, {}", e))),
                }
            },
            Err(e) => Err(GemGuiError::Err(format!("Image file not found: {}, {}", filename, e))),
        }
    }

    ///  New bitmap from a file
    ///  
    /// # Arguments
    /// 
    /// `bytes` - a image file bytes, various formats are supported
    /// 
    /// # Return
    /// 
    /// Bitmap  
    pub fn from_image_bytes(bytes: &[u8]) -> Result<Bitmap> {
        let bytes = Cursor::new(bytes);
        match Reader::new(bytes).with_guessed_format() {
            Ok(reader) => {
                match reader.decode() {
                    Ok(image)  => Self::from_image(image),
                    Err(e) => Err(GemGuiError::Err(format!("Bad format, {}", e))),
                }
            },
            Err(e) =>   Err(GemGuiError::Err(format!("Expected an image format, {:#?}", e))),
        }
    }

    /// Clone a bitmap from another
    /// 
    /// # Arguments
    /// 
    /// `bitmap` - a bitmap data copied
    /// 
    /// # Return
    /// 
    /// Bitmap  
    pub fn from(bitmap: &dyn BitmapData) -> Bitmap {
        let mut new_bitmap = Bitmap::new(bitmap.width(), bitmap.height());
        for y in 0..new_bitmap.height {
            for x in 0..new_bitmap.width {    
            new_bitmap.put(x,  y,  bitmap.get(x, y));
            }
        }
        new_bitmap
    }

    /// Clone a portion of bitmap to another
    ///
    /// Returned bitmap is can be smaller than a given rect is if clip is outsize of bitmap. 
    /// 
    /// # Arguments
    /// 
    /// `rect` - portion extents
    /// 
    /// `bitmap` - Bitmap copied
    /// 
    /// # Return
    /// 
    /// Bitmap  
    pub fn clip(rect: &Rect<u32>, bitmap: &dyn BitmapData) -> Result<Bitmap> {
        let rect = Rect::new(
            rect.x(),
            rect.y(),
            rect.width().min(bitmap.width() - rect.x()),
            rect.height().min(bitmap.height() - rect.y()));
        if rect.width() == 0 || rect.height() == 0 {
            return GemGuiError::error("Invalid size");
        }    
        let width = rect.width.min(bitmap.width() - rect.x);
        let height = rect.height.min(bitmap.height() - rect.y);
        let mut bytes = Bytes::with_capacity((width *  height) as usize);
        for row in rect.y..(rect.y + height) {
            let stride =  bitmap.slice(rect.x, width, row);
            for p in stride.iter() {
                bytes.push(*p);
            }
        }
        assert_eq!(width * height, bytes.len() as u32);
        Ok(Bitmap { width, height, bytes }) 
    }

    /// Create a solid color bitmap
    /// 
    /// # Arguments
    /// 
    /// `width` - bitmap width
    /// 
    /// `height` - bitmap height
    /// 
    /// `pixel`- color applied
    ///
    /// # Return
    /// 
    /// Bitmap  
    pub fn rect(width: u32, height: u32, pixel: Color::Pixel) -> Bitmap {
        assert!(width < BITMAP_MAX && height < BITMAP_MAX);
        let sz = (width * height) as usize;
        let mut bytes: Vec<u32> = Vec::with_capacity(sz);
        bytes.resize(sz, pixel);
        Bitmap::from_bytes(width, height, bytes)
    }

    /// Set a pixel color
    /// 
    /// x and y coordinates are assumed to be inside of bitmap
    /// 
    /// # Arguments
    /// 
    /// `x` - x coordinate
    /// 
    /// `y` - y coordinate
    /// 
    /// `pixel`- color applied
    pub fn put(&mut self, x: u32, y: u32, pixel: Color::Pixel) {
        debug_assert!(x < self.width && y < self.height && y * self.width + x < self.bytes.len() as u32, "{}x{} out from {}x{}", x, y, self.width, self.height);
        self.bytes[(y * self.width + x) as usize] = pixel; 
    }

    /// Merge another bitmap on this bitmap
    /// 
    /// Pixels are either replaced or meld upon the pixel alpha channel.
    /// 
    /// x_pos, y_pos or source bitmap extents does not have to be inside of this bitmap.
    /// 
    /// # Arguments
    /// 
    /// `x_pos` - x coordinate
    /// 
    /// `y_pos` - y coordinate
    /// 
    /// `bitmap`- source bitmap
    pub fn merge(&mut self, x_pos: i32, y_pos: i32, bitmap: &dyn BitmapData) {
      
        let mut width = bitmap.width();
        let mut height = bitmap.height();
        
        if x_pos >= self.width() as i32 || (x_pos + bitmap.width() as i32) < 0 {
            return;
        } 

        if y_pos >= self.height() as i32 || (y_pos + bitmap.height() as i32) < 0 {
            return;
        }
        
        let x: u32;
        let y: u32;
        let b_x: u32;
        let b_y: u32;

        if x_pos < 0 {              // if -10 and width 100
            x = 0;                  // set 0  
            b_x = (-x_pos) as u32;// (-10) => 10
            width -= b_x;           // 100 + (-10) => 90 
        } else {
            x = x_pos as u32;
        }

        if y_pos < 0 {
            y = 0;
            b_y = (-y_pos) as u32;
            height -= b_y;
        } else {
            y = y_pos as u32;
        }

        if  x + width >= self.width {
            width = self.width - x;
        }

        if  y + height >= self.height {
            height = self.height - y;
        }

        debug_assert!(width <= self.width);
        debug_assert!(height <= self.height);
        debug_assert!(width <= bitmap.width());
        debug_assert!(height <= bitmap.height());
    
        for j in 0..height {
            for i in 0..width {
                assert!(x + i < self.width);
                assert!(y + j < self.height);
                let p = self.get(x + i, y + j);
                assert!(i < bitmap.width());
                assert!(j < bitmap.height());
                let po = bitmap.get(i, j);

                let ao = Color::alpha(po);
                let a = Color::alpha(p);
                let r = Color::red(p) * (0xFF - ao);
                let g = Color::green(p) * (0xFF - ao);
                let b = Color::blue(p) * (0xFF - ao);

                let ro = Color::red(po) * ao;
                let go = Color::green(po) * ao;
                let bo = Color::blue(po) * ao;

                let pixel = Color::rgba_clamped(
                    (r + ro) / 0xFF,
                    (g + go) / 0xFF,
                    (b + bo) / 0xFF,
                    a as u32);

                self.put(x + i, y + j, pixel);
            }
        }
    }

    /// Save bitmap to a file
    /// 
    /// # Arguments
    /// 
    /// `path` - file save, the format is deducted from the file extension (e.g. .jpg or png) 
    /// - various format are supported.
    pub fn save<FileName>(&self, filename: FileName) -> Result<()> 
    where FileName: AsRef<Path> {
        let mut rgba: RgbaImage = ImageBuffer::new(self.width as u32, self.height as u32);
        for x in 0..self.width {
            for y in 0..self.height {
                let p = self.get(x, y);
                rgba.put_pixel(x as u32, y as u32, Rgba::from([
                    Color::r(p),
                    Color::g(p),
                    Color::b(p),
                    Color::a(p)]));
            }
        }
        match rgba.save(filename) {
            Ok(_) => Ok(()),
            Err(e) => Err(GemGuiError::Err(format!("{}", e)))
        }
    }

}

impl BitmapData for Bitmap {
        fn iter(&self) ->  std::slice::Iter<'_, Color::Pixel> {
        self.bytes.iter()
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
    
    fn get(&self, x: u32, y: u32) -> Color::Pixel {
        self.bytes[(y  * self.width + x) as usize] 
    }
    
    fn slice(&self, x: u32, width: u32, y: u32) -> &[Color::Pixel] {
        let begin = y * self.width + x;
        let end = begin + width;
        &self.bytes[begin as usize..end as usize]
    }
}

impl BitmapRef<'_> {
    /// New Bitmap from the bytes
    /// 
    /// # Arguments
    /// 
    /// `width` - bitmap width.
    /// 
    /// `height` - bitmap height.
    /// 
    /// `bytes` - bytes that are borrowed for a bitmap.
    pub fn from_bytes(width: u32, height: u32, bytes: &Bytes) -> BitmapRef {
        assert!(width < BITMAP_MAX && height < BITMAP_MAX);
        assert!(width * height == bytes.len() as u32);
        BitmapRef {
            width,
            height,
            bytes,
        }
    }
}

impl BitmapData for BitmapRef<'_> {
    fn iter(&self) ->  std::slice::Iter<'_, Color::Pixel> {
        self.bytes.iter()
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn get(&self, x: u32, y: u32) -> Color::Pixel {
        self.bytes[(y * self.width + x) as usize] 
    }

    fn slice(&self, x: u32, width: u32, y: u32) -> &[Color::Pixel] {
        let begin = y * self.width + x;
        let end = begin + width;
        &self.bytes[begin as usize..end as usize]
    }

}

#[cfg(test)] 
mod tests {

    use super::*;    

    #[test]
    fn test_bitmap() {
        let mut bmp1 = Bitmap::new(10, 10);
        let mut vec: Vec<Color::Pixel> = Vec::with_capacity(10 * 10);
        vec.resize(10 * 10, 0);
        let mut p: Color::Pixel = 0; 
        for i in vec.iter_mut() {
            *i = p;
            p += 1;
        }
        let bmp2 = Bitmap::from_bytes(10, 10, vec);
        for y in 0..10 {
            for x in 0..10 {
                bmp1.put(x, y, bmp2.get(x, y));
            }
        }
    
        let mut p: Color::Pixel = 0;
        for i in bmp1.iter() {
            assert!(*i == p); // test equality of data with same method as set another
            p += 1;
        }
        assert_eq!(p, bmp1.width * bmp1.height());
    
        p = 0;
        for row in 0..bmp1.height() {
            let stride = bmp1.slice(0, bmp1.height(), row);
            for i in stride.iter() {
                assert!(*i == p); // test equality of data with same method as set another
                p += 1;
            }
        }
        assert_eq!(p, bmp1.width * bmp1.height());
    }

    #[test]
    fn  test_clip() {
        let red = Color::rgb(0xFF, 0, 0);
        let bmp = Bitmap::rect(100, 100, red);
        let  bmp1 = Bitmap::clip(&Rect::new(0, 0, 10, 10), &bmp).unwrap();
        assert_eq!(bmp1.width(), 10);
        assert_eq!(bmp1.height(), 10);
        for row in 0..10 {
            for col in 0..10 {
                assert_eq!(bmp1.get(row, col), red);
            }
        }
        let  bmp2 = Bitmap::clip(&Rect::new(95, 95, 10, 10), &bmp).unwrap();
        assert_eq!(bmp2.width(), 5);
        assert_eq!(bmp2.height(), 5);
        for row in 0..5 {
            for col in 0..5 {
                assert_eq!(bmp2.get(row, col), red);
            }
        }
    }

    #[test]
    fn  test_merge() {
        let red_color = Color::rgb(0xFF, 0, 0);
        let red = Bitmap::rect(100, 100, red_color);
        let blue_color = Color::rgb(0, 0x0, 0xFF);
        let blue = Bitmap::rect(100, 100, blue_color);
        let mut bmp = Bitmap::from(&red);
        bmp.merge(0, 0, &blue);
        for row in 0..100 {
            for col in 0..100 {
                assert_eq!(bmp.get(row, col), blue_color, "{:x} vs {:x}", bmp.get(row, col), blue_color);
            }
        }
        let blue = Bitmap::rect(10, 10, blue_color);
        let mut bmp = Bitmap::from(&red);
        bmp.merge(10, 10, &blue);
        for row in 0..100 {
            for col in 0..100 {
                if row >= 10 && row < 20 && col >= 10 && col < 20 {
                    assert_eq!(bmp.get(row, col), blue_color, "{:x} vs {:x}", bmp.get(row, col), blue_color);
                } else {
                    assert_eq!(bmp.get(row, col), red_color, "{:x} vs {:x} {} {}", bmp.get(row, col), red_color, row, col);
                }
            }
        }

        let mut bmp = Bitmap::from(&red);
        bmp.merge(-5, -5, &blue);
        for row in 0..100 {
            for col in 0..100 {
                if row < 5 && col < 5 {
                    assert_eq!(bmp.get(row, col), blue_color, "{:x} vs {:x} {} {}", bmp.get(row, col), blue_color, row, col);
                } else {
                    assert_eq!(bmp.get(row, col), red_color, "{:x} vs {:x} {} {}", bmp.get(row, col), red_color, row, col);
                }
            }
        }

        let mut bmp = Bitmap::from(&red);
        bmp.merge(95, 95, &blue);
        for row in 0..100 {
            for col in 0..100 {
                if row >= 95 && col >= 95 {
                    assert_eq!(bmp.get(row, col), blue_color, "{:x} vs {:x}", bmp.get(row, col), blue_color);
                } else {
                    assert_eq!(bmp.get(row, col), red_color, "{:x} vs {:x} {} {}", bmp.get(row, col), red_color, row, col);
                }
            }
        }

        let mut bmp = Bitmap::from(&red);
        bmp.merge(101, 101, &blue);
        for row in 0..100 {
            for col in 0..100 {
                if row >= 95 && col >= 95 {
                    assert_eq!(bmp.get(row, col), red_color, "{:x} vs {:x} {} {}", bmp.get(row, col), red_color, row, col);
                }
            }
        }

        let mut bmp = Bitmap::from(&red);
        bmp.merge(100, 100, &blue);
        for row in 0..100 {
            for col in 0..100 {
                if row >= 95 && col >= 95 {
                    assert_eq!(bmp.get(row, col), red_color, "{:x} vs {:x} {} {}", bmp.get(row, col), red_color, row, col);
                }
            }
        }

        let mut bmp = Bitmap::from(&red);
        bmp.merge(-11, -11, &blue);
        for row in 0..100 {
            for col in 0..100 {
                if row >= 95 && col >= 95 {
                    assert_eq!(bmp.get(row, col), red_color, "{:x} vs {:x} {} {}", bmp.get(row, col), red_color, row, col);
                }
            }
        }

        let mut bmp = Bitmap::from(&red);
        bmp.merge(-10, -10, &blue);
        for row in 0..100 {
            for col in 0..100 {
                if row >= 95 && col >= 95 {
                    assert_eq!(bmp.get(row, col), red_color, "{:x} vs {:x} {} {}", bmp.get(row, col), red_color, row, col);
                }
            }
        }

    }
}
