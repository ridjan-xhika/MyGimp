use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub pixels: Vec<u8>,
    pub dirty: bool,
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
        }
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
