# MyGimp - Pixel Editor with Import/Export

A GPU-accelerated pixel editor written in Rust using wgpu and winit, now with save/load/import/export functionality powered by **file dialogs**.

## Features

### Drawing
- **Brush control**: Paint with adjustable size (slider or `-`/`=` keys)
- **Color palette**: 8 colors with hotkeys (`1`-`4`) or click panel
- **Brightness slider**: Adjust brush brightness in real-time
- **Drag sliders**: All controls are draggable for smooth adjustment

### Canvas
- **Resize**: Scale canvas up (`L`) or down (`S`), or use panel buttons
- **Canvas size**: Displays current dimensions; resizes on demand

### Save/Load & Export (with File Dialogs! 📁)
- **Ctrl+P**: Save canvas to project folder
  - Opens folder picker → writes `project.json` + `layer_000.png`
- **Ctrl+O**: Load project from folder
  - Opens folder picker → restores canvas from first layer
- **Ctrl+E**: Export canvas as PNG
  - Opens save dialog (defaults to `export.png`) → saves RGBA8 PNG
- **Ctrl+I**: Import PNG into canvas
  - Opens file picker (PNG/JPEG) → imports if size matches, else shows size mismatch error

## Keyboard Shortcuts

| Key       | Action                    |
|-----------|---------------------------|
| `1-4`     | Select color (palette)    |
| `-`/`=`   | Adjust brush size ±1      |
| `[`/`]`   | Adjust brush size ±2      |
| `S`       | Shrink canvas (÷0.75)     |
| `L`       | Enlarge canvas (×1.25)    |
| **Ctrl+E**| **Export canvas as PNG** (file dialog) |
| **Ctrl+I**| **Import PNG** (file picker)  |
| **Ctrl+O**| **Load project** (folder picker) |
| **Ctrl+P**| **Save project** (folder picker) |

## Project Format

A saved project is a folder containing:
```
my_project/
├── project.json      # Metadata: canvas size, layer names, visibility
└── layer_000.png     # Canvas layer (RGBA8 PNG)
```

Example `project.json`:
```json
{
  "name": "my_project",
  "width": 800,
  "height": 600,
  "layers": [
    {
      "name": "canvas",
      "visible": true,
      "filename": "layer_000.png"
    }
  ]
}
```

## Usage Workflow

1. **Start app**: `cargo run --release`
2. **Draw**: Paint on canvas, adjust brush size & color
3. **Export**: Press `Ctrl+E` → pick save location → exports PNG
4. **Import**: Create/prepare a PNG → press `Ctrl+I` → select file
5. **Save**: Press `Ctrl+P` → pick folder → project saved
6. **Load**: Press `Ctrl+O` → select folder → project loaded

## Module Structure

- **`layer.rs`**: Layer and Project data structures (serde-serializable)
- **`io.rs`**: PNG/JPEG load/save, file dialogs, project persistence, layer compositing
- **`main.rs`**: Event loop, hotkey handlers, UI panel

## Building

```bash
cd Gimp
cargo build --release
./target/release/Gimp  # or Gimp.exe on Windows
```

## Dependencies

- `wgpu` (22.1): GPU rendering
- `winit` (0.30): Window & events
- `image` (0.25): PNG/JPEG I/O
- `serde`/`serde_json`: Project serialization
- `rfd` (0.14): Cross-platform file dialogs
- `env_logger`, `log`, `pollster`: Utilities

## Error Messages

All operations log to console:
- ✓ Success: `✓ Canvas exported to export.png`
- ✗ Failure: `✗ Export failed: <reason>`
- ✗ Canceled: `✗ No file selected` (when user cancels dialog)
- ✗ Size issue: `✗ Image size (1024x768) doesn't match canvas (800x600)`

## Future Enhancements

- [ ] True multi-layer editing (edit individual layers, show/hide, reorder)
- [ ] Undo/redo history
- [ ] Text tool, shapes, selection tools
- [ ] Layer effects (blur, brightness, etc.)
- [ ] Keyboard modifier support (Shift, Alt) for mode switching
- [ ] Canvas size editor before import (scale/crop images to fit)
- [ ] Animated GIF export
