use femtovg::Color;
use crate::telemetry::{DriveMode, ColorScheme};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::env;

/// Defines the visual styling for dashboard widgets.
///
/// Theme values are loaded from YAML files in ./assets/themes.
///
/// Example YAML format for a theme file:
///
/// background_color: [40, 40, 80, 230]
/// foreground_color: [230, 230, 230]
/// accent_color: [220, 20, 60]
/// text_color: [255, 255, 255]
/// font_size: 14.0
/// line_width: 2.0
/// circle_colors:
///   - [255, 255, 255, 100]
///   - [255, 255, 255, 70]
///   - [255, 255, 255, 40]
/// dot_color: [220, 20, 60]
/// dot_border_color: [255, 255, 255]
///
/// Place these files in ./assets/themes and name them according to the preset (e.g. dark_road.yml).
///
/// Note: The theme YAML files must exist for the app to run. The app will panic if they are missing.
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub background_color: [u8; 4],
    pub foreground_color: [u8; 3],
    pub accent_color: [u8; 3],
    pub text_color: [u8; 3],
    pub font_size: f32,
    pub line_width: f32,
    pub circle_colors: Vec<[u8; 4]>,
    pub dot_color: [u8; 3],
    pub dot_border_color: [u8; 3],
}

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        self.background_color == other.background_color &&
        self.foreground_color == other.foreground_color &&
        self.accent_color == other.accent_color &&
        self.text_color == other.text_color &&
        self.circle_colors == other.circle_colors &&
        self.dot_color == other.dot_color &&
        self.dot_border_color == other.dot_border_color
    }
}

impl Theme {
    fn get_theme_path() -> String {
        // First try to get the executable's directory
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let theme_dir = exe_dir.join("assets/themes");
                if theme_dir.exists() {
                    return theme_dir.to_string_lossy().into_owned();
                }
            }
        }
        
        // Fallback to current directory
        "assets/themes".to_string()
    }

    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Self {
        let path_str = path.as_ref().to_string_lossy();
        let yaml = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read theme YAML file '{}': {}", path_str, e));
        serde_yaml::from_str(&yaml)
            .unwrap_or_else(|e| panic!("Failed to parse theme YAML file '{}': {}", path_str, e))
    }

    /// Construct a theme based on drive mode and color scheme
    pub fn from_preset(drive_mode: DriveMode, color_scheme: ColorScheme) -> Self {
        let theme_dir = Self::get_theme_path();
        let file = match (drive_mode, color_scheme) {
            (DriveMode::Road, ColorScheme::Light) => format!("{}/light_road.yml", theme_dir),
            (DriveMode::Road, ColorScheme::Dark) => format!("{}/dark_road.yml", theme_dir),
            (DriveMode::Track, ColorScheme::Light) => format!("{}/light_race.yml", theme_dir),
            (DriveMode::Track, ColorScheme::Dark) => format!("{}/dark_race.yml", theme_dir),
        };
        
        if !Path::new(&file).exists() {
            panic!("Theme YAML file not found: {}. Please ensure the file exists in the correct location.", file);
        }
        
        Self::from_yaml_file(file)
    }

    /// Interpolate between two themes (for smooth transitions)
    pub fn interpolate(a: &Theme, b: &Theme, t: f32) -> Self {
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a + (b - a) * t
        }
        let circle_colors = a.circle_colors.iter().zip(&b.circle_colors)
            .map(|(&ac, &bc)| {
                let lerped = [
                    (lerp(ac[0] as f32, bc[0] as f32, t)) as u8,
                    (lerp(ac[1] as f32, bc[1] as f32, t)) as u8,
                    (lerp(ac[2] as f32, bc[2] as f32, t)) as u8,
                    (lerp(ac[3] as f32, bc[3] as f32, t)) as u8,
                ];
                lerped
            })
            .collect();
        Self {
            background_color: [
                (lerp(a.background_color[0] as f32, b.background_color[0] as f32, t)) as u8,
                (lerp(a.background_color[1] as f32, b.background_color[1] as f32, t)) as u8,
                (lerp(a.background_color[2] as f32, b.background_color[2] as f32, t)) as u8,
                (lerp(a.background_color[3] as f32, b.background_color[3] as f32, t)) as u8,
            ],
            foreground_color: [
                (lerp(a.foreground_color[0] as f32, b.foreground_color[0] as f32, t)) as u8,
                (lerp(a.foreground_color[1] as f32, b.foreground_color[1] as f32, t)) as u8,
                (lerp(a.foreground_color[2] as f32, b.foreground_color[2] as f32, t)) as u8,
            ],
            accent_color: [
                (lerp(a.accent_color[0] as f32, b.accent_color[0] as f32, t)) as u8,
                (lerp(a.accent_color[1] as f32, b.accent_color[1] as f32, t)) as u8,
                (lerp(a.accent_color[2] as f32, b.accent_color[2] as f32, t)) as u8,
            ],
            text_color: [
                (lerp(a.text_color[0] as f32, b.text_color[0] as f32, t)) as u8,
                (lerp(a.text_color[1] as f32, b.text_color[1] as f32, t)) as u8,
                (lerp(a.text_color[2] as f32, b.text_color[2] as f32, t)) as u8,
            ],
            font_size: lerp(a.font_size, b.font_size, t),
            line_width: lerp(a.line_width, b.line_width, t),
            circle_colors,
            dot_color: [
                (lerp(a.dot_color[0] as f32, b.dot_color[0] as f32, t)) as u8,
                (lerp(a.dot_color[1] as f32, b.dot_color[1] as f32, t)) as u8,
                (lerp(a.dot_color[2] as f32, b.dot_color[2] as f32, t)) as u8,
            ],
            dot_border_color: [
                (lerp(a.dot_border_color[0] as f32, b.dot_border_color[0] as f32, t)) as u8,
                (lerp(a.dot_border_color[1] as f32, b.dot_border_color[1] as f32, t)) as u8,
                (lerp(a.dot_border_color[2] as f32, b.dot_border_color[2] as f32, t)) as u8,
            ],
        }
    }

    // Helper to convert [u8; 3] or [u8; 4] to femtovg::Color
    pub fn color3(rgb: [u8; 3]) -> femtovg::Color {
        femtovg::Color::rgb(rgb[0], rgb[1], rgb[2])
    }
    pub fn color4(rgba: [u8; 4]) -> femtovg::Color {
        femtovg::Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3])
    }
} 