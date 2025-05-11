use crate::ui::widgets::{Widget, WidgetGeometry, LayoutContext, ThemeTransition};
use crate::ui::widgets::gauge::*;
use crate::ui::theme::Theme;
use crate::telemetry::SharedTelemetryState;
use femtovg::{Canvas, renderer::Renderer};
use std::time::Duration;

pub struct TurboPressureGauge {
    gauge: Gauge,
}

impl TurboPressureGauge {
    pub fn new(theme: &Theme) -> Self {
        let props = GaugeProps {
            label: "TURBO".to_string(),
            unit: "bar".to_string(),
            min_value: -1.0,
            max_value: 2.0,
            danger_zone_start: Some(1.5),
            graduations: GaugeGraduations {
                major_tick_interval: 1.0,
                minor_tick_interval: 0.2,
                show_labels: true,
                label_decimals: 0,
            },
            start_angle: 7.0 * std::f32::consts::PI / 6.0,   // 210°
            end_angle: -1.0 * std::f32::consts::PI / 6.0,    // -30°
            radius_ratio: 0.9,
            center_offset: (0.0, 0.0),
            tick_style: GaugeTickStyle {
                major_tick_width: 3.0,
                major_tick_length: 0.15,
                minor_tick_width: 1.5,
                minor_tick_length: 0.08,
                tick_color: [255, 255, 255, 255],
                danger_zone_color: [255, 0, 0, 180],
            },
            needle: GaugeNeedleStyle {
                sprite_path: None,
                color: [255, 255, 255, 255],
                width: 3.0,
                length: 0.8,
                pivot: (0.0, 0.0),
                shadow: None,
            },
            label_position: (0.5, 0.85), // bottom center
            unit_position: (0.15, 0.15), // top left
            label_font_size: 22.0,
            unit_font_size: 16.0,
            show_value: false,
            value_position: (0.5, 0.7),
            value_font_size: 18.0,
            value_decimals: 1,
            background_color: [0, 0, 0, 255],
            border_color: [255, 255, 255, 255],
            border_width: 2.0,
            track: Some(GaugeTrack {
                color: [80, 80, 80, 180], // semi-transparent gray
                thickness: 0.12,
                start: -1.0,
                end: 2.0,
            }),
            clockwise: true,
        };
        Self {
            gauge: Gauge::new(props),
        }
    }
    pub fn set_value(&mut self, value: f32) {
        self.gauge.set_value(value);
    }
}

impl Widget for TurboPressureGauge {
    fn render<R: Renderer>(&self, canvas: &mut Canvas<R>, rect: WidgetGeometry, telemetry_state: &SharedTelemetryState) {
        self.gauge.render(canvas, rect, telemetry_state);
    }
    fn on_theme_change(&mut self, new_theme: &Theme, transition: ThemeTransition) {
        self.gauge.on_theme_change(new_theme, transition);
    }
    fn update(&mut self, dt: Duration) {
        self.gauge.update(dt);
    }
    fn preferred_size(&self, ctx: &LayoutContext) -> WidgetGeometry {
        self.gauge.preferred_size(ctx)
    }
} 