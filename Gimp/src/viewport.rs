use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ZoomMode {
    Fit,      // Fit to window
    Actual,   // 1:1 pixels
    Zoom(f32), // Custom zoom level (1.0 = 100%)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewState {
    pub zoom: f32,          // Zoom level (1.0 = 100%)
    pub pan_x: f32,         // Pan offset in pixels
    pub pan_y: f32,
    pub zoom_mode: ZoomMode,
    
    // For smooth zoom/pan animation
    pub target_zoom: f32,
    pub target_pan_x: f32,
    pub target_pan_y: f32,
    
    // Inertial panning velocity
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub friction: f32,      // 0.9 = 10% friction per frame
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom_mode: ZoomMode::Fit,
            target_zoom: 1.0,
            target_pan_x: 0.0,
            target_pan_y: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            friction: 0.92,
        }
    }
}

impl ViewState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Zoom in with smooth easing
    pub fn zoom_in(&mut self, center_x: f32, center_y: f32) {
        let old_zoom = self.target_zoom;
        self.target_zoom = (self.target_zoom * 1.2).min(16.0);
        self.adjust_pan_for_zoom(center_x, center_y, old_zoom, self.target_zoom);
    }

    /// Zoom out with smooth easing
    pub fn zoom_out(&mut self, center_x: f32, center_y: f32) {
        let old_zoom = self.target_zoom;
        self.target_zoom = (self.target_zoom / 1.2).max(0.1);
        self.adjust_pan_for_zoom(center_x, center_y, old_zoom, self.target_zoom);
    }

    /// Adjust pan when zooming to keep center point fixed
    fn adjust_pan_for_zoom(&mut self, center_x: f32, center_y: f32, old_zoom: f32, new_zoom: f32) {
        // Calculate how much the center point moves due to zoom change
        let zoom_ratio = new_zoom / old_zoom;
        self.target_pan_x = center_x - (center_x - self.target_pan_x) * zoom_ratio;
        self.target_pan_y = center_y - (center_y - self.target_pan_y) * zoom_ratio;
    }

    /// Fit canvas to window
    pub fn fit_to_window(&mut self, window_width: f32, window_height: f32, canvas_width: u32, canvas_height: u32) {
        let canvas_w = canvas_width as f32;
        let canvas_h = canvas_height as f32;
        
        let zoom_x = window_width / canvas_w;
        let zoom_y = window_height / canvas_h;
        self.target_zoom = zoom_x.min(zoom_y).max(0.1).min(16.0);
        
        self.target_pan_x = 0.0;
        self.target_pan_y = 0.0;
        self.zoom_mode = ZoomMode::Fit;
    }

    /// Actual size (1:1 pixels)
    pub fn actual_size(&mut self) {
        self.target_zoom = 1.0;
        self.target_pan_x = 0.0;
        self.target_pan_y = 0.0;
        self.zoom_mode = ZoomMode::Actual;
    }

    /// Pan with velocity for inertial panning
    pub fn pan_with_velocity(&mut self, delta_x: f32, delta_y: f32) {
        self.velocity_x = delta_x;
        self.velocity_y = delta_y;
    }

    /// Clamp pan to reasonable bounds
    pub fn clamp_pan(&mut self, canvas_width: u32, canvas_height: u32, window_width: f32, window_height: f32) {
        let scaled_canvas_w = canvas_width as f32 * self.zoom;
        let scaled_canvas_h = canvas_height as f32 * self.zoom;

        // Allow panning, but don't let it go too far
        let max_pan_x = (scaled_canvas_w - window_width / 2.0).max(0.0);
        let max_pan_y = (scaled_canvas_h - window_height / 2.0).max(0.0);

        self.pan_x = self.pan_x.clamp(-max_pan_x, max_pan_x);
        self.pan_y = self.pan_y.clamp(-max_pan_y, max_pan_y);
        self.target_pan_x = self.pan_x;
        self.target_pan_y = self.pan_y;
    }

    /// Update smooth animations (call once per frame)
    pub fn update(&mut self, delta_time: f32) {
        let easing_factor = (1.0 - (-5.0 * delta_time).exp()).min(1.0); // Exponential easing

        // Smooth zoom
        self.zoom += (self.target_zoom - self.zoom) * easing_factor;

        // Smooth pan with velocity
        self.velocity_x *= self.friction;
        self.velocity_y *= self.friction;

        self.pan_x += self.velocity_x + (self.target_pan_x - self.pan_x) * easing_factor;
        self.pan_y += self.velocity_y + (self.target_pan_y - self.pan_y) * easing_factor;

        // Stop inertia if velocity is negligible
        if self.velocity_x.abs() < 0.01 {
            self.velocity_x = 0.0;
        }
        if self.velocity_y.abs() < 0.01 {
            self.velocity_y = 0.0;
        }
    }

    /// Calculate mini-map parameters
    /// Returns (minimap_x, minimap_y, minimap_width, minimap_height, viewport_x, viewport_y, viewport_width, viewport_height)
    pub fn calculate_minimap(
        &self,
        canvas_width: u32,
        canvas_height: u32,
        window_width: f32,
        window_height: f32,
        minimap_size: f32,
    ) -> (f32, f32, f32, f32, f32, f32, f32, f32) {
        let scale = (minimap_size / canvas_width as f32).min(minimap_size / canvas_height as f32);
        let minimap_w = canvas_width as f32 * scale;
        let minimap_h = canvas_height as f32 * scale;

        let viewport_w = (window_width / self.zoom).min(canvas_width as f32);
        let viewport_h = (window_height / self.zoom).min(canvas_height as f32);

        let viewport_x = (-self.pan_x / self.zoom).clamp(0.0, canvas_width as f32 - viewport_w);
        let viewport_y = (-self.pan_y / self.zoom).clamp(0.0, canvas_height as f32 - viewport_h);

        let viewport_scale_x = viewport_w / canvas_width as f32 * scale;
        let viewport_scale_y = viewport_h / canvas_height as f32 * scale;
        let viewport_mini_x = viewport_x * scale;
        let viewport_mini_y = viewport_y * scale;

        (0.0, 0.0, minimap_w, minimap_h, viewport_mini_x, viewport_mini_y, viewport_scale_x, viewport_scale_y)
    }

    /// Navigate to a minimap click
    pub fn navigate_to_minimap_click(
        &mut self,
        click_x: f32,
        click_y: f32,
        canvas_width: u32,
        canvas_height: u32,
        window_width: f32,
        window_height: f32,
        minimap_size: f32,
    ) {
        let (_, _, minimap_w, _, _, _, _, _) = self.calculate_minimap(
            canvas_width,
            canvas_height,
            window_width,
            window_height,
            minimap_size,
        );

        let scale = minimap_w / canvas_width as f32;
        let canvas_x = click_x / scale;
        let canvas_y = click_y / scale;

        let viewport_w = window_width / self.zoom;
        let viewport_h = window_height / self.zoom;

        self.target_pan_x = -(canvas_x - viewport_w / 2.0) * self.zoom;
        self.target_pan_y = -(canvas_y - viewport_h / 2.0) * self.zoom;
    }

    /// Convert window coordinates to canvas coordinates
    pub fn window_to_canvas(&self, window_x: f32, window_y: f32) -> (f32, f32) {
        let canvas_x = (window_x - self.pan_x) / self.zoom;
        let canvas_y = (window_y - self.pan_y) / self.zoom;
        (canvas_x, canvas_y)
    }

    /// Convert canvas coordinates to window coordinates
    pub fn canvas_to_window(&self, canvas_x: f32, canvas_y: f32) -> (f32, f32) {
        let window_x = canvas_x * self.zoom + self.pan_x;
        let window_y = canvas_y * self.zoom + self.pan_y;
        (window_x, window_y)
    }
}
