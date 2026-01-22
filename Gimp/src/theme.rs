use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub current_theme: Theme,
    pub colors: ThemeColors,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeColors {
    // Light theme colors
    pub light_bg: [u8; 4],
    pub light_panel: [u8; 4],
    pub light_text: [u8; 4],
    pub light_toolbar: [u8; 4],
    pub light_button: [u8; 4],
    pub light_button_hover: [u8; 4],
    pub light_accent: [u8; 4],

    // Dark theme colors
    pub dark_bg: [u8; 4],
    pub dark_panel: [u8; 4],
    pub dark_text: [u8; 4],
    pub dark_toolbar: [u8; 4],
    pub dark_button: [u8; 4],
    pub dark_button_hover: [u8; 4],
    pub dark_accent: [u8; 4],
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            // Light theme
            light_bg: [240, 240, 240, 255],
            light_panel: [220, 220, 220, 255],
            light_text: [30, 30, 30, 255],
            light_toolbar: [200, 200, 200, 255],
            light_button: [180, 180, 180, 255],
            light_button_hover: [160, 160, 160, 255],
            light_accent: [100, 150, 255, 255],

            // Dark theme
            dark_bg: [30, 30, 30, 255],
            dark_panel: [50, 50, 50, 255],
            dark_text: [220, 220, 220, 255],
            dark_toolbar: [60, 60, 60, 255],
            dark_button: [80, 80, 80, 255],
            dark_button_hover: [100, 100, 100, 255],
            dark_accent: [100, 150, 255, 255],
        }
    }
}

impl ThemeConfig {
    pub fn new() -> Self {
        Self {
            current_theme: Theme::Dark,
            colors: ThemeColors::default(),
        }
    }

    pub fn toggle_theme(&mut self) {
        self.current_theme = match self.current_theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }

    pub fn get_bg_color(&self) -> [u8; 4] {
        match self.current_theme {
            Theme::Light => self.colors.light_bg,
            Theme::Dark => self.colors.dark_bg,
        }
    }

    pub fn get_panel_color(&self) -> [u8; 4] {
        match self.current_theme {
            Theme::Light => self.colors.light_panel,
            Theme::Dark => self.colors.dark_panel,
        }
    }

    pub fn get_text_color(&self) -> [u8; 4] {
        match self.current_theme {
            Theme::Light => self.colors.light_text,
            Theme::Dark => self.colors.dark_text,
        }
    }

    pub fn get_toolbar_color(&self) -> [u8; 4] {
        match self.current_theme {
            Theme::Light => self.colors.light_toolbar,
            Theme::Dark => self.colors.dark_toolbar,
        }
    }

    pub fn get_button_color(&self) -> [u8; 4] {
        match self.current_theme {
            Theme::Light => self.colors.light_button,
            Theme::Dark => self.colors.dark_button,
        }
    }

    pub fn get_button_hover_color(&self) -> [u8; 4] {
        match self.current_theme {
            Theme::Light => self.colors.light_button_hover,
            Theme::Dark => self.colors.dark_button_hover,
        }
    }

    pub fn get_accent_color(&self) -> [u8; 4] {
        match self.current_theme {
            Theme::Light => self.colors.light_accent,
            Theme::Dark => self.colors.dark_accent,
        }
    }

    pub fn save_to_config(&self) -> std::io::Result<()> {
        let config_dir = Self::config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("theme.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        std::fs::write(config_path, json)
    }

    pub fn load_from_config() -> Self {
        let config_dir = Self::config_dir();
        let config_path = config_dir.join("theme.json");

        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<Self>(&content) {
                return config;
            }
        }

        Self::new()
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DpiDensity {
    Standard,  // 1x density
    HighDpi,   // 2x density (HiDPI)
}

pub struct IconSet {
    pub standard_icons: std::collections::HashMap<String, Vec<u8>>, // 32x32 RGBA
    pub hidpi_icons: std::collections::HashMap<String, Vec<u8>>,    // 64x64 RGBA
}

impl IconSet {
    pub fn new() -> Self {
        Self {
            standard_icons: std::collections::HashMap::new(),
            hidpi_icons: std::collections::HashMap::new(),
        }
    }

    /// Get icon with automatic DPI selection
    pub fn get_icon(&self, name: &str, density: DpiDensity) -> Option<&Vec<u8>> {
        match density {
            DpiDensity::Standard => self.standard_icons.get(name),
            DpiDensity::HighDpi => self.hidpi_icons.get(name).or_else(|| self.standard_icons.get(name)),
        }
    }

    /// Register a standard density icon (32x32)
    pub fn register_standard(&mut self, name: String, pixels: Vec<u8>) {
        self.standard_icons.insert(name, pixels);
    }

    /// Register a HiDPI density icon (64x64)
    pub fn register_hidpi(&mut self, name: String, pixels: Vec<u8>) {
        self.hidpi_icons.insert(name, pixels);
    }

    /// Scale a 32x32 icon to 64x64 for simple fallback
    pub fn scale_up_icon(icon_32: &[u8]) -> Vec<u8> {
        let mut icon_64 = vec![0u8; 64 * 64 * 4];

        for y in 0..64 {
            for x in 0..64 {
                let src_x = (x / 2).min(31);
                let src_y = (y / 2).min(31);

                let src_idx = (src_y * 32 + src_x) * 4;
                let dst_idx = (y * 64 + x) * 4;

                if src_idx + 3 < icon_32.len() && dst_idx + 3 < icon_64.len() {
                    icon_64[dst_idx..dst_idx + 4].copy_from_slice(&icon_32[src_idx..src_idx + 4]);
                }
            }
        }

        icon_64
    }

    /// Check if HiDPI display is active (simple detection)
    pub fn detect_dpi_from_scale(scale_factor: f32) -> DpiDensity {
        if scale_factor >= 1.5 {
            DpiDensity::HighDpi
        } else {
            DpiDensity::Standard
        }
    }
}
