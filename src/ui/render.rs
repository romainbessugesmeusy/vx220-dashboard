// Remove all imgui usage and prepare for femtovg integration. Leave a placeholder for femtovg drawing code.

use femtovg::{Canvas, renderer::Renderer, Color, Paint, Path};
use crate::telemetry::SharedTelemetryState;
use crate::logging::UI_NAMESPACE;
use log::debug;

pub fn render_ui<R: Renderer>(canvas: &mut Canvas<R>, telemetry_state: &SharedTelemetryState) {
    //debug!(target: UI_NAMESPACE, "Rendering UI {}x{}", canvas.width(), canvas.height());

    // Clear the canvas with a solid color
    canvas.clear_rect(0, 0, canvas.width() as u32, canvas.height() as u32, Color::rgb(40, 40, 80));

    // Draw a filled red rectangle
    let mut path = Path::new();
    path.rect(50.0, 50.0, 100.0, 100.0);
    let paint = Paint::color(Color::rgb(255, 0, 0));
    canvas.fill_path(&path, &paint);

    // Draw a filled green rectangle
    let mut path = Path::new();
    path.rect(200.0, 50.0, 100.0, 100.0);
    let paint = Paint::color(Color::rgb(0, 255, 0));
    canvas.fill_path(&path, &paint);

    // Draw a filled blue circle
    let mut path = Path::new();
    path.circle(400.0, 100.0, 50.0);
    let paint = Paint::color(Color::rgb(0, 0, 255));
    canvas.fill_path(&path, &paint);

    // Draw some text
    let mut text_paint = Paint::color(Color::rgb(255, 255, 255));
    text_paint.set_font_size(48.0);
    let _ = canvas.fill_text(50.0, 200.0, "VX220 Dashboard", &text_paint);

    // Draw debug info
    let mut debug_paint = Paint::color(Color::rgb(255, 255, 255));
    debug_paint.set_font_size(24.0);
    let _ = canvas.fill_text(
        50.0,
        250.0,
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
    let mut y_position = 300.0;
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
