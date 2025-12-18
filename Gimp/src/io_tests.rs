#[cfg(test)]
mod tests {
    use crate::io::*;
    use crate::layer::{Layer, Project};
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::from_rgba("test".to_string(), 100, 100, vec![255; 40000]);
        assert_eq!(layer.width, 100);
        assert_eq!(layer.height, 100);
        assert_eq!(layer.pixels.len(), 40000);
        assert_eq!(layer.name, "test");
    }

    #[test]
    fn test_project_creation() {
        let mut project = Project::new("TestProject".to_string(), 800, 600);
        project.add_layer_metadata("Layer 0".to_string(), "layer_000.png".to_string());
        assert_eq!(project.layers.len(), 1);
        assert_eq!(project.width, 800);
        assert_eq!(project.height, 600);
    }

    #[test]
    fn test_layer_persistence() {
        // Create a test layer
        let mut pixels = vec![0u8; 40000];
        // Set some pixels to distinguish the layer
        for i in (0..1000).step_by(4) {
            pixels[i] = 255;     // R
            pixels[i + 3] = 255; // A
        }
        let layer = Layer::from_rgba("persist_test".to_string(), 100, 100, pixels.clone());

        // Save it
        let test_path = "test_layer.png";
        let result = export_layer_as_png(&layer, test_path);
        assert!(result.is_ok(), "Export failed: {:?}", result);

        // Load it back
        let loaded = load_image(test_path);
        assert!(loaded.is_ok(), "Import failed: {:?}", loaded);

        let loaded_layer = loaded.unwrap();
        assert_eq!(loaded_layer.width, 100);
        assert_eq!(loaded_layer.height, 100);
        assert_eq!(loaded_layer.pixels.len(), 40000);

        // Cleanup
        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_project_save_load() {
        let test_folder = "test_project_io";
        
        // Clean up any previous test
        let _ = fs::remove_dir_all(test_folder);

        // Create and save a project
        let layer = Layer::from_rgba(
            "test_layer".to_string(),
            200,
            200,
            vec![128; 160000],
        );
        let mut project = Project::new("IOTest".to_string(), 200, 200);
        project.add_layer_metadata("Layer 0".to_string(), "layer_000.png".to_string());

        let save_result = save_project(&project, &[layer], test_folder);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result);

        // Verify files were created
        assert!(Path::new(test_folder).exists(), "Project folder not created");
        assert!(Path::new(&format!("{}/project.json", test_folder)).exists());
        assert!(Path::new(&format!("{}/layer_000.png", test_folder)).exists());

        // Load it back
        let load_result = load_project(test_folder);
        assert!(load_result.is_ok(), "Load failed: {:?}", load_result);

        let (loaded_project, loaded_layers) = load_result.unwrap();
        assert_eq!(loaded_project.name, "IOTest");
        assert_eq!(loaded_project.width, 200);
        assert_eq!(loaded_project.height, 200);
        assert_eq!(loaded_layers.len(), 1);
        assert_eq!(loaded_layers[0].width, 200);
        assert_eq!(loaded_layers[0].height, 200);

        // Cleanup
        let _ = fs::remove_dir_all(test_folder);
    }

    #[test]
    fn test_composite_layers() {
        let layer1 = Layer::from_rgba("layer1".to_string(), 100, 100, vec![255; 40000]);
        let layer2 = Layer::from_rgba("layer2".to_string(), 100, 100, vec![0; 40000]);

        let composite = composite_layers(100, 100, &[layer1, layer2]);
        assert_eq!(composite.len(), 40000);
        // Result should have some blend of both layers
        assert!(!composite.is_empty());
    }
}
