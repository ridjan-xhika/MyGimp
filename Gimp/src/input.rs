use crate::brush::Brush;
use crate::brush_dynamics::{DynamicsState, DynamicsPreset};
use crate::filters::{FilterParams, FilterType};
use crate::selection::{Selection, SelectionMode, SelectionType};
use crate::viewport::ViewState;
use crate::theme::ThemeConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderDrag {
    Size,
    #[allow(dead_code)]
    Brightness,
    #[allow(dead_code)]
    BlurRadius,
    #[allow(dead_code)]
    SharpenStrength,
    #[allow(dead_code)]
    BrightnessFX,
    #[allow(dead_code)]
    Contrast,
    #[allow(dead_code)]
    Saturation,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Brush,
    Eraser,
    FillBucket,
    ColorPicker,
    Move,
    Blur,
    SelectRect,
    SelectEllipse,
    SelectLasso,
    SelectMove,
    ShapeRect,
    ShapeEllipse,
    ShapeLine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorPickerDrag {
    Hue,
    SV,
}

pub struct InputState {
    pub drawing: bool,
    pub last_pos: Option<(f32, f32)>,
    pub brush: Brush,
    pub base_color: [u8; 4],
    pub bg_color: [u8; 4],
    pub brightness: f32,
    pub slider_dragging: Option<SliderDrag>,
    pub pan_offset: (i32, i32), // (x, y) offset for viewing large images
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub current_tool: Tool,
    pub selection_start: Option<(u32, u32)>,
    pub selection_end: Option<(u32, u32)>,
    // Advanced color picker state
    pub show_color_picker: bool,
    pub hue: f32, // 0..1
    pub sat: f32, // 0..1
    pub val: f32, // 0..1
    pub active_is_foreground: bool,
    pub color_dragging: Option<ColorPickerDrag>,
    // Filter state
    pub show_filters: bool,
    pub current_filter: Option<FilterType>,
    pub filter_params: FilterParams,
    // Brush dynamics state
    pub dynamics_state: DynamicsState,
    pub dynamics_enabled: bool,
    // Selection state
    pub selection: Selection,
    pub selection_mode: SelectionMode,
    pub selection_type: SelectionType,
    pub lasso_points: Vec<(f32, f32)>,
    pub shape_start: Option<(f32, f32)>,
    pub shape_end: Option<(f32, f32)>,
    pub move_region_start: Option<(f32, f32)>,
    pub move_region_end: Option<(f32, f32)>,
    pub move_region_buffer: Option<(u32, u32, u32, u32, Vec<u8>)>,
    // Viewport state
    pub view_state: ViewState,
    // Theme state
    pub theme_config: ThemeConfig,
}

impl InputState {
    pub fn new(brush: Brush) -> Self {
        Self {
            drawing: false,
            last_pos: None,
            base_color: brush.color,
            bg_color: [255, 255, 255, 255],
            brightness: 1.0,
            brush,
            slider_dragging: None,
            pan_offset: (0, 0),
            shift_pressed: false,
            ctrl_pressed: false,
            current_tool: Tool::Brush,
            selection_start: None,
            selection_end: None,
            show_color_picker: false,
            hue: 0.0,
            sat: 1.0,
            val: 1.0,
            active_is_foreground: true,
            color_dragging: None,
            show_filters: false,
            current_filter: None,
            filter_params: FilterParams::default(),
            dynamics_state: DynamicsState::default(),
            dynamics_enabled: true,
            selection: Selection::new(800, 600),
            selection_mode: SelectionMode::Replace,
            selection_type: SelectionType::Rectangle,
            lasso_points: vec![],
            shape_start: None,
            shape_end: None,
            move_region_start: None,
            move_region_end: None,
            move_region_buffer: None,
            view_state: ViewState::new(),
            theme_config: ThemeConfig::load_from_config(),
        }
    }

    pub fn stop_drawing(&mut self) {
        self.drawing = false;
        self.last_pos = None;
    }

    pub fn set_brush_color(&mut self, color: [u8; 4]) {
        self.base_color = color;
        self.apply_brightness();
    }

    pub fn set_background_color(&mut self, color: [u8; 4]) {
        self.bg_color = color;
    }

    pub fn adjust_brush_radius(&mut self, delta: f32, min: f32, max: f32) {
        let r = (self.brush.radius + delta).clamp(min, max);
        self.brush.radius = r;
    }

    pub fn set_brush_radius(&mut self, radius: f32, min: f32, max: f32) {
        self.brush.radius = radius.clamp(min, max);
    }

    pub fn set_brightness(&mut self, value: f32, min: f32, max: f32) {
        self.brightness = value.clamp(min, max);
        self.apply_brightness();
    }

    pub fn adjust_brightness(&mut self, delta: f32, min: f32, max: f32) {
        self.brightness = (self.brightness + delta).clamp(min, max);
        self.apply_brightness();
    }

    pub fn set_slider_drag(&mut self, target: Option<SliderDrag>) {
        self.slider_dragging = target;
        if target.is_none() {
            self.last_pos = None;
        }
    }

    pub fn set_color_drag(&mut self, target: Option<ColorPickerDrag>) {
        self.color_dragging = target;
        if target.is_none() {
            self.last_pos = None;
        }
    }

    fn apply_brightness(&mut self) {
        let factor = self.brightness;
        let mut c = self.base_color;
        for i in 0..3 {
            c[i] = ((c[i] as f32 * factor).clamp(0.0, 255.0)).round() as u8;
        }
        self.brush.color = c;
    }

    pub fn toggle_color_picker(&mut self) {
        self.show_color_picker = !self.show_color_picker;
    }

    pub fn open_color_picker_foreground(&mut self) {
        self.active_is_foreground = true;
        self.show_color_picker = true;
    }

    pub fn open_color_picker_background(&mut self) {
        self.active_is_foreground = false;
        self.show_color_picker = true;
    }

    pub fn set_hsv(&mut self, h: f32, s: f32, v: f32) {
        self.hue = h.clamp(0.0, 1.0);
        self.sat = s.clamp(0.0, 1.0);
        self.val = v.clamp(0.0, 1.0);
        let rgb = hsv_to_rgb(self.hue, self.sat, self.val);
        if self.active_is_foreground {
            self.set_brush_color([rgb[0], rgb[1], rgb[2], 255]);
        } else {
            self.set_background_color([rgb[0], rgb[1], rgb[2], 255]);
        }
    }

    /// Toggle brush dynamics on/off
    pub fn toggle_dynamics(&mut self) {
        self.dynamics_enabled = !self.dynamics_enabled;
        self.dynamics_state.dynamics.enabled = self.dynamics_enabled;
    }

    /// Set active dynamics preset
    pub fn set_dynamics_preset(&mut self, preset: DynamicsPreset) {
        self.dynamics_state.set_preset(preset);
        // Try to save
        let _ = self.dynamics_state.save();
    }

    /// Get effective brush size with dynamics applied
    pub fn get_dynamic_brush_size(&self, pressure: f32, speed_mult: f32) -> f32 {
        if !self.dynamics_enabled {
            return self.brush.radius;
        }
        let (size, _) = self.dynamics_state.dynamics.apply_dynamics(
            self.brush.radius,
            self.brush.color,
            pressure,
            speed_mult,
        );
        size
    }

    /// Get effective brush color with dynamics applied
    pub fn get_dynamic_brush_color(&self, pressure: f32, speed_mult: f32) -> [u8; 4] {
        if !self.dynamics_enabled {
            return self.brush.color;
        }
        let (_, color) = self.dynamics_state.dynamics.apply_dynamics(
            self.brush.radius,
            self.brush.color,
            pressure,
            speed_mult,
        );
        color
    }

    /// Set selection mode (Replace, Add, Subtract, Intersect)
    pub fn set_selection_mode(&mut self, mode: SelectionMode) {
        self.selection_mode = mode;
    }

    /// Set selection type (Rectangle, Ellipse, Lasso)
    pub fn set_selection_type(&mut self, sel_type: SelectionType) {
        self.selection_type = sel_type;
        self.lasso_points.clear();
    }

    /// Clear current selection
    pub fn clear_selection(&mut self) {
        self.selection.clear();
        self.lasso_points.clear();
    }

    /// Select all
    pub fn select_all(&mut self) {
        self.selection.select_all();
    }

    /// Invert selection
    pub fn invert_selection(&mut self) {
        self.selection.invert();
    }

    /// Finalize lasso selection
    pub fn finalize_lasso_selection(&mut self) {
        if self.lasso_points.len() >= 3 {
            self.selection.select_lasso(&self.lasso_points, self.selection_mode);
        }
        self.lasso_points.clear();
    }

    /// Toggle theme and save
    pub fn toggle_theme(&mut self) {
        self.theme_config.toggle_theme();
        let _ = self.theme_config.save_to_config();
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
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
