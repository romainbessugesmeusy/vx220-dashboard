use log::{debug, error, info, trace, warn, LevelFilter};
use std::env;

pub const UI_NAMESPACE: &str = "vx220::ui";
pub const TELEMETRY_NAMESPACE: &str = "vx220::telemetry";
pub const RACEBOX_NAMESPACE: &str = "vx220::racebox";
pub const ESP32_NAMESPACE: &str = "vx220::esp32";

pub fn init_logging() {
    // Set default log level if not specified in environment
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }

    // Configure env_logger
    env_logger::Builder::from_env(env_logger::Env::default())
        .format_timestamp_millis()
        .format_module_path(true)
        .format_target(true)
        .filter(Some(UI_NAMESPACE), LevelFilter::Debug)
        .filter(Some(TELEMETRY_NAMESPACE), LevelFilter::Debug)
        .filter(Some(RACEBOX_NAMESPACE), LevelFilter::Debug)
        .filter(Some(ESP32_NAMESPACE), LevelFilter::Debug)
        .init();

    info!("Logging initialized");
}

// Convenience macros for each namespace
#[macro_export]
macro_rules! ui_log {
    ($($arg:tt)*) => {
        log::log!(target: $crate::logging::UI_NAMESPACE, $($arg)*)
    };
}

#[macro_export]
macro_rules! telemetry_log {
    ($($arg:tt)*) => {
        log::log!(target: $crate::logging::TELEMETRY_NAMESPACE, $($arg)*)
    };
}

#[macro_export]
macro_rules! racebox_log {
    ($($arg:tt)*) => {
        log::log!(target: $crate::logging::RACEBOX_NAMESPACE, $($arg)*)
    };
}

#[macro_export]
macro_rules! esp32_log {
    ($($arg:tt)*) => {
        log::log!(target: $crate::logging::ESP32_NAMESPACE, $($arg)*)
    };
} 