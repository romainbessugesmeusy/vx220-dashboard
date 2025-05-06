use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::telemetry::{SharedTelemetryState, TelemetryError, ESP32Data};
use crate::racebox::parser::RaceBoxData;
use rand::rngs::SmallRng;
use rand::{SeedableRng, Rng};

pub async fn start_mock_telemetry(telemetry_state: SharedTelemetryState) {
    tokio::spawn(async move {
        let mut t: f32 = 0.0;
        loop {
            // Use a local SmallRng for Send safety
            let mut rng = SmallRng::from_entropy();
            let g_force_x = (t).sin() * 1.2 + rng.gen_range(-0.05..0.05);
            let g_force_y = (t * 0.7).cos() * 1.0 + rng.gen_range(-0.05..0.05);
            let g_force_z = 1.0 + (t * 0.3).sin() * 0.2 + rng.gen_range(-0.02..0.02);
            let speed_kph = 80.0 + (t * 0.2).sin() * 40.0;
            let heading_deg = (t * 10.0) % 360.0;

            let racebox_data = RaceBoxData {
                timestamp_ms: (t * 1000.0) as u32,
                year: 2024,
                month: 6,
                day: 1,
                hour: 12,
                minute: 0,
                second: 0,
                valid_time: true,
                valid_date: true,
                fix_status: 3,
                fix_ok: true,
                num_sv: 12,
                latitude: 48.123456,
                longitude: 11.654321,
                wgs_alt: 500.0,
                msl_alt: 495.0,
                horiz_acc_mm: 1000,
                vert_acc_mm: 1500,
                speed_kph,
                heading_deg,
                speed_acc: 0.2,
                heading_acc: 0.5,
                pdop: 1.2,
                g_force_x,
                g_force_y,
                g_force_z,
                rot_rate_x: (t * 0.5).sin() * 10.0,
                rot_rate_y: (t * 0.3).cos() * 10.0,
                rot_rate_z: (t * 0.2).sin() * 10.0,
            };

            let esp32_data = ESP32Data {
                fuel_level: Some(3000 + ((t * 0.1).sin() * 500.0) as u16),
                oil_pressure: Some(2000 + ((t * 0.2).cos() * 200.0) as u16),
                boost_pressure: Some(1500 + ((t * 0.3).sin() * 300.0) as u16),
                rpm: Some(2000 + ((t * 1.5).sin() * 1500.0) as u16),
                speed: Some(speed_kph as u16),
                status_flags: None,
                steering_angle: Some(((t * 0.5).sin() * 300.0) as i16),
                brake_pressure: Some(1000 + ((t * 0.7).cos() * 500.0) as u16),
                throttle_position: Some((50.0 + (t * 0.8).sin() * 40.0) as u8),
                gear_position: Some(3 + ((t * 0.2).sin() * 2.0) as u8),
                tyre_pressures: [Some(2200), Some(2200), Some(2100), Some(2100)],
                tyre_temps: [Some(300), Some(305), Some(295), Some(290)],
            };

            {
                let mut state = telemetry_state.lock().await;
                state.latest_racebox_data = Some(racebox_data);
                state.latest_esp32_data = esp32_data;
                state.racebox_error = None;
                state.esp32_error = None;
            }

            t += 0.05;
            sleep(Duration::from_millis(50)).await;
        }
    });
} 