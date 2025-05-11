# vx220-dashboard

Modular Racecar/Roadcar dashboard for the Vauxhall VX220 Turbo.

## Physical Components

- Raspberry Pi 4
- UPS Hat for Raspberry Pi 4
- ESP32 WROOM
- FPV Camera for real-time rear view
- Racebox Micro

## Software Features

### Regular Features

- RPM Gauge
- Speedometer
- Telltale for various warnings
- Fuel gauge
- Water temperature gauge
- Oil temperature gauge
- Oil pressure gauge
- Battery voltage gauge
- Gear indicator
- Shift light
- Turbo pressure gauge
- Turbo temperature gauge

### Racing Features

- Telemetry from Racebox Micro
- GPS
- Speed
- Gyroscope
- G-force
- Lap timer
- Lap counter

### Nice to have

- Steering angle sensor
- Brake pressure sensor
- Throttle position sensor
- Gear position sensor

## Software Architecture

### Main Application

The main application is built with femtovg and Rust, and runs on the Raspberry Pi 4. The UI is running on the main thread, while the telemetry is running on a separate thread. 

The raspberry pi is sending video signal to two HDMI monitors. One behind the steering wheel and one in the center console. The center console monitor will primarily display the rear view camera feed in race mode, and a custom UI in road mode.

#### UI

The UI is built with femtovg, and is located under the ui/ folder. It will make use of custom widgets defined in the src/ui/widgets/ folder. Graphical assets are located under the assets/ folder.

__Why femtovg?__
- femtovg is a lightweight, high-performance 2D vector graphics library
- It provides modern OpenGL-based rendering
- It has good Rust bindings and is actively maintained
- It's well-suited for real-time dashboard applications

#### Connection to Racebox Micro

The Racebox Micro is running its own firmware that sends the telemetry data to the Raspberry Pi 4 over BLE. The code responsible for interacting with the Racebox Micro is located under the racebox module. It has been generated according to the specifications of the manufacturer, accessible under the docs/ folder. Because this file is proprietary, it is gitignored. For more information, please contact the manufacturer.

1. The application initiates a scan for BLE devices.
2. The application connects to the first "Racebox Micro" device found.
3. The application subscribes to the notifications from the Racebox Micro (TX_CHAR_UUID).
4. The application parses the incoming data and stores it in a buffer in the form of a RaceBoxData struct.
5. The application periodically flushes the buffer to the main thread.

The RaceBoxData struct is defined in the racebox/parser.rs file. It is used by the main application to populate UI widgets.

#### ESP32 WROOM

The ESP32 WROOM is running a custom firmware that acts as a analog to digital converter for the sensors. It then sends the data to the Raspberry Pi 4 over UART. 

```ESP32 GPIO17 (TX) ──────────────> Pi GPIO15 (RX) [Pin 10]
ESP32 GPIO16 (RX) <───────────── Pi GPIO14 (TX) [Pin 8]
ESP32 GND        ─────────────── Pi GND         [Pin 6]
``` 

Analog information will be harvested from the existing dashboard connector, while additional 3rd-party analog feeds (for now turbo pressure & oil temperature), will be connected independently. 

| Signal Name | Signal Type | Source Unit | Voltage Level | Connector Pin (est.) | Output to Raspberry Pi GPIO | ADC Resolution / Interface Suggestion |
| --- | --- | --- | --- | --- | --- | --- |
| RPM | Frequency-based (pulse train) | ECU tach output or coil trigger | ~12V square wave | ECU pin C8 (to cluster) | Frequency counter via microcontroller (interrupt-based) | Not applicable – count pulses per second |
| Coolant Temp | PWM duty-cycle | ECU PWM output | ~12V PWM | ECU pin C14 (to cluster) | Duty cycle reader via microcontroller PWM input or filtered ADC | 10-bit+ ADC or timer-capable MCU |
| Speed | Frequency-based (pulse train) | ABS module | ~12V square wave | ABS pin 25 (speed pulse) | Frequency counter or pulse capture (optocoupler to GPIO) | Not applicable – frequency-based |
| Fuel Level | Resistive analog | Tank level sender | 0–5V (via divider) | Cluster pin 11 (X30 plug) | Analog input via voltage divider to ADC | 10–12 bit ADC recommended |
| Oil Pressure (stock)m | On/Off (binary switch) | Oil pressure switch | 12V switched ground | Cluster pin 6 (X30 plug) | Digital GPIO with optocoupler or level shifter | None – digital only |
| Oil Pressure (aftermarket) | Analog 0–5V sensor | External sensor (0–150 psi) | 0–5V | Custom input (T-fitting) | Buffered analog input to ADC | 10–12 bit ADC (e.g. MCP3008) |
| Turbo Boost (MAP) | Analog 0–5V sensor | MAP sensor on intake manifold | 0–5V | ECU pin C18 or C5 | Analog input (buffered) | 10–12 bit ADC |
| MIL (Check Engine) | On/Off (binary) | ECU warning output | 12V when active | ECU pin C16 or cluster pin 17 | GPIO with optocoupler | None – digital only |
| ABS Warning | On/Off (binary) | ABS module | 12V switched | Cluster pin 12 (X30) | GPIO with level shifting or optocoupler | None |
| Airbag Warning | On/Off (binary) | Airbag ECU | 12V switched | Cluster pin 5 (X30) | GPIO with optocoupler | None |
| Turn Signals (L/R) | On/Off (12V pulsed) | Flasher relay outputs | 12V flashing | Cluster pins 9/10 (X30) | Edge detection or GPIO with pulse detection | Optional smoothing or interrupt logic |
| High Beam | On/Off (12V line) | Headlight relay | 12V constant | Cluster pin 8 (X30) | GPIO with resistor divider or opto-isolation | None |
| Parking Brake | On/Off (switch) | Handbrake lever switch | 12V switched ground | Cluster pin 4 (X30) | GPIO digital input | None |

Documentation for the ESP32 -> Raspberry Pi 4 payload can be found in the [ESP32 TLV Payload Specification](docs/esp32-payload.md) file. This document details the TLV (Type-Length-Value) frame structure, sensor data types, and provides examples of the communication protocol between the ESP32 and Raspberry Pi.

#### Video Feed

The Video feed is captured by a PAL/SECAM FPV camera (Toothless), connected to a USB converter. The camera is mounted on top of the car, where was previously mounted the FM antenna. The USB converter is connected to the Raspberry Pi 4 directly to a USB 3.0 port.

### Nice to have

- [ ] OpenStreetMap integration for navigation
- [ ] Tracks database for lap times and other track information

## Features

- Real-time telemetry display
- RaceBox data integration (speed)
- ESP32 data integration (RPM, boost pressure)
- Modern UI with femtovg rendering
- Cross-platform support

## Requirements

- Rust (latest stable)
- OpenGL 3.3+ compatible graphics card
- RaceBox device (optional)
- ESP32 device (optional)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Project Structure

- `src/`
  - `main.rs` - Application entry point
  - `telemetry/` - Telemetry data handling
  - `ui/` - User interface components
    - `render.rs` - UI rendering with femtovg
    - `window.rs` - Window management
  - `logging.rs` - Logging configuration

## UI Themes

UI themes are now loaded from YAML files in `assets/themes/`. The following files are required:

- `dark_road.yml`
- `dark_race.yml`
- `light_road.yml`
- `light_race.yml`

Each file defines the color and style settings for a specific drive mode and color scheme combination. The expected YAML format is documented at the top of `src/ui/theme.rs`.

If any of these files are missing, the app will panic at startup.

## License

## Live UI State Control via TCP Command Interface

The dashboard supports live UI state changes via a simple TCP command interface. This is useful for development, testing, and demo purposes.

### How it works
- The app listens on TCP port 7878 for commands.
- You can send commands from another terminal or script to change the drive mode or color scheme in real time.
- The UI will update and animate the transition accordingly.

### Example Usage

```
# Set drive mode to Track
 echo set_mode Track | nc 127.0.0.1 7878

# Set drive mode to Road
 echo set_mode Road | nc 127.0.0.1 7878

# Set color scheme to Dark
 echo set_scheme Dark | nc 127.0.0.1 7878

# Set color scheme to Light
 echo set_scheme Light | nc 127.0.0.1 7878
```

You can use `nc` (netcat), `telnet`, or any TCP client to send these commands while the app is running.

### Supported Commands
- `set_mode Road` — Switch to Road drive mode
- `set_mode Track` — Switch to Track drive mode
- `set_scheme Light` — Switch to Light color scheme
- `set_scheme Dark` — Switch to Dark color scheme

This system is extensible: you can add more commands for other mock/test features as needed by editing `src/main.rs`.

