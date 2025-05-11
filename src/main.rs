mod ui;
mod telemetry;
mod racebox;
mod esp32;
mod logging;

use winit::event_loop::EventLoop;
use tokio::sync::Mutex;
use std::sync::Arc;
use log::Level::Error;
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use crate::telemetry::{DriveMode, ColorScheme};

#[tokio::main]
async fn main() {
    // Initialize logging
    logging::init_logging();

    // Create shared telemetry state
    let telemetry_state = Arc::new(Mutex::new(telemetry::TelemetryState::new()));

    // Start mock telemetry if enabled
    telemetry::maybe_start_mock_telemetry(telemetry_state.clone()).await;

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

    // Start the command listener (in a background thread)
    start_command_listener(telemetry_state.clone());

    // Create event loop
    let event_loop = EventLoop::new();

    // Run UI
    ui::run_ui(event_loop, telemetry_state);
}

fn start_command_listener(telemetry_state: Arc<Mutex<telemetry::TelemetryState>>) {
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind TCP listener");
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                handle_command(stream, &telemetry_state);
            }
        }
    });
}

fn handle_command(mut stream: TcpStream, telemetry_state: &Arc<Mutex<telemetry::TelemetryState>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    for line in reader.lines() {
        if let Ok(cmd) = line {
            let mut state = telemetry_state.blocking_lock();
            let tokens: Vec<_> = cmd.trim().split_whitespace().collect();
            let mut response = "OK\n".to_string();
            if tokens.len() == 2 && tokens[0] == "set_mode" {
                match tokens[1] {
                    "Road" => state.set_drive_mode(DriveMode::Road),
                    "Track" => state.set_drive_mode(DriveMode::Track),
                    _ => response = format!("ERR invalid mode: {}\n", tokens[1]),
                }
            } else if tokens.len() == 2 && tokens[0] == "set_scheme" {
                match tokens[1] {
                    "Light" => state.set_color_scheme(ColorScheme::Light),
                    "Dark" => state.set_color_scheme(ColorScheme::Dark),
                    _ => response = format!("ERR invalid scheme: {}\n", tokens[1]),
                }
            } else {
                response = "ERR unknown command\n".to_string();
            }
            let _ = stream.write_all(response.as_bytes());
            break;
        }
    }
}