// Copyright (Your Name or Organization) (Year)
// SPDX-License-Identifier: (Your chosen SPDX license, e.g., MIT OR Apache-2.0)

use crate::ui::widgets::{Widget, WidgetGeometry, LayoutContext, ThemeTransition};
use crate::telemetry::SharedTelemetryState;
use femtovg::{Align, Baseline, Canvas, Paint, Path, Solidity, renderer::Renderer}; // Ensure all are imported
use std::time::Duration;
use std::f32::consts::PI;

// Defines a colored track segment that can be drawn behind the ticks.
// This is useful for things like a permanent redline background or optimal shift range.
pub struct GaugeTrack {
    pub color: [u8; 4],      // Color of the track (R, G, B, A)
    pub thickness: f32,      // Thickness as a fraction of the gauge radius (e.g., 0.1 for 10% of radius)
    pub start: f32,          // Value at which the track starts (in gauge units, e.g., RPM)
    pub end: f32,            // Value at which the track ends (in gauge units)
}

// Properties defining the appearance and behavior of a gauge.
// These are typically set once when the gauge is created.
pub struct GaugeProps {
    // Data & Scale
    pub label: String,               // Main label for the gauge (e.g., "RPM", "TURBO")
    pub unit: String,                // Unit label for the gauge values (e.g., "rpm", "bar")
    pub min_value: f32,              // Minimum value on the gauge scale
    pub max_value: f32,              // Maximum value on the gauge scale
    pub danger_zone_start: Option<f32>,// Value at which the danger zone (redline) starts
    pub graduations: GaugeGraduations, // Defines how ticks and their labels are drawn

    // Angular Layout (angles in radians)
    pub start_angle: f32,            // Angle for `min_value` (0 rad = 3 o'clock, angles increase CCW)
    pub end_angle: f32,              // Angle for `max_value` (intended visual end)
    pub clockwise: bool,             // If true, values progress visually clockwise; otherwise, counter-clockwise.

    // Physical Layout & Style
    pub radius_ratio: f32,           // Fraction of the widget's half-min-dimension to use for the gauge radius (0.0 to 1.0)
    pub center_offset: (f32, f32),   // Offset of the gauge center from widget center (as fraction of widget width/height)
    pub tick_style: GaugeTickStyle,  // Defines appearance of major and minor ticks
    pub needle: GaugeNeedleStyle,    // Defines appearance of the needle
    pub label_position: (f32, f32),  // Position of the main label (fraction of widget width/height from top-left)
    pub unit_position: (f32, f32),   // Position of the unit label (fraction of widget width/height from top-left)
    pub label_font_size: f32,        // Base font size for the main label (scaled dynamically)
    pub unit_font_size: f32,         // Base font size for the unit label (scaled dynamically)
    pub show_value: bool,            // If true, display the current numerical value as text
    pub value_position: (f32, f32),  // Position for the current value text
    pub value_font_size: f32,        // Base font size for the current value text
    pub value_decimals: u8,          // Number of decimal places for the displayed value text
    pub background_color: [u8; 4],   // Gauge background circle color
    pub border_color: [u8; 4],       // Gauge border circle color
    pub border_width: f32,           // Gauge border circle width
    pub track: Option<GaugeTrack>,   // Optional track segment to draw behind ticks
}

// Defines how gauge graduations (ticks and their numerical labels) are drawn.
pub struct GaugeGraduations {
    pub major_tick_interval: f32,    // Interval between major ticks (e.g., 1000 RPM)
    pub minor_tick_interval: f32,    // Interval between minor ticks (e.g., 100 RPM)
    pub show_labels: bool,           // If true, draw numerical labels at major ticks
    pub label_decimals: u8,          // Number of decimal places for tick labels
}

// Defines the style of the tick marks on the gauge.
pub struct GaugeTickStyle {
    pub major_tick_width: f32,       // Width (thickness) of major tick lines
    pub major_tick_length: f32,      // Length of major ticks (as fraction of radius, e.g., 0.15 for 15%)
    pub minor_tick_width: f32,       // Width of minor tick lines
    pub minor_tick_length: f32,      // Length of minor ticks (as fraction of radius, e.g., 0.08 for 8%)
    pub tick_color: [u8; 4],         // Color for normal ticks
    pub danger_zone_color: [u8; 4],  // Color for the danger zone arc segment
}

// Defines the style of the gauge needle.
pub struct GaugeNeedleStyle {
    pub sprite_path: Option<String>, // Optional path to an image for the needle (not implemented yet)
    pub color: [u8; 4],              // Color of the needle line
    pub width: f32,                  // Width (thickness) of the needle line
    pub length: f32,                 // Length of the needle (as fraction of radius, e.g., 0.8 for 80%)
    pub pivot: (f32, f32),           // Pivot point of the needle relative to gauge center (fraction of radius)
                                     // (0,0) is gauge center. Use for needles not rotating around dead center.
    pub shadow: Option<NeedleShadowProps>, // Optional shadow properties (not implemented yet)
}

// Properties for a needle shadow (currently unused).
pub struct NeedleShadowProps {
    pub color: [u8; 4],
    pub offset: (f32, f32),
    pub blur: f32,
}

// Represents a generic gauge widget.
// It holds its configuration (GaugeProps) and current value.
pub struct Gauge {
    pub props: GaugeProps,
    pub value: f32, // The current value the gauge should display
}

// DESIGN_REFERENCE_WIDTH is the width for which the font sizes in GaugeProps are designed.
// When the gauge is rendered at a different width, fonts will be scaled proportionally.
const GAUGE_DESIGN_REFERENCE_WIDTH: f32 = 200.0;

impl Gauge {
    // Creates a new Gauge with the given properties, initialized to its minimum value.
    pub fn new(props: GaugeProps) -> Self {
        let initial_value = props.min_value;
        Self {
            props,
            value: initial_value,
        }
    }

    // Sets the current value of the gauge.
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}

impl Widget for Gauge {
    fn render<R: Renderer>(&self, canvas: &mut Canvas<R>, rect: WidgetGeometry, _telemetry_state: &SharedTelemetryState) {
        // --- PREPARATION ---
        let props = &self.props;
        let current_gauge_value = self.value.clamp(props.min_value, props.max_value);

        let center_x = rect.center_x() + props.center_offset.0 * rect.width;
        let center_y = rect.center_y() + props.center_offset.1 * rect.height;
        let gauge_radius = rect.width.min(rect.height) * 0.5 * props.radius_ratio;

        // FONT SCALING:
        // Font sizes are defined in GaugeProps for a gauge of GAUGE_DESIGN_REFERENCE_WIDTH.
        // They are scaled here based on the actual rendered width of the gauge.
        let font_scale_factor = rect.width / GAUGE_DESIGN_REFERENCE_WIDTH;

        // SWEEP ANGLE AND DIRECTION LOGIC:
        // Standard graphics coordinate system: 0 rad = 3 o'clock, angles increase Counter-Clockwise (CCW).
        // `props.start_angle`: Angle for `props.min_value`.
        // `props.end_angle`: User-intended angle for `props.max_value`.
        // `props.clockwise`: True if values should progress visually Clockwise (CW).
        //
        // `sweep_angle_rad`: The signed angular distance (radians) from `props.start_angle` to the angle for `props.max_value`,
        // respecting `props.clockwise`. A negative sweep means CW progression of values, positive means CCW.
        let p_start_angle_rad = props.start_angle;
        let p_end_angle_rad = props.end_angle;
        let mut sweep_angle_rad = p_end_angle_rad - p_start_angle_rad;

        if props.clockwise { // User wants values to progress visually Clockwise
            if sweep_angle_rad >= 0.0 { // If raw angular diff is CCW or zero, make it the shortest CW path
                sweep_angle_rad -= 2.0 * PI; // e.g. 0 to PI (CCW) becomes 0 to -PI (CW)
            }
        } else { // User wants values to progress visually Counter-Clockwise
            if sweep_angle_rad <= 0.0 { // If raw angular diff is CW or zero, make it the shortest CCW path
                sweep_angle_rad += 2.0 * PI; // e.g. 0 to -PI (CW) becomes 0 to PI (CCW)
            }
        }
        // Now, `p_start_angle_rad + sweep_angle_rad` is the angle for `props.max_value`.
        // Values are interpolated linearly along this `sweep_angle_rad`.

        // --- DRAW BACKGROUND ---
        let mut bg_path = Path::new();
        bg_path.circle(center_x, center_y, gauge_radius);
        let bg_paint = Paint::color(femtovg::Color::rgba(
            props.background_color[0],
            props.background_color[1],
            props.background_color[2],
            props.background_color[3],
        ));
        // Anti-aliasing for filled paths is often on by default or best handled by MSAA at canvas level.
        canvas.fill_path(&bg_path, &bg_paint);
        
        // --- DRAW TRACK ARC (Optional) ---
        // femtovg::Path::arc(cx, cy, radius, start_angle_arg, end_angle_arg, solidity) draws an arc.
        // It is assumed to draw CCW by default from start_angle_arg to end_angle_arg (shortest path).
        // To get a visually CW or CCW segment based on `props.clockwise`:
        if let Some(track) = &props.track {
            let track_val_start_frac = (track.start - props.min_value) / (props.max_value - props.min_value);
            let track_val_end_frac   = (track.end   - props.min_value) / (props.max_value - props.min_value);
            
            let track_segment_point_a_angle = p_start_angle_rad + track_val_start_frac * sweep_angle_rad;
            let track_segment_point_b_angle = p_start_angle_rad + track_val_end_frac * sweep_angle_rad;
            
            let (arc_draw_start_arg, arc_draw_end_arg) = if props.clockwise {
                // For a visually CW segment with a CCW arc primitive: draw from B to A.
                (track_segment_point_b_angle, track_segment_point_a_angle)
            } else {
                // For a visually CCW segment with a CCW arc primitive: draw from A to B.
                (track_segment_point_a_angle, track_segment_point_b_angle)
            };

            let mut track_path = Path::new();
            track_path.arc(center_x, center_y, gauge_radius * 0.96, arc_draw_start_arg, arc_draw_end_arg, Solidity::Hole);
            let mut track_paint = Paint::color(femtovg::Color::rgba(
                track.color[0], track.color[1], track.color[2], track.color[3],
            ));
            track_paint.set_line_width(gauge_radius * track.thickness);
            track_paint.set_anti_alias(true); // Enable AA for stroked paths
            canvas.stroke_path(&track_path, &track_paint);
        }

        // --- DRAW BORDER ARC ---
        if props.border_width > 0.0 {
            let mut border_path = Path::new();
            border_path.circle(center_x, center_y, gauge_radius);
            let mut border_paint = Paint::color(femtovg::Color::rgba(
                props.border_color[0], props.border_color[1], props.border_color[2], props.border_color[3],
            ));
            border_paint.set_line_width(props.border_width);
            border_paint.set_anti_alias(true); // Enable AA for stroked paths
            canvas.stroke_path(&border_path, &border_paint);
        }

        // --- DRAW DANGER ZONE ARC (Optional) ---
        if let Some(danger_zone_start_value) = props.danger_zone_start {
            if danger_zone_start_value < props.max_value {
                let danger_val_start_frac = (danger_zone_start_value - props.min_value) / (props.max_value - props.min_value);
                // Danger zone always extends to max_value (fraction = 1.0)
                let danger_segment_point_a_angle = p_start_angle_rad + danger_val_start_frac * sweep_angle_rad;
                let danger_segment_point_b_angle = p_start_angle_rad + 1.0 * sweep_angle_rad; // Angle for props.max_value
                
                let (arc_draw_start_arg, arc_draw_end_arg) = if props.clockwise {
                    (danger_segment_point_b_angle, danger_segment_point_a_angle)
                } else {
                    (danger_segment_point_a_angle, danger_segment_point_b_angle)
                };

                let mut danger_arc_path = Path::new();
                danger_arc_path.arc(center_x, center_y, gauge_radius * 0.92, arc_draw_start_arg, arc_draw_end_arg, Solidity::Hole);
                let mut danger_arc_paint = Paint::color(femtovg::Color::rgba(
                    props.tick_style.danger_zone_color[0],
                    props.tick_style.danger_zone_color[1],
                    props.tick_style.danger_zone_color[2],
                    props.tick_style.danger_zone_color[3],
                ));
                danger_arc_paint.set_line_width(props.tick_style.major_tick_width * 1.5); // Make distinct
                danger_arc_paint.set_anti_alias(true); // Enable AA for stroked paths
                canvas.stroke_path(&danger_arc_path, &danger_arc_paint);
            }
        }

        // --- DRAW TICKS AND LABELS ---
        // Tick/label/needle *positions* are determined by `p_start_angle_rad + fraction * sweep_angle_rad`.
        // This correctly places them visually CW or CCW based on the sign of `sweep_angle_rad`.
        let mut current_tick_value = props.min_value;
        while current_tick_value <= props.max_value + 0.0001 { // Add epsilon for float comparison robustness
            let is_major_tick = ((current_tick_value - props.min_value) % props.graduations.major_tick_interval).abs() < 0.001;
            let value_fraction = (current_tick_value - props.min_value) / (props.max_value - props.min_value);
            let angle_for_this_tick = p_start_angle_rad + value_fraction * sweep_angle_rad;

            let (tick_length_ratio, tick_line_width) = if is_major_tick {
                (props.tick_style.major_tick_length, props.tick_style.major_tick_width)
            } else {
                (props.tick_style.minor_tick_length, props.tick_style.minor_tick_width)
            };
            
            let r_outer = gauge_radius;
            let r_inner = gauge_radius * (1.0 - tick_length_ratio);
            let x0 = center_x + angle_for_this_tick.cos() * r_inner;
            let y0 = center_y + angle_for_this_tick.sin() * r_inner;
            let x1 = center_x + angle_for_this_tick.cos() * r_outer;
            let y1 = center_y + angle_for_this_tick.sin() * r_outer;
            
            let mut tick_path = Path::new();
            tick_path.move_to(x0, y0);
            tick_path.line_to(x1, y1);
            
            let mut tick_paint = Paint::color(femtovg::Color::rgba(
                props.tick_style.tick_color[0], props.tick_style.tick_color[1], 
                props.tick_style.tick_color[2], props.tick_style.tick_color[3],
            ));
            tick_paint.set_line_width(tick_line_width);
            tick_paint.set_anti_alias(true); // Enable AA for stroked lines
            canvas.stroke_path(&tick_path, &tick_paint);

            // Draw numerical label for major ticks
            if is_major_tick && props.graduations.show_labels {
                let label_distance_ratio = 1.0 - tick_length_ratio - 0.08; // Position labels inset from major ticks
                let lx = center_x + angle_for_this_tick.cos() * (gauge_radius * label_distance_ratio);
                let ly = center_y + angle_for_this_tick.sin() * (gauge_radius * label_distance_ratio);
                
                let mut tick_label_paint = Paint::color(femtovg::Color::rgb(255, 255, 255)); // Assuming white
                let scaled_tick_font_size = props.label_font_size * 0.7 * font_scale_factor; // Tick labels are 70% of main label size
                tick_label_paint.set_font_size(scaled_tick_font_size);
                tick_label_paint.set_text_align(Align::Center);
                tick_label_paint.set_text_baseline(Baseline::Middle);
                tick_label_paint.set_anti_alias(true); // Enable AA for text
                
                let label_text_content = format!("{:.*}", props.graduations.label_decimals as usize, current_tick_value);
                let _ = canvas.fill_text(lx, ly, &label_text_content, &tick_label_paint);
            }
            current_tick_value += props.graduations.minor_tick_interval;
        }

        // --- DRAW NEEDLE ---
        let current_value_fraction = (current_gauge_value - props.min_value) / (props.max_value - props.min_value);
        let angle_for_needle = p_start_angle_rad + current_value_fraction * sweep_angle_rad;
        let needle_length_abs = gauge_radius * props.needle.length;
        
        let needle_tip_x = center_x + angle_for_needle.cos() * needle_length_abs;
        let needle_tip_y = center_y + angle_for_needle.sin() * needle_length_abs;
        
        let mut needle_path = Path::new();
        let needle_pivot_x = center_x + props.needle.pivot.0 * gauge_radius;
        let needle_pivot_y = center_y + props.needle.pivot.1 * gauge_radius;
        needle_path.move_to(needle_pivot_x, needle_pivot_y);
        needle_path.line_to(needle_tip_x, needle_tip_y);
        
        let mut needle_paint = Paint::color(femtovg::Color::rgba(
            props.needle.color[0], props.needle.color[1], 
            props.needle.color[2], props.needle.color[3],
        ));
        needle_paint.set_line_width(props.needle.width);
        needle_paint.set_anti_alias(true); // Enable AA for stroked lines
        canvas.stroke_path(&needle_path, &needle_paint);

        // --- DRAW TEXT LABELS ---
        // Main Label (e.g., "RPM", "TURBO")
        let main_label_x = rect.x + props.label_position.0 * rect.width;
        let main_label_y = rect.y + props.label_position.1 * rect.height;
        let mut main_label_paint = Paint::color(femtovg::Color::rgb(255, 255, 255));
        main_label_paint.set_font_size(props.label_font_size * font_scale_factor);
        main_label_paint.set_text_align(Align::Center);
        main_label_paint.set_text_baseline(Baseline::Middle);
        main_label_paint.set_anti_alias(true);
        let _ = canvas.fill_text(main_label_x, main_label_y, &props.label, &main_label_paint);

        // Unit Label (e.g., "rpm", "bar")
        let unit_label_x = rect.x + props.unit_position.0 * rect.width;
        let unit_label_y = rect.y + props.unit_position.1 * rect.height;
        let mut unit_label_paint = Paint::color(femtovg::Color::rgb(255, 255, 255));
        unit_label_paint.set_font_size(props.unit_font_size * font_scale_factor);
        unit_label_paint.set_text_align(Align::Center);
        unit_label_paint.set_text_baseline(Baseline::Middle);
        unit_label_paint.set_anti_alias(true);
        let _ = canvas.fill_text(unit_label_x, unit_label_y, &props.unit, &unit_label_paint);

        // Current Value Text (Optional)
        if props.show_value {
            let value_text_x = rect.x + props.value_position.0 * rect.width;
            let value_text_y = rect.y + props.value_position.1 * rect.height;
            let mut value_text_paint = Paint::color(femtovg::Color::rgb(255, 255, 255));
            value_text_paint.set_font_size(props.value_font_size * font_scale_factor);
            value_text_paint.set_text_align(Align::Center);
            value_text_paint.set_text_baseline(Baseline::Middle);
            value_text_paint.set_anti_alias(true);
            let value_display_string = format!("{:.*}", props.value_decimals as usize, current_gauge_value);
            let _ = canvas.fill_text(value_text_x, value_text_y, &value_display_string, &value_text_paint);
        }
    }

    fn on_theme_change(&mut self, _new_theme: &crate::ui::theme::Theme, _transition: ThemeTransition) {
        // This method would be used if GaugeProps needed to be updated based on theme changes.
        // For example, if colors were not directly part of GaugeProps but derived from the Theme.
        // e.g., self.props.tick_color = new_theme.primary_color.into();
    }

    fn update(&mut self, _dt: Duration) {
        // This method is for time-based updates, like animations (e.g., needle smoothing).
        // Currently not used for basic gauge rendering.
    }

    fn preferred_size(&self, _ctx: &LayoutContext) -> WidgetGeometry {
        // Suggests a default size for the widget. The actual layout system might override this.
        WidgetGeometry::new(0.0, 0.0, GAUGE_DESIGN_REFERENCE_WIDTH, GAUGE_DESIGN_REFERENCE_WIDTH)
    }
} 