use tokio::io::AsyncReadExt;
use tokio_serial::SerialStream;
use std::time::Duration;
use crate::telemetry::{SharedTelemetryState, ESP32Data, StatusFlags};
use std::error::Error;

const UART_BAUD_RATE: u32 = 115200;
const UART_DEVICE: &str = "/dev/ttyAMA0"; // Default UART device on Raspberry Pi

pub struct ESP32Connection {
    port: SerialStream,
}

impl ESP32Connection {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let port = SerialStream::open(&tokio_serial::new(UART_DEVICE, UART_BAUD_RATE)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_millis(100)))?;

        Ok(Self { port })
    }

    pub async fn start_listener(
        telemetry_state: SharedTelemetryState,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = Self::new().await?;
        
        let mut buffer = Vec::with_capacity(1024);
        let mut frame_buffer = Vec::with_capacity(256);
        
        loop {
            let mut byte = [0u8; 1];
            match connection.port.read_exact(&mut byte).await {
                Ok(_) => {
                    buffer.push(byte[0]);
                    
                    // Look for frame start (0xAA)
                    if byte[0] == 0xAA {
                        frame_buffer.clear();
                        frame_buffer.push(byte[0]);
                    } else if !frame_buffer.is_empty() {
                        frame_buffer.push(byte[0]);
                        
                        // Check for frame end (0x55)
                        if byte[0] == 0x55 && frame_buffer.len() >= 4 {
                            if let Ok(data) = Self::parse_frame(&frame_buffer) {
                                let state = telemetry_state.clone();
                                tokio::spawn(async move {
                                    let mut state = state.lock().await;
                                    state.latest_esp32_data = data;
                                });
                            }
                            frame_buffer.clear();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ESP32 UART Error: {:?}", e);
                    // Attempt to reconnect after a delay
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    connection = Self::new().await?;
                }
            }
        }
    }

    fn parse_frame(frame: &[u8]) -> Result<ESP32Data, Box<dyn Error + Send + Sync>> {
        if frame.len() < 4 {
            return Err("Frame too short".into());
        }

        let mut data = ESP32Data::default();
        let mut pos = 3; // Skip header (0xAA), length, and version

        while pos < frame.len() - 3 { // Leave room for CRC and EOF
            let id = frame[pos];
            let len = frame[pos + 1] as usize;
            pos += 2;

            match id {
                0x01 => data.fuel_level = Some(u16::from_be_bytes([frame[pos], frame[pos + 1]])),
                0x02 => data.oil_pressure = Some(u16::from_be_bytes([frame[pos], frame[pos + 1]])),
                0x03 => data.boost_pressure = Some(u16::from_be_bytes([frame[pos], frame[pos + 1]])),
                0x04 => data.rpm = Some(u16::from_be_bytes([frame[pos], frame[pos + 1]])),
                0x05 => data.speed = Some(u16::from_be_bytes([frame[pos], frame[pos + 1]])),
                0x06 => data.status_flags = Some(StatusFlags::from_byte(frame[pos])),
                0x07 => data.steering_angle = Some(i16::from_be_bytes([frame[pos], frame[pos + 1]])),
                0x08 => data.brake_pressure = Some(u16::from_be_bytes([frame[pos], frame[pos + 1]])),
                0x09 => data.throttle_position = Some(frame[pos]),
                0x0A => data.gear_position = Some(frame[pos]),
                0x0B..=0x0E => {
                    let idx = (id - 0x0B) as usize;
                    data.tyre_pressures[idx] = Some(u16::from_be_bytes([frame[pos], frame[pos + 1]]));
                }
                0x0F..=0x12 => {
                    let idx = (id - 0x0F) as usize;
                    data.tyre_temps[idx] = Some(i16::from_be_bytes([frame[pos], frame[pos + 1]]));
                }
                _ => {} // Ignore unknown IDs
            }

            pos += len;
        }

        Ok(data)
    }
} 