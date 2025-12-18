use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
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
            pixels: vec![255; size], // White by default
        }
    }

    pub fn from_rgba(name: String, width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            name,
            width,
            height,
            visible: true,
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
            filename,
        });
    }
}
