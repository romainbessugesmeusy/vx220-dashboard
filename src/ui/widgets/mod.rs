use femtovg::{Canvas, renderer::Renderer};
use crate::telemetry::SharedTelemetryState;
use crate::ui::theme::Theme;
use crate::telemetry::{DriveMode, ColorScheme};
use std::time::Duration;

pub mod g_force_meter;
pub mod gauge;
pub mod turbo_pressure_gauge;
pub mod rpm_gauge;

/// Defines the position and size of a widget
#[derive(Debug, Clone, Copy)]
pub struct WidgetGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WidgetGeometry {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }
    
    pub fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }
}

/// Information about the current layout context (window size, drive mode, etc.)
#[derive(Debug, Clone)]
pub struct LayoutContext {
    pub window_width: f32,
    pub window_height: f32,
    pub drive_mode: DriveMode,
    pub color_scheme: ColorScheme,
    // ...future extensibility
}

/// Information about a theme transition (for interpolation)
#[derive(Debug, Clone)]
pub struct ThemeTransition {
    pub from: Theme,
    pub to: Theme,
    pub progress: f32, // 0.0..=1.0
    // pub easing: EasingFunction, // Add if you want per-widget easing
}

/// Base trait for all dashboard widgets
pub trait Widget {
    /// Render the widget inside the given rectangle.
    fn render<R: Renderer>(
        &self,
        canvas: &mut Canvas<R>,
        rect: WidgetGeometry,
        telemetry_state: &SharedTelemetryState,
    );

    /// Called when the drive mode or color scheme changes.
    fn on_theme_change(&mut self, new_theme: &Theme, transition: ThemeTransition);

    /// Called every frame to update internal state (e.g., for animations).
    fn update(&mut self, dt: Duration);

    /// Widgets can suggest their preferred size for layout.
    fn preferred_size(&self, ctx: &LayoutContext) -> WidgetGeometry;
} 