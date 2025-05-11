use std::sync::Arc;
use tokio::sync::Mutex;
use crate::racebox::parser::RaceBoxData;
use std::time::Instant;

/// Status flags from the ESP32, representing various vehicle warning states
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusFlags {
    /// Check Engine Light (MIL - Malfunction Indicator Lamp)
    pub mil: bool,
    /// ABS Warning Light
    pub abs_warning: bool,
    /// Airbag Warning Light
    pub airbag_warning: bool,
    /// Left Turn Signal Active
    pub left_turn: bool,
    /// Right Turn Signal Active
    pub right_turn: bool,
    /// High Beam Headlights Active
    pub high_beam: bool,
    /// Parking Brake Engaged
    pub parking_brake: bool,
    /// Reserved for future use
    pub reserved: bool,
}

impl StatusFlags {
    /// Convert a byte into StatusFlags
    pub fn from_byte(byte: u8) -> Self {
        Self {
            mil: (byte & 0x01) != 0,
            abs_warning: (byte & 0x02) != 0,
            airbag_warning: (byte & 0x04) != 0,
            left_turn: (byte & 0x08) != 0,
            right_turn: (byte & 0x10) != 0,
            high_beam: (byte & 0x20) != 0,
            parking_brake: (byte & 0x40) != 0,
            reserved: (byte & 0x80) != 0,
        }
    }

    /// Convert StatusFlags into a byte
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.mil { byte |= 0x01; }
        if self.abs_warning { byte |= 0x02; }
        if self.airbag_warning { byte |= 0x04; }
        if self.left_turn { byte |= 0x08; }
        if self.right_turn { byte |= 0x10; }
        if self.high_beam { byte |= 0x20; }
        if self.parking_brake { byte |= 0x40; }
        if self.reserved { byte |= 0x80; }
        byte
    }
}

#[derive(Default)]
pub struct ESP32Data {
    pub fuel_level: Option<u16>,
    pub oil_pressure: Option<u16>,
    pub boost_pressure: Option<u16>,
    pub rpm: Option<u16>,
    pub speed: Option<u16>,
    pub status_flags: Option<StatusFlags>,
    pub steering_angle: Option<i16>,
    pub brake_pressure: Option<u16>,
    pub throttle_position: Option<u8>,
    pub gear_position: Option<u8>,
    pub tyre_pressures: [Option<u16>; 4],
    pub tyre_temps: [Option<i16>; 4],
}

#[derive(Debug, Clone)]
pub enum TelemetryError {
    BLE(String),
    ESP32(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveMode {
    Road,
    Track,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
    // HighContrast, // for future use
}

pub struct TelemetryState {
    pub latest_racebox_data: Option<RaceBoxData>,
    pub latest_esp32_data: ESP32Data,
    pub racebox_error: Option<(TelemetryError, Instant)>,
    pub esp32_error: Option<(TelemetryError, Instant)>,
    pub drive_mode: DriveMode,
    pub color_scheme: ColorScheme,
}

impl TelemetryState {
    pub fn new() -> Self {
        Self {
            latest_racebox_data: None,
            latest_esp32_data: ESP32Data::default(),
            racebox_error: None,
            esp32_error: None,
            drive_mode: DriveMode::Road,
            color_scheme: ColorScheme::Light,
        }
    }

    pub fn set_racebox_error(&mut self, error: String) {
        self.racebox_error = Some((TelemetryError::BLE(error), Instant::now()));
    }

    pub fn set_esp32_error(&mut self, error: String) {
        self.esp32_error = Some((TelemetryError::ESP32(error), Instant::now()));
    }

    pub fn clear_racebox_error(&mut self) {
        self.racebox_error = None;
    }

    pub fn clear_esp32_error(&mut self) {
        self.esp32_error = None;
    }

    pub fn set_drive_mode(&mut self, mode: DriveMode) {
        self.drive_mode = mode;
    }

    pub fn get_drive_mode(&self) -> DriveMode {
        self.drive_mode
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    pub fn get_color_scheme(&self) -> ColorScheme {
        self.color_scheme
    }
}

pub type SharedTelemetryState = Arc<Mutex<TelemetryState>>;

#[cfg(feature = "mock_telemetry")]
pub mod mock;

#[cfg(feature = "mock_telemetry")]
pub async fn maybe_start_mock_telemetry(telemetry_state: SharedTelemetryState) {
    mock::start_mock_telemetry(telemetry_state).await;
}

#[cfg(not(feature = "mock_telemetry"))]
pub async fn maybe_start_mock_telemetry(_telemetry_state: SharedTelemetryState) {
    // No-op in real mode
} 