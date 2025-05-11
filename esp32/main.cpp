#include <Arduino.h>
#include <math.h>
#include <stdint.h>

// --- TLV Type IDs (example, adjust to your protocol) ---
#define TLV_TYPE_RPM            0x01
#define TLV_TYPE_BOOST_PRESSURE 0x02
#define TLV_TYPE_OIL_PRESSURE   0x03
#define TLV_TYPE_FUEL_LEVEL     0x04
#define TLV_TYPE_SPEED          0x05
#define TLV_TYPE_STATUS_FLAGS   0x06
#define TLV_TYPE_STEERING_ANGLE 0x07
#define TLV_TYPE_BRAKE_PRESSURE 0x08
#define TLV_TYPE_THROTTLE_POS   0x09
#define TLV_TYPE_GEAR_POS       0x0A

// --- Serial port config ---
#define SERIAL_BAUD 115200

// --- Helper: Send TLV packet ---
void send_tlv(uint8_t type, uint8_t len, const uint8_t* value) {
    Serial.write(type);
    Serial.write(len);
    Serial.write(value, len);
}

// --- Helper: Send uint16_t as TLV ---
void send_tlv_u16(uint8_t type, uint16_t value) {
    uint8_t buf[2];
    buf[0] = value & 0xFF;
    buf[1] = (value >> 8) & 0xFF;
    send_tlv(type, 2, buf);
}

// --- Helper: Send int16_t as TLV ---
void send_tlv_i16(uint8_t type, int16_t value) {
    uint8_t buf[2];
    buf[0] = value & 0xFF;
    buf[1] = (value >> 8) & 0xFF;
    send_tlv(type, 2, buf);
}

// --- Helper: Send uint8_t as TLV ---
void send_tlv_u8(uint8_t type, uint8_t value) {
    send_tlv(type, 1, &value);
}

void setup() {
    Serial.begin(SERIAL_BAUD);
}

void loop() {
    static uint32_t t_ms = 0;
    float t = millis() / 1000.0f;

    // --- Mock values (oscillating, realistic ranges) ---
    uint16_t rpm            = 2000 + (int)(sin(t * 1.5) * 1500);         // 500..3500
    uint16_t boost_mbar     = (uint16_t)(500.0 + sin(t * 0.3) * 700.0);  // -0.2..1.2 bar (500..1200 mbar)
    uint16_t oil_pressure   = 2000 + (int)(cos(t * 0.2) * 200);          // 1800..2200
    uint16_t fuel_level     = 3000 + (int)(sin(t * 0.1) * 500);          // 2500..3500
    uint16_t speed          = 80 + (int)(sin(t * 0.2) * 40);             // 40..120
    uint8_t  status_flags   = 0b00000000;                                // All off
    int16_t  steering_angle = (int16_t)(sin(t * 0.5) * 300);             // -300..+300
    uint16_t brake_pressure = 1000 + (int)(cos(t * 0.7) * 500);          // 500..1500
    uint8_t  throttle_pos   = 50 + (uint8_t)(sin(t * 0.8) * 40);         // 10..90
    uint8_t  gear_pos       = 3 + (uint8_t)(sin(t * 0.2) * 2);           // 1..5

    // --- Send as TLV packets ---
    send_tlv_u16(TLV_TYPE_RPM,            rpm);
    send_tlv_u16(TLV_TYPE_BOOST_PRESSURE, boost_mbar);
    send_tlv_u16(TLV_TYPE_OIL_PRESSURE,   oil_pressure);
    send_tlv_u16(TLV_TYPE_FUEL_LEVEL,     fuel_level);
    send_tlv_u16(TLV_TYPE_SPEED,          speed);
    send_tlv_u8 (TLV_TYPE_STATUS_FLAGS,   status_flags);
    send_tlv_i16(TLV_TYPE_STEERING_ANGLE, steering_angle);
    send_tlv_u16(TLV_TYPE_BRAKE_PRESSURE, brake_pressure);
    send_tlv_u8 (TLV_TYPE_THROTTLE_POS,   throttle_pos);
    send_tlv_u8 (TLV_TYPE_GEAR_POS,       gear_pos);

    // --- Send at 20Hz ---
    delay(50);
}
