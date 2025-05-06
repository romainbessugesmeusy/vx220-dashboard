use femtovg::{Canvas, renderer::Renderer};
use crate::telemetry::SharedTelemetryState;

pub mod theme;
pub mod g_force_meter;

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

/// Base trait for all dashboard widgets
pub trait Widget {
    /// Render the widget to the canvas
    fn render<R: Renderer>(&self, canvas: &mut Canvas<R>, telemetry_state: &SharedTelemetryState);
    
    /// Get the geometry of the widget
    fn geometry(&self) -> WidgetGeometry;
    
    /// Set the geometry of the widget
    fn set_geometry(&mut self, geometry: WidgetGeometry);
} 