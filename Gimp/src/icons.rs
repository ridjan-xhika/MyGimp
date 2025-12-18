use image::ImageReader;
use std::io::Cursor;

#[derive(Clone)]
pub struct Icon {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Icon {
    pub fn empty() -> Self {
        Icon {
            pixels: vec![],
            width: 0,
            height: 0,
        }
    }
}

pub struct IconCache {
    pub brush: Icon,
    pub eraser: Icon,
    pub fill: Icon,
    pub picker: Icon,
    pub move_tool: Icon,
    pub import: Icon,
    pub export: Icon,
    pub save: Icon,
    pub invert: Icon,
    pub grayscale: Icon,
    pub brightness: Icon,
    pub blur: Icon,
}

impl IconCache {
    pub fn load() -> Self {
        IconCache {
            brush: load_icon("assets/brush.png"),
            eraser: load_icon("assets/eraser.png"),
            fill: load_icon("assets/fillbucket.png"),
            picker: load_icon("assets/colorpicker.png"),
            move_tool: load_icon("assets/move.png"),
            import: load_icon("assets/import.png"),
            export: load_icon("assets/export.png"),
            save: load_icon("assets/save.png"),
            invert: load_icon("assets/invert.png"),
            grayscale: load_icon("assets/grayscale.png"),
            brightness: load_icon("assets/brightness.png"),
            blur: load_icon("assets/blur.png"),
        }
    }
}

fn load_icon(path: &str) -> Icon {
    match std::fs::read(path) {
        Ok(data) => {
            if let Ok(img) = ImageReader::new(Cursor::new(&data))
                .and_then(|r| r.with_guessed_format())
                .and_then(|r| r.decode())
            {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                Icon {
                    pixels: rgba.into_raw(),
                    width,
                    height,
                }
            } else {
                Icon::empty()
            }
        }
        Err(_) => Icon::empty(),
    }
}
