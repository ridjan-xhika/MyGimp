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
    // Try multiple path variants
    let paths = [
        path.to_string(),
        format!("Gimp/{}", path),
        format!("./Gimp/{}", path),
    ];
    
    for try_path in &paths {
        match std::fs::read(try_path) {
            Ok(data) => {
                match ImageReader::new(Cursor::new(&data))
                    .with_guessed_format()
                {
                    Ok(reader) => {
                        match reader.decode() {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                eprintln!("✓ Loaded icon: {} ({} × {})", try_path, width, height);
                                return Icon {
                                    pixels: rgba.into_raw(),
                                    width,
                                    height,
                                };
                            }
                            Err(e) => {
                                eprintln!("✗ Failed to decode {}: {}", try_path, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to read format for {}: {}", try_path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to read {}: {}", try_path, e);
            }
        }
    }
    
    eprintln!("✗ Could not load icon from any path variant for: {}", path);
    Icon::empty()
}
