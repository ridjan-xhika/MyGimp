use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixels: Vec<u8>,
    pub dirty: bool,
    pub loaded_image_size: Option<(u32, u32)>, // Track size of loaded image for panning
    pub loaded_image_data: Option<Vec<u8>>, // Store loaded image for re-panning
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let stride = aligned_stride(width);
        let pixels = vec![255; stride * height as usize];
        Self {
            width,
            height,
            stride,
            pixels,
            dirty: true,
            loaded_image_size: None,
            loaded_image_data: None,
        }
    }

    /// Load pixels from a tight-packed RGBA buffer (width*height*4)
    /// and expand them to canvas stride
    pub fn load_pixels(&mut self, width: u32, height: u32, tight_pixels: Vec<u8>) {
        if width != self.width || height != self.height {
            return;
        }
        
        let tight_stride = width as usize * 4;
        for y in 0..height as usize {
            let tight_offset = y * tight_stride;
            let canvas_offset = y * self.stride;
            
            if tight_offset + tight_stride <= tight_pixels.len() 
                && canvas_offset + tight_stride <= self.pixels.len() {
                self.pixels[canvas_offset..canvas_offset + tight_stride]
                    .copy_from_slice(&tight_pixels[tight_offset..tight_offset + tight_stride]);
            }
        }
        self.dirty = true;
    }

    /// Extract tight-packed RGBA pixels (without stride padding) for saving
    pub fn extract_tight_pixels(&self) -> Vec<u8> {
        let row_bytes = self.width as usize * 4;
        let mut tight_pixels = Vec::with_capacity(row_bytes * self.height as usize);
        
        for y in 0..self.height as usize {
            let row_offset = y * self.stride;
            if row_offset + row_bytes <= self.pixels.len() {
                tight_pixels.extend_from_slice(&self.pixels[row_offset..row_offset + row_bytes]);
            }
        }
        
        tight_pixels
    }

    /// Paste an image onto the canvas with offset (for panning large images)
    pub fn paste_image_with_offset(&mut self, img_width: u32, img_height: u32, img_pixels: &[u8], offset_x: i32, offset_y: i32) {
        self.loaded_image_size = Some((img_width, img_height));
        
        let img_stride = img_width as usize * 4;
        
        for canvas_y in 0..self.height {
            let img_y = canvas_y as i32 - offset_y;
            if img_y < 0 || img_y >= img_height as i32 {
                continue;
            }
            
            for canvas_x in 0..self.width {
                let img_x = canvas_x as i32 - offset_x;
                if img_x < 0 || img_x >= img_width as i32 {
                    continue;
                }
                
                let img_idx = (img_y as usize * img_stride) + (img_x as usize * 4);
                let canvas_idx = (canvas_y as usize * self.stride) + (canvas_x as usize * 4);
                
                if img_idx + 4 <= img_pixels.len() && canvas_idx + 4 <= self.pixels.len() {
                    self.pixels[canvas_idx..canvas_idx + 4].copy_from_slice(&img_pixels[img_idx..img_idx + 4]);
                }
            }
        }
        self.dirty = true;
    }

    /// Paste an image onto the canvas at position (0,0) without scaling
    /// Only the overlapping region is copied (like GIMP)
    pub fn paste_image(&mut self, img_width: u32, img_height: u32, img_pixels: &[u8]) {
        self.paste_image_with_offset(img_width, img_height, img_pixels, 0, 0);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y as usize * self.stride + x as usize * 4;
        self.pixels[idx..idx + 4].copy_from_slice(&color);
        self.dirty = true;
    }

    pub fn blend_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y as usize * self.stride + x as usize * 4;
        let dst = &mut self.pixels[idx..idx + 4];
        let a = color[3] as f32 / 255.0;
        for i in 0..4 {
            let src_v = color[i] as f32;
            let dst_v = dst[i] as f32;
            dst[i] = (src_v * a + dst_v * (1.0 - a)).round() as u8;
        }
        self.dirty = true;
    }

    pub fn stamp_circle(&mut self, cx: f32, cy: f32, radius: f32, color: [u8; 4]) {
        if radius <= 0.0 {
            return;
        }
        let r2 = radius * radius;
        let min_x = (cx - radius).floor().max(0.0) as i32;
        let max_x = (cx + radius).ceil().min((self.width - 1) as f32) as i32;
        let min_y = (cy - radius).floor().max(0.0) as i32;
        let max_y = (cy + radius).ceil().min((self.height - 1) as f32) as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                if dx * dx + dy * dy <= r2 {
                    self.blend_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        if w == 0 || h == 0 {
            return;
        }
        let max_x = (x + w).min(self.width);
        let max_y = (y + h).min(self.height);
        for yy in y..max_y {
            let row = yy as usize * self.stride;
            for xx in x..max_x {
                let idx = row + xx as usize * 4;
                self.pixels[idx..idx + 4].copy_from_slice(&color);
            }
        }
        self.dirty = true;
    }
}

fn aligned_stride(width: u32) -> usize {
    let row = width as usize * 4;
    let align = COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    (row + align - 1) / align * align
}
