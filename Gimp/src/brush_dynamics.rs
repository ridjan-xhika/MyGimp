use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrushDynamics {
    pub enabled: bool,
    pub pressure_to_size: Vec<(f32, f32)>, // Curve points: (input, output)
    pub pressure_to_opacity: Vec<(f32, f32)>,
    pub speed_sensitivity: f32, // 0 to 1, how much speed affects size
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DynamicsPreset {
    Soft,
    Hard,
    Calligraphy,
}

impl BrushDynamics {
    pub fn new() -> Self {
        Self {
            enabled: true,
            pressure_to_size: vec![
                (0.0, 0.1),
                (0.5, 0.5),
                (1.0, 1.0),
            ],
            pressure_to_opacity: vec![
                (0.0, 0.2),
                (0.5, 0.8),
                (1.0, 1.0),
            ],
            speed_sensitivity: 0.5,
        }
    }

    pub fn from_preset(preset: DynamicsPreset) -> Self {
        let mut dynamics = Self::new();
        match preset {
            DynamicsPreset::Soft => {
                // Soft brush: linear size response, gentle opacity
                dynamics.pressure_to_size = vec![
                    (0.0, 0.2),
                    (0.5, 0.6),
                    (1.0, 1.0),
                ];
                dynamics.pressure_to_opacity = vec![
                    (0.0, 0.3),
                    (0.5, 0.7),
                    (1.0, 1.0),
                ];
                dynamics.speed_sensitivity = 0.3;
            }
            DynamicsPreset::Hard => {
                // Hard brush: aggressive size response at high pressure
                dynamics.pressure_to_size = vec![
                    (0.0, 0.05),
                    (0.3, 0.3),
                    (0.7, 0.8),
                    (1.0, 1.0),
                ];
                dynamics.pressure_to_opacity = vec![
                    (0.0, 0.1),
                    (0.5, 0.6),
                    (1.0, 1.0),
                ];
                dynamics.speed_sensitivity = 0.8;
            }
            DynamicsPreset::Calligraphy => {
                // Calligraphy: non-linear, rapid size changes
                dynamics.pressure_to_size = vec![
                    (0.0, 0.1),
                    (0.2, 0.3),
                    (0.4, 0.6),
                    (0.6, 0.8),
                    (1.0, 1.0),
                ];
                dynamics.pressure_to_opacity = vec![
                    (0.0, 0.4),
                    (0.5, 0.9),
                    (1.0, 1.0),
                ];
                dynamics.speed_sensitivity = 0.2;
            }
        }
        dynamics
    }

    /// Evaluate curve at a given input value (0.0 to 1.0)
    #[allow(dead_code)]
    pub fn evaluate_curve(curve: &[(f32, f32)], input: f32) -> f32 {
        let clamped = input.clamp(0.0, 1.0);

        // Find the two points to interpolate between
        for i in 0..curve.len() - 1 {
            let (x1, y1) = curve[i];
            let (x2, y2) = curve[i + 1];

            if clamped >= x1 && clamped <= x2 {
                // Linear interpolation between points
                let t = if x2 == x1 { 0.0 } else { (clamped - x1) / (x2 - x1) };
                return y1 + t * (y2 - y1);
            }
        }

        // Clamp to last point
        curve.last().map(|&(_, y)| y).unwrap_or(1.0)
    }

    /// Get size multiplier based on pressure (0.0 to 1.0)
    #[allow(dead_code)]
    pub fn size_from_pressure(&self, pressure: f32) -> f32 {
        Self::evaluate_curve(&self.pressure_to_size, pressure.clamp(0.0, 1.0))
    }

    /// Get opacity multiplier based on pressure (0.0 to 1.0)
    #[allow(dead_code)]
    pub fn opacity_from_pressure(&self, pressure: f32) -> f32 {
        Self::evaluate_curve(&self.pressure_to_opacity, pressure.clamp(0.0, 1.0))
    }

    /// Get size multiplier based on speed. Speed is in pixels per frame.
    #[allow(dead_code)]
    pub fn size_from_speed(&self, distance_pixel: f32, time_delta_secs: f32) -> f32 {
        if time_delta_secs <= 0.0 {
            return 1.0;
        }
        let speed = distance_pixel / time_delta_secs;
        // Normalize speed to 0..1 range (assuming 0-500 px/sec is normal)
        let normalized_speed = (speed / 500.0).clamp(0.0, 1.0);
        // Faster strokes = smaller size
        1.0 - (normalized_speed * self.speed_sensitivity * 0.5)
    }

    /// Apply dynamics to radius and color with pressure and speed
    #[allow(dead_code)]
    pub fn apply_dynamics(
        &self,
        base_radius: f32,
        base_color: [u8; 4],
        pressure: f32,
        speed_multiplier: f32,
    ) -> (f32, [u8; 4]) {
        if !self.enabled {
            return (base_radius, base_color);
        }

        let pressure_clamped = pressure.clamp(0.0, 1.0);

        // Apply pressure to size
        let size_mult = self.size_from_pressure(pressure_clamped);
        let speed_mult = speed_multiplier.clamp(0.5, 1.0);
        let final_radius = base_radius * size_mult * speed_mult;

        // Apply pressure to opacity
        let opacity_mult = self.opacity_from_pressure(pressure_clamped);
        let mut final_color = base_color;
        final_color[3] = (base_color[3] as f32 * opacity_mult) as u8;

        (final_radius, final_color)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicsState {
    pub preset: DynamicsPreset,
    pub dynamics: BrushDynamics,
    pub last_pressure: f32,
    pub last_distance: f32,
}

impl Default for DynamicsState {
    fn default() -> Self {
        Self {
            preset: DynamicsPreset::Soft,
            dynamics: BrushDynamics::from_preset(DynamicsPreset::Soft),
            last_pressure: 0.0,
            last_distance: 0.0,
        }
    }
}

impl DynamicsState {
    #[allow(dead_code)]
    pub fn new(preset: DynamicsPreset) -> Self {
        Self {
            preset,
            dynamics: BrushDynamics::from_preset(preset),
            last_pressure: 0.0,
            last_distance: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn set_preset(&mut self, preset: DynamicsPreset) {
        self.preset = preset;
        self.dynamics = BrushDynamics::from_preset(preset);
    }

    /// Load preset from file or create default
    #[allow(dead_code)]
    pub fn load_or_default(preset: DynamicsPreset) -> Self {
        let config_dir = Self::config_dir();
        let preset_path = config_dir.join(format!("{:?}.json", preset));

        if let Ok(content) = std::fs::read_to_string(&preset_path) {
            if let Ok(state) = serde_json::from_str::<Self>(&content) {
                return state;
            }
        }

        Self::new(preset)
    }

    /// Save preset to file
    #[allow(dead_code)]
    pub fn save(&self) -> std::io::Result<()> {
        let config_dir = Self::config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let preset_path = config_dir.join(format!("{:?}.json", self.preset));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        std::fs::write(preset_path, json)
    }

    #[allow(dead_code)]
    fn config_dir() -> std::path::PathBuf {
        #[cfg(target_os = "windows")]
        {
            std::path::PathBuf::from("./config")
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(home) = std::env::var("HOME") {
                std::path::PathBuf::from(home).join(".config/MyGimp")
            } else {
                std::path::PathBuf::from("./config")
            }
        }
    }
}
