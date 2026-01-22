# MyGimp - GPU-Powered Pixel Editor

A lightweight, GPU-accelerated pixel art editor written in Rust using **wgpu** for rendering and **winit** for windowing. Draw, paint, import/export images, and save/load projects with a simple yet powerful UI.

![Screenshot](#) *(Coming soon)*

## Features

✨ **Core Drawing**
- Brush-based painting with adjustable size and brightness
- 8-color palette (black, dark red, red, orange, yellow, green, blue, purple)
- Real-time canvas rendering with GPU acceleration
- Undo/redo via canvas save/load

✨ **File Operations**
- **Import**: Load PNG/JPEG images (auto-scales to canvas size)
- **Export**: Save canvas as PNG
- **Save**: Persist projects as JSON metadata + PNG layers
- **Load**: Restore projects from saved folders
- Native file dialogs (Windows/macOS/Linux)

✨ **UI Controls**
- Left panel with color palette buttons
- Horizontal size slider with +/- buttons
- Brightness slider with drag support
- Canvas resize buttons (shrink/grow)
- File operation buttons (Import/Export/Save/Open)

✨ **Rendering**
- GPU-powered using wgpu fullscreen triangle shader
- 800×600 canvas by default
- Smooth brush strokes with anti-aliasing

## Installation

### Prerequisites

- **Rust 1.92.0+** (download from [rustup.rs](https://rustup.rs/))
- **GPU** (supports modern graphics cards)

### Build

```bash
git clone https://github.com/ridix/MyGimp.git
cd MyGimp/Gimp
cargo build --release
```

The executable will be at `target/release/Gimp.exe` (Windows) or `target/release/Gimp` (macOS/Linux).

### Run

```bash
cargo run --release
```

Or directly execute the built binary:
```bash
./target/release/Gimp
```

## Usage

### Keyboard Shortcuts

| Action | Hotkey |
|--------|--------|
| **Drawing** | Left-click drag on canvas |
| **Color** | `1` = Black, `2` = Red, `3` = Blue, `4` = Purple |
| **Brush Size** | `-`/`=` to decrease/increase (or drag Size slider) |
| **Brightness** | `[`/`]` to decrease/increase (or drag Brightness slider) |
| **Clear Canvas** | `C` (fills white) |
| **Import Image** | `Ctrl+I` or click **I** button |
| **Export PNG** | `Ctrl+E` or click **E** button |
| **Save Project** | `Ctrl+P` or click **S** button |
| **Load Project** | `Ctrl+O` or click **O** button |

### UI Panel (Left Side)

```
┌────────────────────┐
│  [1] [2] [3] [4]   │ ← Color palette (click to select)
├────────────────────┤
│  - ▫────●──  +     │ ← Size slider (drag to adjust)
├────────────────────┤
│  ↑ [S] [L] ↓       │ ← Canvas buttons (shrink/grow)
├────────────────────┤
│  - ▫─────●─  +     │ ← Brightness slider
├────────────────────┤
│  ┌──┬──────────┐   │
│  │ I │    E    │   │ ← File buttons (Import/Export)
│  ├──┼──────────┤   │
│  │ S │    O    │   │ ← Save/Open buttons
│  └──┴──────────┘   │
└────────────────────┘
```

### Workflow Example

1. **Start Drawing**: Click and drag on the white canvas to paint
2. **Change Color**: Press `2` for red or click the color palette
3. **Adjust Size**: Drag the Size slider left/right
4. **Import Reference**: Press `Ctrl+I` to load a PNG (auto-scaled to canvas)
5. **Paint Over**: Draw on top of the imported image
6. **Save Work**: Press `Ctrl+P` to save as a project folder
7. **Export Final**: Press `Ctrl+E` to save as PNG

## Project Format

Projects are saved as folders containing:

```
MyProject/
├── project.json          ← Metadata (project name, dimensions)
└── layer_000.png         ← Canvas pixel data
```

`project.json` structure:
```json
{
  "name": "My Drawing",
  "width": 800,
  "height": 600,
  "layers": [
    {
      "name": "canvas",
      "filename": "layer_000.png",
      "visible": true
    }
  ]
}
```

## Canvas Dimensions

- **Default**: 800×600 pixels
- **Resize**: Use shrink (↓) / grow (↑) buttons in left panel
- **On Import**: Images are automatically scaled to match canvas size

## Architecture

### Core Modules

| Module | Purpose |
|--------|---------|
| `src/main.rs` | Event loop, UI rendering, input handling |
| `src/gpu.rs` | wgpu GPU pipeline and shader |
| `src/canvas.rs` | CPU pixel buffer management |
| `src/brush.rs` | Brush stroke interpolation & rendering |
| `src/input.rs` | Input state tracking |
| `src/layer.rs` | Layer and project data structures |
| `src/io.rs` | File I/O, import/export, dialogs |

### Dependencies

```toml
wgpu = "0.22"          # GPU rendering
winit = "0.30"         # Windowing
image = "0.25"         # Image loading/saving
serde = "1.0"          # Serialization
serde_json = "1.0"     # JSON format
rfd = "0.14"           # File dialogs
pollster = "0.3"       # Async executor
env_logger = "0.11"    # Logging
```

## Performance

- **GPU-Accelerated**: All rendering offloaded to GPU
- **60 FPS Target**: Maintains high frame rate on modern GPUs
- **Efficient Paint**: Brush strokes use alpha blending
- **Project Size**: Typical 800×600 project ~2MB on disk

## Troubleshooting

### Canvas Disappears After Zoom
- This is a known limitation; canvas resizing clears pixels
- **Workaround**: Use File → Save before zooming, then reload

### "Size Mismatch" on Import (Older Build)
- **Fixed in latest version**: Images now auto-scale to canvas size
- Update via `git pull && cargo build --release`

### File Dialog Not Opening
- Ensure you're using the latest build (`cargo build --release`)
- On Linux, may require `zenity` or `kdialog` installed

### Out of Memory
- Extremely large canvases (>4K) may cause issues
- Stick to 800×600 or similar resolutions

## Roadmap

- [ ] Multi-layer editing
- [ ] Undo/redo stack
- [ ] Selection tools (rect, lasso)
- [ ] Fill bucket tool
- [ ] Gradient tool
- [ ] Text tool
- [ ] Zoom/pan with mouse wheel
- [ ] Color picker
- [ ] Animation support (frame playback)
- [ ] Plugin system

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit changes (`git commit -am 'Add my feature'`)
4. Push to branch (`git push origin feature/my-feature`)
5. Open a Pull Request

## Credits

- **wgpu** community for excellent GPU graphics library
- **Rust** for safe systems programming
- **Image** crate for image processing

## Contact

For issues, questions, or suggestions, please open a [GitHub Issue](https://github.com/ridix/MyGimp/issues).

---

**MyGimp** — Fast. Simple. GPU-Powered. 🎨
