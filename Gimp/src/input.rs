use crate::brush::Brush;

pub struct InputState {
    pub drawing: bool,
    pub last_pos: Option<(f32, f32)>,
    pub brush: Brush,
}

impl InputState {
    pub fn new(brush: Brush) -> Self {
        Self {
            drawing: false,
            last_pos: None,
            brush,
        }
    }

    pub fn stop_drawing(&mut self) {
        self.drawing = false;
        self.last_pos = None;
    }

    pub fn set_brush_color(&mut self, color: [u8; 4]) {
        self.brush.color = color;
    }

    pub fn adjust_brush_radius(&mut self, delta: f32, min: f32, max: f32) {
        let r = (self.brush.radius + delta).clamp(min, max);
        self.brush.radius = r;
    }
}
