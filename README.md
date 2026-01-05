# Train Set Control

A Rust package for controlling a model train set, providing self-test functionality and hardware interface management. Designed to run exclusively on Raspberry Pi.

## Features

- **LED Indicators**: Control LED status indicators
- **Track Motive Power**: Manage track power delivery
- **Points Control**: Switch track points/switches
- **Sensor Monitoring**: Read and monitor track sensors

## Hardware

This package interfaces with the **Sequent Micro Systems** series of cards for hardware control via I2C, SPI, and GPIO interfaces on Raspberry Pi.

## Prerequisites

### On Development Machine (macOS/Linux)

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install ARM cross-compilation toolchain**:

   For macOS (using Homebrew):
   ```bash
   # For 32-bit Raspberry Pi (Pi 1, 2, 3, Zero)
   brew install arm-linux-gnueabihf-binutils
   
   # For 64-bit Raspberry Pi (Pi 4, 5)
   brew install aarch64-elf-gcc
   ```

   For Linux (Ubuntu/Debian):
   ```bash
   # For 32-bit Raspberry Pi
   sudo apt-get install gcc-arm-linux-gnueabihf
   
   # For 64-bit Raspberry Pi
   sudo apt-get install gcc-aarch64-linux-gnu
   ```

3. **Add Rust targets**:
   ```bash
   # For 32-bit Raspberry Pi (Pi 1, 2, 3, Zero) - Recommended for compatibility
   rustup target add arm-unknown-linux-gnueabihf
   
   # For 64-bit Raspberry Pi (Pi 4, 5)
   rustup target add aarch64-unknown-linux-gnu
   ```

   **Note**: If you're unsure which to use, install `arm-unknown-linux-gnueabihf` as it works on all Raspberry Pi models.

### On Raspberry Pi

1. **Enable I2C and SPI** (if not already enabled):
   ```bash
   sudo raspi-config
   # Navigate to: Interface Options -> I2C -> Enable
   # Navigate to: Interface Options -> SPI -> Enable
   ```

2. **Install required system packages** (if needed):
   ```bash
   sudo apt-get update
   sudo apt-get install libc6-dev
   ```

## Building

### Cross-Compilation (Recommended - from macOS/Linux)

Cross-compilation allows you to build on your development machine and deploy to the Raspberry Pi.

#### Quick Start:

1. **Set up cross-compilation** (one-time setup):
   ```bash
   # Install Rust target
   rustup target add arm-unknown-linux-gnueabihf
   
   # Install cross-compiler (macOS)
   brew install arm-linux-gnueabihf-binutils
   
   # Or on Linux
   sudo apt-get install gcc-arm-linux-gnueabihf
   ```

2. **Build using the script**:
   ```bash
   chmod +x build-pi.sh
   ./build-pi.sh
   ```

   Or manually:
   ```bash
   cargo build --target arm-unknown-linux-gnueabihf --release
   ```

3. **Deploy to Raspberry Pi**:
   ```bash
   ./deploy.sh
   ```

   Or manually:
   ```bash
   scp target/arm-unknown-linux-gnueabihf/release/train pi@raspberrypi.local:/home/pi/
   ```

#### Using the build script:

```bash
# For 32-bit Raspberry Pi (default - works on all models)
./build-pi.sh arm-unknown-linux-gnueabihf

# For 64-bit Raspberry Pi (Pi 4, 5 only)
./build-pi.sh aarch64-unknown-linux-gnu
```

#### Manual build:

```bash
# For 32-bit Raspberry Pi (recommended)
cargo build --target arm-unknown-linux-gnueabihf --release

# For 64-bit Raspberry Pi
cargo build --target aarch64-unknown-linux-gnu --release
```

The compiled binary will be at:
- `target/arm-unknown-linux-gnueabihf/release/train` (32-bit)
- `target/aarch64-unknown-linux-gnu/release/train` (64-bit)

### Local Development (on Raspberry Pi)

If you prefer to build directly on the Raspberry Pi:

```bash
cargo build --release
```

**Note**: Cross-compilation is faster and doesn't require the Raspberry Pi to be available during development.

## Deployment

### Using the deployment script:

```bash
chmod +x deploy.sh

# Deploy to Raspberry Pi (default: pi@raspberrypi.local)
./deploy.sh arm-unknown-linux-gnueabihf

# Or specify custom host and path
./deploy.sh arm-unknown-linux-gnueabihf pi@192.168.1.100 /home/pi/train-control
```

### Manual deployment:

1. **Build for Raspberry Pi** (see Building section above)

2. **Copy binary to Raspberry Pi**:
   ```bash
   scp target/arm-unknown-linux-gnueabihf/release/train pi@raspberrypi.local:/home/pi/
   ```

3. **SSH into Raspberry Pi and run**:
   ```bash
   ssh pi@raspberrypi.local
   chmod +x train
   ./train
   ```

### Running as a System Service

To run the application as a systemd service (starts automatically on boot):

1. **Copy the service file to Raspberry Pi**:
   ```bash
   scp train-control.service pi@raspberrypi.local:/tmp/
   ```

2. **SSH into Raspberry Pi and install the service**:
   ```bash
   ssh pi@raspberrypi.local
   sudo cp /tmp/train-control.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable train-control.service
   sudo systemctl start train-control.service
   ```

3. **Check service status**:
   ```bash
   sudo systemctl status train-control.service
   ```

4. **View logs**:
   ```bash
   journalctl -u train-control.service -f
   ```

## Running

### On Raspberry Pi:

The application runs in test mode with subcommands to test individual components:

```bash
# Test LED indicators (see LED subcommands below)
./train test led all

# Test points/switches
./train test points

# Test sensors
./train test sensors

# Test track power
./train test tracks
```

### Command-Line Usage

The application supports two modes:

#### Test Mode

```bash
train test <component>

Subcommands:
  led      Test LED indicators (24 LEDs on GPIO pins 4-27)
    Subcommands:
      all     Turn all LEDs on
      off     Turn all LEDs off
      seq     Sequential test (each LED on for 250ms)
      random  Random LED test (200 iterations)
  points   Test points/switches
  sensors  Test sensors
  tracks   Test track power
```

#### Server Mode

```bash
train server [OPTIONS]

Options:
  -p, --port <PORT>    Port to listen on (default: 8080)
  -H, --host <HOST>    Host to bind to (default: 0.0.0.0)
```

### Examples

#### Test Mode

```bash
# LED Tests (24 LEDs on GPIO pins 4-27)
./train test led all      # Turn all LEDs on
./train test led off      # Turn all LEDs off
./train test led seq      # Sequential test (each LED on for 250ms)
./train test led random   # Random LED test (200 iterations)

# Test points control
./train test points

# Monitor sensors (continuous reading)
./train test sensors

# Test track power control
./train test tracks
```

Note: 
- The sensors test runs continuously until interrupted (Ctrl+C)
- The LED "all" test waits for Enter before turning LEDs off
- All other tests run once and exit

#### Server Mode

```bash
# Start server on default port 8080
./train server

# Start server on custom port
./train server --port 3000

# Start server on specific host and port
./train server --host 0.0.0.0 --port 8080
```

## Web Server API

When running in server mode, the application provides a REST API for controlling the train set.

### Base URL

```
http://<raspberry-pi-ip>:8080
```

### API Endpoints

#### LEDs

- `GET /api/leds` - Get all LEDs
- `GET /api/leds/:index` - Get LED state
- `POST /api/leds/:index/on` - Turn LED on
- `POST /api/leds/:index/off` - Turn LED off
- `POST /api/leds/:index/toggle` - Toggle LED
- `POST /api/leds/all/on` - Turn all LEDs on
- `POST /api/leds/all/off` - Turn all LEDs off

#### Track Power

- `GET /api/power` - Get power state
- `POST /api/power/enable` - Enable track power
- `POST /api/power/disable` - Disable track power
- `POST /api/power/toggle` - Toggle track power

#### Points

- `GET /api/points` - Get all points
- `GET /api/points/:point_id` - Get point position
- `POST /api/points/:point_id/normal` - Set point to normal
- `POST /api/points/:point_id/reverse` - Set point to reverse
- `POST /api/points/:point_id/toggle` - Toggle point
- `POST /api/points/all/normal` - Set all points to normal

#### Sensors

- `GET /api/sensors` - Get all sensor states
- `GET /api/sensors/:sensor_id` - Get sensor state

### Example API Usage

```bash
# Enable track power
curl -X POST http://raspberrypi.local:8080/api/power/enable

# Get power state
curl http://raspberrypi.local:8080/api/power

# Turn on LED 0
curl -X POST http://raspberrypi.local:8080/api/leds/0/on

# Set point 1 to reverse
curl -X POST http://raspberrypi.local:8080/api/points/1/reverse

# Get all sensor states
curl http://raspberrypi.local:8080/api/sensors
```

### API Response Format

All endpoints return JSON. Success responses include a `status` field set to `"ok"` and a `message` field. Error responses use standard HTTP status codes.

Example response:
```json
{
  "status": "ok",
  "message": "LED 0 turned on"
}
```

### Configuration

The application uses GPIO pins for hardware control. You can modify the pin assignments in `src/main.rs`:

- **LEDs**: GPIO pins 4-27 (24 LEDs total)
- **Power Control**: Default pin 5
- **Points**: Default pins 6, 7
- **Sensors**: Default pins 8, 9

Adjust these pin numbers based on your hardware setup and Sequent Micro Systems card configuration.

## API Usage

```rust
use train::{HardwareInterface, LedController, PowerController, PointsController, SensorController, PointsPosition};
use std::sync::Arc;
use std::collections::HashMap;

// Initialize hardware
let hardware = Arc::new(HardwareInterface::new()?);

// Control LEDs
let leds = LedController::new(Arc::clone(&hardware), vec![2, 3, 4])?;
leds.led_on(0)?;

// Control track power
let power = PowerController::new(Arc::clone(&hardware), 5)?;
power.enable()?;

// Control points
let mut points_pins = HashMap::new();
points_pins.insert(1, 6);
let points = PointsController::new(Arc::clone(&hardware), points_pins)?;
points.set_position(1, PointsPosition::Reverse)?;

// Monitor sensors
let mut sensor_pins = HashMap::new();
sensor_pins.insert(1, 8);
let sensors = SensorController::new(Arc::clone(&hardware), sensor_pins)?;
let state = sensors.read_sensor(1)?;
```

## Testing

```bash
cargo test
```

Note: Hardware-dependent tests may require a Raspberry Pi with connected hardware.

## Project Structure

```
src/
├── main.rs          # Application entry point with self-test
├── lib.rs           # Library root
├── error.rs         # Error types and handling
├── hardware.rs      # Hardware interface abstraction
├── leds.rs          # LED controller
├── power.rs         # Track power controller
├── points.rs        # Points/switches controller
└── sensors.rs       # Sensor monitoring
```

## Troubleshooting

### Cross-compilation issues:

- **Linker errors**: Ensure the cross-compilation toolchain is properly installed
- **Missing libraries**: Some dependencies may require additional system libraries on the target

### Runtime issues on Raspberry Pi:

- **Permission denied**: Ensure the user has access to GPIO/I2C/SPI devices:
  ```bash
  sudo usermod -a -G gpio,i2c,spi $USER
  # Log out and back in for changes to take effect
  ```

- **I2C/SPI not found**: Ensure I2C and SPI are enabled in `raspi-config`

- **GPIO access errors**: The application may need to run with appropriate permissions or be added to the gpio group

## License

MIT License

Copyright (c) 2026 AdrianChallinorOsiris

## Author

Adrian Challinor <adrian.challinor@osiris.co.uk>
