use tokio::io::AsyncReadExt;
use tokio_serial::SerialStream;
use std::time::{Duration, Instant};
use crate::telemetry::{SharedTelemetryState, ESP32Data, StatusFlags};
use std::error::Error;
use crate::logging::ESP32_NAMESPACE;
use log::{debug, error, warn};

const UART_BAUD_RATE: u32 = 115200;
const UART_DEVICE: &str = "/dev/ttyS0"; // Default UART device on Raspberry Pi
const VALUE_RETENTION_MS: u64 = 500; // Keep values for 500ms after last update

pub struct ESP32Connection {
    port: SerialStream,
    last_update: Instant,
    last_values: ESP32Data,
}

impl ESP32Connection {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let port = SerialStream::open(&tokio_serial::new(UART_DEVICE, UART_BAUD_RATE)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_millis(1000)))?;

        Ok(Self { 
            port,
            last_update: Instant::now(),
            last_values: ESP32Data::default(),
        })
    }

    pub async fn start_listener(
        telemetry_state: SharedTelemetryState,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut connection = Self::new().await?;
        
        let mut buffer = Vec::with_capacity(1024);
        let mut frame_buffer = Vec::with_capacity(256);
        let mut last_successful_update = Instant::now();
        
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
                        if byte[0] == 0x55 && frame_buffer.len() >= 8 {
                            if let Ok(data) = Self::parse_frame(&frame_buffer) {
                                let state = telemetry_state.clone();
                                let update_time = Instant::now();
                                
                                // Update last successful update time and last_values
                                last_successful_update = update_time;
                                connection.last_update = update_time;
                                connection.last_values = data.clone();
                                
                                tokio::spawn(async move {
                                    let mut state = state.lock().await;
                                    state.latest_esp32_data = data;
                                    //debug!(target: ESP32_NAMESPACE, "Updated ESP32 data successfully");
                                });
                            }
                            frame_buffer.clear();
                        }
                    }
                }
                Err(e) => {
                    error!(target: ESP32_NAMESPACE, "ESP32 UART Error: {:?}", e);
                    // Attempt to reconnect after a delay
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    connection = Self::new().await?;
                }
            }

            // Only use retention logic if no new data has arrived for the retention period
            if last_successful_update.elapsed() >= Duration::from_millis(VALUE_RETENTION_MS) {
                let state = telemetry_state.clone();
                let last_values = connection.last_values.clone();
                debug!(target: ESP32_NAMESPACE, "Using retained ESP32 data due to signal interruption");
                tokio::spawn(async move {
                    if let Ok(mut state) = state.try_lock() {
                        state.latest_esp32_data = last_values;
                    }
                });
                // Update last_successful_update so we don't spam the UI
                last_successful_update = Instant::now();
            }
        }
    }

    fn parse_frame(frame: &[u8]) -> Result<ESP32Data, Box<dyn Error + Send + Sync>> {
        if frame.len() < 8 { // Minimum: HDR, LEN, VER, CRC16, EOF
            return Err("Frame too short".into());
        }
        // Frame: [0xAA][LEN][VER][TLV...][CRC16][0x55]
        let len = frame[1] as usize;
        let ver = frame[2];
        let tlv_start = 3;
        let tlv_end = 3 + (len - 1); // len includes VER + TLV
        if frame.len() < tlv_end + 3 {
            return Err("Frame length mismatch".into());
        }
        // CRC check
        let crc_offset = tlv_end;
        let crc_frame = &frame[2..crc_offset]; // VER + TLV
        let crc_recv = u16::from_be_bytes([frame[crc_offset], frame[crc_offset + 1]]);
        let crc_calc = {
            let mut crc = 0x0000u16;
            for &b in crc_frame {
                crc ^= (b as u16) << 8;
                for _ in 0..8 {
                    if crc & 0x8000 != 0 {
                        crc = (crc << 1) ^ 0x1021;
                    } else {
                        crc <<= 1;
                    }
                }
            }
            crc
        };
        if crc_recv != crc_calc {
            return Err("CRC mismatch".into());
        }
        if frame[crc_offset + 2] != 0x55 {
            return Err("Missing EOF byte".into());
        }
        let mut data = ESP32Data::default();
        let mut pos = tlv_start;
        while pos < crc_offset {
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
                _ => {}
            }
            pos += len;
        }
        Ok(data)
    }
} 