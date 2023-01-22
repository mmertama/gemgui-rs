use std::{ops::Deref, collections::HashMap};

use byteorder::{WriteBytesExt, LittleEndian};
use crate::{graphics::{color as Color}, ui_data::UiData, Rect, Result, GemGuiError};
use futures::Future;
use crate::{element::Element, ui_ref::UiRef, JSMessageTx, graphics::bitmap::BitmapData};


// this value depends if websocket payload size
// in theory is big one (i.e. huge 2^63), but in practice it seems to be 
// capped. 64 / 63 was ok - but we try bigger values 
static TILE_WIDTH: u32 = 640; 
static TILE_HEIGHT: u32 = 640; 

static CANVAS_ID: u32 = 0xAAA;
use LittleEndian as Endianness;

use super::context::Context2D;

/// Canvas is-an Element
#[derive(Clone)]
pub struct Canvas {
    canvas: Element,
}


fn align(a: u32) -> u32{
    (a + 3u32) & !3u32
}

fn write_prelude(vec8: &mut Vec<u8>, data_type: u32, owner: &str, sz: u32, header: &Vec<u32>) {
        vec8.write_u32::<Endianness>(data_type).unwrap();
        vec8.write_u32::<Endianness>(sz as u32).unwrap();
        vec8.write_u32::<Endianness>(align(owner.len() as u32)).unwrap();
        vec8.write_u32::<Endianness>(align(header.len() as u32)).unwrap();
}

fn write_epilog(vec8: &mut Vec<u8>, owner: &str, header: &Vec<u32>)  {      
        
    for header_item in header {
            vec8.write_u32::<Endianness>(*header_item).unwrap();
        }

    assert!(!owner.is_empty());
    for byte in owner.chars() {
            vec8.write_u16::<Endianness>(byte as u16).unwrap();
        }

    let padding = align(owner.len() as u32) - owner.len() as u32;
    for _i in 0..padding {
        vec8.write_u16::<Endianness>(0).unwrap();
    }     
}



// Considered as a anti pattern, but so handy to avoid tedious copy-paste code
// Rust may have a good way to do this, but for this case Deref works fine and
// is easy(?) to understand. You can always use "fn element()" instead 
impl Deref for Canvas {
    type Target = Element;
    fn deref(&self) -> &Element {
        &self.canvas
    }
}

/// For on_draw
pub enum DrawNotify {
    /// Call on_start immediately without a draw.
    Kick,
    /// Call on_start only after a draw.
    NoKick,
}

impl Canvas {
    /// A new canvas cloned from an element
    /// 
    /// It is assumed that element's  HTML element type is <canvas>
    /// but that is not ensured here. If not, results are
    /// unpredictable 
    /// 
    /// # Arguments
    /// 
    /// `element` - cloned 
    /// 
    /// # Return
    /// 
    /// new Canvas 
    pub fn new(element: &Element) -> Canvas {   
        Canvas{
            canvas: element.clone(),
        }
    }

    /// A new canvas from an element
    ///
    /// It is assumed that element's  HTML element type is <canvas>
    /// but that is not ensured here. If not, results are
    /// unpredictable 
    ///
    /// # Arguments
    /// 
    /// `element` - consumed Element 
    ///
    /// # Return
    /// 
    /// new Canvas  
    pub fn from(element: Element) -> Canvas {   
        Canvas{
            canvas: element,
        }
    }

    /// Reference to this as Element
    /// 
    /// # Return
    /// 
    /// This canvas's Element
    /// 
    pub fn element(&self) -> &Element {
        &self.canvas   
    }

    /// Reference to this as mut Element
    /// 
    /// # Return
    ///
    /// This canvas's Element 
    pub fn element_mut(&mut self) -> &mut Element {
        &mut self.canvas   
    }

    /// Cancels on draw callbacks 
    pub fn on_draw_cancel(&self) {
        self.element().unsubscribe("event_notify");
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "event_notify",
            name: Some("canvas_draw"),
            add: Some(false), 
            ..Default::default()
        };
        self.element().send(msg);
    }
    
    fn init_on_draw(&self, kick: DrawNotify) {
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "event_notify",
            name: Some("canvas_draw"),
            add: Some(true), 
            ..Default::default()
        };
        self.element().send(msg);
        match kick  {
            DrawNotify::Kick => {
                let mut prop = HashMap::new();
                prop.insert("name".to_string(), "canvas_draw".to_string());
            self.element().call("event_notify", prop);
            },
            DrawNotify::NoKick => (),
        }    
    }

    /// Set on draw callback
    /// 
    /// See [on_draw](Self::on_draw)
    pub fn on_draw_async<CB, Fut>(&self, draw_completed_cb: CB, kick: DrawNotify) 
        where CB: FnOnce(UiRef)  ->Fut + Send + Clone + 'static,
        Fut: Future<Output =  ()> + Send +  'static {
        //let interval = Arc::new(Mutex::new(Instant::now()));        
        self.element().subscribe_async("event_notify", |ui, ev| async move {
            if let Some(prop) = ev.property_str("name") {
                if prop == "canvas_draw" {
                    //let duration = Self::elapsed(interval.clone());
                    draw_completed_cb(ui).await;
                    //Self::set_now(interval.clone());
                }
            }
        });
        self.init_on_draw(kick);
    }


    /// Set on draw callback
    /// 
    /// Given callback is called immediately after UI has drawn the UI and hence enables synchronized draw.
    /// 
    /// # Arguments
    /// 
    /// `draw_completed` - callback to be called once draw has been completed
    /// 
    /// `kick` - make the callback called immediately without a draw 
    /// (so all draw can be initiated in/via the callback)
    /// 
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// 
    pub fn on_draw<CB>(&self, mut draw_completed_cb: CB, kick: DrawNotify)
    where CB: FnMut(UiRef) + Send + 'static { // FnOnce or FnMut (or Fn ????)
        //let interval = Arc::new(Mutex::new(Instant::now()));    
        self.element().subscribe("event_notify", move |ui, ev| {
            if let Some(prop) = ev.property_str("name") {
                if prop == "canvas_draw" {
                    //let duration = Self::elapsed(interval.clone());
                    draw_completed_cb(ui);
                    //Self::set_now(interval.clone()); 
                }
            }
        });
        self.init_on_draw(kick);
    }


    /// Draws a given bitmap
    /// 
    /// Draws to top-left corner
    /// draw_callback is called if enabled
    /// 
    /// # Arguments
    /// 
    /// `bitmap` - to be drawn.
    pub fn draw_bitmap(&self, bitmap: &dyn BitmapData) {
        self.paint_bitmap(0, 0, bitmap, true).unwrap()
    }

    /// Draws a given bitmap to the coordinates
    /// 
    /// draw_callback is called if enabled
    /// 
    /// # Arguments
    /// 
    /// `x` - x coordinate, does not have to be in canvas
    /// 
    /// `y` - y coordinate, does not have to be in canvas
    /// 
    /// `bitmap` - to be drawn
    pub fn draw_bitmap_at(&self,  x: i32, y: i32, bitmap: &dyn BitmapData) -> Result<()> {
        self.paint_bitmap(x, y, bitmap, true)
    }

    /// Draws a given bitmap to the coordinates
    /// 
    /// draw_callback is NOT called even enabled
    /// 
    /// # Arguments
    /// 
    /// `x` - x coordinate, does not have to be in canvas
    /// 
    /// `y` - y coordinate, does not have to be in canvas
    /// 
    /// `bitmap` - to be drawn
    pub fn paint(&self, x: i32, y: i32, bitmap: &dyn BitmapData) -> Result<()> {
        self.paint_bitmap(x, y, bitmap, false)
    }

    fn paint_bitmap(&self, x: i32, y: i32, bitmap: &dyn BitmapData, as_draw: bool) -> Result<()> {
        if (y + (bitmap.height() as i32) <= 0) || (x + (bitmap.width() as i32) <= 0) {
            return GemGuiError::error(&format!("Invalid bitmap rect x:{} y:{} width:{} height:{}", x, y, bitmap.width(), bitmap.height()));
        }
        
        let height = if y < 0 {(bitmap.height() as i32 + y) as u32 } else {bitmap.height()};
        let y = y.max(0) as u32;
        let width = if x < 0 {(bitmap.width() as i32 + x) as u32 } else {bitmap.width()};
        let x = x.max(0) as u32;

        let mut is_last = false;

        for j in (y..height).step_by(TILE_HEIGHT as usize) {
            let height = TILE_HEIGHT.min(bitmap.height() - j);
            if height == 0 {
                continue;
            }
            for i in (x..width).step_by(TILE_WIDTH as usize) {
                let width = TILE_WIDTH.min(bitmap.width() - i);
                debug_assert!(!is_last);
                is_last = (bitmap.height() - j < TILE_HEIGHT) && (bitmap.width() - i < TILE_WIDTH);
                let header = vec!(
                    i as u32,
                    j as u32,
                    width as u32,
                    height as u32,
                    // figure out if this is the last one if set notification
                    (as_draw && is_last) as u32
                );
                let mut vec8: Vec<u8> = vec![];
                write_prelude(
                    &mut vec8,
                    CANVAS_ID,
                    self.id(),
                    width * height,
                    &header
                    );    
                
                for row in j..(j + height) {
                    let stride = bitmap.slice(i, width, row);
                    for elem in stride.iter() {
                       let p = Color::rgba(
                        Color::r(*elem),
                        Color::g(*elem),
                        Color::b(*elem),
                        Color::a(*elem));
                       vec8.write_u32::<Endianness>(p).unwrap();
                    }
                }
                
                write_epilog(
                    &mut vec8,
                    self.id(),
                    &header,
                    );
               

                assert!(vec8.len() as u32 == (width * height + 4) * 4 + 20 + 16, "w:{} h:{} len:{}", i, j, vec8.len()); 
                self.element().send_bin(vec8);
            }
        }
        
        if ! is_last {
            let header = vec!(
                0,
                0,
                width as u32,
                height as u32,
                true as u32
            );
            let mut vec8: Vec<u8> = vec![];
            write_prelude(
                &mut vec8,
                CANVAS_ID,
                self.id(),
                0,
                &header
                );
            write_epilog(
                &mut vec8,
                self.id(),
                &header,
                );
            self.element().send_bin(vec8);            
        }
        

        Ok(()) 
    } 

     

    // Add an image to canvas for drawing
    /// 
    /// See (add_image) [Self::add_image]
    pub async fn add_image_async<OptCB, CB, Fut>(&self, url: &str, image_added_cb: OptCB) -> Result<String>
        where OptCB: Into<Option<CB>>,
        CB: FnOnce(UiRef, String)-> Fut + Send + Clone + 'static,
        Fut:  Future<Output = ()> + Send + 'static {
            let cb = image_added_cb.into();
            match cb {
                Some(cf) =>  self.add_image(url, UiData::as_sync_fn(cf)).await,
                _ => self.add_image(url, |_,_|{}).await
            }
        }

    
    
    /// Add an image to canvas for drawing
    /// 
    /// # Arguments
    /// 
    /// `url` - image URL, can be a filemap or external image
    /// 
    /// `image_added_cb` - Optional callback called when image is ready for draw
    /// 
    /// # Callback
    /// 
    /// `UiRef`- Reference to UI
    /// 
    /// `String` - Image id
    /// 
    /// # Return
    /// 
    /// Image id
    pub async fn add_image<OptCB, CB>(&self, url: &str, image_added_cb: OptCB) -> Result<String>
        where
        CB: FnMut(UiRef, String) + Send + 'static,
        OptCB: Into<Option<CB>> {  // FnOnce or FnMut (or Fn ????)  {
            let name = UiData::random("image");
            let id_name = name.clone();
            let image_element = 
            self.ui().add_element_with_id(&name, "IMG", self).await?;
            let cb = image_added_cb.into();
            if cb.is_some() {
                let mut f = cb.unwrap();
                image_element.subscribe("load", move |ui, _| f(ui, name.clone()));
            }
            image_element.set_attribute("style", "display:none");
            image_element.set_attribute("src", url);
            Ok(id_name)
        }

    /// Draws an image
    /// 
    /// # Arguments
    /// 
    /// `id` - Image id e.g. <IMG ID="some_id"> or a String get from add_image
    /// 
    /// `x` - x coordinate, does not have to be in canvas
    /// 
    /// `y` - y coordinate, does not have to be in canvas
    /// 
    /// `clipping_rect` - Optional clipping rect.
    pub fn draw_image<ORect>(&self, image_id: &str, x: i32, y: i32, clipping_rect: ORect)
    where ORect: Into<Option<Rect<i32>>> {
        self.paint_image_internal(image_id, Some((x, y)), None, clipping_rect.into())
    } 

    /// Draws an image rectangle
    /// 
    /// # Arguments
    /// 
    /// `id` - Image id e.g. <IMG ID="some_id"> or a String get from add_image
    /// 
    /// `rect` - Rect in target image where image is drawn - a scaling may occur
    /// 
    /// `clipping_rect` - Optional clipping rect.
    pub fn draw_image_rect<ORect>(&self, image_id: &str, rect: Rect<i32>, clipping_rect: ORect)
    where ORect: Into<Option<Rect<i32>>> {
        if rect.width <= 0 || rect.height <= 0 {
            return;
        }
        self.paint_image_internal(image_id, None, Some(rect), clipping_rect.into())
    } 

    fn paint_image_internal(&self, image_id: &str, pos: Option<(i32, i32)>, target_rect: Option<Rect<i32>>, clipping_rect: Option<Rect<i32>>) {
        let clipping_rect = clipping_rect.map(|rect| 
            vec!(rect.x as f32, rect.y as f32, rect.width as f32, rect.height as f32));
        let target_rect = target_rect.map(|rect|
            vec!(rect.x as f32, rect.y as f32, rect.width as f32, rect.height as f32));
        let pos = pos.map(|pp| vec!(pp.0 as f32, pp.1 as f32));

        let msg =  JSMessageTx {
        element: self.id(),
        _type: "paint_image",
        image: Some(image_id),
        rect: target_rect,
        clip: clipping_rect,
        pos, 
        ..Default::default()
    };
        self.send(msg);
    }

    /// Draw context
    /// 
    /// # Arguments
    /// 
    /// `context` - context drawn
    pub fn draw_context(&self, context: &Context2D) {
        let commands = context.composed();
        if commands.is_empty() {
            return;
        }
        let msg =  JSMessageTx {
            element: self.id(),
            _type: "canvas_draw",
            commands: Some(commands),
            ..Default::default()
        };
        self.send(msg);
    }
    
    /// Erase an area from Canvas
    /// 
    /// # Arguments
    /// 
    /// `rect` - erase area
    pub fn erase(&self, rect: Rect<usize>) {
        let f_rect = Rect::<f32>{
            x: rect.x as f32,
            y:  rect.y as f32,
            width: rect.width as f32,
            height: rect.height as f32};
        self.draw_context(Context2D::new().clear_rect(&f_rect));
    }

}


#[cfg(test)]
mod tests {

    use std::io::Cursor;

    use byteorder::ReadBytesExt;

    use crate::graphics::bitmap::{Bitmap, BitmapRef};

    use super::*;    

    #[test]
    fn test_data() {
        let mut bitmap = Bitmap::new(10, 10);
        bitmap.put(3, 3, 0x1);
        bitmap.put(5, 5, 0x100);
        bitmap.put(9, 9, 0x10000);

        let mut vec8: Vec<u8> = vec![];
        let name = "kissa";
        let header = vec!(0, 0, bitmap.width() as u32, bitmap.height() as u32, true as u32);
        write_prelude (
            &mut vec8,
            CANVAS_ID,
            name,
            bitmap.width() * bitmap.height(),
            &header,
            );

        for elem in bitmap.iter() {
            vec8.write_u32::<Endianness>(*elem).unwrap();
        }

        write_epilog (
            &mut vec8,
            name,
            &header,
            );
        
        let len = vec8.len() as u32;    
        let mut c = Cursor::new(vec8);

        let read32 = |c: &mut Cursor<Vec<u8>>| {c.read_u32::<Endianness>().unwrap()};

        assert_eq!(read32(&mut c), 0xAAA);
        let data_sz = read32(&mut c);
        assert_eq!(data_sz, 100); // w * h
        let id_len = read32(&mut c);
        assert_eq!(id_len, align(name.len() as u32));
        let header_len = read32(&mut c);
        assert_eq!(header_len, align(5)); // header size above 

        let header_offset = (data_sz + 4) * 4;
        assert!(header_offset < len - (4 * 4));

        let data_offset = 4 * 4;
        let id_offset = (4 * 4) + data_offset + header_len * 4;
        assert!(id_offset <= len - id_len * 2, "{} <= {} - {} * 2", id_offset, len, id_len);

        assert!(len == (10 * 10 + 4) * 4 + 20 + 16); 

        //data
        let mut pixels = Vec::new();
        for i in 0..align(100) {
            let pix = c.read_u32::<Endianness>();
            assert!(pix.is_ok(), "Error at {} ", i); 
            pixels.push(pix.unwrap());
        }

        let result = BitmapRef::from_bytes(10, 10, &pixels);
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(bitmap.get(x, y), result.get(x, y), "at {}x{}", x, y);
            }
        }

        // header
       
        assert_eq!(read32(&mut c), 0); // header 1
        assert_eq!(read32(&mut c), 0); // header 2
        assert_eq!(read32(&mut c), 10); // header 3
        assert_eq!(read32(&mut c), 10); // header 4
        assert_eq!(read32(&mut c), 1);  // header 5

        // id
        let mut chrs: Vec<char> = Vec::new();
        for i in 0..id_len {
            let ch =   c.read_u16::<Endianness>();
            assert!(ch.is_ok(), "Error at {} of {} - {} vs {:#?}", i, id_len, name, chrs); 
            let c_v = ch.unwrap() as u32;
            if c_v > 0 { // clean off padding
                chrs.push(char::from_u32(c_v).unwrap());
            }
        }
        let name_string: String = chrs.into_iter().collect();
        assert_eq!(name, name_string);
    
    }
}