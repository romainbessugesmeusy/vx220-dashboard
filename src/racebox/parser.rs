#[derive(Debug, Clone)]
pub struct RaceBoxData {
    pub timestamp_ms: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub valid_time: bool,
    pub valid_date: bool,
    pub fix_status: u8,
    pub fix_ok: bool,
    pub num_sv: u8,
    pub latitude: f64,
    pub longitude: f64,
    pub wgs_alt: f64,
    pub msl_alt: f64,
    pub horiz_acc_mm: u32,
    pub vert_acc_mm: u32,
    pub speed_kph: f32,
    pub heading_deg: f32,
    pub speed_acc: f32,
    pub heading_acc: f32,
    pub pdop: f32,
    pub g_force_x: f32,
    pub g_force_y: f32,
    pub g_force_z: f32,
    pub rot_rate_x: f32,
    pub rot_rate_y: f32,
    pub rot_rate_z: f32,
}

pub fn parse_packet(data: &[u8]) -> Option<RaceBoxData> {
    if data.len() < 80 || data[0] != 0xB5 || data[1] != 0x62 || data[2] != 0xFF || data[3] != 0x01 {
        return None;
    }

    let timestamp_ms = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
    let year = u16::from_le_bytes([data[10], data[11]]);
    let month = data[12];
    let day = data[13];
    let hour = data[14];
    let minute = data[15];
    let second = data[16];
    let valid_flags = data[17];
    let valid_date = valid_flags & 0b0000_0001 != 0;
    let valid_time = valid_flags & 0b0000_0010 != 0;

    let fix_status = data[20];
    let fix_flags = data[21];
    let fix_ok = fix_flags & 0b0000_0001 != 0;
    let num_sv = data[23];

    let lon = i32::from_le_bytes([data[24], data[25], data[26], data[27]]);
    let lat = i32::from_le_bytes([data[28], data[29], data[30], data[31]]);
    let wgs_alt = i32::from_le_bytes([data[32], data[33], data[34], data[35]]) as f64 / 1000.0;
    let msl_alt = i32::from_le_bytes([data[36], data[37], data[38], data[39]]) as f64 / 1000.0;

    let horiz_acc_mm = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);
    let vert_acc_mm = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);
    let speed_mmps = i32::from_le_bytes([data[48], data[49], data[50], data[51]]);
    let heading_raw = i32::from_le_bytes([data[52], data[53], data[54], data[55]]);
    let speed_acc = u32::from_le_bytes([data[56], data[57], data[58], data[59]]) as f32 / 1000.0;
    let heading_acc = u32::from_le_bytes([data[60], data[61], data[62], data[63]]) as f32 / 100000.0;

    // Standard Deviation of Position in meters
    // It's a GNSS quality metric and helps to understand how precise the position is
    let pdop_raw = u16::from_le_bytes([data[64], data[65]]);
    let pdop = pdop_raw as f32 / 100.0;

    // G-Forces in milli-g's
    let g_force_x_raw = [data[68], data[69]];
    let g_force_y_raw = [data[70], data[71]];
    let g_force_z_raw = [data[72], data[73]];
    let g_force_x = i16::from_le_bytes(g_force_x_raw) as f32 / 1000.0;
    let g_force_y = i16::from_le_bytes(g_force_y_raw) as f32 / 1000.0;
    let g_force_z = i16::from_le_bytes(g_force_z_raw) as f32 / 1000.0;
    crate::racebox_log!(log::Level::Debug, "g_force_x_raw: {:?}, g_force_y_raw: {:?}, g_force_z_raw: {:?} | g_force_x: {}, g_force_y: {}, g_force_z: {}", g_force_x_raw, g_force_y_raw, g_force_z_raw, g_force_x, g_force_y, g_force_z);

    // Rotation Rates in centi-degrees per second
    let rot_rate_x = i16::from_le_bytes([data[74], data[75]]) as f32 / 100.0;
    let rot_rate_y = i16::from_le_bytes([data[76], data[77]]) as f32 / 100.0;
    let rot_rate_z = i16::from_le_bytes([data[78], data[79]]) as f32 / 100.0;

    Some(RaceBoxData {
        timestamp_ms,
        year,
        month,
        day,
        hour,
        minute,
        second,
        valid_date,
        valid_time,
        fix_status,
        fix_ok,
        num_sv,
        latitude: lat as f64 / 1e7,
        longitude: lon as f64 / 1e7,
        wgs_alt,
        msl_alt,
        horiz_acc_mm,
        vert_acc_mm,
        speed_kph: speed_mmps as f32 * 3.6 / 1000.0,
        heading_deg: heading_raw as f32 / 100000.0,
        speed_acc,
        heading_acc,
        pdop,
        g_force_x,
        g_force_y,
        g_force_z,
        rot_rate_x,
        rot_rate_y,
        rot_rate_z,
    })
} 