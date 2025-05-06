# ESP32 WROOM TLV-Style Payload

This document describes the structure of the payload sent by the ESP32 to the Raspberry Pi through UART, using a TLV (Type-Length-Value) format for extensibility and robustness.

---

## 1. Frame Structure

Each UART frame sent from the ESP32 to the Raspberry Pi has the following structure:

```
| HDR | LEN | VER |   TLV Payload   | CRC16 | EOF |
|-----|-----|-----|-----------------|-------|-----|
|0xAA |  1B | 1B  |   variable      |  2B   |0x55 |
```
- **HDR**: Start byte (0xAA)
- **LEN**: Payload length in bytes (excluding HDR, LEN, CRC16, EOF)
- **VER**: Protocol version (start at 0x01)
- **TLV Payload**: One or more TLV-encoded sensor values
- **CRC16**: CRC-16-CCITT (XModem or CCITT-FALSE) over VER and TLV Payload
- **EOF**: End byte (0x55)

---

## 2. TLV Payload Format

The payload consists of a sequence of TLV (Type-Length-Value) entries:

```
| ID  | L   | Value         |
|-----|-----|---------------|
| 1B  | 1B  | L bytes       |
```
- **ID**: Sensor or data type identifier (see table below)
- **L**: Length of Value in bytes
- **Value**: Raw data, big-endian encoding

Multiple TLVs can be concatenated in a single frame.

---

## 3. Sensor/Data Type ID Map

| ID   | Name                | Type    | Encoding         | Range/Scale         |
|------|---------------------|---------|------------------|---------------------|
| 0x01 | Fuel level          | uint16  | Big-endian       | 0–4095 (ADC)        |
| 0x02 | Oil pressure        | uint16  | Big-endian       | 0–4095 (ADC)        |
| 0x03 | Boost pressure      | uint16  | Big-endian       | 0–4095 (ADC)        |
| 0x04 | RPM                 | uint16  | Big-endian       | 0–16000             |
| 0x05 | Vehicle speed       | uint16  | Big-endian       | 0–400               |
| 0x06 | Status flags        | uint8   | Bitfield         | See below           |
| 0x07 | Steering angle      | int16   | Big-endian       | -7200 to +7200 (0.1°/LSB) |
| 0x08 | Brake pressure      | uint16  | Big-endian       | 0–20000 (0.01 bar/LSB) |
| 0x09 | Throttle position   | uint8   |                  | 0–100 (%)           |
| 0x0A | Gear position       | uint8   |                  | 0–7                 |
| 0x0B | Tyre pressure FL    | uint16  | Big-endian       | 0–400 (0.01 bar/LSB)|
| 0x0C | Tyre pressure FR    | uint16  | Big-endian       | 0–400 (0.01 bar/LSB)|
| 0x0D | Tyre pressure RL    | uint16  | Big-endian       | 0–400 (0.01 bar/LSB)|
| 0x0E | Tyre pressure RR    | uint16  | Big-endian       | 0–400 (0.01 bar/LSB)|
| 0x0F | Tyre temp FL        | int16   | Big-endian       | -200 to +2000 (0.1°C/LSB)|
| 0x10 | Tyre temp FR        | int16   | Big-endian       | -200 to +2000 (0.1°C/LSB)|
| 0x11 | Tyre temp RL        | int16   | Big-endian       | -200 to +2000 (0.1°C/LSB)|
| 0x12 | Tyre temp RR        | int16   | Big-endian       | -200 to +2000 (0.1°C/LSB)|

Unused codes (0x80–0xFF) are reserved for future use.

---

## 4. Status Flags (0x06)

Status flags are sent as a single byte (bitfield):
- Bit 0: MIL (Check Engine)
- Bit 1: ABS Warning
- Bit 2: Airbag Warning
- Bit 3: Left Turn
- Bit 4: Right Turn
- Bit 5: High Beam
- Bit 6: Parking Brake
- Bit 7: Reserved

---

## 5. Example Frame

Suppose the ESP32 sends fuel level, oil pressure, RPM, speed, status flags, and steering angle:

```
AA                  // HDR
14                  // LEN (20 bytes payload)
01                  // VER
01 02 0B D2         // TLV: Fuel level = 3026
02 02 03 20         // TLV: Oil pressure = 800
04 02 30 39         // TLV: RPM = 12345
05 02 00 96         // TLV: Speed = 150
06 01 12            // TLV: Flags = 0b00010010
07 02 01 F4         // TLV: Steering angle = +500 (50.0°)
A1 B4               // CRC16
55                  // EOF
```

---

## 6. Notes
- All multi-byte values are big-endian.
- CRC16 is calculated over VER and the entire TLV payload.
- The frame is extensible: new TLV IDs can be added without breaking compatibility.
- The Raspberry Pi should ignore unknown TLV IDs.

