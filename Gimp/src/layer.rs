use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

impl BlendMode {
    /// Blend a foreground color with a background color using this blend mode
    pub fn blend(&self, fg: [u8; 4], bg: [u8; 4]) -> [u8; 4] {
        match self {
            BlendMode::Normal => Self::normal_blend(fg, bg),
            BlendMode::Multiply => Self::multiply_blend(fg, bg),
            BlendMode::Screen => Self::screen_blend(fg, bg),
            BlendMode::Overlay => Self::overlay_blend(fg, bg),
            BlendMode::SoftLight => Self::soft_light_blend(fg, bg),
        }
    }

    fn normal_blend(fg: [u8; 4], bg: [u8; 4]) -> [u8; 4] {
        let fg_alpha = fg[3] as f32 / 255.0;
        let bg_alpha = bg[3] as f32 / 255.0;
        let out_alpha = fg_alpha + bg_alpha * (1.0 - fg_alpha);

        if out_alpha == 0.0 {
            return [0, 0, 0, 0];
        }

        let mut result = [0u8; 4];
        for i in 0..3 {
            let fg_val = fg[i] as f32 / 255.0;
            let bg_val = bg[i] as f32 / 255.0;
            let blended = (fg_val * fg_alpha + bg_val * bg_alpha * (1.0 - fg_alpha)) / out_alpha;
            result[i] = (blended.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        result[3] = (out_alpha * 255.0).round() as u8;
        result
    }

    fn multiply_blend(fg: [u8; 4], bg: [u8; 4]) -> [u8; 4] {
        let mut result = [0u8; 4];
        for i in 0..3 {
            let fg_val = fg[i] as f32 / 255.0;
            let bg_val = bg[i] as f32 / 255.0;
            result[i] = ((fg_val * bg_val) * 255.0).round() as u8;
        }
        result[3] = ((fg[3] as f32 + bg[3] as f32) / 2.0).round() as u8;
        result
    }

    fn screen_blend(fg: [u8; 4], bg: [u8; 4]) -> [u8; 4] {
        let mut result = [0u8; 4];
        for i in 0..3 {
            let fg_val = fg[i] as f32 / 255.0;
            let bg_val = bg[i] as f32 / 255.0;
            let blended = 1.0 - (1.0 - fg_val) * (1.0 - bg_val);
            result[i] = (blended.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        result[3] = ((fg[3] as f32 + bg[3] as f32) / 2.0).round() as u8;
        result
    }

    fn overlay_blend(fg: [u8; 4], bg: [u8; 4]) -> [u8; 4] {
        let mut result = [0u8; 4];
        for i in 0..3 {
            let fg_val = fg[i] as f32 / 255.0;
            let bg_val = bg[i] as f32 / 255.0;
            let blended = if bg_val < 0.5 {
                2.0 * fg_val * bg_val
            } else {
                1.0 - 2.0 * (1.0 - fg_val) * (1.0 - bg_val)
            };
            result[i] = (blended.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        result[3] = ((fg[3] as f32 + bg[3] as f32) / 2.0).round() as u8;
        result
    }

    fn soft_light_blend(fg: [u8; 4], bg: [u8; 4]) -> [u8; 4] {
        let mut result = [0u8; 4];
        for i in 0..3 {
            let fg_val = fg[i] as f32 / 255.0;
            let bg_val = bg[i] as f32 / 255.0;
            let blended = if fg_val < 0.5 {
                bg_val - (1.0 - 2.0 * fg_val) * bg_val * (1.0 - bg_val)
            } else {
                bg_val + (2.0 * fg_val - 1.0)
                    * (Self::g_function(bg_val) - bg_val)
            };
            result[i] = (blended.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        result[3] = ((fg[3] as f32 + bg[3] as f32) / 2.0).round() as u8;
        result
    }

    fn g_function(x: f32) -> f32 {
        if x <= 0.25 {
            ((16.0 * x - 12.0) * x + 4.0) * x
        } else {
            x.sqrt()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub blend_mode: BlendMode,
    pub opacity: u8,  // 0-255
    pub pixels: Vec<u8>, // RGBA8, packed in row-major order
}

impl Layer {
    #[allow(dead_code)]
    pub fn new(name: String, width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            name,
            width,
            height,
            visible: true,
            blend_mode: BlendMode::default(),
            opacity: 255,
            pixels: vec![255; size], // White by default
        }
    }

    pub fn from_rgba(name: String, width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            name,
            width,
            height,
            visible: true,
            blend_mode: BlendMode::default(),
            opacity: 255,
            pixels,
        }
    }

    #[allow(dead_code)]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            if idx + 3 < self.pixels.len() {
                self.pixels[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            if idx + 3 < self.pixels.len() {
                let mut color = [0u8; 4];
                color.copy_from_slice(&self.pixels[idx..idx + 4]);
                return color;
            }
        }
        [0, 0, 0, 0]
    }

    #[allow(dead_code)]
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let new_size = (new_width as usize) * (new_height as usize) * 4;
        self.pixels.resize(new_size, 255);
        self.width = new_width;
        self.height = new_height;
    }

    #[allow(dead_code)]
    pub fn clear(&mut self, color: [u8; 4]) {
        for i in (0..self.pixels.len()).step_by(4) {
            self.pixels[i..i + 4].copy_from_slice(&color);
        }
    }

    /// Set the blend mode for this layer
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    /// Set the opacity (0-255)
    pub fn set_opacity(&mut self, opacity: u8) {
        self.opacity = opacity;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<LayerMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub name: String,
    pub visible: bool,
    pub blend_mode: BlendMode,
    pub opacity: u8,
    pub filename: String,
}

impl Project {
    pub fn new(name: String, width: u32, height: u32) -> Self {
        Self {
            name,
            width,
            height,
            layers: vec![],
        }
    }

    pub fn add_layer_metadata(&mut self, name: String, filename: String) {
        self.layers.push(LayerMetadata {
            name,
            visible: true,
            blend_mode: BlendMode::default(),
            opacity: 255,
            filename,
        });
    }
}
