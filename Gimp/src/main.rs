mod brush;
mod canvas;
mod gpu;
mod input;

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
    input::InputState,
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

fn draw_ui(canvas: &mut Canvas, brush: &Brush, brightness: f32) {
    // Background
    canvas.fill_rect(0, 0, PANEL_WIDTH.min(canvas.width), canvas.height, [230, 230, 230, 255]);

    let mut y = UI_MARGIN;
    let x = UI_MARGIN;
    let w = (PANEL_WIDTH - UI_MARGIN * 2).max(1);

    // Palette buttons (more colors, smaller height)
    for color in PALETTE {
        canvas.fill_rect(x, y, w, UI_BUTTON_H, color);
        y += UI_BUTTON_H + UI_GAP;
    }

    // Size buttons (- and +) simple grayscale cues
    let minus_color = [180, 180, 180, 255];
    let plus_color = [140, 140, 140, 255];
    canvas.fill_rect(x, y, w / 2 - UI_GAP / 2, UI_BUTTON_H, minus_color);
    canvas.fill_rect(x + w / 2 + UI_GAP / 2, y, w / 2 - UI_GAP / 2, UI_BUTTON_H, plus_color);
    y += UI_BUTTON_H + UI_GAP;

    // Canvas resize buttons (small / large)
    let small_color = [200, 200, 200, 255];
    let large_color = [120, 120, 120, 255];
    canvas.fill_rect(x, y, w / 2 - UI_GAP / 2, UI_BUTTON_H, small_color);
    canvas.fill_rect(x + w / 2 + UI_GAP / 2, y, w / 2 - UI_GAP / 2, UI_BUTTON_H, large_color);

    // Brightness slider area
    let slider_track_color = [200, 200, 200, 255];
    let (sx, sy, sw, _) = slider_rect();
    canvas.fill_rect(sx, sy, sw, SLIDER_H, slider_track_color);
    let knob_color = [60, 60, 60, 255];
    let knob_w: u32 = 12;
    let t = ((brightness - BRIGHT_MIN) / (BRIGHT_MAX - BRIGHT_MIN)).clamp(0.0, 1.0);
    let knob_x = sx + ((t * (sw.saturating_sub(knob_w)).max(0) as f32).round() as u32);
    canvas.fill_rect(knob_x, sy, knob_w.min(sw), SLIDER_H, knob_color);
    y = sy + SLIDER_H + UI_GAP;

    // Brush preview bar
    let preview_w = (brush.radius * 2.0).min(w as f32) as u32;
    let preview_x = x + (w.saturating_sub(preview_w)) / 2;
    canvas.fill_rect(preview_x, y, preview_w.max(4), UI_BUTTON_H / 2, brush.color);
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

    // Size buttons area
    let half_w = full_w / 2 - UI_GAP / 2;
    if y >= current_y && y < current_y + UI_BUTTON_H {
        let rel_x = x.saturating_sub(UI_MARGIN);
        if rel_x < half_w {
            return Some(PanelAction::SizeDown);
        } else if rel_x > half_w + UI_GAP {
            return Some(PanelAction::SizeUp);
        }
    }
    current_y += UI_BUTTON_H + UI_GAP;

    // Canvas resize buttons
    if y >= current_y && y < current_y + UI_BUTTON_H {
        let rel_x = x.saturating_sub(UI_MARGIN);
        if rel_x < half_w {
            return Some(PanelAction::CanvasSmaller);
        } else if rel_x > half_w + UI_GAP {
            return Some(PanelAction::CanvasLarger);
        }
    }
    // Brightness slider
    let (sx, sy, sw, sh) = slider_rect();
    if y >= sy && y < sy + sh && x >= sx && x < sx + sw {
        let value = brightness_from_x(x as f32);
        return Some(PanelAction::Brightness(value));
    }

    None
}

fn slider_rect() -> (u32, u32, u32, u32) {
    let mut y = UI_MARGIN;
    y += (UI_BUTTON_H + UI_GAP) * PALETTE.len() as u32; // palette rows
    y += UI_BUTTON_H + UI_GAP; // size buttons
    y += UI_BUTTON_H + UI_GAP; // canvas resize
    let x = UI_MARGIN;
    let w = (PANEL_WIDTH - UI_MARGIN * 2).max(1);
    (x, y, w, SLIDER_H)
}

fn brightness_from_x(x: f32) -> f32 {
    let (sx, _, sw, _) = slider_rect();
    let width = (sw as f32 - 1.0).max(1.0);
    let t = ((x - sx as f32) / width).clamp(0.0, 1.0);
    BRIGHT_MIN + t * (BRIGHT_MAX - BRIGHT_MIN)
}

enum PanelAction {
    Color(u8),
    SizeUp,
    SizeDown,
    CanvasSmaller,
    CanvasLarger,
    Brightness(f32),
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
        PanelAction::SizeUp => input.adjust_brush_radius(2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
        PanelAction::SizeDown => input.adjust_brush_radius(-2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
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
                                *c = Canvas::new(new_size.width.max(1), new_size.height.max(1));
                                w.request_redraw();
                            }
                            WindowEvent::KeyboardInput { event, .. } => {
                                if event.state == ElementState::Pressed {
                                    if let PhysicalKey::Code(code) = event.physical_key {
                                        match code {
                                            KeyCode::Digit1 => input.set_brush_color(PALETTE[0]),
                                            KeyCode::Digit2 => input.set_brush_color(PALETTE[1]),
                                            KeyCode::Digit3 => input.set_brush_color(PALETTE[2]),
                                            KeyCode::Digit4 => input.set_brush_color(PALETTE[3]),
                                            KeyCode::Minus => input.adjust_brush_radius(-1.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::Equal => input.adjust_brush_radius(1.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::BracketLeft => input.adjust_brush_radius(-2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::BracketRight => input.adjust_brush_radius(2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
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
                                                input.set_slider_drag(true);
                                            }
                                            handle_panel_action(action, &mut input, &mut window_size, g, c, w);
                                            input.stop_drawing();
                                            return;
                                        }
                                        if pos.0 >= PANEL_WIDTH as f32 {
                                            input.drawing = true;
                                        }
                                    }
                                } else {
                                    input.set_slider_drag(false);
                                    input.stop_drawing();
                                }
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                if let Some(p) = window_to_canvas(position, window_size, c) {
                                    let prev = input.last_pos;
                                    input.last_pos = Some(p);
                                    if input.slider_dragging {
                                        let value = brightness_from_x(p.0);
                                        input.set_brightness(value, BRIGHT_MIN, BRIGHT_MAX);
                                        w.request_redraw();
                                        return;
                                    }
                                    if input.drawing {
                                        if p.0 < PANEL_WIDTH as f32 {
                                            input.stop_drawing();
                                            return;
                                        }
                                        if let Some(last) = prev {
                                            input.brush.stroke(c, last, p);
                                        } else {
                                            input.brush.stamp(c, p);
                                        }
                                        w.request_redraw();
                                    }
                                }
                            }
                            WindowEvent::RedrawRequested => {
                                draw_ui(c, &input.brush, input.brightness);
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
