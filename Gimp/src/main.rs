mod brush;
mod canvas;
mod gpu;
mod input;
mod layer;
mod io;

use std::sync::Arc;
use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};

use crate::{
    brush::Brush,
    canvas::Canvas,
    gpu::Gpu,
    input::{InputState, SliderDrag},
};

const BRUSH_COLOR: [u8; 4] = [0, 0, 0, 255];
const BRUSH_RADIUS: f32 = 6.0;
const BRUSH_RADIUS_MIN: f32 = 1.0;
const BRUSH_RADIUS_MAX: f32 = 64.0;
const BRIGHT_MIN: f32 = 0.3;
const BRIGHT_MAX: f32 = 1.6;
const PANEL_WIDTH: u32 = 88;
const UI_MARGIN: u32 = 6;
const UI_BUTTON_H: u32 = 20;
const UI_GAP: u32 = 6;
const SLIDER_H: u32 = 8;
const SLIDER_LABEL_W: u32 = 12;
const SLIDER_ICON_W: u32 = 10;
const SLIDER_KNOB_W: u32 = 12;
const PALETTE: [[u8; 4]; 8] = [
    [0, 0, 0, 255],       // Black
    [255, 0, 0, 255],     // Red
    [0, 128, 255, 255],   // Blue-ish
    [0, 180, 0, 255],     // Green
    [255, 200, 0, 255],   // Orange
    [255, 255, 0, 255],   // Yellow
    [255, 0, 255, 255],   // Magenta
    [255, 255, 255, 255], // White
];

fn window_to_canvas(
    pos: PhysicalPosition<f64>,
    window_size: PhysicalSize<u32>,
    canvas: &Canvas,
) -> Option<(f32, f32)> {
    if window_size.width == 0 || window_size.height == 0 {
        return None;
    }
    let x = (pos.x as f32) * canvas.width as f32 / window_size.width as f32;
    let y = (pos.y as f32) * canvas.height as f32 / window_size.height as f32;
    Some((x.clamp(0.0, (canvas.width - 1) as f32), y.clamp(0.0, (canvas.height - 1) as f32)))
}

fn draw_ui(canvas: &mut Canvas, brush: &Brush, brightness: f32, input: &InputState) {
    // Background
    canvas.fill_rect(0, 0, PANEL_WIDTH.min(canvas.width), canvas.height, [230, 230, 230, 255]);

    let x = UI_MARGIN;
    let w = (PANEL_WIDTH - UI_MARGIN * 2).max(1);

    // Palette buttons (more colors, smaller height)
    for (i, color) in PALETTE.iter().enumerate() {
        let y = UI_MARGIN + i as u32 * (UI_BUTTON_H + UI_GAP);
        canvas.fill_rect(x, y, w, UI_BUTTON_H, *color);
    }

    // Size slider (radius)
    let size_geom = size_slider_geom();
    draw_slider(canvas, size_geom, brush.radius, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX, 'S');
    let canvas_y = size_geom.row_y + size_geom.row_h + UI_GAP;

    // Canvas resize buttons (small / large)
    let small_color = [200, 200, 200, 255];
    let large_color = [120, 120, 120, 255];
    canvas.fill_rect(x, canvas_y, w / 2 - UI_GAP / 2, UI_BUTTON_H, small_color);
    canvas.fill_rect(x + w / 2 + UI_GAP / 2, canvas_y, w / 2 - UI_GAP / 2, UI_BUTTON_H, large_color);

    // Brightness slider
    let bright_geom = brightness_slider_geom();
    draw_slider(canvas, bright_geom, brightness, BRIGHT_MIN, BRIGHT_MAX, 'B');
    let preview_y = bright_geom.row_y + bright_geom.row_h + UI_GAP;

    // Brush preview bar
    let preview_w = (brush.radius * 2.0).min(w as f32) as u32;
    let preview_x = x + (w.saturating_sub(preview_w)) / 2;
    canvas.fill_rect(preview_x, preview_y, preview_w.max(4), UI_BUTTON_H / 2, brush.color);

    // Tool selection buttons - larger and more readable
    let tools_y = preview_y + UI_BUTTON_H / 2 + UI_GAP;
    let tool_btn_h = 24; // Taller buttons
    let tool_gap = 4; // More spacing
    
    let tools = [
        (input::Tool::Brush, "BRUSH"),
        (input::Tool::Eraser, "ERASER"),
        (input::Tool::FillBucket, "FILL"),
        (input::Tool::ColorPicker, "PICKER"),
        (input::Tool::Move, "MOVE"),
    ];
    
    let mut tool_y = tools_y;
    for (tool, name) in &tools {
        let is_active = input.current_tool == *tool;
        let btn_color = if is_active { [100, 150, 255, 255] } else { [180, 180, 180, 255] };
        canvas.fill_rect(x, tool_y, w, tool_btn_h, btn_color);
        draw_button_text(canvas, x + 6, tool_y + 7, name);
        tool_y += tool_btn_h + tool_gap;
    }

    // File operation buttons
    let file_buttons_y = tool_y + UI_GAP;
    let btn_w = (w - UI_GAP) / 2;
    let file_btn_color = [170, 170, 200, 255];
    
    // Import / Export row
    canvas.fill_rect(x, file_buttons_y, btn_w, UI_BUTTON_H, file_btn_color);
    canvas.fill_rect(x + btn_w + UI_GAP, file_buttons_y, btn_w, UI_BUTTON_H, file_btn_color);
    draw_button_text(canvas, x + 4, file_buttons_y + 6, "Import");
    draw_button_text(canvas, x + btn_w + UI_GAP + 4, file_buttons_y + 6, "Export");
    
    // Save / Open row
    let second_row_y = file_buttons_y + UI_BUTTON_H + UI_GAP;
    canvas.fill_rect(x, second_row_y, btn_w, UI_BUTTON_H, file_btn_color);
    canvas.fill_rect(x + btn_w + UI_GAP, second_row_y, btn_w, UI_BUTTON_H, file_btn_color);
    draw_button_text(canvas, x + 4, second_row_y + 6, "Save");
    draw_button_text(canvas, x + btn_w + UI_GAP + 4, second_row_y + 6, "Open");
    
    // Pan controls (if large image is loaded)
    if let Some((img_w, img_h)) = canvas.loaded_image_size {
        if img_w > canvas.width || img_h > canvas.height {
            // Show image info
            let info_y = second_row_y + UI_BUTTON_H + UI_GAP;
            draw_button_text(canvas, x + 2, info_y, &format!("{}x{}", img_w, img_h));
        }
    }
}

fn panel_hit_test(pos: (f32, f32), canvas: &Canvas) -> Option<PanelAction> {
    if pos.0 < 0.0 || pos.1 < 0.0 {
        return None;
    }
    let x = pos.0 as u32;
    let y = pos.1 as u32;
    if x >= PANEL_WIDTH || y >= canvas.height {
        return None;
    }

    let mut current_y = UI_MARGIN;
    let full_w = (PANEL_WIDTH - UI_MARGIN * 2).max(1);

    // Palette buttons
    for (i, _) in PALETTE.iter().enumerate() {
        if y >= current_y && y < current_y + UI_BUTTON_H {
            return Some(PanelAction::Color(i as u8));
        }
        current_y += UI_BUTTON_H + UI_GAP;
    }

    // Size slider
    let size_geom = size_slider_geom();
    if y >= size_geom.row_y && y < size_geom.row_y + size_geom.row_h && x >= size_geom.track_x && x < size_geom.track_x + size_geom.track_w {
        let value = slider_value_from_x(x as f32, size_geom, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX);
        return Some(PanelAction::SizeValue(value));
    }
    current_y = size_geom.row_y + size_geom.row_h + UI_GAP;

    // Canvas resize buttons
    let half_w = full_w / 2 - UI_GAP / 2;
    if y >= current_y && y < current_y + UI_BUTTON_H {
        let rel_x = x.saturating_sub(UI_MARGIN);
        if rel_x < half_w {
            return Some(PanelAction::CanvasSmaller);
        } else if rel_x > half_w + UI_GAP {
            return Some(PanelAction::CanvasLarger);
        }
    }
    // Brightness slider
    let bright_geom = brightness_slider_geom();
    if y >= bright_geom.row_y && y < bright_geom.row_y + bright_geom.row_h && x >= bright_geom.track_x && x < bright_geom.track_x + bright_geom.track_w {
        let value = slider_value_from_x(x as f32, bright_geom, BRIGHT_MIN, BRIGHT_MAX);
        return Some(PanelAction::Brightness(value));
    }

    // Tool selection buttons
    let preview_y = bright_geom.row_y + bright_geom.row_h + UI_GAP + UI_BUTTON_H / 2 + UI_GAP;
    let tools_y = preview_y;
    let tool_btn_h = 24;
    let tool_gap = 4;
    
    let tools = [
        input::Tool::Brush,
        input::Tool::Eraser,
        input::Tool::FillBucket,
        input::Tool::ColorPicker,
        input::Tool::Move,
    ];
    
    let mut tool_y = tools_y;
    for tool in &tools {
        if y >= tool_y && y < tool_y + tool_btn_h && x >= UI_MARGIN && x < PANEL_WIDTH - UI_MARGIN {
            return Some(PanelAction::Tool(*tool));
        }
        tool_y += tool_btn_h + tool_gap;
    }

    // File operation buttons
    let file_buttons_y = tool_y + UI_GAP;
    let btn_w = (full_w - UI_GAP) / 2;
    
    // Import / Export row
    if y >= file_buttons_y && y < file_buttons_y + UI_BUTTON_H {
        let rel_x = x.saturating_sub(UI_MARGIN);
        if rel_x < btn_w {
            return Some(PanelAction::FileImport);
        } else if rel_x > btn_w + UI_GAP {
            return Some(PanelAction::FileExport);
        }
    }
    
    // Save / Open row
    let second_row_y = file_buttons_y + UI_BUTTON_H + UI_GAP;
    if y >= second_row_y && y < second_row_y + UI_BUTTON_H {
        let rel_x = x.saturating_sub(UI_MARGIN);
        if rel_x < btn_w {
            return Some(PanelAction::FileSave);
        } else if rel_x > btn_w + UI_GAP {
            return Some(PanelAction::FileOpen);
        }
    }

    None
}

#[derive(Copy, Clone)]
struct SliderGeom {
    row_x: u32,
    row_y: u32,
    row_h: u32,
    track_x: u32,
    track_w: u32,
}

fn size_slider_geom() -> SliderGeom {
    let row_x = UI_MARGIN;
    let row_w = (PANEL_WIDTH - UI_MARGIN * 2).max(1);
    let row_y = UI_MARGIN + (UI_BUTTON_H + UI_GAP) * PALETTE.len() as u32;
    slider_geom(row_x, row_y, row_w)
}

fn brightness_slider_geom() -> SliderGeom {
    let row_x = UI_MARGIN;
    let row_w = (PANEL_WIDTH - UI_MARGIN * 2).max(1);
    let row_y = UI_MARGIN
        + (UI_BUTTON_H + UI_GAP) * PALETTE.len() as u32
        + (SLIDER_H + UI_GAP)
        + UI_BUTTON_H
        + UI_GAP;
    slider_geom(row_x, row_y, row_w)
}

fn slider_geom(row_x: u32, row_y: u32, row_w: u32) -> SliderGeom {
    let row_h = SLIDER_H;
    let track_x = row_x + SLIDER_LABEL_W + SLIDER_ICON_W;
    let track_w = row_w.saturating_sub(SLIDER_LABEL_W + SLIDER_ICON_W * 2).max(1);
    SliderGeom {
        row_x,
        row_y,
        row_h,
        track_x,
        track_w,
    }
}

fn slider_value_from_x(x: f32, geom: SliderGeom, min: f32, max: f32) -> f32 {
    let travel = (geom.track_w as f32 - SLIDER_KNOB_W as f32).max(1.0);
    let t = ((x - geom.track_x as f32) / travel).clamp(0.0, 1.0);
    min + t * (max - min)
}

fn size_value_from_x(x: f32) -> f32 {
    slider_value_from_x(x, size_slider_geom(), BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX)
}

fn brightness_value_from_x(x: f32) -> f32 {
    slider_value_from_x(x, brightness_slider_geom(), BRIGHT_MIN, BRIGHT_MAX)
}

fn draw_button_text(canvas: &mut Canvas, x: u32, y: u32, text: &str) {
    // Simple text drawing: draw characters as small pixel patterns
    let text_color = [0, 0, 0, 255];
    for (i, ch) in text.chars().enumerate() {
        let char_x = x + i as u32 * 6;
        draw_char(canvas, char_x, y, ch, text_color);
    }
}

fn draw_char(canvas: &mut Canvas, x: u32, y: u32, ch: char, color: [u8; 4]) {
    // 4x6 pixel character patterns
    match ch {
        'I' | 'i' => {
            canvas.fill_rect(x + 1, y, 2, 6, color);
        }
        'N' | 'n' => {
            canvas.fill_rect(x, y, 1, 6, color);
            canvas.fill_rect(x + 3, y, 1, 6, color);
            canvas.fill_rect(x + 1, y + 2, 2, 1, color);
        }
        'E' | 'e' => {
            canvas.fill_rect(x, y, 4, 1, color);
            canvas.fill_rect(x, y + 2, 4, 1, color);
            canvas.fill_rect(x, y + 5, 4, 1, color);
            canvas.fill_rect(x, y, 1, 6, color);
        }
        'X' | 'x' => {
            canvas.fill_rect(x, y, 1, 2, color);
            canvas.fill_rect(x + 3, y, 1, 2, color);
            canvas.fill_rect(x + 1, y + 2, 2, 2, color);
            canvas.fill_rect(x, y + 4, 1, 2, color);
            canvas.fill_rect(x + 3, y + 4, 1, 2, color);
        }
        'S' | 's' => {
            canvas.fill_rect(x + 1, y, 3, 1, color);
            canvas.fill_rect(x, y + 1, 2, 1, color);
            canvas.fill_rect(x + 1, y + 2, 3, 1, color);
            canvas.fill_rect(x + 2, y + 3, 2, 1, color);
            canvas.fill_rect(x + 1, y + 4, 3, 1, color);
        }
        'V' | 'v' => {
            canvas.fill_rect(x, y, 1, 4, color);
            canvas.fill_rect(x + 3, y, 1, 4, color);
            canvas.fill_rect(x + 1, y + 5, 2, 1, color);
        }
        'O' | 'o' => {
            canvas.fill_rect(x + 1, y, 2, 1, color);
            canvas.fill_rect(x + 1, y + 5, 2, 1, color);
            canvas.fill_rect(x, y, 1, 6, color);
            canvas.fill_rect(x + 3, y, 1, 6, color);
        }
        'P' | 'p' => {
            canvas.fill_rect(x, y, 1, 6, color);
            canvas.fill_rect(x + 1, y, 3, 1, color);
            canvas.fill_rect(x + 3, y + 1, 1, 1, color);
            canvas.fill_rect(x + 1, y + 2, 3, 1, color);
        }
        _ => {}
    }
}

fn draw_slider(canvas: &mut Canvas, geom: SliderGeom, value: f32, min: f32, max: f32, label: char) {
    let track_color = [200, 200, 200, 255];
    let knob_color = [60, 60, 60, 255];
    let icon_color = [30, 30, 30, 255];
    // Label glyph
    draw_glyph(canvas, geom.row_x, geom.row_y, label, icon_color);
    // Minus icon
    let minus_x = geom.row_x + SLIDER_LABEL_W;
    draw_minus_icon(canvas, minus_x, geom.row_y, icon_color);
    // Plus icon
    let plus_x = geom.track_x + geom.track_w;
    draw_plus_icon(canvas, plus_x.saturating_sub(SLIDER_ICON_W), geom.row_y, icon_color);

    // Track and knob
    canvas.fill_rect(geom.track_x, geom.row_y, geom.track_w, geom.row_h, track_color);
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let knob_travel = geom.track_w.saturating_sub(SLIDER_KNOB_W);
    let knob_x = geom.track_x + ((t * knob_travel as f32).round() as u32);
    canvas.fill_rect(knob_x, geom.row_y, SLIDER_KNOB_W.min(geom.track_w), geom.row_h, knob_color);
}

fn draw_glyph(canvas: &mut Canvas, x: u32, y: u32, ch: char, color: [u8; 4]) {
    // Simple 5x5 pixel glyphs for labels
    let pattern: [u8; 25] = match ch {
        'S' => [
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
        ],
        'B' => [
            1, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
        ],
        _ => [0; 25],
    };
    for row in 0..5u32 {
        for col in 0..5u32 {
            if pattern[(row * 5 + col) as usize] == 1 {
                canvas.fill_rect(x + col, y + row + 1, 1, 1, color);
            }
        }
    }
}

fn draw_minus_icon(canvas: &mut Canvas, x: u32, y: u32, color: [u8; 4]) {
    let pad = 2;
    canvas.fill_rect(x + pad, y + SLIDER_H / 2, SLIDER_ICON_W.saturating_sub(pad * 2), 1, color);
}

fn draw_plus_icon(canvas: &mut Canvas, x: u32, y: u32, color: [u8; 4]) {
    let pad = 2;
    let w = SLIDER_ICON_W.saturating_sub(pad * 2);
    let cx = x + pad + w / 2;
    canvas.fill_rect(x + pad, y + SLIDER_H / 2, w, 1, color);
    canvas.fill_rect(cx, y + 1, 1, SLIDER_H.saturating_sub(2), color);
}

enum PanelAction {
    Color(u8),
    SizeValue(f32),
    CanvasSmaller,
    CanvasLarger,
    Brightness(f32),
    FileImport,
    FileExport,
    FileSave,
    FileOpen,
    Tool(input::Tool),
}

fn handle_panel_action(
    action: PanelAction,
    input: &mut InputState,
    window_size: &mut PhysicalSize<u32>,
    gpu: &mut Gpu,
    canvas: &mut Canvas,
    window: &winit::window::Window,
) {
    match action {
        PanelAction::Color(idx) => {
            if let Some(color) = PALETTE.get(idx as usize) {
                input.set_brush_color(*color);
            }
        }
        PanelAction::SizeValue(v) => input.set_brush_radius(v, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
        PanelAction::CanvasSmaller => {
            let new_w = (window_size.width.max(1) as f32 * 0.75).round() as u32;
            let new_h = (window_size.height.max(1) as f32 * 0.75).round() as u32;
            *window_size = PhysicalSize::new(new_w.max(1), new_h.max(1));
            gpu.resize(*window_size);
            *canvas = Canvas::new(window_size.width.max(1), window_size.height.max(1));
            window.request_redraw();
        }
        PanelAction::CanvasLarger => {
            let new_w = (window_size.width.max(1) as f32 * 1.25).round() as u32;
            let new_h = (window_size.height.max(1) as f32 * 1.25).round() as u32;
            *window_size = PhysicalSize::new(new_w.max(1), new_h.max(1));
            gpu.resize(*window_size);
            *canvas = Canvas::new(window_size.width.max(1), window_size.height.max(1));
            window.request_redraw();
        }
        PanelAction::Brightness(value) => {
            input.set_brightness(value, BRIGHT_MIN, BRIGHT_MAX);
            window.request_redraw();
        }
        PanelAction::FileImport => {
            match io::select_image_file() {
                Ok(path) => {
                    match io::load_image(&path) {
                        Ok(img_layer) => {
                            canvas.pan_offset = (0, 0);
                            canvas.paste_image(img_layer.width, img_layer.height, &img_layer.pixels);
                            window.request_redraw();
                            println!("✓ Imported ({}x{}) - Use arrow keys to pan", img_layer.width, img_layer.height);
                        }
                        Err(e) => eprintln!("✗ Import failed: {}", e),
                    }
                }
                Err(e) => eprintln!("✗ {}", e),
            }
        }
        PanelAction::FileExport => {
            match io::select_export_png_path() {
                Ok(path) => {
                    match io::export_canvas_as_png(canvas, &path) {
                        Ok(_) => println!("✓ Exported"),
                        Err(e) => eprintln!("✗ Export failed: {}", e),
                    }
                }
                Err(e) => eprintln!("✗ {}", e),
            }
        }
        PanelAction::FileSave => {
            match io::select_save_project_folder() {
                Ok(path) => {
                    let layer = layer::Layer::from_rgba(
                        "canvas".to_string(),
                        canvas.width,
                        canvas.height,
                        canvas.extract_tight_pixels(),
                    );
                    let project_name = std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Project")
                        .to_string();
                    let mut project = layer::Project::new(project_name, canvas.width, canvas.height);
                    project.add_layer_metadata("canvas".to_string(), "layer_000.png".to_string());
                    
                    match io::save_project(&project, &[layer], &path) {
                        Ok(_) => println!("✓ Saved"),
                        Err(e) => eprintln!("✗ Save failed: {}", e),
                    }
                }
                Err(e) => eprintln!("✗ {}", e),
            }
        }
        PanelAction::FileOpen => {
            match io::select_load_project_folder() {
                Ok(path) => {
                    match io::load_project(&path) {
                        Ok((project, layers)) => {
                            if !layers.is_empty() && layers[0].width == canvas.width && layers[0].height == canvas.height {
                                canvas.load_pixels(layers[0].width, layers[0].height, layers[0].pixels.clone());
                                window.request_redraw();
                                println!("✓ Loaded: {}", project.name);
                            } else {
                                eprintln!("✗ Size mismatch");
                            }
                        }
                        Err(e) => eprintln!("✗ Load failed: {}", e),
                    }
                }
                Err(e) => eprintln!("✗ {}", e),
            }
        }
        PanelAction::Tool(tool) => {
            input.current_tool = tool;
            println!("Tool: {:?}", tool);
            window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut gpu: Option<Gpu> = None;
    let mut window_size: PhysicalSize<u32> = PhysicalSize::new(0, 0);
    let mut window: Option<Arc<winit::window::Window>> = None;
    let mut canvas: Option<Canvas> = None;
    let mut input = InputState::new(Brush {
        radius: BRUSH_RADIUS,
        color: BRUSH_COLOR,
    });

    event_loop
        .run(move |event, elwt| match event {
            Event::Resumed => {
                if gpu.is_none() {
                    let attrs = WindowAttributes::default()
                        .with_title("Pixel Editor")
                        .with_inner_size(LogicalSize::new(800.0, 600.0));
                    let w = Arc::new(elwt.create_window(attrs).unwrap());
                    let (g, s) = pollster::block_on(Gpu::new(&w));
                    window_size = s;
                    canvas = Some(Canvas::new(s.width.max(1), s.height.max(1)));
                    window = Some(w);
                    gpu = Some(g);
                }
            }

            Event::WindowEvent { event, window_id } => {
                if let (Some(g), Some(w), Some(c)) = (gpu.as_mut(), window.as_ref(), canvas.as_mut()) {
                    if window_id == w.id() {
                        match event {
                            WindowEvent::CloseRequested => elwt.exit(),
                            WindowEvent::Resized(new_size) => {
                                window_size = new_size;
                                g.resize(new_size);
                                
                                // Preserve old canvas pixels when resizing
                                let old_pixels = c.pixels.clone();
                                let old_width = c.width;
                                let old_height = c.height;
                                let old_stride = c.stride;
                                
                                // Create new canvas
                                *c = Canvas::new(new_size.width.max(1), new_size.height.max(1));
                                
                                // Copy old pixels to new canvas (preserve what fits)
                                let copy_width = old_width.min(c.width);
                                let copy_height = old_height.min(c.height);
                                for y in 0..copy_height {
                                    let old_row_offset = y as usize * old_stride;
                                    let new_row_offset = y as usize * c.stride;
                                    let row_bytes = copy_width as usize * 4;
                                    
                                    if old_row_offset + row_bytes <= old_pixels.len() 
                                        && new_row_offset + row_bytes <= c.pixels.len() {
                                        c.pixels[new_row_offset..new_row_offset + row_bytes]
                                            .copy_from_slice(&old_pixels[old_row_offset..old_row_offset + row_bytes]);
                                    }
                                }
                                c.dirty = true;
                                
                                w.request_redraw();
                            }
                            WindowEvent::KeyboardInput { event, .. } => {
                                // Track modifier keys
                                if let PhysicalKey::Code(code) = event.physical_key {
                                    match code {
                                        KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                                            input.shift_pressed = event.state == ElementState::Pressed;
                                        }
                                        KeyCode::ControlLeft | KeyCode::ControlRight => {
                                            input.ctrl_pressed = event.state == ElementState::Pressed;
                                        }
                                        _ => {}
                                    }
                                }
                                
                                let shift_pressed = input.shift_pressed;
                                let ctrl_pressed = input.ctrl_pressed;
                                if event.state == ElementState::Pressed {
                                    if let PhysicalKey::Code(code) = event.physical_key {
                                        match code {
                                            // Check zoom first (with shift modifier)
                                            KeyCode::PageUp | KeyCode::Equal if shift_pressed => {
                                                // Zoom in (Shift+= or Page Up)
                                                if c.loaded_image_size.is_some() {
                                                    c.zoom_scale = (c.zoom_scale * 1.25).min(5.0);
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                    println!("Zoom: {:.0}%", c.zoom_scale * 100.0);
                                                }
                                            }
                                            KeyCode::PageDown | KeyCode::Minus if shift_pressed => {
                                                // Zoom out (Shift+- or Page Down)
                                                if c.loaded_image_size.is_some() {
                                                    c.zoom_scale = (c.zoom_scale / 1.25).max(0.1);
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                    println!("Zoom: {:.0}%", c.zoom_scale * 100.0);
                                                }
                                            }
                                            KeyCode::Digit0 if shift_pressed => {
                                                // Reset zoom to 100% (Shift+0)
                                                if c.loaded_image_size.is_some() {
                                                    c.zoom_scale = 1.0;
                                                    c.pan_offset = (0, 0);
                                                    c.repan_image(0, 0);
                                                    w.request_redraw();
                                                    println!("Zoom: 100%");
                                                }
                                            }
                                            // Color palette selection
                                            KeyCode::Digit1 => input.set_brush_color(PALETTE[0]),
                                            KeyCode::Digit2 => input.set_brush_color(PALETTE[1]),
                                            KeyCode::Digit3 => input.set_brush_color(PALETTE[2]),
                                            KeyCode::Digit4 => input.set_brush_color(PALETTE[3]),
                                            // Brush size adjustments (without shift)
                                            KeyCode::Minus if !shift_pressed => input.adjust_brush_radius(-1.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::Equal if !shift_pressed => input.adjust_brush_radius(1.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::BracketLeft => input.adjust_brush_radius(-2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::BracketRight => input.adjust_brush_radius(2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::ArrowLeft => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_offset.0 += 50;
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                }
                                            }
                                            KeyCode::ArrowRight => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_offset.0 -= 50;
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                }
                                            }
                                            KeyCode::ArrowUp => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_offset.1 += 50;
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                }
                                            }
                                            KeyCode::ArrowDown => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_offset.1 -= 50;
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                }
                                            }
                                            KeyCode::KeyS => {
                                                let new_w = (window_size.width.max(1) as f32 * 0.75).round() as u32;
                                                let new_h = (window_size.height.max(1) as f32 * 0.75).round() as u32;
                                                window_size = PhysicalSize::new(new_w.max(1), new_h.max(1));
                                                g.resize(window_size);
                                                *c = Canvas::new(window_size.width.max(1), window_size.height.max(1));
                                                w.request_redraw();
                                            }
                                            KeyCode::KeyL => {
                                                let new_w = (window_size.width.max(1) as f32 * 1.25).round() as u32;
                                                let new_h = (window_size.height.max(1) as f32 * 1.25).round() as u32;
                                                window_size = PhysicalSize::new(new_w.max(1), new_h.max(1));
                                                g.resize(window_size);
                                                *c = Canvas::new(window_size.width.max(1), window_size.height.max(1));
                                                w.request_redraw();
                                            }
                                            // IO shortcuts (require Ctrl)
                                            KeyCode::KeyE if ctrl_pressed => {
                                                // Ctrl+E: Export canvas as PNG
                                                match io::select_export_png_path() {
                                                    Ok(path) => {
                                                        match io::export_canvas_as_png(c, &path) {
                                                            Ok(_) => {
                                                                let filename = std::path::Path::new(&path)
                                                                    .file_name()
                                                                    .and_then(|n| n.to_str())
                                                                    .unwrap_or("file");
                                                                println!("✓ Canvas exported to {}", filename);
                                                            }
                                                            Err(e) => eprintln!("✗ Export failed: {}", e),
                                                        }
                                                    }
                                                    Err(e) => eprintln!("✗ {}", e),
                                                }
                                            }
                                            KeyCode::KeyI if ctrl_pressed => {
                                                // Ctrl+I: Import PNG
                                                match io::select_image_file() {
                                                    Ok(path) => {
                                                        match io::load_image(&path) {
                                                            Ok(img_layer) => {
                                                                c.pan_offset = (0, 0);
                                                                c.paste_image(img_layer.width, img_layer.height, &img_layer.pixels);
                                                                w.request_redraw();
                                                                let filename = std::path::Path::new(&path)
                                                                    .file_name()
                                                                    .and_then(|n| n.to_str())
                                                                    .unwrap_or("image");
                                                                println!("✓ Imported {} - Use arrow keys to pan", filename);
                                                            }
                                                            Err(e) => eprintln!("✗ Import failed: {}", e),
                                                        }
                                                    }
                                                    Err(e) => eprintln!("✗ {}", e),
                                                }
                                            }
                                            KeyCode::KeyO if ctrl_pressed => {
                                                // Ctrl+O: Load project
                                                match io::select_load_project_folder() {
                                                    Ok(path) => {
                                                        match io::load_project(&path) {
                                                            Ok((project, layers)) => {
                                                                if !layers.is_empty() && layers[0].width == c.width && layers[0].height == c.height {
                                                                    c.load_pixels(layers[0].width, layers[0].height, layers[0].pixels.clone());
                                                                    w.request_redraw();
                                                                    println!("✓ Project loaded: {} ({} layers)", project.name, layers.len());
                                                                } else if layers.is_empty() {
                                                                    eprintln!("✗ Project has no layers");
                                                                } else {
                                                                    eprintln!("✗ Layer size mismatch");
                                                                }
                                                            }
                                                            Err(e) => eprintln!("✗ Load failed: {}", e),
                                                        }
                                                    }
                                                    Err(e) => eprintln!("✗ {}", e),
                                                }
                                            }
                                            KeyCode::KeyP if ctrl_pressed => {
                                                // Ctrl+P: Save project
                                                match io::select_save_project_folder() {
                                                    Ok(path) => {
                                                        let layer = layer::Layer::from_rgba(
                                                            "canvas".to_string(),
                                                            c.width,
                                                            c.height,
                                                            c.extract_tight_pixels(),
                                                        );
                                                        let project_name = std::path::Path::new(&path)
                                                            .file_name()
                                                            .and_then(|n| n.to_str())
                                                            .unwrap_or("Project")
                                                            .to_string();
                                                        let mut project = layer::Project::new(project_name, c.width, c.height);
                                                        project.add_layer_metadata("canvas".to_string(), "layer_000.png".to_string());
                                                        
                                                        match io::save_project(&project, &[layer], &path) {
                                                            Ok(_) => {
                                                                let folder_name = std::path::Path::new(&path)
                                                                    .file_name()
                                                                    .and_then(|n| n.to_str())
                                                                    .unwrap_or("project");
                                                                println!("✓ Project saved to {}/", folder_name);
                                                            }
                                                            Err(e) => eprintln!("✗ Save failed: {}", e),
                                                        }
                                                    }
                                                    Err(e) => eprintln!("✗ {}", e),
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                                if state == ElementState::Pressed {
                                    if let Some(pos) = input.last_pos {
                                        if let Some(action) = panel_hit_test(pos, c) {
                                            if matches!(action, PanelAction::Brightness(_)) {
                                                input.set_slider_drag(Some(SliderDrag::Brightness));
                                            } else if matches!(action, PanelAction::SizeValue(_)) {
                                                input.set_slider_drag(Some(SliderDrag::Size));
                                            }
                                            handle_panel_action(action, &mut input, &mut window_size, g, c, w);
                                            input.stop_drawing();
                                            return;
                                        }
                                        if pos.0 >= PANEL_WIDTH as f32 {
                                            // Handle different tools
                                            match input.current_tool {
                                                input::Tool::Brush | input::Tool::Eraser => {
                                                    input.drawing = true;
                                                }
                                                input::Tool::FillBucket => {
                                                    let canvas_x = (pos.0 - PANEL_WIDTH as f32).max(0.0) as u32;
                                                    let canvas_y = pos.1 as u32;
                                                    c.flood_fill(canvas_x, canvas_y, input.brush.color);
                                                    w.request_redraw();
                                                }
                                                input::Tool::ColorPicker => {
                                                    let canvas_x = (pos.0 - PANEL_WIDTH as f32).max(0.0) as u32;
                                                    let canvas_y = pos.1 as u32;
                                                    if let Some(color) = c.get_pixel(canvas_x, canvas_y) {
                                                        input.set_brush_color(color);
                                                        println!("Picked color: {:?}", color);
                                                    }
                                                    w.request_redraw();
                                                }
                                                input::Tool::Move => {
                                                    input.drawing = true;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Mouse released
                                    if input.current_tool == input::Tool::Move && input.drawing {
                                        // Apply move if we dragged

                                    }
                                    input.set_slider_drag(None);
                                    input.stop_drawing();
                                    input.selection_start = None;
                                    input.selection_end = None;
                                }
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                if let Some(p) = window_to_canvas(position, window_size, c) {
                                    let prev = input.last_pos;
                                    input.last_pos = Some(p);
                                    if let Some(target) = input.slider_dragging {
                                        match target {
                                            SliderDrag::Brightness => {
                                                let value = brightness_value_from_x(p.0);
                                                input.set_brightness(value, BRIGHT_MIN, BRIGHT_MAX);
                                            }
                                            SliderDrag::Size => {
                                                let value = size_value_from_x(p.0);
                                                input.set_brush_radius(value, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX);
                                            }
                                        }
                                        w.request_redraw();
                                        return;
                                    }
                                    if input.drawing {
                                        if p.0 < PANEL_WIDTH as f32 {
                                            input.stop_drawing();
                                            return;
                                        }
                                        
                                        match input.current_tool {
                                            input::Tool::Brush => {
                                                if let Some(last) = prev {
                                                    input.brush.stroke(c, last, p);
                                                } else {
                                                    input.brush.stamp(c, p);
                                                }
                                                w.request_redraw();
                                            }
                                            input::Tool::Eraser => {
                                                // Eraser directly sets pixels to transparent
                                                c.erase_circle((p.0 - PANEL_WIDTH as f32).max(0.0), p.1, input.brush.radius);
                                                if let Some(last) = prev {
                                                    // Draw line of eraser stamps
                                                    let dist = ((p.0 - last.0).powi(2) + (p.1 - last.1).powi(2)).sqrt();
                                                    let steps = (dist / (input.brush.radius / 2.0)).ceil().max(1.0) as i32;
                                                    for i in 0..=steps {
                                                        let t = i as f32 / steps as f32;
                                                        let ix = last.0 + (p.0 - last.0) * t;
                                                        let iy = last.1 + (p.1 - last.1) * t;
                                                        c.erase_circle((ix - PANEL_WIDTH as f32).max(0.0), iy, input.brush.radius);
                                                    }
                                                }
                                                w.request_redraw();
                                            }
                                            input::Tool::Move => {
                                                if let Some(last) = prev {
                                                    let dx = ((p.0 - last.0) / c.zoom_scale) as i32;
                                                    let dy = ((p.1 - last.1) / c.zoom_scale) as i32;
                                                    if dx != 0 || dy != 0 {
                                                        c.move_layer(dx, dy);
                                                        w.request_redraw();
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            WindowEvent::RedrawRequested => {
                                draw_ui(c, &input.brush, input.brightness, &input);
                                
                                if let Err(e) = g.render(c) {
                                    match e {
                                        wgpu::SurfaceError::Lost => {
                                            g.resize(window_size);
                                            c.dirty = true;
                                        }
                                        wgpu::SurfaceError::OutOfMemory => elwt.exit(),
                                        other => eprintln!("{other:?}"),
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            Event::AboutToWait => {
                if let (Some(w), Some(c)) = (window.as_ref(), canvas.as_ref()) {
                    if c.dirty {
                        w.request_redraw();
                    }
                }
            }

            _ => {}
        })
        .unwrap();
}
