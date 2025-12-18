use std::fs;
use std::path::Path;
use image::{ImageBuffer, RgbaImage};
use serde_json;
use rfd::FileDialog;

use crate::layer::{Layer, Project};
use crate::canvas::Canvas;

pub type IoResult<T> = Result<T, String>;

/// Open file dialog to select an image file (PNG/JPEG)
pub fn select_image_file() -> IoResult<String> {
    FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg"])
        .add_filter("PNG", &["png"])
        .add_filter("JPEG", &["jpg", "jpeg"])
        .pick_file()
        .ok_or_else(|| "No file selected".to_string())
        .map(|p| p.to_string_lossy().to_string())
}

/// Save file dialog to export as PNG
pub fn select_export_png_path() -> IoResult<String> {
    FileDialog::new()
        .add_filter("PNG", &["png"])
        .set_file_name("export.png")
        .save_file()
        .ok_or_else(|| "No file selected".to_string())
        .map(|p| p.to_string_lossy().to_string())
}

/// Select folder for project save
pub fn select_save_project_folder() -> IoResult<String> {
    FileDialog::new()
        .set_directory(".")
        .pick_folder()
        .ok_or_else(|| "No folder selected".to_string())
        .map(|p| p.to_string_lossy().to_string())
}

/// Select folder for project load
pub fn select_load_project_folder() -> IoResult<String> {
    FileDialog::new()
        .set_directory(".")
        .pick_folder()
        .ok_or_else(|| "No folder selected".to_string())
        .map(|p| p.to_string_lossy().to_string())
}


/// Load a PNG or JPEG from disk into a Layer.
pub fn load_image(path: &str) -> IoResult<Layer> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to load image {}: {}", path, e))?;
    
    let rgba_img = img.to_rgba8();
    let (_width, _height) = rgba_img.dimensions();
    let pixels = rgba_img.to_vec();
    
    let filename = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported")
        .to_string();
    
    Ok(Layer::from_rgba(filename, _width, _height, pixels))
}

/// Load and resize image to fit canvas dimensions
pub fn load_image_scaled(path: &str, target_width: u32, target_height: u32) -> IoResult<Layer> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to load image {}: {}", path, e))?;
    
    let rgba_img = img.to_rgba8();
    
    // If dimensions match, return as-is
    if rgba_img.width() == target_width && rgba_img.height() == target_height {
        let pixels = rgba_img.to_vec();
        let filename = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported")
            .to_string();
        return Ok(Layer::from_rgba(filename, target_width, target_height, pixels));
    }
    
    // Resize the image using nearest neighbor (fast)
    let resized = image::imageops::resize(
        &rgba_img,
        target_width,
        target_height,
        image::imageops::FilterType::Nearest,
    );
    
    let pixels = resized.to_vec();
    let filename = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported")
        .to_string();
    
    Ok(Layer::from_rgba(filename, target_width, target_height, pixels))
}

/// Export a Layer as a PNG file.
pub fn export_layer_as_png(layer: &Layer, path: &str) -> IoResult<()> {
    let img: RgbaImage = ImageBuffer::from_raw(
        layer.width,
        layer.height,
        layer.pixels.clone(),
    ).ok_or("Failed to create image buffer".to_string())?;
    
    img.save(path)
        .map_err(|e| format!("Failed to save PNG {}: {}", path, e))
}

/// Export a Canvas as a PNG file.
pub fn export_canvas_as_png(canvas: &Canvas, path: &str) -> IoResult<()> {
    // Extract tight-packed pixels from stride-aligned canvas
    let tight_pixels = canvas.extract_tight_pixels();
    
    let img: RgbaImage = ImageBuffer::from_raw(
        canvas.width,
        canvas.height,
        tight_pixels,
    ).ok_or("Failed to create image buffer".to_string())?;
    
    img.save(path)
        .map_err(|e| format!("Failed to save PNG {}: {}", path, e))
}

/// Save a Project (JSON + PNGs) to a folder.
pub fn save_project(project: &Project, layers: &[Layer], folder_path: &str) -> IoResult<()> {
    // Create folder if it doesn't exist
    fs::create_dir_all(folder_path)
        .map_err(|e| format!("Failed to create folder {}: {}", folder_path, e))?;
    
    // Save each layer as PNG
    for (idx, layer) in layers.iter().enumerate() {
        let layer_filename = format!("layer_{:03}.png", idx);
        let layer_path = Path::new(folder_path).join(&layer_filename);
        export_layer_as_png(layer, layer_path.to_str().unwrap())?;
    }
    
    // Save project JSON
    let project_json = serde_json::to_string_pretty(project)
        .map_err(|e| format!("Failed to serialize project: {}", e))?;
    
    let json_path = Path::new(folder_path).join("project.json");
    fs::write(&json_path, project_json)
        .map_err(|e| format!("Failed to write project.json: {}", e))?;
    
    Ok(())
}

/// Load a Project (JSON + PNGs) from a folder.
pub fn load_project(folder_path: &str) -> IoResult<(Project, Vec<Layer>)> {
    // Read project JSON
    let json_path = Path::new(folder_path).join("project.json");
    let json_content = fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read project.json: {}", e))?;
    
    let project: Project = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse project.json: {}", e))?;
    
    // Load layers
    let mut layers = Vec::new();
    for (idx, metadata) in project.layers.iter().enumerate() {
        let layer_filename = format!("layer_{:03}.png", idx);
        let layer_path = Path::new(folder_path).join(&layer_filename);
        
        let mut layer = load_image(layer_path.to_str().unwrap())?;
        layer.name = metadata.name.clone();
        layer.visible = metadata.visible;
        layers.push(layer);
    }
    
    Ok((project, layers))
}

/// Composite all visible layers into a single Canvas-like buffer.
#[allow(dead_code)]
pub fn composite_layers(width: u32, height: u32, layers: &[Layer]) -> Vec<u8> {
    let mut result = vec![255u8; width as usize * height as usize * 4];
    
    for layer in layers {
        if !layer.visible {
            continue;
        }
        // Simple alpha blend
        for y in 0..layer.height.min(height) {
            for x in 0..layer.width.min(width) {
                let src_idx = ((y * layer.width + x) * 4) as usize;
                let dst_idx = ((y * width + x) * 4) as usize;
                
                if src_idx + 3 < layer.pixels.len() && dst_idx + 3 < result.len() {
                    let src = &layer.pixels[src_idx..src_idx + 4];
                    let dst = &mut result[dst_idx..dst_idx + 4];
                    
                    let alpha = src[3] as f32 / 255.0;
                    for i in 0..3 {
                        dst[i] = (src[i] as f32 * alpha + dst[i] as f32 * (1.0 - alpha)) as u8;
                    }
                    dst[3] = 255;
                }
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::from_rgba("test".to_string(), 100, 100, vec![255; 40000]);
        assert_eq!(layer.width, 100);
        assert_eq!(layer.height, 100);
        assert_eq!(layer.pixels.len(), 40000);
    }

    #[test]
    fn test_project_save_load() {
        let test_folder = "test_project_io";
        let _ = std::fs::remove_dir_all(test_folder);

        let layer = Layer::from_rgba("test".to_string(), 64, 64, vec![200; 16384]);
        let mut project = Project::new("Test".to_string(), 64, 64);
        project.add_layer_metadata("L0".to_string(), "layer_000.png".to_string());

        assert!(save_project(&project, &[layer], test_folder).is_ok());
        assert!(std::path::Path::new(&format!("{}/project.json", test_folder)).exists());

        let result = load_project(test_folder);
        assert!(result.is_ok());
        let (proj, layers) = result.unwrap();
        assert_eq!(proj.name, "Test");
        assert_eq!(layers.len(), 1);

        let _ = std::fs::remove_dir_all(test_folder);
    }
}
