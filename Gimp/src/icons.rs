use image::ImageReader;
use std::io::Cursor;

pub struct IconCache {
    pub brush: Vec<u8>,
    pub eraser: Vec<u8>,
    pub fill: Vec<u8>,
    pub picker: Vec<u8>,
    pub move_tool: Vec<u8>,
    pub import: Vec<u8>,
    pub export: Vec<u8>,
    pub save: Vec<u8>,
    pub invert: Vec<u8>,
    pub grayscale: Vec<u8>,
    pub brightness: Vec<u8>,
    pub blur: Vec<u8>,
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

fn load_icon(path: &str) -> Vec<u8> {
    match std::fs::read(path) {
        Ok(data) => {
            if let Ok(img) = ImageReader::new(Cursor::new(&data))
                .and_then(|r| r.with_guessed_format())
                .and_then(|r| r.decode())
            {
                img.to_rgba8().into_raw()
            } else {
                vec![]
            }
        }
        Err(_) => vec![],
    }
}
