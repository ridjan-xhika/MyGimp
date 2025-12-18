use crate::brush::Brush;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SliderDrag {
    Size,
    Brightness,
}

pub struct InputState {
    pub drawing: bool,
    pub last_pos: Option<(f32, f32)>,
    pub brush: Brush,
    pub base_color: [u8; 4],
    pub brightness: f32,
    pub slider_dragging: Option<SliderDrag>,
}

impl InputState {
    pub fn new(brush: Brush) -> Self {
        Self {
            drawing: false,
            last_pos: None,
            base_color: brush.color,
            brightness: 1.0,
            brush,
            slider_dragging: None,
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

    pub fn adjust_brush_radius(&mut self, delta: f32, min: f32, max: f32) {
        let r = (self.brush.radius + delta).clamp(min, max);
        self.brush.radius = r;
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

    fn apply_brightness(&mut self) {
        let factor = self.brightness;
        let mut c = self.base_color;
        for i in 0..3 {
            c[i] = ((c[i] as f32 * factor).clamp(0.0, 255.0)).round() as u8;
        }
        self.brush.color = c;
    }
}
