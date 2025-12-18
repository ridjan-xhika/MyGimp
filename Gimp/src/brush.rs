use crate::canvas::Canvas;

#[derive(Clone)]
pub struct Brush {
    pub radius: f32,
    pub color: [u8; 4],
}

impl Brush {
    pub fn stamp(&self, canvas: &mut Canvas, pos: (f32, f32)) {
        canvas.stamp_circle(pos.0, pos.1, self.radius, self.color);
    }

    pub fn stroke(&self, canvas: &mut Canvas, from: (f32, f32), to: (f32, f32)) {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let dist = (dx * dx + dy * dy).sqrt();
        let steps = dist.max(1.0).ceil();
        let step_x = dx / steps;
        let step_y = dy / steps;
        let mut x = from.0;
        let mut y = from.1;
        for _ in 0..=steps as i32 {
            canvas.stamp_circle(x, y, self.radius, self.color);
            x += step_x;
            y += step_y;
        }
    }
}
