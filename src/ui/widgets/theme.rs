use femtovg::Color;

/// Defines the visual styling for dashboard widgets
#[derive(Debug, Clone)]
pub struct Theme {
    pub background_color: Color,
    pub foreground_color: Color,
    pub accent_color: Color,
    pub text_color: Color,
    pub font_size: f32,
    pub line_width: f32,
    pub circle_colors: Vec<Color>,
    pub dot_color: Color,
    pub dot_border_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background_color: Color::rgba(40, 40, 80, 230),
            foreground_color: Color::rgb(230, 230, 230),
            accent_color: Color::rgb(220, 20, 60),
            text_color: Color::rgb(255, 255, 255),
            font_size: 14.0,
            line_width: 2.0,
            circle_colors: vec![
                Color::rgba(255, 255, 255, 100),
                Color::rgba(255, 255, 255, 70),
                Color::rgba(255, 255, 255, 40),
            ],
            dot_color: Color::rgb(220, 20, 60),
            dot_border_color: Color::rgb(255, 255, 255),
        }
    }
} 