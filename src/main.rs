mod ui;
mod telemetry;
mod racebox;
mod esp32;
mod logging;

use winit::event_loop::EventLoop;
use tokio::sync::Mutex;
use std::sync::Arc;
use log::Level::Error;

#[tokio::main]
async fn main() {
    // Initialize logging
    logging::init_logging();

    // Create shared telemetry state
    let telemetry_state = Arc::new(Mutex::new(telemetry::TelemetryState::new()));

    // Start BLE listener for RaceBox Micro
    let telemetry_state_ble = telemetry_state.clone();
    tokio::spawn(async move {
        let telemetry_state_ble = telemetry_state_ble.clone();
        let telemetry_state_ble_data = telemetry_state_ble.clone();
        let telemetry_state_ble_error = telemetry_state_ble.clone();
        
        racebox::ble::start_ble_listener(
            move |data| {
                let state = telemetry_state_ble_data.clone();
                tokio::spawn(async move {
                    let mut state = state.lock().await;
                    state.latest_racebox_data = Some(data);
                    state.clear_racebox_error(); // Clear error on successful data
                });
            },
            move |error| {
                let state = telemetry_state_ble_error.clone();
                racebox_log!(Error, "BLE Error: {:?}", error);
                tokio::spawn(async move {
                    let mut state = state.lock().await;
                    state.set_racebox_error(format!("{:?}", error));
                });
            },
        );
    });

    // Start ESP32 connection
    let telemetry_state_esp32 = telemetry_state.clone();
    tokio::spawn(async move {
        let telemetry_state_esp32 = telemetry_state_esp32.clone();
        match esp32::ESP32Connection::start_listener(telemetry_state_esp32.clone()).await {
            Ok(_) => {
                let mut state = telemetry_state_esp32.lock().await;
                state.clear_esp32_error();
            }
            Err(e) => {
                esp32_log!(Error, "ESP32 Error: {:?}", e);
                let mut state = telemetry_state_esp32.lock().await;
                state.set_esp32_error(e.to_string());
            }
        }
    });

    // Create event loop
    let event_loop = EventLoop::new();

    // Run UI
    ui::run_ui(event_loop, telemetry_state);
}