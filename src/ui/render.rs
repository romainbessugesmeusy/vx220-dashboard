// Remove all imgui usage and prepare for femtovg integration. Leave a placeholder for femtovg drawing code.

use femtovg::{Canvas, renderer::Renderer, Color, Paint, Path};
use crate::telemetry::SharedTelemetryState;
use crate::logging::UI_NAMESPACE;
use log::debug;
use crate::ui::widgets::{Widget, WidgetGeometry};
use crate::ui::widgets::g_force_meter::GForceMeter;
use crate::ui::widgets::theme::Theme;

pub fn render_ui<R: Renderer>(canvas: &mut Canvas<R>, telemetry_state: &SharedTelemetryState) {
    //debug!(target: UI_NAMESPACE, "Rendering UI {}x{}", canvas.width(), canvas.height());

    // Clear the canvas with a solid color
    canvas.clear_rect(0, 0, canvas.width() as u32, canvas.height() as u32, Color::rgb(40, 40, 80));

    // Create a Theme instance
    let theme = Theme::default();

    // Create a GForceMeter widget
    let g_force_meter = GForceMeter::new(
        WidgetGeometry::new(
            canvas.width() * 0.6, // X position - right side of screen
            canvas.height() * 0.3, // Y position - upper portion of screen
            canvas.width() * 0.3, // Width - 30% of screen width
            canvas.width() * 0.3, // Height - make it square with same size as width
        ),
        theme,
        2.0, // max_g_force_displayed
    );
    
    // Render the GForceMeter
    g_force_meter.render(canvas, telemetry_state);

    // Draw some text
    let mut text_paint = Paint::color(Color::rgb(255, 255, 255));
    text_paint.set_font_size(48.0);
    let _ = canvas.fill_text(50.0, 100.0, "VX220 Dashboard", &text_paint);

    // Draw debug info
    let mut debug_paint = Paint::color(Color::rgb(255, 255, 255));
    debug_paint.set_font_size(24.0);
    let _ = canvas.fill_text(
        50.0,
        150.0,
        &format!("Canvas size: {}x{}", canvas.width(), canvas.height()),
        &debug_paint,
    );

    // Draw telemetry data if available
    let state = match telemetry_state.try_lock() {
        Ok(state) => state,
        Err(_) => {
            debug!(target: UI_NAMESPACE, "Could not acquire telemetry state lock, skipping telemetry data");
            return;
        }
    };
    let mut y_position = 200.0;
    let x_position = 50.0;
    let y_spacing = 40.0;

    let mut text_paint = Paint::color(Color::rgb(255, 255, 255));
    text_paint.set_font_size(24.0);

    if let Some(data) = &state.latest_racebox_data {
        let _ = canvas.fill_text(
            x_position,
            y_position,
            &format!("Speed: {:.1} km/h", data.speed_kph),
            &text_paint,
        );
        y_position += y_spacing;
    } else {
        let _ = canvas.fill_text(
            x_position,
            y_position,
            "No RaceBox data available",
            &text_paint,
        );
        y_position += y_spacing;
    }

    if let Some(rpm) = state.latest_esp32_data.rpm {
        let _ = canvas.fill_text(
            x_position,
            y_position,
            &format!("RPM: {}", rpm),
            &text_paint,
        );
        y_position += y_spacing;
    } else {
        let _ = canvas.fill_text(
            x_position,
            y_position,
            "No RPM data available",
            &text_paint,
        );
        y_position += y_spacing;
    }

    if let Some(boost) = state.latest_esp32_data.boost_pressure {
        let _ = canvas.fill_text(
            x_position,
            y_position,
            &format!("Boost: {} mbar", boost),
            &text_paint,
        );
    } else {
        let _ = canvas.fill_text(
            x_position,
            y_position,
            "No boost data available",
            &text_paint,
        );
    }

    // Force a flush of the canvas
    canvas.flush();
} 
