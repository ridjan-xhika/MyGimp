mod brush;
mod brush_dynamics;
mod canvas;
mod gpu;
mod input;
mod layer;
mod io;
mod icons;
mod history;
mod filters;
mod selection;
mod viewport;
mod theme;

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
    history::History,
};

const BRUSH_COLOR: [u8; 4] = [0, 0, 0, 255];
const BRUSH_RADIUS: f32 = 6.0;
const BRUSH_RADIUS_MIN: f32 = 1.0;
const BRUSH_RADIUS_MAX: f32 = 64.0;
const BRIGHT_MIN: f32 = 0.3;
const BRIGHT_MAX: f32 = 1.6;
const TOOLBAR_HEIGHT: u32 = 64;
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

fn ensure_selection_dimensions(input: &mut InputState, canvas: &Canvas) {
    if input.selection.width != canvas.width || input.selection.height != canvas.height {
        input.selection = crate::selection::Selection::new(canvas.width, canvas.height);
    }
}

fn draw_ui(canvas: &mut Canvas, brush: &Brush, _brightness: f32, input: &InputState, icons: &crate::icons::IconCache) {
    // Top toolbar background (dark)
    canvas.fill_rect(0, 0, canvas.width, TOOLBAR_HEIGHT, [50, 50, 50, 255]);
    
    // Left side panel background (light gray)
    canvas.fill_rect(0, TOOLBAR_HEIGHT, PANEL_WIDTH, canvas.height - TOOLBAR_HEIGHT, [220, 220, 220, 255]);
    
    // Toolbar: Tool buttons with icons
    let tool_gap = 6;
    let tool_size = 40;
    let mut tool_x = 8;
    let tool_y = 8;
    
    // Tool buttons with selection highlight
    let tools = [
        (input::Tool::Brush, &icons.brush, Some("B")),
        (input::Tool::Eraser, &icons.eraser, Some("E")),
        (input::Tool::FillBucket, &icons.fill, Some("F")),
        (input::Tool::ColorPicker, &icons.picker, Some("P")),
        (input::Tool::Move, &icons.move_tool, Some("M")),
        (input::Tool::Blur, &icons.blur, Some("U")),
        (input::Tool::SelectRect, &icons.select_rect, Some("R")),
        (input::Tool::SelectEllipse, &icons.select_ellipse, Some("O")),
        (input::Tool::SelectLasso, &icons.select_lasso, Some("L")),
        (input::Tool::SelectMove, &icons.move_tool, Some("SM")),
        (input::Tool::ShapeRect, &icons.shape_rect, Some("SR")),
        (input::Tool::ShapeEllipse, &icons.shape_ellipse, Some("SE")),
        (input::Tool::ShapeLine, &icons.shape_line, Some("SL")),
    ];
    
    for (tool, icon, label) in &tools {
        let is_active = input.current_tool == *tool;
        let btn_color = if is_active { [100, 150, 255, 255] } else { [80, 80, 80, 255] };
        let border_color = [200, 200, 200, 255];
        
        canvas.fill_rect(tool_x, tool_y, tool_size, tool_size, btn_color);
        // Simple border
        canvas.fill_rect(tool_x, tool_y, tool_size, 2, border_color);
        canvas.fill_rect(tool_x, tool_y, 2, tool_size, border_color);
        
        // Draw icon centered in the button
        if !icon.pixels.is_empty() {
            let icon_display_size = (tool_size - 8).max(20);
            let icon_x = tool_x + (tool_size - icon_display_size) / 2;
            let icon_y = tool_y + (tool_size - icon_display_size) / 2;
            draw_icon(canvas, icon, icon_x, icon_y, icon_display_size);
        }
        // Fallback/overlay label for clarity when icons are reused
        if let Some(text) = label {
            let text_w = (text.len() as u32 * 6).min(tool_size);
            let text_x = tool_x + (tool_size.saturating_sub(text_w)) / 2;
            let text_y = tool_y + tool_size / 2 - 3;
            draw_button_text(canvas, text_x, text_y, text);
        }
        tool_x += tool_size + tool_gap;
    }
    
    // Right side of toolbar: File operations with icons + filters
    let file_x = canvas.width.saturating_sub(240);
    let file_btns = [
        (file_x, &icons.import, None),           // Import
        (file_x + 30, &icons.export, None),      // Export
        (file_x + 60, &icons.save, None),        // Save
        (file_x + 90, &icons.brightness, None),  // Brightness filter
        (file_x + 120, &icons.invert, None),     // Invert filter
        (file_x + 150, &icons.grayscale, None),  // Grayscale filter (toggle)
        (file_x + 180, &icons.brightness, Some("X")), // Remove brightness (marked)
        (file_x + 210, &icons.grayscale, Some("X")),  // Remove grayscale (marked)
    ];
    
    for (x, icon, label) in &file_btns {
        canvas.fill_rect(*x, tool_y, 24, tool_size, [100, 100, 100, 255]);
        if !icon.pixels.is_empty() {
            let icon_display_size = 20;
            let icon_x_pos = *x + (24 - icon_display_size) / 2;
            let icon_y_pos = tool_y + (tool_size - icon_display_size) / 2;
            draw_icon(canvas, icon, icon_x_pos, icon_y_pos, icon_display_size);
        }
        if let Some(text) = label {
            draw_button_text(canvas, *x + 8, tool_y + 16, text);
        }
    }
    
    // New feature buttons (bottom right of toolbar)
    let new_btns_x = canvas.width.saturating_sub(180);
    let new_btns = [
        (new_btns_x, "U", [150, 80, 80, 255]),       // Undo
        (new_btns_x + 30, "R", [150, 80, 80, 255]),  // Redo
        (new_btns_x + 60, "+", [80, 150, 80, 255]),  // Zoom In
        (new_btns_x + 90, "-", [80, 150, 80, 255]),  // Zoom Out
        (new_btns_x + 120, "F", [80, 150, 80, 255]), // Zoom Fit
        (new_btns_x + 150, "T", [80, 80, 150, 255]), // Theme Toggle
    ];
    
    for (x, label, color) in &new_btns {
        canvas.fill_rect(*x, tool_y, 24, tool_size, *color);
        draw_button_text(canvas, *x + 6, tool_y + 16, label);
    }
    
    // Left panel: Colors, Size, Filters
    let panel_x = 8;
    let panel_y = TOOLBAR_HEIGHT + 8;
    
    // Foreground / Background swatches (click either to open HSV picker)
    // Foreground (front, larger)
    canvas.fill_rect(panel_x, panel_y, 24, 24, input.brush.color);
    let border = [30, 30, 30, 255];
    canvas.fill_rect(panel_x, panel_y, 24, 1, border);
    canvas.fill_rect(panel_x, panel_y, 1, 24, border);
    canvas.fill_rect(panel_x + 23, panel_y, 1, 24, border);
    canvas.fill_rect(panel_x, panel_y + 23, 24, 1, border);
    // Background (behind, smaller)
    canvas.fill_rect(panel_x + 26, panel_y + 4, 22, 22, input.bg_color);
    canvas.fill_rect(panel_x + 26, panel_y + 4, 22, 1, border);
    canvas.fill_rect(panel_x + 26, panel_y + 4, 1, 22, border);
    canvas.fill_rect(panel_x + 26 + 21, panel_y + 4, 1, 22, border);
    canvas.fill_rect(panel_x + 26, panel_y + 4 + 21, 22, 1, border);
    // Active swatch highlight (white border)
    if input.active_is_foreground {
        canvas.fill_rect(panel_x.saturating_sub(2), panel_y.saturating_sub(2), 28, 2, [255, 255, 255, 255]);
        canvas.fill_rect(panel_x.saturating_sub(2), panel_y + 24, 28, 2, [255, 255, 255, 255]);
        canvas.fill_rect(panel_x.saturating_sub(2), panel_y.saturating_sub(2), 2, 28, [255, 255, 255, 255]);
        canvas.fill_rect(panel_x + 26, panel_y.saturating_sub(2), 2, 28, [255, 255, 255, 255]);
    } else {
        let bx = panel_x + 26;
        let by = panel_y + 4;
        canvas.fill_rect(bx.saturating_sub(2), by.saturating_sub(2), 26, 2, [255, 255, 255, 255]);
        canvas.fill_rect(bx.saturating_sub(2), by + 22, 26, 2, [255, 255, 255, 255]);
        canvas.fill_rect(bx.saturating_sub(2), by.saturating_sub(2), 2, 26, [255, 255, 255, 255]);
        canvas.fill_rect(bx + 22, by.saturating_sub(2), 2, 26, [255, 255, 255, 255]);
    }

    // Color palette
    let mut col_y = panel_y + 30;
    for (i, color) in PALETTE.iter().enumerate() {
        if i >= 4 { break; }
        canvas.fill_rect(panel_x, col_y, PANEL_WIDTH - 16, 20, *color);
        col_y += 24;
    }
    
    // Brush size display
    let size_y = col_y + 4;
    draw_button_text(canvas, panel_x, size_y, &format!("SIZE:{:.0}", brush.radius));
    
    // Brush preview
    let preview_w = (brush.radius * 2.0).min((PANEL_WIDTH - 16) as f32) as u32;
    let preview_x = panel_x + ((PANEL_WIDTH - 16).saturating_sub(preview_w)) / 2;
    canvas.fill_rect(preview_x, size_y + 16, preview_w.max(4), 12, brush.color);

    // Brush size slider under toolbar icons
    let slider_y = TOOLBAR_HEIGHT.saturating_sub(10);
    let slider_x = 8;
    let slider_w = 280u32;
    let track_color = [100, 100, 100, 255];
    canvas.fill_rect(slider_x, slider_y, slider_w, 6, track_color);
    let t = ((brush.radius - BRUSH_RADIUS_MIN) / (BRUSH_RADIUS_MAX - BRUSH_RADIUS_MIN)).clamp(0.0, 1.0);
    let knob_x = slider_x + (t * slider_w as f32).round() as u32;
    canvas.fill_rect(knob_x.saturating_sub(3), slider_y.saturating_sub(2), 6, 10, [200, 200, 200, 255]);

    // Advanced color picker UI (Hue bar + SV square)
    if input.show_color_picker {
        // Geometry
        let hue_x = panel_x;
        let hue_y = size_y + 36;
        let hue_w = 14u32; // slightly wider like GIMP
        let hue_h = 120u32; // taller for smoother gradient
        let sv_x = hue_x + hue_w + 6;
        let sv_y = hue_y;
        let sv_w = (PANEL_WIDTH - 16).saturating_sub(hue_w + 6);
        let sv_h = sv_w;

        // Draw hue bar (fast)
        draw_hue_bar_fast(canvas, hue_x, hue_y, hue_w, hue_h);
        // Hue bar border
        canvas.fill_rect(hue_x.saturating_sub(1), hue_y.saturating_sub(1), hue_w + 2, 1, [0, 0, 0, 255]);
        canvas.fill_rect(hue_x.saturating_sub(1), hue_y + hue_h, hue_w + 2, 1, [0, 0, 0, 255]);
        canvas.fill_rect(hue_x.saturating_sub(1), hue_y.saturating_sub(1), 1, hue_h + 2, [0, 0, 0, 255]);
        canvas.fill_rect(hue_x + hue_w, hue_y.saturating_sub(1), 1, hue_h + 2, [0, 0, 0, 255]);
        // Hue indicator
        let hue_pos = (input.hue.clamp(0.0, 1.0) * hue_h as f32).round() as u32;
        canvas.fill_rect(hue_x, hue_y + hue_pos.saturating_sub(1), hue_w, 2, [255, 255, 255, 255]);

        // Draw SV square for current hue (fast)
        draw_sv_square_fast(canvas, sv_x, sv_y, sv_w, sv_h, input.hue);
        // SV square border
        canvas.fill_rect(sv_x.saturating_sub(1), sv_y.saturating_sub(1), sv_w + 2, 1, [0, 0, 0, 255]);
        canvas.fill_rect(sv_x.saturating_sub(1), sv_y + sv_h, sv_w + 2, 1, [0, 0, 0, 255]);
        canvas.fill_rect(sv_x.saturating_sub(1), sv_y.saturating_sub(1), 1, sv_h + 2, [0, 0, 0, 255]);
        canvas.fill_rect(sv_x + sv_w, sv_y.saturating_sub(1), 1, sv_h + 2, [0, 0, 0, 255]);
        // SV indicator
        let sv_ix = (input.sat.clamp(0.0, 1.0) * sv_w as f32).round() as u32;
        let sv_iy = ((1.0 - input.val.clamp(0.0, 1.0)) * sv_h as f32).round() as u32;
        // Crosshair indicator similar to GIMP
        let cx = sv_x + sv_ix;
        let cy = sv_y + sv_iy;
        // small cross in white with black shadow for visibility
        canvas.fill_rect(cx.saturating_sub(6), cy, 12, 1, [255, 255, 255, 255]);
        canvas.fill_rect(cx, cy.saturating_sub(6), 1, 12, [255, 255, 255, 255]);
        canvas.fill_rect(cx.saturating_sub(6), cy.saturating_sub(1), 12, 1, [0, 0, 0, 255]);
        canvas.fill_rect(cx.saturating_sub(1), cy.saturating_sub(6), 1, 12, [0, 0, 0, 255]);
    }
    
    // No filter intensity UI; simple toolbar-only filters
    

fn hsv_to_rgb_u8(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = (h * 6.0) % 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let (r, g, b) = match i as i32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

fn draw_hue_bar_fast(canvas: &mut Canvas, x: u32, y: u32, w: u32, h: u32) {
    for iy in 0..h {
        let hh = iy as f32 / h as f32;
        let rgb = hsv_to_rgb_u8(hh, 1.0, 1.0);
        let row = (y + iy) as usize * canvas.stride;
        for ix in 0..w {
            let idx = row + (x + ix) as usize * 4;
            if idx + 3 <= canvas.pixels.len() {
                canvas.pixels[idx] = rgb[0];
                canvas.pixels[idx + 1] = rgb[1];
                canvas.pixels[idx + 2] = rgb[2];
                canvas.pixels[idx + 3] = 255;
            }
        }
    }
}

fn draw_sv_square_fast(canvas: &mut Canvas, x: u32, y: u32, w: u32, h: u32, hue: f32) {
    for iy in 0..h {
        let v = 1.0 - (iy as f32 / h as f32);
        let row = (y + iy) as usize * canvas.stride;
        for ix in 0..w {
            let s = ix as f32 / w as f32;
            let rgb = hsv_to_rgb_u8(hue, s, v);
            let idx = row + (x + ix) as usize * 4;
            if idx + 3 <= canvas.pixels.len() {
                canvas.pixels[idx] = rgb[0];
                canvas.pixels[idx + 1] = rgb[1];
                canvas.pixels[idx + 2] = rgb[2];
                canvas.pixels[idx + 3] = 255;
            }
        }
    }
}
    // Status bar at bottom - only in the left panel area, not over the canvas
    let status_bar_height = 20;
    let status_bar_y = canvas.height.saturating_sub(status_bar_height);
    
    // Draw status bar only in the left panel (first PANEL_WIDTH pixels)
    if status_bar_y < canvas.height {
        canvas.fill_rect(0, status_bar_y, PANEL_WIDTH.min(canvas.width), status_bar_height, [80, 80, 80, 255]);
    }
    
    // Display coordinates and color under cursor (in panel only)
    if let Some(pos) = input.last_pos {
        // Only show status if cursor is in the panel area, not over canvas
        if pos.0 < PANEL_WIDTH as f32 && status_bar_y < canvas.height {
            let status_text = format!("Panel Status");
            draw_button_text(canvas, 8, status_bar_y + 2, &status_text);
        }
    }
}

fn panel_hit_test(pos: (f32, f32), canvas: &Canvas) -> Option<PanelAction> {
    if pos.0 < 0.0 || pos.1 < 0.0 {
        return None;
    }
    let x = pos.0 as u32;
    let y = pos.1 as u32;
    
    // Toolbar hit test
    if y < TOOLBAR_HEIGHT {
        // Tools
        let tool_gap = 6;
        let tool_size = 40;
        let mut tool_x = 8;
        let tool_y = 8;
        
        let tools = [
            input::Tool::Brush,
            input::Tool::Eraser,
            input::Tool::FillBucket,
            input::Tool::ColorPicker,
            input::Tool::Move,
            input::Tool::Blur,
            input::Tool::SelectRect,
            input::Tool::SelectEllipse,
            input::Tool::SelectLasso,
            input::Tool::SelectMove,
            input::Tool::ShapeRect,
            input::Tool::ShapeEllipse,
            input::Tool::ShapeLine,
        ];
        
        for tool in &tools {
            if x >= tool_x && x < tool_x + tool_size && y >= tool_y && y < tool_y + tool_size {
                return Some(PanelAction::Tool(*tool));
            }
            tool_x += tool_size + tool_gap;
        }
        
        // File operations + filters in toolbar
        let file_x = canvas.width.saturating_sub(240);
        if x >= file_x && x < canvas.width {
            let rel_x = x - file_x;
            if rel_x < 24 { return Some(PanelAction::FileImport); }
            else if rel_x < 54 { return Some(PanelAction::FileExport); }
            else if rel_x < 84 { return Some(PanelAction::FileSave); }
            else if rel_x < 114 { return Some(PanelAction::FilterBrightness); }
            else if rel_x < 144 { return Some(PanelAction::FilterInvert); }
            else if rel_x < 174 { return Some(PanelAction::FilterGrayscale); }
            else if rel_x < 204 { return Some(PanelAction::RemoveBrightness); }
            else if rel_x < 234 { return Some(PanelAction::RemoveGrayscale); }
        }
        
        // New feature buttons (bottom right of toolbar)
        let new_btns_x = canvas.width.saturating_sub(180);
        if x >= new_btns_x && x < canvas.width {
            let rel_x = x - new_btns_x;
            if rel_x < 24 { return Some(PanelAction::HistoryUndo); }
            else if rel_x < 54 { return Some(PanelAction::HistoryRedo); }
            else if rel_x < 84 { return Some(PanelAction::ZoomIn); }
            else if rel_x < 114 { return Some(PanelAction::ZoomOut); }
            else if rel_x < 144 { return Some(PanelAction::ZoomFit); }
            else if rel_x < 174 { return Some(PanelAction::ThemeToggle); }
        }

        // Brush size slider
        let slider_y = TOOLBAR_HEIGHT.saturating_sub(10);
        let slider_x = 8;
        let slider_w = 280u32;
        if y >= slider_y && y < slider_y + 6 && x >= slider_x && x < slider_x + slider_w {
            let t = (x.saturating_sub(slider_x)) as f32 / slider_w as f32;
            let value = BRUSH_RADIUS_MIN + t * (BRUSH_RADIUS_MAX - BRUSH_RADIUS_MIN);
            return Some(PanelAction::SizeValue(value));
        }
        
        return None;
    }
    
    // Side panel hit test
    if x >= PANEL_WIDTH {
        return None;
    }
    
    let panel_x = 8;
    let panel_y = TOOLBAR_HEIGHT + 8;
    
    // Foreground swatch click
    if y >= panel_y && y < panel_y + 24 && x >= panel_x && x < panel_x + 24 {
        return Some(PanelAction::OpenColorPickerForeground);
    }
    // Background swatch click
    let bg_x = panel_x + 26;
    let bg_y = panel_y + 4;
    if y >= bg_y && y < bg_y + 22 && x >= bg_x && x < bg_x + 22 {
        return Some(PanelAction::OpenColorPickerBackground);
    }
    
    // Color palette
    for (i, _) in PALETTE.iter().enumerate() {
        if i >= 4 { break; }
        let row_y = panel_y + 30 + i as u32 * 24;
        if y >= row_y && y < row_y + 20 && x >= panel_x && x < panel_x + PANEL_WIDTH - 16 {
            return Some(PanelAction::Color(i as u8));
        }
    }

    // Color picker interactions
    // Geometry mirrored from draw_ui() EXACTLY
    // Palette rows: start at panel_y + 30, 4 rows, each adds 24
    let size_y = (panel_y + 30) + 24 * 4 + 4; // col_y after palette + 4
    let hue_x = panel_x;
    let hue_y = size_y + 36;
    let hue_w = 14u32;
    let hue_h = 120u32;
    let sv_x = hue_x + hue_w + 6;
    let sv_y = hue_y;
    let sv_w = (PANEL_WIDTH - 16).saturating_sub(hue_w + 6);
    let sv_h = sv_w;
    // Hue bar
    if y >= hue_y && y < hue_y + hue_h && x >= hue_x && x < hue_x + hue_w {
        let hh = (y - hue_y) as f32 / hue_h as f32;
        return Some(PanelAction::PickerHue(hh));
    }
    // SV square
    if y >= sv_y && y < sv_y + sv_h && x >= sv_x && x < sv_x + sv_w {
        let s = (x - sv_x) as f32 / sv_w as f32;
        let v = 1.0 - (y - sv_y) as f32 / sv_h as f32;
        return Some(PanelAction::PickerSV(s, v));
    }
    
    None
}

fn size_value_from_x(x: f32) -> f32 {
    // Map canvas X to slider percentage using actual slider geometry
    let slider_x = 8.0;
    let slider_w = 280.0;
    let t = ((x - slider_x) / slider_w).clamp(0.0, 1.0);
    BRUSH_RADIUS_MIN + t * (BRUSH_RADIUS_MAX - BRUSH_RADIUS_MIN)
}

fn brightness_value_from_x(x: f32) -> f32 {
    // Map canvas X to brightness using same slider geometry for consistency
    let slider_x = 8.0;
    let slider_w = 280.0;
    let t = ((x - slider_x) / slider_w).clamp(0.0, 1.0);
    BRIGHT_MIN + t * (BRIGHT_MAX - BRIGHT_MIN)
}

fn draw_icon(canvas: &mut Canvas, icon: &crate::icons::Icon, x: u32, y: u32, size: u32) {
    if icon.pixels.is_empty() || icon.width == 0 || icon.height == 0 {
        return;
    }
    let src_w = icon.width as f32;
    let src_h = icon.height as f32;
    // Iterate destination pixels only (fast), sample source with nearest neighbor
    for dy in 0..size {
        let screen_y = y + dy;
        if screen_y >= canvas.height { break; }
        let row = screen_y as usize * canvas.stride;
        for dx in 0..size {
            let screen_x = x + dx;
            if screen_x >= canvas.width { break; }
            let sx = ((dx as f32 / size as f32) * src_w).floor() as u32;
            let sy = ((dy as f32 / size as f32) * src_h).floor() as u32;
            let sidx = ((sy * icon.width + sx) * 4) as usize;
            if sidx + 3 >= icon.pixels.len() { continue; }
            let a = icon.pixels[sidx + 3];
            if a < 10 { continue; }
            let dest_idx = row + screen_x as usize * 4;
            canvas.pixels[dest_idx] = icon.pixels[sidx];
            canvas.pixels[dest_idx + 1] = icon.pixels[sidx + 1];
            canvas.pixels[dest_idx + 2] = icon.pixels[sidx + 2];
            canvas.pixels[dest_idx + 3] = 255;
        }
    }
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

enum PanelAction {
    Color(u8),
    SizeValue(f32),
    CanvasSmaller,
    CanvasLarger,
    FileImport,
    FileExport,
    FileSave,
    FileOpen,
    Tool(input::Tool),
    FilterInvert,
    FilterGrayscale,
    FilterBrightness,
    RemoveGrayscale,
    RemoveBrightness,
    FilterBlur,
    ToggleColorPicker,
    OpenColorPickerForeground,
    OpenColorPickerBackground,
    PickerHue(f32),
    PickerSV(f32, f32),
    // New actions
    ZoomIn,
    ZoomOut,
    ZoomFit,
    ZoomActual,
    HistoryUndo,
    HistoryRedo,
    ThemeToggle,
}

fn handle_panel_action(
    action: PanelAction,
    input: &mut InputState,
    window_size: &mut PhysicalSize<u32>,
    gpu: &mut Gpu,
    canvas: &mut Canvas,
    window: &winit::window::Window,
    history: &mut History,
) {
    match action {
        PanelAction::Color(idx) => {
            if let Some(color) = PALETTE.get(idx as usize) {
                if input.active_is_foreground {
                    input.set_brush_color(*color);
                } else {
                    input.set_background_color(*color);
                }
            }
        }
        PanelAction::SizeValue(v) => input.set_brush_radius(v, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
        // No filter intensity sliders; filters are applied from toolbar buttons
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
        // No brightness slider action; brightness filter is applied via toolbar button
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
                    let image_pixels = canvas.extract_image_pixels();
                    let (width, height) = canvas.loaded_image_size.unwrap_or((canvas.width, canvas.height));
                    let layer = layer::Layer::from_rgba(
                        "canvas".to_string(),
                        width,
                        height,
                        image_pixels,
                    );
                    let project_name = std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Project")
                        .to_string();
                    let mut project = layer::Project::new(project_name, width, height);
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
        PanelAction::FilterInvert => {
            canvas.filter_invert();
            history.push(canvas, "Invert filter".to_string());
            window.request_redraw();
            println!("✓ Applied Invert filter");
        }
        PanelAction::FilterGrayscale => {
            canvas.filter_grayscale();
            history.push(canvas, "Grayscale filter".to_string());
            window.request_redraw();
            println!("✓ Applied Grayscale filter");
        }
        PanelAction::FilterBrightness => {
            canvas.filter_brightness_contrast(30.0, 20.0);
            history.push(canvas, "Brightness filter".to_string());
            window.request_redraw();
            println!("✓ Applied Brightness filter");
        }
        PanelAction::RemoveBrightness => {
            canvas.remove_brightness();
            history.push(canvas, "Remove brightness".to_string());
            window.request_redraw();
            println!("✓ Removed Brightness");
        }
        PanelAction::RemoveGrayscale => {
            canvas.remove_grayscale();
            history.push(canvas, "Remove grayscale".to_string());
            window.request_redraw();
            println!("✓ Removed Grayscale");
        }
        PanelAction::FilterBlur => {
            canvas.filter_blur(2);
            history.push(canvas, "Blur filter".to_string());
            window.request_redraw();
            println!("✓ Applied Blur filter");
        }
        PanelAction::ToggleColorPicker => {
            input.toggle_color_picker();
            window.request_redraw();
        }
        PanelAction::OpenColorPickerForeground => {
            input.open_color_picker_foreground();
            window.request_redraw();
        }
        PanelAction::OpenColorPickerBackground => {
            input.open_color_picker_background();
            window.request_redraw();
        }
        PanelAction::PickerHue(h) => {
            input.set_hsv(h, input.sat, input.val);
            window.request_redraw();
        }
        PanelAction::PickerSV(s, v) => {
            input.set_hsv(input.hue, s, v);
            window.request_redraw();
        }
        PanelAction::ZoomIn => {
            let center_x = window_size.width as f32 / 2.0;
            let center_y = window_size.height as f32 / 2.0;
            input.view_state.zoom_in(center_x, center_y);
            window.request_redraw();
        }
        PanelAction::ZoomOut => {
            let center_x = window_size.width as f32 / 2.0;
            let center_y = window_size.height as f32 / 2.0;
            input.view_state.zoom_out(center_x, center_y);
            window.request_redraw();
        }
        PanelAction::ZoomFit => {
            input.view_state.fit_to_window(
                window_size.width as f32,
                window_size.height as f32,
                canvas.width,
                canvas.height
            );
            window.request_redraw();
        }
        PanelAction::ZoomActual => {
            input.view_state.actual_size();
            window.request_redraw();
        }
        PanelAction::HistoryUndo => {
            if history.can_undo() {
                history.undo(canvas);
                window.request_redraw();
                println!("✓ Undo");
            }
        }
        PanelAction::HistoryRedo => {
            if history.can_redo() {
                history.redo(canvas);
                window.request_redraw();
                println!("✓ Redo");
            }
        }
        PanelAction::ThemeToggle => {
            input.theme_config.toggle_theme();
            window.request_redraw();
            println!("✓ Theme: {:?}", input.theme_config.current_theme);
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
    input.open_color_picker_foreground();  // Show color picker by default
    
    // Load icons at startup
    let icons = crate::icons::IconCache::load();
    
    // Initialize history
    let mut history = History::new();

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
                                            // Zoom with Ctrl+Plus/Minus (new hotkeys)
                                            KeyCode::Equal if ctrl_pressed && !shift_pressed => {
                                                // Ctrl+=: Zoom in
                                                if c.loaded_image_size.is_some() {
                                                    c.zoom_scale = (c.zoom_scale * 1.25).min(5.0);
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                    println!("Zoom in: {:.0}%", c.zoom_scale * 100.0);
                                                }
                                            }
                                            KeyCode::Minus if ctrl_pressed && !shift_pressed => {
                                                // Ctrl+-: Zoom out
                                                if c.loaded_image_size.is_some() {
                                                    c.zoom_scale = (c.zoom_scale / 1.25).max(0.1);
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                    println!("Zoom out: {:.0}%", c.zoom_scale * 100.0);
                                                }
                                            }
                                            // Check zoom with shift modifier (legacy)
                                            KeyCode::PageUp | KeyCode::Equal if shift_pressed && !ctrl_pressed => {
                                                // Zoom in (Shift+= or Page Up)
                                                if c.loaded_image_size.is_some() {
                                                    c.zoom_scale = (c.zoom_scale * 1.25).min(5.0);
                                                    c.repan_image(c.pan_offset.0, c.pan_offset.1);
                                                    w.request_redraw();
                                                    println!("Zoom: {:.0}%", c.zoom_scale * 100.0);
                                                }
                                            }
                                            KeyCode::PageDown | KeyCode::Minus if shift_pressed && !ctrl_pressed => {
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
                                            // Brush size adjustments (only when ctrl and shift not pressed)
                                            KeyCode::Minus if !shift_pressed && !ctrl_pressed => input.adjust_brush_radius(-1.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::Equal if !shift_pressed && !ctrl_pressed => input.adjust_brush_radius(1.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::BracketLeft => input.adjust_brush_radius(-2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            KeyCode::BracketRight => input.adjust_brush_radius(2.0, BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX),
                                            // Undo/Redo shortcuts
                                            KeyCode::KeyZ if ctrl_pressed && !shift_pressed => {
                                                // Ctrl+Z: Undo
                                                if let Some(c) = canvas.as_mut() {
                                                    if history.undo(c) {
                                                        w.request_redraw();
                                                        println!("↶ Undo");
                                                    }
                                                }
                                            }
                                            KeyCode::KeyZ if ctrl_pressed && shift_pressed => {
                                                // Ctrl+Shift+Z: Redo
                                                if let Some(c) = canvas.as_mut() {
                                                    if history.redo(c) {
                                                        w.request_redraw();
                                                        println!("↷ Redo");
                                                    }
                                                }
                                            }
                                            // Filter shortcuts
                                            KeyCode::KeyG if ctrl_pressed => {
                                                // Ctrl+G: Grayscale
                                                c.filter_grayscale();
                                                history.push(c, "Grayscale (shortcut)".to_string());
                                                w.request_redraw();
                                                println!("✓ Applied Grayscale filter");
                                            }
                                            KeyCode::KeyB if ctrl_pressed && shift_pressed => {
                                                // Ctrl+Shift+B: Brightness/Contrast
                                                c.filter_brightness_contrast(30.0, 20.0);
                                                history.push(c, "Brightness/Contrast (shortcut)".to_string());
                                                w.request_redraw();
                                                println!("✓ Applied Brightness/Contrast filter");
                                            }
                                            KeyCode::KeyU if ctrl_pressed => {
                                                // Ctrl+U: Blur
                                                c.filter_blur(2);
                                                history.push(c, "Blur (shortcut)".to_string());
                                                w.request_redraw();
                                                println!("✓ Applied Blur filter");
                                            }
                                            KeyCode::KeyT if ctrl_pressed => {
                                                // Ctrl+T: Toggle Theme
                                                input.theme_config.toggle_theme();
                                                w.request_redraw();
                                                println!("✓ Theme: {:?}", input.theme_config.current_theme);
                                            }
                                            KeyCode::KeyR if !ctrl_pressed => {
                                                // R: Rectangle Select Tool
                                                input.current_tool = input::Tool::SelectRect;
                                                w.request_redraw();
                                            }
                                            KeyCode::KeyE if !ctrl_pressed => {
                                                // E: Ellipse Select Tool
                                                input.current_tool = input::Tool::SelectEllipse;
                                                w.request_redraw();
                                            }
                                            KeyCode::KeyL if !ctrl_pressed => {
                                                // L: Lasso Select Tool
                                                input.current_tool = input::Tool::SelectLasso;
                                                w.request_redraw();
                                            }
                                            KeyCode::KeyV if !ctrl_pressed => {
                                                // V: Shape Rectangle Tool
                                                input.current_tool = input::Tool::ShapeRect;
                                                w.request_redraw();
                                            }
                                            KeyCode::KeyC if !ctrl_pressed => {
                                                // C: Shape Ellipse Tool
                                                input.current_tool = input::Tool::ShapeEllipse;
                                                w.request_redraw();
                                            }
                                            KeyCode::KeyN if !ctrl_pressed => {
                                                // N: Shape Line Tool
                                                input.current_tool = input::Tool::ShapeLine;
                                                w.request_redraw();
                                            }
                                            KeyCode::KeyQ if !ctrl_pressed => {
                                                // Q: Select & Move Tool
                                                input.current_tool = input::Tool::SelectMove;
                                                w.request_redraw();
                                            }
                                            // Pan/zoom controls
                                            KeyCode::ArrowLeft => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_image(50, 0);
                                                    w.request_redraw();
                                                }
                                            }
                                            KeyCode::ArrowRight => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_image(-50, 0);
                                                    w.request_redraw();
                                                }
                                            }
                                            KeyCode::ArrowUp => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_image(0, 50);
                                                    w.request_redraw();
                                                }
                                            }
                                            KeyCode::ArrowDown => {
                                                if c.loaded_image_size.is_some() {
                                                    c.pan_image(0, -50);
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
                                            KeyCode::KeyI if ctrl_pressed && shift_pressed => {
                                                // Ctrl+Shift+I: Invert filter
                                                c.filter_invert();
                                                w.request_redraw();
                                                println!("✓ Applied Invert filter");
                                            }
                                            KeyCode::KeyI if ctrl_pressed && !shift_pressed => {
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
                                                        let image_pixels = c.extract_image_pixels();
                                                        let (width, height) = c.loaded_image_size.unwrap_or((c.width, c.height));
                                                        let layer = layer::Layer::from_rgba(
                                                            "canvas".to_string(),
                                                            width,
                                                            height,
                                                            image_pixels,
                                                        );
                                                        let project_name = std::path::Path::new(&path)
                                                            .file_name()
                                                            .and_then(|n| n.to_str())
                                                            .unwrap_or("Project")
                                                            .to_string();
                                                        let mut project = layer::Project::new(project_name, width, height);
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
                                            if matches!(action, PanelAction::SizeValue(_)) {
                                                input.set_slider_drag(Some(SliderDrag::Size));
                                            } else if matches!(action, PanelAction::PickerHue(_)) {
                                                input.set_color_drag(Some(input::ColorPickerDrag::Hue));
                                            } else if matches!(action, PanelAction::PickerSV(_, _)) {
                                                input.set_color_drag(Some(input::ColorPickerDrag::SV));
                                            }
                                            handle_panel_action(action, &mut input, &mut window_size, g, c, w, &mut history);
                                            input.stop_drawing();
                                            return;
                                        }
                                        if pos.0 >= PANEL_WIDTH as f32 {
                                            // Handle different tools
                                            match input.current_tool {
                                                input::Tool::Brush | input::Tool::Eraser | input::Tool::Blur => {
                                                    input.drawing = true;
                                                }
                                                input::Tool::SelectRect | input::Tool::SelectEllipse | input::Tool::SelectLasso => {
                                                    // Transform screen coordinates to canvas coordinates
                                                    let canvas_x = ((pos.0 - PANEL_WIDTH as f32) / input.view_state.zoom + input.view_state.pan_x).max(0.0) as u32;
                                                    let canvas_y = ((pos.1 - TOOLBAR_HEIGHT as f32) / input.view_state.zoom + input.view_state.pan_y).max(0.0) as u32;
                                                    input.selection_start = Some((canvas_x, canvas_y));
                                                    input.selection_end = Some((canvas_x, canvas_y));
                                                    input.lasso_points.clear();
                                                    if matches!(input.current_tool, input::Tool::SelectLasso) {
                                                        input.lasso_points.push((canvas_x as f32, canvas_y as f32));
                                                    }
                                                    input.drawing = true;
                                                }
                                                input::Tool::ShapeRect | input::Tool::ShapeEllipse | input::Tool::ShapeLine => {
                                                    input.shape_start = Some(pos);
                                                    input.shape_end = Some(pos);
                                                    input.drawing = true;
                                                    w.request_redraw();
                                                }
                                                input::Tool::SelectMove => {
                                                    input.move_region_start = Some(pos);
                                                    input.move_region_end = Some(pos);
                                                    input.drawing = true;
                                                    w.request_redraw();
                                                }
                                                input::Tool::FillBucket => {
                                                    if pos.0 >= PANEL_WIDTH as f32 && pos.1 >= TOOLBAR_HEIGHT as f32 {
                                                        let canvas_x = ((pos.0 - PANEL_WIDTH as f32) / input.view_state.zoom + input.view_state.pan_x).max(0.0) as u32;
                                                        let canvas_y = ((pos.1 - TOOLBAR_HEIGHT as f32) / input.view_state.zoom + input.view_state.pan_y).max(0.0) as u32;
                                                        c.flood_fill(canvas_x, canvas_y, input.brush.color);
                                                        history.push(c, "Flood fill".to_string());
                                                        w.request_redraw();
                                                    }
                                                }
                                                input::Tool::ColorPicker => {
                                                    if pos.0 >= PANEL_WIDTH as f32 && pos.1 >= TOOLBAR_HEIGHT as f32 {
                                                        // Transform screen coordinates to canvas coordinates with zoom/pan
                                                        let canvas_x = ((pos.0 - PANEL_WIDTH as f32) / input.view_state.zoom + input.view_state.pan_x).max(0.0) as u32;
                                                        let canvas_y = ((pos.1 - TOOLBAR_HEIGHT as f32) / input.view_state.zoom + input.view_state.pan_y).max(0.0) as u32;
                                                        if let Some(color) = c.get_pixel(canvas_x, canvas_y) {
                                                            if input.active_is_foreground {
                                                                input.set_brush_color(color);
                                                            } else {
                                                                input.set_background_color(color);
                                                            }
                                                            println!("Picked color: {:?}", color);
                                                        }
                                                        w.request_redraw();
                                                    }
                                                }
                                                input::Tool::Move => {
                                                    input.drawing = true;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Mouse released - save to history after drawing
                                    if input.drawing {
                                        let mut pushed_history = false;
                                        match input.current_tool {
                                            input::Tool::SelectRect | input::Tool::SelectEllipse => {
                                                if let Some(start) = input.selection_start {
                                                    let end = input.selection_end.unwrap_or(start);
                                                    ensure_selection_dimensions(&mut input, c);
                                                    match input.current_tool {
                                                        input::Tool::SelectRect => {
                                                            input.selection.select_rectangle(
                                                                start.0,
                                                                start.1,
                                                                end.0,
                                                                end.1,
                                                                input.selection_mode,
                                                            );
                                                        }
                                                        input::Tool::SelectEllipse => {
                                                            let cx = (start.0 as f32 + end.0 as f32) / 2.0;
                                                            let cy = (start.1 as f32 + end.1 as f32) / 2.0;
                                                            let rx = ((start.0 as i64 - end.0 as i64).abs() as f32 / 2.0).max(1.0);
                                                            let ry = ((start.1 as i64 - end.1 as i64).abs() as f32 / 2.0).max(1.0);
                                                            input.selection.select_ellipse(cx, cy, rx, ry, input.selection_mode);
                                                        }
                                                        _ => {}
                                                    }
                                                    history.push(c, "Selection".to_string());
                                                    pushed_history = true;
                                                    w.request_redraw();
                                                }
                                            }
                                            input::Tool::SelectLasso => {
                                                if !input.lasso_points.is_empty() {
                                                    ensure_selection_dimensions(&mut input, c);
                                                    input.selection.select_lasso(&input.lasso_points, input.selection_mode);
                                                    println!("✓ Lasso selection created");
                                                    w.request_redraw();
                                                }
                                            }
                                            input::Tool::SelectMove => {
                                                if let (Some(start), Some(end)) = (input.move_region_start, input.move_region_end) {
                                                    let x1 = start.0.min(end.0) as u32;
                                                    let y1 = start.1.min(end.1) as u32;
                                                    let x2 = start.0.max(end.0) as u32;
                                                    let y2 = start.1.max(end.1) as u32;
                                                    let w_region = x2.saturating_sub(x1).max(1);
                                                    let h_region = y2.saturating_sub(y1).max(1);
                                                    
                                                    // If we have a buffer, paste it at the new location
                                                    if let Some((src_x, src_y, src_w, src_h, buffer)) = input.move_region_buffer.take() {
                                                        // Center the region at cursor position (not top-left at cursor)
                                                        let cursor_x = end.0 as i32;
                                                        let cursor_y = end.1 as i32;
                                                        let offset_x = (src_w as i32) / 2;
                                                        let offset_y = (src_h as i32) / 2;
                                                        let dest_x = (cursor_x - offset_x).max(0) as u32;
                                                        let dest_y = (cursor_y - offset_y).max(0) as u32;
                                                        
                                                        // Erase original region (fill with white)
                                                        for y in src_y..src_y + src_h {
                                                            let offset = y as usize * c.stride + src_x as usize * 4;
                                                            let len = src_w as usize * 4;
                                                            if offset + len <= c.pixels.len() {
                                                                for i in (0..len).step_by(4) {
                                                                    c.pixels[offset + i] = 255;     // R
                                                                    c.pixels[offset + i + 1] = 255; // G
                                                                    c.pixels[offset + i + 2] = 255; // B
                                                                    c.pixels[offset + i + 3] = 255; // A
                                                                }
                                                            }
                                                        }
                                                        
                                                        // Paste at destination (centered at cursor)
                                                        let mut buf_cursor = 0;
                                                        for y in dest_y..(dest_y + src_h).min(c.height) {
                                                            let offset = y as usize * c.stride + dest_x as usize * 4;
                                                            let len = src_w as usize * 4;
                                                            if offset + len <= c.pixels.len() && buf_cursor + len <= buffer.len() {
                                                                c.pixels[offset..offset + len].copy_from_slice(&buffer[buf_cursor..buf_cursor + len]);
                                                            }
                                                            buf_cursor += len;
                                                        }
                                                        
                                                        // Clear move region tracking for next operation
                                                        input.move_region_start = None;
                                                        input.move_region_end = None;
                                                        
                                                        history.push(c, "Move region".to_string());
                                                        pushed_history = true;
                                                        println!("✓ Region moved from ({}, {}) to ({}, {})", src_x, src_y, dest_x, dest_y);
                                                        w.request_redraw();
                                                    } else {
                                                        // First selection: copy region to buffer
                                                        let mut region_data = Vec::with_capacity(w_region as usize * h_region as usize * 4);
                                                        for y in y1..=(y1 + h_region).min(c.height.saturating_sub(1)) {
                                                            let offset = y as usize * c.stride + x1 as usize * 4;
                                                            let len = w_region as usize * 4;
                                                            if offset + len <= c.pixels.len() {
                                                                region_data.extend_from_slice(&c.pixels[offset..offset + len]);
                                                            }
                                                        }
                                                        
                                                        input.move_region_buffer = Some((x1, y1, w_region, h_region, region_data));
                                                        println!("✓ Region copied at ({}, {}) size {}x{} - drag SelectMove to paste", x1, y1, w_region, h_region);
                                                    }
                                                }
                                            }
                                            input::Tool::ShapeRect | input::Tool::ShapeEllipse | input::Tool::ShapeLine => {
                                                if let (Some(start), Some(end)) = (input.shape_start, input.shape_end) {
                                                    let thickness = input.brush.radius.max(1.0);
                                                    let color = input.brush.color;
                                                    match input.current_tool {
                                                        input::Tool::ShapeRect => {
                                                            c.draw_rect_outline(start, end, thickness, color);
                                                        }
                                                        input::Tool::ShapeEllipse => {
                                                            c.draw_ellipse_outline(start, end, thickness, color);
                                                        }
                                                        input::Tool::ShapeLine => {
                                                            c.draw_line(start, end, thickness, color);
                                                        }
                                                        _ => {}
                                                    }
                                                    // Clear shape state
                                                    input.shape_start = None;
                                                    input.shape_end = None;
                                                    history.push(c, "Shape".to_string());
                                                    pushed_history = true;
                                                    w.request_redraw();
                                                }
                                            }
                                            input::Tool::Move => {
                                                // Move handled during drag; nothing extra on release
                                            }
                                            _ => {
                                                history.push(c, "Drawing".to_string());
                                                pushed_history = true;
                                            }
                                        }
                                        if !pushed_history {
                                            history.push(c, "Drawing".to_string());
                                        }
                                    }
                                    input.set_slider_drag(None);
                                    input.set_color_drag(None);
                                    input.stop_drawing();
                                    input.selection_start = None;
                                    input.selection_end = None;
                                    input.lasso_points.clear();
                                    input.shape_start = None;
                                    input.shape_end = None;
                                    input.move_region_start = None;
                                    input.move_region_end = None;
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
                                            SliderDrag::BlurRadius => {
                                                let value = (p.0.clamp(8.0, 280.0) - 8.0) / 272.0 * 15.0 + 1.0;
                                                input.filter_params.blur_radius = value.clamp(1.0, 16.0);
                                            }
                                            SliderDrag::SharpenStrength => {
                                                let value = (p.0.clamp(8.0, 280.0) - 8.0) / 272.0 * 2.0;
                                                input.filter_params.sharpen_strength = value.clamp(0.0, 2.0);
                                            }
                                            SliderDrag::BrightnessFX => {
                                                let value = (p.0.clamp(8.0, 280.0) - 8.0) / 272.0 * 2.0 - 1.0;
                                                input.filter_params.brightness = value.clamp(-1.0, 1.0);
                                            }
                                            SliderDrag::Contrast => {
                                                let value = (p.0.clamp(8.0, 280.0) - 8.0) / 272.0 * 1.5 + 0.5;
                                                input.filter_params.contrast = value.clamp(0.5, 2.0);
                                            }
                                            SliderDrag::Saturation => {
                                                let value = (p.0.clamp(8.0, 280.0) - 8.0) / 272.0 * 2.0;
                                                input.filter_params.saturation = value.clamp(0.0, 2.0);
                                            }
                                        }
                                        w.request_redraw();
                                        return;
                                    }
                                    if let Some(cp_drag) = input.color_dragging {
                                        // Geometry: same as panel_hit_test/draw_ui
                                        let panel_x = 8.0;
                                        let panel_y = TOOLBAR_HEIGHT as f32 + 8.0;
                                        let size_y = (panel_y + 30.0) + 24.0 * 4.0 + 4.0;
                                        let hue_x = panel_x;
                                        let hue_y = size_y + 36.0;
                                        let hue_w = 14.0;
                                        let hue_h = 120.0;
                                        let sv_x = hue_x + hue_w + 6.0;
                                        let sv_y = hue_y;
                                        let sv_w = (PANEL_WIDTH as f32 - 16.0) - (hue_w + 6.0);
                                        let sv_h = sv_w;
                                        match cp_drag {
                                            input::ColorPickerDrag::Hue => {
                                                // Clamp to hue bar and update hue
                                                let yy = p.1.clamp(hue_y, hue_y + hue_h);
                                                let hh = (yy - hue_y) / hue_h;
                                                input.set_hsv(hh, input.sat, input.val);
                                            }
                                            input::ColorPickerDrag::SV => {
                                                let xx = p.0.clamp(sv_x, sv_x + sv_w);
                                                let yy = p.1.clamp(sv_y, sv_y + sv_h);
                                                let s = (xx - sv_x) / sv_w;
                                                let v = 1.0 - (yy - sv_y) / sv_h;
                                                input.set_hsv(input.hue, s, v);
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
                                            input::Tool::SelectRect | input::Tool::SelectEllipse => {
                                                // Update selection end in canvas coordinates while dragging
                                                let canvas_x = ((p.0 - PANEL_WIDTH as f32) / input.view_state.zoom + input.view_state.pan_x).max(0.0) as u32;
                                                let canvas_y = ((p.1 - TOOLBAR_HEIGHT as f32) / input.view_state.zoom + input.view_state.pan_y).max(0.0) as u32;
                                                input.selection_end = Some((canvas_x, canvas_y));
                                            }
                                            input::Tool::SelectLasso => {
                                                let canvas_x = ((p.0 - PANEL_WIDTH as f32) / input.view_state.zoom + input.view_state.pan_x).max(0.0) as u32;
                                                let canvas_y = ((p.1 - TOOLBAR_HEIGHT as f32) / input.view_state.zoom + input.view_state.pan_y).max(0.0) as u32;
                                                let point = (canvas_x as f32, canvas_y as f32);
                                                if input.lasso_points.last().map_or(true, |last| {
                                                    (last.0 - point.0).abs() > 0.5 || (last.1 - point.1).abs() > 0.5
                                                }) {
                                                    input.lasso_points.push(point);
                                                }
                                            }
                                            input::Tool::ShapeRect | input::Tool::ShapeEllipse | input::Tool::ShapeLine => {
                                                input.shape_end = Some(p);
                                                w.request_redraw();
                                            }
                                            input::Tool::SelectMove => {
                                                input.move_region_end = Some(p);
                                                w.request_redraw();
                                            }
                                            input::Tool::Brush => {
                                                // Block drawing in UI regions
                                                if p.0 >= PANEL_WIDTH as f32 && p.1 >= TOOLBAR_HEIGHT as f32 {
                                                    if let Some(last) = prev {
                                                        input.brush.stroke(c, last, p);
                                                    } else {
                                                        input.brush.stamp(c, p);
                                                    }
                                                    w.request_redraw();
                                                }
                                            }
                                            input::Tool::Eraser => {
                                                if p.0 >= PANEL_WIDTH as f32 && p.1 >= TOOLBAR_HEIGHT as f32 {
                                                    c.erase_circle(p.0, p.1, input.brush.radius);
                                                    if let Some(last) = prev {
                                                        let dist = ((p.0 - last.0).powi(2) + (p.1 - last.1).powi(2)).sqrt();
                                                        let steps = (dist / (input.brush.radius / 2.0)).ceil().max(1.0) as i32;
                                                        for i in 0..=steps {
                                                            let t = i as f32 / steps as f32;
                                                            let ix = last.0 + (p.0 - last.0) * t;
                                                            let iy = last.1 + (p.1 - last.1) * t;
                                                            c.erase_circle(ix, iy, input.brush.radius);
                                                        }
                                                    }
                                                    w.request_redraw();
                                                }
                                            }
                                            input::Tool::Blur => {
                                                if p.0 >= PANEL_WIDTH as f32 && p.1 >= TOOLBAR_HEIGHT as f32 {
                                                    c.blur_circle(p.0, p.1, input.brush.radius);
                                                    if let Some(last) = prev {
                                                        let dist = ((p.0 - last.0).powi(2) + (p.1 - last.1).powi(2)).sqrt();
                                                        let steps = (dist / (input.brush.radius / 2.0)).ceil().max(1.0) as i32;
                                                        for i in 0..=steps {
                                                            let t = i as f32 / steps as f32;
                                                            let ix = last.0 + (p.0 - last.0) * t;
                                                            let iy = last.1 + (p.1 - last.1) * t;
                                                            c.blur_circle(ix, iy, input.brush.radius);
                                                        }
                                                    }
                                                    w.request_redraw();
                                                }
                                            }
                                            input::Tool::Move => {
                                                if let Some(last) = prev {
                                                    let dx = ((p.0 - last.0) / c.zoom_scale) as i32;
                                                    let dy = ((p.1 - last.1) / c.zoom_scale) as i32;
                                                    if dx != 0 || dy != 0 {
                                                        c.pan_image(dx, dy);
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
                                draw_ui(c, &input.brush, input.brightness, &input, &icons);

                                // Optional preview overlays during drag (backed up and restored)
                                let mut preview_backup: Option<(u32, u32, u32, u32, Vec<u8>)> = None;
                                if input.drawing {
                                    let mut backup_regions = Vec::new();

                                    // Shape tool preview
                                    if let Some((start, end)) = input.shape_start.zip(input.shape_end) {
                                        let thickness = input.brush.radius.max(1.0);
                                        let preview_color = [200, 150, 255, 200]; // Light purple preview overlay
                                        let margin = (thickness.ceil() as u32).saturating_add(2);
                                        let min_x = start.0.min(end.0).floor().max(0.0) as u32;
                                        let max_x = start.0.max(end.0).ceil().min((c.width - 1) as f32) as u32;
                                        let min_y = start.1.min(end.1).floor().max(0.0) as u32;
                                        let max_y = start.1.max(end.1).ceil().min((c.height - 1) as f32) as u32;
                                        let bx0 = min_x.saturating_sub(margin).min(c.width.saturating_sub(1));
                                        let by0 = min_y.saturating_sub(margin).min(c.height.saturating_sub(1));
                                        let bx1 = (max_x + margin).min(c.width.saturating_sub(1));
                                        let by1 = (max_y + margin).min(c.height.saturating_sub(1));
                                        let region_w = bx1.saturating_sub(bx0).saturating_add(1);
                                        let region_h = by1.saturating_sub(by0).saturating_add(1);
                                        if region_w > 0 && region_h > 0 {
                                            let mut backup = Vec::with_capacity(region_w as usize * region_h as usize * 4);
                                            for y in by0..=by1 {
                                                let offset = y as usize * c.stride + bx0 as usize * 4;
                                                let len = region_w as usize * 4;
                                                if offset + len <= c.pixels.len() {
                                                    backup.extend_from_slice(&c.pixels[offset..offset + len]);
                                                }
                                            }

                                            match input.current_tool {
                                                input::Tool::ShapeRect => c.draw_rect_outline(start, end, thickness, preview_color),
                                                input::Tool::ShapeEllipse => c.draw_ellipse_outline(start, end, thickness, preview_color),
                                                input::Tool::ShapeLine => c.draw_line(start, end, thickness, preview_color),
                                                _ => {}
                                            }

                                            backup_regions.push((bx0, by0, region_w, region_h, backup));
                                        }
                                    }

                                    // Selection tool preview (marching ants style border) - BLUE
                                    if matches!(input.current_tool, input::Tool::SelectRect | input::Tool::SelectEllipse) {
                                        if let Some((start, end)) = input.selection_start.zip(input.selection_end) {
                                            let sx = start.0.min(end.0);
                                            let sy = start.1.min(end.1);
                                            let ex = start.0.max(end.0);
                                            let ey = start.1.max(end.1);
                                            let border_color = [0, 150, 255, 255]; // Blue marquee
                                            
                                            // Backup the area around the selection border
                                            let margin = 2u32;
                                            let bx0 = sx.saturating_sub(margin);
                                            let by0 = sy.saturating_sub(margin);
                                            let bx1 = (ex + margin).min(c.width.saturating_sub(1));
                                            let by1 = (ey + margin).min(c.height.saturating_sub(1));
                                            let region_w = bx1.saturating_sub(bx0).saturating_add(1);
                                            let region_h = by1.saturating_sub(by0).saturating_add(1);
                                            
                                            if region_w > 0 && region_h > 0 {
                                                let mut backup = Vec::with_capacity(region_w as usize * region_h as usize * 4);
                                                for y in by0..=by1 {
                                                    let offset = y as usize * c.stride + bx0 as usize * 4;
                                                    let len = region_w as usize * 4;
                                                    if offset + len <= c.pixels.len() {
                                                        backup.extend_from_slice(&c.pixels[offset..offset + len]);
                                                    }
                                                }
                                                
                                                // Draw marching ants (dashed rectangle) around selection
                                                let dash_len = 4u32;
                                                let mut dash_count = 0u32;
                                                for x in sx..=ex.min(c.width.saturating_sub(1)) {
                                                    dash_count += 1;
                                                    if dash_count % (dash_len * 2) < dash_len {
                                                        if sy < c.height { c.set_pixel(x, sy, border_color); }
                                                        if ey < c.height { c.set_pixel(x, ey, border_color); }
                                                    }
                                                }
                                                for y in sy..=ey.min(c.height.saturating_sub(1)) {
                                                    dash_count += 1;
                                                    if dash_count % (dash_len * 2) < dash_len {
                                                        if sx < c.width { c.set_pixel(sx, y, border_color); }
                                                        if ex < c.width { c.set_pixel(ex, y, border_color); }
                                                    }
                                                }
                                                
                                                backup_regions.push((bx0, by0, region_w, region_h, backup));
                                            }
                                        }
                                    }

                                    // Move region preview - MAGENTA for destination
                                    if matches!(input.current_tool, input::Tool::SelectMove) {
                                        if let Some((start, end)) = input.move_region_start.zip(input.move_region_end) {
                                            let x1 = start.0.min(end.0) as u32;
                                            let y1 = start.1.min(end.1) as u32;
                                            let x2 = start.0.max(end.0) as u32;
                                            let y2 = start.1.max(end.1) as u32;
                                            let border_color = [255, 0, 255, 255]; // Magenta marquee for destination preview
                                            
                                            let margin = 2u32;
                                            let bx0 = x1.saturating_sub(margin);
                                            let by0 = y1.saturating_sub(margin);
                                            let bx1 = (x2 + margin).min(c.width.saturating_sub(1));
                                            let by1 = (y2 + margin).min(c.height.saturating_sub(1));
                                            let region_w = bx1.saturating_sub(bx0).saturating_add(1);
                                            let region_h = by1.saturating_sub(by0).saturating_add(1);
                                            
                                            if region_w > 0 && region_h > 0 {
                                                let mut backup = Vec::with_capacity(region_w as usize * region_h as usize * 4);
                                                for y in by0..=by1 {
                                                    let offset = y as usize * c.stride + bx0 as usize * 4;
                                                    let len = region_w as usize * 4;
                                                    if offset + len <= c.pixels.len() {
                                                        backup.extend_from_slice(&c.pixels[offset..offset + len]);
                                                    }
                                                }
                                                
                                                // Draw marching ants for move destination
                                                let dash_len = 4u32;
                                                let mut dash_count = 0u32;
                                                for x in x1..=x2.min(c.width.saturating_sub(1)) {
                                                    dash_count += 1;
                                                    if dash_count % (dash_len * 2) < dash_len {
                                                        if y1 < c.height { c.set_pixel(x, y1, border_color); }
                                                        if y2 < c.height { c.set_pixel(x, y2, border_color); }
                                                    }
                                                }
                                                for y in y1..=y2.min(c.height.saturating_sub(1)) {
                                                    dash_count += 1;
                                                    if dash_count % (dash_len * 2) < dash_len {
                                                        if x1 < c.width { c.set_pixel(x1, y, border_color); }
                                                        if x2 < c.width { c.set_pixel(x2, y, border_color); }
                                                    }
                                                }
                                                
                                                backup_regions.push((bx0, by0, region_w, region_h, backup));
                                            }
                                        }
                                    }

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

                                    // Restore all backed-up regions so previews don't persist
                                    for (bx0, by0, region_w, region_h, backup) in backup_regions {
                                        let mut cursor = 0;
                                        for y in by0..by0 + region_h {
                                            let offset = y as usize * c.stride + bx0 as usize * 4;
                                            let len = region_w as usize * 4;
                                            if offset + len <= c.pixels.len() && cursor + len <= backup.len() {
                                                c.pixels[offset..offset + len].copy_from_slice(&backup[cursor..cursor + len]);
                                            }
                                            cursor += len;
                                        }
                                    }
                                } else {
                                    // When not dragging, show persistent blue border around current selection
                                    if let Some((start, end)) = input.selection_start.zip(input.selection_end) {
                                        let sx = start.0.min(end.0);
                                        let sy = start.1.min(end.1);
                                        let ex = start.0.max(end.0);
                                        let ey = start.1.max(end.1);
                                        let border_color = [0, 150, 255, 255]; // Blue marquee
                                        
                                        let dash_len = 4u32;
                                        let mut dash_count = 0u32;
                                        for x in sx..=ex.min(c.width.saturating_sub(1)) {
                                            dash_count += 1;
                                            if dash_count % (dash_len * 2) < dash_len {
                                                if sy < c.height { c.set_pixel(x, sy, border_color); }
                                                if ey < c.height { c.set_pixel(x, ey, border_color); }
                                            }
                                        }
                                        for y in sy..=ey.min(c.height.saturating_sub(1)) {
                                            dash_count += 1;
                                            if dash_count % (dash_len * 2) < dash_len {
                                                if sx < c.width { c.set_pixel(sx, y, border_color); }
                                                if ex < c.width { c.set_pixel(ex, y, border_color); }
                                            }
                                        }
                                    }

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
