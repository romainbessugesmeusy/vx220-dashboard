// Remove all imgui usage and prepare for femtovg integration. Leave a placeholder for femtovg drawing code.

use femtovg::{Canvas, renderer::Renderer, Color, Paint, Path};
use crate::telemetry::SharedTelemetryState;
use crate::logging::UI_NAMESPACE;
use log::debug;
use crate::ui::widgets::{Widget, WidgetGeometry};
use crate::ui::widgets::g_force_meter::GForceMeter;
use crate::ui::theme::Theme;
use crate::telemetry::{DriveMode, ColorScheme};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use crate::ui::widgets::turbo_pressure_gauge::TurboPressureGauge;
use crate::ui::widgets::rpm_gauge::RpmGauge;

#[derive(Clone, Copy)]
enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

fn apply_easing(easing: EasingFunction, t: f32) -> f32 {
    match easing {
        EasingFunction::Linear => t,
        EasingFunction::EaseIn => t * t,
        EasingFunction::EaseOut => t * (2.0 - t),
        EasingFunction::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
    }
}

struct ThemeTransitionState {
    current_theme: Theme,
    next_theme: Option<Theme>,
    start_time: Option<Instant>,
    duration: Duration,
    easing: EasingFunction,
}

impl ThemeTransitionState {
    fn new(initial_theme: Theme, duration: Duration, easing: EasingFunction) -> Self {
        Self {
            current_theme: initial_theme,
            next_theme: None,
            start_time: None,
            duration,
            easing,
        }
    }
    fn start_transition(&mut self, new_theme: Theme) {
        if self.current_theme.background_color != new_theme.background_color {
            self.next_theme = Some(new_theme);
            self.start_time = Some(Instant::now());
        } else {
            self.current_theme = new_theme;
            self.next_theme = None;
            self.start_time = None;
        }
    }
    fn update(&mut self) {
        if let (Some(next), Some(start)) = (&self.next_theme, self.start_time) {
            let elapsed = start.elapsed();
            let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
            let t_eased = apply_easing(self.easing, t);
            let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t_eased) as u8;
            let bg = [
                lerp(self.current_theme.background_color[0], next.background_color[0]),
                lerp(self.current_theme.background_color[1], next.background_color[1]),
                lerp(self.current_theme.background_color[2], next.background_color[2]),
                lerp(self.current_theme.background_color[3], next.background_color[3]),
            ];
            self.current_theme.background_color = bg;
            if t >= 1.0 {
                self.current_theme = next.clone();
                self.next_theme = None;
                self.start_time = None;
            }
        }
    }
    fn get_theme(&self) -> &Theme {
        &self.current_theme
    }
}

thread_local! {
    static THEME_TRANSITION_STATE: RefCell<Option<ThemeTransitionState>> = RefCell::new(None);
    static LAST_PRESET: RefCell<Option<(DriveMode, ColorScheme)>> = RefCell::new(None);
}

pub fn render_ui<R: Renderer>(canvas: &mut Canvas<R>, telemetry_state: &SharedTelemetryState) {
    //debug!(target: UI_NAMESPACE, "Rendering UI {}x{}", canvas.width(), canvas.height());

    // Get drive mode and color scheme from state
    let (drive_mode, color_scheme) = {
        let state = match telemetry_state.try_lock() {
            Ok(state) => state,
            Err(_) => {
                debug!(target: UI_NAMESPACE, "Could not acquire telemetry state lock, skipping telemetry data");
                return;
            }
        };
        (state.get_drive_mode(), state.get_color_scheme())
    };
    let target_theme = Theme::from_preset(drive_mode, color_scheme);

    // Check if we need to start a new transition
    let last_preset = LAST_PRESET.with(|lp| *lp.borrow());
    let need_transition = match last_preset {
        Some((last_drive, last_color)) => last_drive != drive_mode || last_color != color_scheme,
        None => true,
    };
    let transition_duration = Duration::from_secs(1);
    let easing = EasingFunction::EaseInOut;
    let theme = THEME_TRANSITION_STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.is_none() {
            *state = Some(ThemeTransitionState::new(target_theme.clone(), transition_duration, easing));
        }
        let s = state.as_mut().unwrap();
        if s.next_theme.is_none() && s.current_theme != target_theme {
            s.start_transition(target_theme.clone());
        }
        s.update();
        s.get_theme().clone()
    });
    if need_transition {
        LAST_PRESET.with(|lp| *lp.borrow_mut() = Some((drive_mode, color_scheme)));
    }

    // Clear the canvas with the theme's background color
    canvas.clear_rect(0, 0, canvas.width() as u32, canvas.height() as u32, Theme::color4(theme.background_color));

    // Create a GForceMeter widget
    let mut g_force_meter = GForceMeter::new(
        theme.clone(),
        2.0, // max_g_force_displayed
    );
    // Example: handle theme change (in a real app, this would be tracked across frames)
    // g_force_meter.on_theme_change(&theme, ThemeTransition { from: theme.clone(), to: theme.clone(), progress: 1.0 });
    // Example: update per frame (dt should be passed in from main loop)
    // g_force_meter.update(dt);
    // Layout: place it on the right side of the screen, 30% width, square
    let g_force_rect = WidgetGeometry::new(
        canvas.width() * 0.6, // X position - right side of screen
        canvas.height() * 0.3, // Y position - upper portion of screen
        canvas.width() * 0.3, // Width - 30% of screen width
        canvas.width() * 0.3, // Height - make it square with same size as width
    );
    g_force_meter.render(canvas, g_force_rect, telemetry_state);

    // Create a TurboPressureGauge widget
    let mut turbo_gauge = TurboPressureGauge::new(&theme);
    // Set value from telemetry if available
    if let Ok(state) = telemetry_state.try_lock() {
        if let Some(boost) = state.latest_esp32_data.boost_pressure {
            // Convert mbar to bar if needed (assuming boost is in mbar)
            turbo_gauge.set_value(boost as f32 / 1000.0);
        }
    }
    // Layout: place it on the left side of the screen, 30% width, square
    let turbo_gauge_rect = WidgetGeometry::new(
        canvas.width() * 0.05, // X position - left margin
        canvas.height() * 0.3, // Y position - upper portion of screen
        canvas.width() * 0.3, // Width - 30% of screen width
        canvas.width() * 0.3, // Height - make it square with same size as width
    );
    turbo_gauge.render(canvas, turbo_gauge_rect, telemetry_state);

    // Create an RPM Gauge widget
    let mut rpm_gauge = RpmGauge::new(&theme);
    // Set value from telemetry if available
    if let Ok(state) = telemetry_state.try_lock() {
        if let Some(rpm) = state.latest_esp32_data.rpm {
            rpm_gauge.set_value(rpm as f32);
        }
    }
    // Layout: place it in the center of the screen, 30% width, square
    let rpm_gauge_rect = WidgetGeometry::new(
        canvas.width() * 0.35, // X position - center
        canvas.height() * 0.3, // Y position - upper portion of screen
        canvas.width() * 0.3, // Width - 30% of screen width
        canvas.width() * 0.3, // Height - make it square with same size as width
    );
    rpm_gauge.render(canvas, rpm_gauge_rect, telemetry_state);

    // Draw some text
    let mut text_paint = Paint::color(Theme::color3(theme.text_color));
    text_paint.set_font_size(48.0);
    let _ = canvas.fill_text(50.0, 100.0, "VX220 Dashboard", &text_paint);

    // Draw debug info
    let mut debug_paint = Paint::color(Theme::color3(theme.text_color));
    debug_paint.set_font_size(24.0);
    let _ = canvas.fill_text(
        50.0,
        150.0,
        &format!("Canvas size: {}x{} | Mode: {:?} | Scheme: {:?}", canvas.width(), canvas.height(), drive_mode, color_scheme),
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

    let mut text_paint = Paint::color(Theme::color3(theme.text_color));
    text_paint.set_font_size(24.0);

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
