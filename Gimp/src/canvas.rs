use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixels: Vec<u8>, // Composite display buffer
    pub dirty: bool,
    pub loaded_image_size: Option<(u32, u32)>, // Track size of loaded image for panning
    pub loaded_image_data: Option<Vec<u8>>, // Store loaded image for re-panning
    pub zoom_scale: f32, // Zoom level (1.0 = 100%, 2.0 = 200%, etc.)
    pub drawing_layer: Vec<u8>, // User drawings layer in IMAGE-SPACE coordinates
    pub pan_offset: (i32, i32), // Store pan offset so drawings can use it
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let stride = aligned_stride(width);
        let pixels = vec![255; stride * height as usize];
        let drawing_layer = vec![]; // Will be sized to match loaded image
        Self {
            width,
            height,
            stride,
            pixels,
            dirty: false,
            loaded_image_size: None,
            loaded_image_data: None,
            zoom_scale: 1.0,
            drawing_layer,
            pan_offset: (0, 0),
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
    /// This updates the background layer only, not the drawing layer
    pub fn paste_image_with_offset(&mut self, img_width: u32, img_height: u32, img_pixels: &[u8], offset_x: i32, offset_y: i32) {
        let is_new_image = self.loaded_image_size != Some((img_width, img_height));
        
        self.loaded_image_size = Some((img_width, img_height));
        self.loaded_image_data = Some(img_pixels.to_vec());
        self.pan_offset = (offset_x, offset_y);
        
        // Initialize drawing layer to match image size if new image
        if is_new_image {
            self.drawing_layer = vec![0; (img_width * img_height * 4) as usize];
        }
        
        let img_stride = img_width as usize * 4;
        
        // Render the background image with zoom/pan
        for canvas_y in 0..self.height {
            for canvas_x in 0..self.width {
                // Calculate source image coordinates with zoom
                let img_x = ((canvas_x as f32 / self.zoom_scale) as i32) - offset_x;
                let img_y = ((canvas_y as f32 / self.zoom_scale) as i32) - offset_y;
                
                let canvas_idx = (canvas_y as usize * self.stride) + (canvas_x as usize * 4);
                
                if img_x >= 0 && img_x < img_width as i32 && img_y >= 0 && img_y < img_height as i32 {
                    let img_idx = (img_y as usize * img_stride) + (img_x as usize * 4);
                    
                    if img_idx + 4 <= img_pixels.len() && canvas_idx + 4 <= self.pixels.len() {
                        self.pixels[canvas_idx..canvas_idx + 4].copy_from_slice(&img_pixels[img_idx..img_idx + 4]);
                    }
                } else {
                    // Fill with white outside image bounds
                    if canvas_idx + 4 <= self.pixels.len() {
                        self.pixels[canvas_idx..canvas_idx + 4].copy_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }
        }
        
        // Composite drawing layer on top
        self.composite_layers();
        self.dirty = true;
    }
    
    /// Composite the drawing layer on top of the background
    /// Drawing layer is in image-space, so we need to transform coordinates
    fn composite_layers(&mut self) {
        if let Some((img_w, img_h)) = self.loaded_image_size {
            let img_stride = img_w as usize * 4;
            let (offset_x, offset_y) = self.pan_offset;
            
            for canvas_y in 0..self.height {
                for canvas_x in 0..self.width {
                    // Convert canvas coords to image coords
                    let img_x = ((canvas_x as f32 / self.zoom_scale) as i32) - offset_x;
                    let img_y = ((canvas_y as f32 / self.zoom_scale) as i32) - offset_y;
                    
                    if img_x >= 0 && img_x < img_w as i32 && img_y >= 0 && img_y < img_h as i32 {
                        let img_idx = (img_y as usize * img_stride) + (img_x as usize * 4);
                        let canvas_idx = (canvas_y as usize * self.stride) + (canvas_x as usize * 4);
                        
                        if img_idx + 3 < self.drawing_layer.len() && canvas_idx + 3 < self.pixels.len() {
                            let alpha = self.drawing_layer[img_idx + 3] as f32 / 255.0;
                            if alpha > 0.0 {
                                // Alpha blend drawing on top of background
                                for j in 0..3 {
                                    let bg = self.pixels[canvas_idx + j] as f32;
                                    let fg = self.drawing_layer[img_idx + j] as f32;
                                    self.pixels[canvas_idx + j] = (fg * alpha + bg * (1.0 - alpha)) as u8;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Re-render the loaded image with a new offset
    pub fn repan_image(&mut self, offset_x: i32, offset_y: i32) {
        if let Some((img_w, img_h)) = self.loaded_image_size {
            if let Some(img_data) = self.loaded_image_data.clone() {
                self.paste_image_with_offset(img_w, img_h, &img_data, offset_x, offset_y);
            }
        }
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
        
        // Convert canvas coordinates to image coordinates (so drawing moves with pan/zoom)
        if let Some((img_w, img_h)) = self.loaded_image_size {
            let (offset_x, offset_y) = self.pan_offset;
            
            let img_x = ((x as f32 / self.zoom_scale) as i32) - offset_x;
            let img_y = ((y as f32 / self.zoom_scale) as i32) - offset_y;
            
            if img_x >= 0 && img_x < img_w as i32 && img_y >= 0 && img_y < img_h as i32 {
                // Store in drawing layer at image coordinates
                let img_stride = img_w as usize * 4;
                let img_idx = (img_y as usize * img_stride) + (img_x as usize * 4);
                
                if img_idx + 4 <= self.drawing_layer.len() {
                    let dst = &mut self.drawing_layer[img_idx..img_idx + 4];
                    let a = color[3] as f32 / 255.0;
                    for i in 0..4 {
                        let src_v = color[i] as f32;
                        let dst_v = dst[i] as f32;
                        dst[i] = (src_v * a + dst_v * (1.0 - a)).round() as u8;
                    }
                }
            }
        }
        
        // Also update the display buffer at canvas coordinates
        let idx = y as usize * self.stride + x as usize * 4;
        if idx + 4 <= self.pixels.len() {
            let display_dst = &mut self.pixels[idx..idx + 4];
            let a = color[3] as f32 / 255.0;
            for i in 0..4 {
                let src_v = color[i] as f32;
                let dst_v = display_dst[i] as f32;
                display_dst[i] = (src_v * a + dst_v * (1.0 - a)).round() as u8;
            }
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

    /// Erase a circle (set pixels to transparent in drawing layer)
    pub fn erase_circle(&mut self, cx: f32, cy: f32, radius: f32) {
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
                    let canvas_x = x as u32;
                    let canvas_y = y as u32;
                    
                    if canvas_x >= self.width || canvas_y >= self.height {
                        continue;
                    }
                    
                    // Erase from drawing layer if we have an image
                    if let Some((img_w, img_h)) = self.loaded_image_size {
                        let (offset_x, offset_y) = self.pan_offset;
                        let img_x = ((canvas_x as f32 / self.zoom_scale) as i32) - offset_x;
                        let img_y = ((canvas_y as f32 / self.zoom_scale) as i32) - offset_y;
                        
                        if img_x >= 0 && img_x < img_w as i32 && img_y >= 0 && img_y < img_h as i32 {
                            let img_stride = img_w as usize * 4;
                            let img_idx = (img_y as usize * img_stride) + (img_x as usize * 4);
                            if img_idx + 4 <= self.drawing_layer.len() {
                                self.drawing_layer[img_idx..img_idx + 4].copy_from_slice(&[0, 0, 0, 0]);
                            }
                        }
                    }
                    
                    // Also update display buffer
                    let idx = canvas_y as usize * self.stride + canvas_x as usize * 4;
                    if idx + 4 <= self.pixels.len() {
                        // Get the background pixel (re-render from loaded image if available)
                        if let Some((img_w, img_h)) = self.loaded_image_size {
                            if let Some(img_data) = &self.loaded_image_data {
                                let (offset_x, offset_y) = self.pan_offset;
                                let img_x = ((canvas_x as f32 / self.zoom_scale) as i32) - offset_x;
                                let img_y = ((canvas_y as f32 / self.zoom_scale) as i32) - offset_y;
                                
                                if img_x >= 0 && img_x < img_w as i32 && img_y >= 0 && img_y < img_h as i32 {
                                    let img_stride = img_w as usize * 4;
                                    let img_idx = (img_y as usize * img_stride) + (img_x as usize * 4);
                                    if img_idx + 4 <= img_data.len() {
                                        self.pixels[idx..idx + 4].copy_from_slice(&img_data[img_idx..img_idx + 4]);
                                    }
                                } else {
                                    self.pixels[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
                                }
                            }
                        } else {
                            // No image, set to white
                            self.pixels[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
                        }
                    }
                }
            }
        }
        
        self.dirty = true;
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
    
    /// Get pixel color at canvas coordinates (for color picker)
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = y as usize * self.stride + x as usize * 4;
        if idx + 4 <= self.pixels.len() {
            Some([
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            ])
        } else {
            None
        }
    }
    
    /// Flood fill at canvas coordinates
    pub fn flood_fill(&mut self, start_x: u32, start_y: u32, fill_color: [u8; 4]) {
        if start_x >= self.width || start_y >= self.height {
            return;
        }
        
        let idx = start_y as usize * self.stride + start_x as usize * 4;
        if idx + 4 > self.pixels.len() {
            return;
        }
        
        let target_color = [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ];
        
        // Don't fill if already the same color
        if target_color == fill_color {
            return;
        }
        
        let mut stack = vec![(start_x, start_y)];
        let mut visited = vec![false; (self.width * self.height) as usize];
        
        while let Some((x, y)) = stack.pop() {
            if x >= self.width || y >= self.height {
                continue;
            }
            
            let visit_idx = (y * self.width + x) as usize;
            if visited[visit_idx] {
                continue;
            }
            visited[visit_idx] = true;
            
            let pixel_idx = y as usize * self.stride + x as usize * 4;
            if pixel_idx + 4 > self.pixels.len() {
                continue;
            }
            
            let current = [
                self.pixels[pixel_idx],
                self.pixels[pixel_idx + 1],
                self.pixels[pixel_idx + 2],
                self.pixels[pixel_idx + 3],
            ];
            
            if current != target_color {
                continue;
            }
            
            // Fill this pixel
            self.pixels[pixel_idx..pixel_idx + 4].copy_from_slice(&fill_color);
            
            // Also fill in drawing layer if we have an image
            if let Some((img_w, img_h)) = self.loaded_image_size {
                let (offset_x, offset_y) = self.pan_offset;
                let img_x = ((x as f32 / self.zoom_scale) as i32) - offset_x;
                let img_y = ((y as f32 / self.zoom_scale) as i32) - offset_y;
                
                if img_x >= 0 && img_x < img_w as i32 && img_y >= 0 && img_y < img_h as i32 {
                    let img_stride = img_w as usize * 4;
                    let img_idx = (img_y as usize * img_stride) + (img_x as usize * 4);
                    if img_idx + 4 <= self.drawing_layer.len() {
                        self.drawing_layer[img_idx..img_idx + 4].copy_from_slice(&fill_color);
                    }
                }
            }
            
            // Add neighbors
            if x > 0 { stack.push((x - 1, y)); }
            if x + 1 < self.width { stack.push((x + 1, y)); }
            if y > 0 { stack.push((x, y - 1)); }
            if y + 1 < self.height { stack.push((x, y + 1)); }
        }
        
        self.dirty = true;
    }
    
    /// Move/translate the drawing layer by offset
    pub fn move_layer(&mut self, offset_x: i32, offset_y: i32) {
        if let Some((img_w, img_h)) = self.loaded_image_size {
            let img_stride = img_w as usize * 4;
            let mut new_layer = vec![0u8; self.drawing_layer.len()];
            
            for y in 0..img_h {
                for x in 0..img_w {
                    let new_x = x as i32 + offset_x;
                    let new_y = y as i32 + offset_y;
                    
                    if new_x >= 0 && new_x < img_w as i32 && new_y >= 0 && new_y < img_h as i32 {
                        let src_idx = (y as usize * img_stride) + (x as usize * 4);
                        let dst_idx = (new_y as usize * img_stride) + (new_x as usize * 4);
                        
                        if src_idx + 4 <= self.drawing_layer.len() && dst_idx + 4 <= new_layer.len() {
                            new_layer[dst_idx..dst_idx + 4].copy_from_slice(&self.drawing_layer[src_idx..src_idx + 4]);
                        }
                    }
                }
            }
            
            self.drawing_layer = new_layer;
            
            // Re-render
            if let Some(img_data) = self.loaded_image_data.clone() {
                let (offset_x, offset_y) = self.pan_offset;
                self.paste_image_with_offset(img_w, img_h, &img_data, offset_x, offset_y);
            }
        }
        
        self.dirty = true;
    }
    
    /// Apply invert filter to drawing layer
    pub fn filter_invert(&mut self) {
        if let Some((img_w, img_h)) = self.loaded_image_size {
            let img_stride = img_w as usize * 4;
            for y in 0..img_h {
                for x in 0..img_w {
                    let idx = (y as usize * img_stride) + (x as usize * 4);
                    if idx + 3 < self.drawing_layer.len() {
                        // Only invert if pixel has been drawn on (has some alpha)
                        if self.drawing_layer[idx + 3] > 0 {
                            self.drawing_layer[idx] = 255 - self.drawing_layer[idx];
                            self.drawing_layer[idx + 1] = 255 - self.drawing_layer[idx + 1];
                            self.drawing_layer[idx + 2] = 255 - self.drawing_layer[idx + 2];
                        }
                    }
                }
            }
            // Re-render
            if let Some(img_data) = self.loaded_image_data.clone() {
                let (offset_x, offset_y) = self.pan_offset;
                self.paste_image_with_offset(img_w, img_h, &img_data, offset_x, offset_y);
            }
        }
        self.dirty = true;
    }
    
    /// Apply grayscale filter to drawing layer
    pub fn filter_grayscale(&mut self) {
        if let Some((img_w, img_h)) = self.loaded_image_size {
            let img_stride = img_w as usize * 4;
            for y in 0..img_h {
                for x in 0..img_w {
                    let idx = (y as usize * img_stride) + (x as usize * 4);
                    if idx + 3 < self.drawing_layer.len() {
                        if self.drawing_layer[idx + 3] > 0 {
                            let r = self.drawing_layer[idx] as f32;
                            let g = self.drawing_layer[idx + 1] as f32;
                            let b = self.drawing_layer[idx + 2] as f32;
                            // Luminosity method
                            let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                            self.drawing_layer[idx] = gray;
                            self.drawing_layer[idx + 1] = gray;
                            self.drawing_layer[idx + 2] = gray;
                        }
                    }
                }
            }
            if let Some(img_data) = self.loaded_image_data.clone() {
                let (offset_x, offset_y) = self.pan_offset;
                self.paste_image_with_offset(img_w, img_h, &img_data, offset_x, offset_y);
            }
        }
        self.dirty = true;
    }
    
    /// Apply brightness/contrast adjustment to drawing layer
    pub fn filter_brightness_contrast(&mut self, brightness: f32, contrast: f32) {
        if let Some((img_w, img_h)) = self.loaded_image_size {
            let img_stride = img_w as usize * 4;
            let factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));
            
            for y in 0..img_h {
                for x in 0..img_w {
                    let idx = (y as usize * img_stride) + (x as usize * 4);
                    if idx + 3 < self.drawing_layer.len() {
                        if self.drawing_layer[idx + 3] > 0 {
                            for i in 0..3 {
                                let pixel = self.drawing_layer[idx + i] as f32;
                                // Apply contrast
                                let contrasted = factor * (pixel - 128.0) + 128.0;
                                // Apply brightness
                                let adjusted = contrasted + brightness;
                                self.drawing_layer[idx + i] = adjusted.clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                }
            }
            if let Some(img_data) = self.loaded_image_data.clone() {
                let (offset_x, offset_y) = self.pan_offset;
                self.paste_image_with_offset(img_w, img_h, &img_data, offset_x, offset_y);
            }
        }
        self.dirty = true;
    }
    
    /// Apply box blur filter to drawing layer
    pub fn filter_blur(&mut self, radius: u32) {
        if radius == 0 {
            return;
        }
        if let Some((img_w, img_h)) = self.loaded_image_size {
            let img_stride = img_w as usize * 4;
            let mut temp_layer = self.drawing_layer.clone();
            
            for y in 0..img_h {
                for x in 0..img_w {
                    let idx = (y as usize * img_stride) + (x as usize * 4);
                    if idx + 3 >= self.drawing_layer.len() {
                        continue;
                    }
                    
                    // Only blur pixels that have been drawn on
                    if self.drawing_layer[idx + 3] == 0 {
                        continue;
                    }
                    
                    let mut sum_r = 0u32;
                    let mut sum_g = 0u32;
                    let mut sum_b = 0u32;
                    let mut sum_a = 0u32;
                    let mut count = 0u32;
                    
                    // Box blur: average pixels in radius
                    let min_y = y.saturating_sub(radius);
                    let max_y = (y + radius).min(img_h - 1);
                    let min_x = x.saturating_sub(radius);
                    let max_x = (x + radius).min(img_w - 1);
                    
                    for by in min_y..=max_y {
                        for bx in min_x..=max_x {
                            let bidx = (by as usize * img_stride) + (bx as usize * 4);
                            if bidx + 3 < self.drawing_layer.len() {
                                sum_r += self.drawing_layer[bidx] as u32;
                                sum_g += self.drawing_layer[bidx + 1] as u32;
                                sum_b += self.drawing_layer[bidx + 2] as u32;
                                sum_a += self.drawing_layer[bidx + 3] as u32;
                                count += 1;
                            }
                        }
                    }
                    
                    if count > 0 {
                        temp_layer[idx] = (sum_r / count) as u8;
                        temp_layer[idx + 1] = (sum_g / count) as u8;
                        temp_layer[idx + 2] = (sum_b / count) as u8;
                        temp_layer[idx + 3] = (sum_a / count) as u8;
                    }
                }
            }
            
            self.drawing_layer = temp_layer;
            
            if let Some(img_data) = self.loaded_image_data.clone() {
                let (offset_x, offset_y) = self.pan_offset;
                self.paste_image_with_offset(img_w, img_h, &img_data, offset_x, offset_y);
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
