use femtovg::{Canvas, renderer::Renderer, Paint, Path};
use crate::telemetry::SharedTelemetryState;
use super::{Widget, WidgetGeometry};
use crate::ui::theme::Theme;
use std::f32::consts::PI;
use crate::ui::widgets::{ThemeTransition, LayoutContext};
use std::time::Duration;

const DIRECTION_LABELS: [&str; 4] = ["FRONT", "RIGHT", "REAR", "LEFT"];

/// A widget that displays G-Force as a moving dot in a circular display
pub struct GForceMeter {
    theme: Theme,
    max_g_force_displayed: f32,
    // For theme transition animation
    theme_transition: Option<ThemeTransition>,
    theme_anim_time: f32, // 0.0..=1.0
}

impl GForceMeter {
    /// Create a new GForceMeter widget
    pub fn new(theme: Theme, max_g_force_displayed: f32) -> Self {
        Self {
            theme,
            max_g_force_displayed,
            theme_transition: None,
            theme_anim_time: 1.0,
        }
    }
    
    /// Set the theme for this GForceMeter
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.theme_transition = None;
        self.theme_anim_time = 1.0;
    }
    /// Set the max G-Force displayed
    pub fn set_max_g_force_displayed(&mut self, max_g: f32) {
        self.max_g_force_displayed = max_g;
    }
}

impl Widget for GForceMeter {
    fn render<R: Renderer>(&self, canvas: &mut Canvas<R>, rect: WidgetGeometry, telemetry_state: &SharedTelemetryState) {
        let state = match telemetry_state.try_lock() {
            Ok(state) => state,
            Err(_) => return, // Skip rendering if we can't acquire the lock
        };
        
        let g_force_data = match &state.latest_racebox_data {
            Some(data) => (data.g_force_x, data.g_force_y, data.g_force_z),
            None => return, // Skip rendering if no g-force data is available
        };
        
        let (g_force_x, g_force_y, g_force_z) = g_force_data;
        
        // Prepare drawing constants
        let geometry = rect;
        let center_x = geometry.center_x();
        let center_y = geometry.center_y();
        let min_dimension = geometry.width.min(geometry.height);
        let radius = min_dimension * 0.45; // Main circle takes 90% of the size
        
        // Draw background
        let mut path = Path::new();
        path.rect(geometry.x, geometry.y, geometry.width, geometry.height);
        let paint = Paint::color(Theme::color4(self.theme.background_color));
        canvas.fill_path(&path, &paint);
        
        // Draw concentric circles
        let num_circles = self.theme.circle_colors.len();
        for (i, color) in self.theme.circle_colors.iter().enumerate() {
            let circle_radius = radius * (i + 1) as f32 / num_circles as f32;
            let mut path = Path::new();
            path.circle(center_x, center_y, circle_radius);
            
            let mut paint = Paint::color(Theme::color4(*color));
            paint.set_line_width(self.theme.line_width);
            paint.set_line_join(femtovg::LineJoin::Round);
            canvas.stroke_path(&path, &paint);
        }
        
        // Draw the cross in the middle
        let cross_size = radius * 0.1;
        let mut path = Path::new();
        // Horizontal line
        path.move_to(center_x - cross_size, center_y);
        path.line_to(center_x + cross_size, center_y);
        // Vertical line
        path.move_to(center_x, center_y - cross_size);
        path.line_to(center_x, center_y + cross_size);
        
        let mut paint = Paint::color(Theme::color3(self.theme.foreground_color));
        paint.set_line_width(self.theme.line_width * 0.5);
        canvas.stroke_path(&path, &paint);
        
        // Calculate dot position based on g-forces
        // Negative X is left, positive X is right
        // Negative Y is forward, positive Y is backward
        let max_g = self.max_g_force_displayed;
        let scaled_x = (g_force_x / max_g).clamp(-1.0, 1.0);
        let scaled_y = (g_force_y / max_g).clamp(-1.0, 1.0);
        
        let dot_x = center_x + scaled_x * radius;
        let dot_y = center_y - scaled_y * radius; // Invert Y to match physical coordinate system
        
        // Calculate dot size based on Z g-force (scaled between 0.5 and 1.5)
        let base_dot_size = radius * 0.1;
        let scale_factor = 1.0 + (g_force_z / max_g).clamp(-0.5, 0.5);
        let dot_size = base_dot_size * scale_factor;
        
        // Draw the g-force indicator dot
        let mut path = Path::new();
        path.circle(dot_x, dot_y, dot_size);
        let paint = Paint::color(Theme::color3(self.theme.dot_color));
        canvas.fill_path(&path, &paint);
        
        // Draw the dot border
        let mut path = Path::new();
        path.circle(dot_x, dot_y, dot_size);
        let mut paint = Paint::color(Theme::color3(self.theme.dot_border_color));
        paint.set_line_width(self.theme.line_width * 0.5);
        canvas.stroke_path(&path, &paint);
        
        // Draw direction labels and g-force values
        let mut text_paint = Paint::color(Theme::color3(self.theme.text_color));
        text_paint.set_font_size(self.theme.font_size);
        text_paint.set_text_align(femtovg::Align::Center);
        text_paint.set_text_baseline(femtovg::Baseline::Middle);
        
        // Draw direction labels and values
        for i in 0..4 {
            let angle = (i as f32 * PI / 2.0) - (PI / 2.0); // Start from top and go clockwise
            let label_radius = radius * 1.15;
            let value_radius = radius * 1.3;
            
            let label_x = center_x + angle.cos() * label_radius;
            let label_y = center_y + angle.sin() * label_radius;
            
            let value_x = center_x + angle.cos() * value_radius;
            let value_y = center_y + angle.sin() * value_radius;
            
            // Draw direction label
            let _ = canvas.fill_text(label_x, label_y, DIRECTION_LABELS[i], &text_paint);
            
            // Calculate g-force value for this direction
            let g_value = match i {
                0 => -g_force_y, // Forward is negative Y
                1 => g_force_x,  // Right is positive X
                2 => g_force_y,  // Rear is positive Y
                3 => -g_force_x, // Left is negative X
                _ => 0.0,
            };
            
            // Draw g-force value with one decimal place
            let _ = canvas.fill_text(
                value_x, 
                value_y, 
                &format!("{:.1}G", g_value),
                &text_paint
            );
        }
    }

    fn on_theme_change(&mut self, new_theme: &Theme, transition: ThemeTransition) {
        self.theme_transition = Some(transition);
        self.theme_anim_time = 0.0;
    }

    fn update(&mut self, dt: Duration) {
        if let Some(ref mut transition) = self.theme_transition {
            self.theme_anim_time += dt.as_secs_f32();
            let t = self.theme_anim_time.min(1.0);
            self.theme = Theme::interpolate(&transition.from, &transition.to, t);
            if t >= 1.0 {
                self.theme_transition = None;
                self.theme_anim_time = 1.0;
            }
        }
    }

    fn preferred_size(&self, _ctx: &LayoutContext) -> WidgetGeometry {
        // Default to a square 200x200, can be dynamic based on context
        WidgetGeometry::new(0.0, 0.0, 200.0, 200.0)
    }
} 