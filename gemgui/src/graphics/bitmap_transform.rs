
use crate::GemGuiError;
use crate::Result;

use super::{bitmap::{Bytes, BitmapData}};
use super::color::TRANSPARENT;
use super::color as Color;

use Color::Pixel as BitmapPixel;

static BITMAP_MAX: i32 = crate::graphics::bitmap::BITMAP_MAX as i32;

/// Basic bitmap transformations
#[derive(Clone)]
pub struct BitmapTransform {
    width: i32,
    height: i32,
    saved_width: i32,
    saved_height: i32,
    data: Bytes,
    saved: Bytes,
}

impl BitmapTransform {

    /// Bitmap width
    pub fn width(&self) -> u32 {
        self.width as u32
    }

    /// Bitmap height
    pub fn height(&self) -> u32 {
        self.height as u32
    }

    /// Readonly bytes
    pub fn data(&self) -> &[BitmapPixel] {
        &self.data
    }

    fn alloc(sz: i32) -> Bytes {
        let sz = sz as usize;
        vec![0; sz]
    }

    /// New bitmap from BitmapData
    /// 
    /// # Arguments
    ///
    /// `bitmap` - data copied
    /// 
    /// # Return
    /// 
    /// New BitmapTransform
    ///  
    pub fn new(bitmap: &dyn BitmapData) -> BitmapTransform {
        let mut data = Bytes::with_capacity((bitmap.width() * bitmap.height()) as usize);
        for p in bitmap.iter() {
            data.push(*p);
        }
        BitmapTransform {
            width: bitmap.width() as i32,
            height: bitmap.height() as i32,
            saved_width: bitmap.width() as i32,
            saved_height: bitmap.height() as i32,
            data: data.clone(),
            saved: data,
        }
    }      
     

/// Change image dimensions extending or cropping by value.
/// Positive pads and negative crops by given amount of pixels.
/// 
/// # Arguments
/// 
/// `width` - new width
/// 
/// `height` - new height
/// 
/// `color` - padding pixel
pub fn enlarge(&mut self, width: i32, height: i32, color: BitmapPixel) -> Result<()> {
    let new_width = self.width + width * 2;
    let new_height = self.height + height * 2;

    if new_height > BITMAP_MAX || new_width > BITMAP_MAX || new_height <= 0 || new_width < 0 {
        return GemGuiError::error("Bad size");
    }

    let mut temp = Self::alloc(new_width * new_height); 

    let mut index = 0;
    for y in 0..new_height {
        for x in 0..new_width {
            let old_x = x - width;
            let old_y = y - height;

            if old_x < 0 || old_x >= self.width || old_y < 0 || old_y >= self.height {
                    temp[index] = color;
                    index += 1;
            } else {
                let old_index = (old_y * self.width + old_x) as usize;
                temp[index] = self.data[old_index];
                index += 1;
            }
        }
    }

    self.width = new_width;
    self.height = new_height;

    self.data = temp;

    Ok(())
}

/// Take a snapshot of current state
pub fn store(&mut self) {
    self.saved = self.data.clone();
    self.saved_width = self.width;
    self.saved_height = self.height;
}

/// Restore from a snapshot, if `store` is not called, the initial state is used.
pub fn restore(&mut self) {
    self.data = self.saved.clone();
    self.width = self.saved_width;
    self.height = self.saved_height;
}

/// Rotates the image 
/// Image dimensions are not changed.
/// 
/// # Argument
/// 
/// `angle` - counter clock wise rotation
pub fn rotate(&mut self, angle: f64) {

    let sin_a = angle.sin();
    let cos_a = angle.cos();

    let half_w = self.width / 2;
    let half_h = self.height / 2;

    let mut temp: Bytes = Self::alloc(self.data.len() as i32);

    let mut index = 0;
    for y  in 0..self.height {
        let  y_sin =  sin_a * ((y - half_h) as f64);
        let y_cos = cos_a * ((y - half_h) as f64);
        for x in 0..self.width {
            let new_x = (cos_a * ((x - half_w) as f64) - y_sin).round() as i32 + half_w - 1; // why this 1 makes round go right?
            let new_y = (sin_a * ((x - half_w) as f64) + y_cos).round() as i32 + half_h;
            let new_index = (new_y * self.width + new_x) as usize;

            if new_x < 0 || new_x >= self.width || new_y < 0 || new_y >= self.height {
                temp[index] = self.data[index];
                }
            else {
                temp[index] = self.data[new_index];
                }
            index += 1;    
        }
    }
    self.data = temp;
}


/// Scales the image  by x_factor and y_factor
/// 
/// # Argument
/// 
/// `x_factor`
/// 
/// `y_factor`
pub fn scale(&mut self, x_factor: f64, y_factor: f64) -> Result<()>{
    assert!(x_factor > 0. && y_factor > 0.);
    let new_width =  (self.width as f64 * x_factor).ceil() as i32;
    let new_height = (self.height as f64 * y_factor).ceil() as i32;

    if new_width > BITMAP_MAX || new_height > BITMAP_MAX || new_width <= 0 || new_height < 0 {
        return GemGuiError::error("Bad size");
    }

    let vec_sz = (new_width * new_height) as usize;
    let mut temp: Bytes = Self::alloc(new_width * new_height);

    let mut index = 0;
    for y in 0..new_height {
        for x in 0..new_width {
            let old_x = ((x as f64) / x_factor) as i32;
            let old_y = ((y as f64) / y_factor) as i32;
            let old_index = (old_y * self.width + old_x) as usize;
            debug_assert!(index < vec_sz);
            debug_assert!(old_index < (self.width * self.height) as usize);
            temp[index] = self.data[old_index];
            index += 1;
        }
    }

    self.width = new_width;
    self.height = new_height;

    self.data = temp;
    Ok(())
}


/// Resize and center of given size
/// 
/// # Argument
/// 
/// `width` - new size
/// 
/// `height` - new size
pub fn center(&mut self, width: u32, height: u32) -> Result<()>{
    self.enlarge(
        ((width as i32 - self.width) as  f64 / 2.0).ceil() as i32,
        ((height as i32 - self.height) as f64 / 2.0).ceil() as i32,
        TRANSPARENT)
}


/// Resize image inside the given extents, i.e. keep proportions
/// 
/// # Argument
/// 
/// `width` - maximum width
/// 
/// `height` - maximum height
/// 
pub fn resize_in(&mut self, width: u32, height: u32) -> Result<()>{
    self.resize_with(self.width(), self.height(), width, height, true)
}

/// Resize image out from the given extents, i.e. keep proportions
/// 
/// # Argument
/// 
/// `width`- minimum width
/// 
/// `height` - minimum height
pub fn resize_out(&mut self, width: u32, height: u32) ->Result<()> {
    self.resize_with(self.width(), self.height(), width, height, false)
}

// could be public if needed, but parameter are probably too tricky
fn resize_with(&mut self, w: u32, h: u32, window_width: u32, window_height: u32, inside: bool) ->Result<()> {
    let xs = (window_width as f64) / (w as f64);
    let ys = (window_height as f64) / (h as f64);
    let s = if inside {xs.min(ys)} else {xs.max(ys)};
    self.scale(s, s)
}

/// Resize image to the given size
/// 
/// # Argument
/// 
/// `width` - new width
/// 
/// `height` - new height
pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
    let xs = (width as f64) / (self.width as f64);
    let ys = (height as f64) / (self.height as f64);
    self.scale(xs, ys)
}

}

impl BitmapData for BitmapTransform {
    fn iter(&self) ->  std::slice::Iter<'_, BitmapPixel> {
    self.data.iter()
}

fn width(&self) -> u32 {
    self.width as u32
}

fn height(&self) -> u32 {
    self.height as u32
}

fn get(&self, x: u32, y: u32) -> BitmapPixel {
    self.data[(y  * self.width() + x) as usize] 
}

fn slice(&self, x: u32, width: u32, y: u32) -> &[BitmapPixel] {
    let begin = y * self.width() + x;
    let end = begin + width;
    &self.data[begin as usize..end as usize]
}

}

#[cfg(test)] 
mod tests {

    #[allow(unused)]
    use std::f64::consts::PI;
    #[allow(unused)]
    use crate::graphics::color::{self as Color, RED, BLUE, BLACK, Pixel};
    use crate::graphics::bitmap::Bitmap;

    use super::*;    

    #[test]
    fn test_transform() {
        let b = Bitmap::rect(20, 10, BLACK);
        let t = BitmapTransform::new(&b);
        assert_eq!(t.width(), 20);
        assert_eq!(t.height(), 10); 
        assert_eq!(b.width(), 20);
        assert_eq!(b.height(), 10);

        let r = Bitmap::rect(20, 10, RED);
        let mut b = Bitmap::rect(20, 20, BLUE);
        b.merge(0, 5, &r);
        assert!(test_pixels(&b, 0, 0, 20, 5, BLUE));
        assert!(test_pixels(&b, 0, 5, 20, 10, RED));
        assert!(test_pixels(&b, 0, 15, 20, 5, BLUE));
        let t = BitmapTransform::new(&b);
        assert!(test_pixels(&t, 0, 0, 20, 5, BLUE));
        assert!(test_pixels(&t, 0, 5, 20, 10, RED));
        assert!(test_pixels(&t, 0, 15, 20, 5, BLUE));

    }


    fn test_pixels(bmp: &dyn BitmapData, x: u32, y: u32, width: u32, height: u32, col: Pixel) -> bool{
        for xx in x..(x + width) {
            for yy in y..(y + height) {
                if bmp.get(xx, yy) !=  col {
                    Bitmap::from(bmp).save("image.png").unwrap();
                    eprintln!("mismatch {}x{} -> {:x} != {:x}", xx, yy,  bmp.get(xx, yy), col);
                    return false;
                }
            }
        }
        return true;
    }

    #[test]
    fn test_transform_rot() {
        let r = Bitmap::rect(200, 100, RED);
        let mut b = Bitmap::rect(200, 200, BLUE);
        b.merge(0, 50, &r);
        let mut t = BitmapTransform::new(&b);
        t.rotate(PI / 2.0);
        assert_eq!(t.width(), 200);
        assert_eq!(t.height(), 200); 
        assert_eq!(b.width(), 200);
        assert_eq!(b.height(), 200);
        assert!(test_pixels(&t, 0, 0, 50, 200, BLUE));
        assert!(test_pixels(&t, 50, 0, 100, 200, RED));
        assert!(test_pixels(&t, 150, 0, 50, 200, BLUE));
        t.rotate(PI / 2.0);
        assert_eq!(t.width(), 200);
        assert_eq!(t.height(), 200);
        assert!(test_pixels(&t, 0, 0, 100, 50, BLUE));
        assert!(test_pixels(&t, 0, 50, 100, 100, RED));
        assert!(test_pixels(&t, 0, 150, 100, 50, BLUE));
        t.rotate(PI / 2.0);
        assert_eq!(t.width(), 200);
        assert_eq!(t.height(), 200);
        assert!(test_pixels(&t, 0, 0, 50, 200, BLUE));
        assert!(test_pixels(&t, 50, 0, 100, 200, RED));
        assert!(test_pixels(&t, 150, 0, 50, 200, BLUE)); 
        t.restore();
        assert_eq!(t.width(), 200);
        assert_eq!(t.height(), 200);
        assert!(test_pixels(&t, 0, 0, 100, 50, BLUE));
        assert!(test_pixels(&t, 0, 50, 100, 100, RED));
        assert!(test_pixels(&t, 0, 150, 100, 50, BLUE));

    }

    #[test]
    fn test_scale() {
        let r = Bitmap::rect(20, 10, RED);
        let mut b = Bitmap::rect(20, 20, BLUE);
        b.merge(0, 5, &r);
        let mut t = BitmapTransform::new(&b);
        t.scale(2.0, 2.0).unwrap();
        assert_eq!(t.width(), 40);
        assert_eq!(t.height(), 40);
        assert!(test_pixels(&t, 0, 0, 20, 10, BLUE));
        assert!(test_pixels(&t, 0, 10, 20, 20, RED));
        assert!(test_pixels(&t, 0, 30, 20, 10, BLUE));
        t.restore();
        assert_eq!(t.width(), 20);
        assert_eq!(t.height(), 20);
        assert!(test_pixels(&t, 0, 0, 10, 5, BLUE));
        assert!(test_pixels(&t, 0, 5, 10, 10, RED));
        assert!(test_pixels(&t, 0, 15, 10, 5, BLUE));
        t.scale(0.5, 0.5).unwrap();
        assert_eq!(t.width(), 10);
        assert_eq!(t.height(), 10);
        assert!(test_pixels(&t, 0, 0, 10, 2, BLUE));
        assert!(test_pixels(&t, 0, 3, 10, 5, RED));
        assert!(test_pixels(&t, 0, 8, 10, 2, BLUE));
    }

    #[test]
    fn test_resize() {
        let r = Bitmap::rect(20, 10, RED);
        let mut b = Bitmap::rect(20, 20, BLUE);
        b.merge(0, 5, &r);
        let mut t = BitmapTransform::new(&b);
        t.resize(40, 40).unwrap();
        assert_eq!(t.width(), 40);
        assert_eq!(t.height(), 40);
        assert!(test_pixels(&t, 0, 0, 40, 10, BLUE));
        assert!(test_pixels(&t, 0, 10, 40, 20, RED));
        assert!(test_pixels(&t, 0, 30, 40, 10, BLUE));
        t.restore();
        assert_eq!(t.width(), 20);
        assert_eq!(t.height(), 20);
        assert!(test_pixels(&t, 0, 0, 20, 5, BLUE));
        assert!(test_pixels(&t, 0, 5, 20, 10, RED));
        assert!(test_pixels(&t, 0, 15, 20, 5, BLUE));
        t.resize(10, 10).unwrap();
        assert_eq!(t.width(), 10);
        assert_eq!(t.height(), 10);
        assert!(test_pixels(&t, 0, 0, 10, 2, BLUE));
        assert!(test_pixels(&t, 0, 3, 10, 5, RED));
        assert!(test_pixels(&t, 0, 8, 10, 2, BLUE));


    }
}