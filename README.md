
# PoKeys Core Library

[![CI](https://github.com/pokeys-toolkit/core/actions/workflows/ci.yml/badge.svg)](https://github.com/pokeys-toolkit/core/actions/workflows/ci.yml)
[![Release](https://github.com/pokeys-toolkit/core/actions/workflows/release.yml/badge.svg)](https://github.com/pokeys-toolkit/core/actions/workflows/release.yml)

A pure Rust implementation of the PoKeysLib for controlling PoKeys devices. This is the **core library** that provides all fundamental device communication and control functionality for the PoKeys ecosystem.

## ✨ Core Features

### Device Connectivity
- **USB Devices**: Full support for USB-connected PoKeys devices
- **Network Devices**: Discovery and connection to network-enabled devices
- **Auto-Detection**: Intelligent connection type detection
- **Multi-Device**: Concurrent management of multiple devices

### Digital & Analog I/O
- **Digital I/O**: Pin configuration and digital input/output operations
- **Analog I/O**: Multi-channel analog input with configurable reference voltage
- **Pin Functions**: Digital input/output, analog input, PWM (pins 17-22), encoder, counter, keyboard matrix
- **Bulk Operations**: Optimized bulk pin configuration and state reading

### Advanced Control Systems
- **PWM Control**: 6 hardware PWM channels (pins 17-22) with 25MHz clock precision for servo control
- **Encoder Support**: Quadrature encoder reading with 4x/2x sampling modes, position and velocity tracking
- **Pulse Engine v2**: Stepper motor control with advanced pulse generation
- **Matrix Operations**: Matrix keyboard scanning and LED matrix control
- **Matrix Keyboard**: 4x4 to 16x8 matrix keyboard support with real-time key detection

### Communication Protocols
- **SPI**: Full SPI master support with multiple chip select pins
- **I2C**: Enhanced I2C master operations with automatic packet fragmentation, intelligent retry mechanisms, and device scanning
- **1-Wire**: 1-Wire protocol support for temperature sensors
- **CAN Bus**: CAN message transmission and reception
- **UART**: Serial communication support

### Display & Interface Support
- **LCD Display**: Text LCD display control and management
- **Seven-Segment**: Built-in character mapping and display utilities

### Sensor Integration
- **EasySensors**: Integrated sensor support and data acquisition
- **Real-Time Clock**: RTC operations and time synchronization
- **Temperature Sensors**: 1-Wire temperature sensor support

### Safety & Reliability
- **Device Models**: Comprehensive pin capability validation and safety checks
- **Enhanced Error Handling**: Detailed error types with context, recovery suggestions, and intelligent retry mechanisms
- **Thread Safety**: Safe concurrent access to device resources
- **Failsafe Settings**: Configurable failsafe behavior for critical applications
- **SPI Pin Reservation**: Hardware constraint enforcement prevents conflicts
- **I2C Reliability**: Automatic packet fragmentation, exponential backoff retry, and configurable validation levels
- **Health Monitoring**: Real-time performance metrics and device health diagnostics

## 🔧 Supported Devices

- **PoKeys55 v3** - 55-pin development board
- **PoKeys56U** - USB-enabled 56-pin device (55 usable pins, 31 CS-capable)
- **PoKeys56E** - Ethernet-enabled 56-pin device (55 usable pins, 31 CS-capable)
- **PoKeys57U** - Advanced USB device (57 usable pins, 33 CS-capable)
- **PoKeys57E** - Ethernet-enabled industrial device (57 usable pins, 33 CS-capable)
- **PoKeys58EU** - Ethernet-enabled device with extended features
- **Custom Devices** - Extensible model system for custom hardware

## 🛠️ Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pokeys-lib = "0.3.0"
```

Or install from git:

```toml
[dependencies]
pokeys-lib = { git = "https://github.com/pokeys-toolkit/core" }
```

## 📖 Usage Examples

### Basic Device Control
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    // Enumerate and connect to first device
    let device_count = enumerate_usb_devices()?;
    if device_count > 0 {
        let mut device = connect_to_device(0)?;

        // Configure pin 1 as digital output
        device.set_pin_function(1, PinFunction::DigitalOutput)?;

        // Turn on pin 1
        device.set_digital_output(1, true)?;

        println!("Pin 1 is now HIGH");
    }
    Ok(())
}
```

### Encoder Monitoring
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;

    // Configure encoder on pins 10-11
    let options = EncoderOptions::with_4x_sampling();
    device.configure_encoder(0, 10, 11, options)?;

    loop {
        let position = device.get_encoder_value(0)?;
        println!("Encoder position: {}", position);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

### PWM Servo Control
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;

    // Configure PWM for servo control on pin 22 (PWM1)
    // PoKeys PWM operates at 25MHz clock frequency
    device.set_pwm_period(500000)?; // 20ms period (0.020 × 25,000,000)
    device.enable_pwm_for_pin(22, true)?;

    // Servo positions in clock cycles
    let positions = [
        (60000, "0°"),   // Custom calibrated 0° position
        (36000, "90°"),  // Custom calibrated 90° position  
        (12000, "180°"), // Custom calibrated 180° position
    ];

    for (duty_cycles, angle) in positions.iter() {
        println!("Moving to {}", angle);
        device.set_pwm_duty_cycle_for_pin(22, *duty_cycles)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Disable PWM
    device.enable_pwm_for_pin(22, false)?;
    Ok(())
}
```

### Advanced Servo Control
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;

    // 180-degree position servo
    let servo_180 = ServoConfig::one_eighty(22, 60000, 12000);
    device.configure_servo(servo_180.clone())?;
    device.set_servo_angle(&servo_180, 90.0)?; // Move to 90°

    // 360-degree speed servo (continuous rotation)
    // Datasheet: Counterclockwise 1-1.5ms, Stop 1.5ms, Clockwise 1.5-2ms
    let servo_speed = ServoConfig::three_sixty_speed(21, 37500, 50000, 25000);
    device.configure_servo(servo_speed.clone())?;
    device.set_servo_speed(&servo_speed, 50.0)?; // 50% clockwise
    device.stop_servo(&servo_speed)?; // Stop rotation

    Ok(())
}
```

### Matrix Keyboard
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;

    // Configure 4x4 matrix keyboard
    let column_pins = [5, 6, 7, 8];  // Column pins
    let row_pins = [1, 2, 3, 4];     // Row pins
    device.configure_matrix_keyboard(4, 4, &column_pins, &row_pins)?;

    // Monitor for key presses
    let mut previous_states = vec![vec![false; 4]; 4];
    
    loop {
        device.read_matrix_keyboard()?;
        
        for row in 0..4 {
            for col in 0..4 {
                let current_state = device.matrix_keyboard.get_key_state(row, col);
                if current_state != previous_states[row][col] {
                    if current_state {
                        println!("Key PRESSED at ({}, {})", row, col);
                    }
                    previous_states[row][col] = current_state;
                }
            }
        }
        
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
```

### Enhanced I2C Communication
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;

    // Configure I2C with enhanced features
    device.i2c_init()?;
    
    // Configure retry behavior
    let retry_config = RetryConfig {
        max_attempts: 3,
        base_delay_ms: 100,
        backoff_multiplier: 2.0,
        jitter: true,
        ..Default::default()
    };
    
    // Write large data with automatic fragmentation
    let large_data = vec![0x42; 100]; // 100 bytes
    device.i2c_write_fragmented(0x50, &large_data)?;
    
    // Write with intelligent retry on failure
    let data = vec![0x01, 0x02, 0x03];
    device.i2c_write_with_retry(0x50, &data, &retry_config)?;
    
    // Monitor device health
    let health = device.health_check();
    println!("I2C Success Rate: {:.1}%", health.performance.success_rate * 100.0);

    Ok(())
}
```

## 🛡️ Enhanced Reliability & Hardware Support

### SPI Pin Reservation & Device Model Updates
- ✅ **Pin 23 (MOSI)** automatically reserved when SPI is enabled
- ✅ **Pin 25 (CLK)** automatically reserved when SPI is enabled
- ✅ **Configuration validation** prevents hardware conflicts
- ✅ **31-33 CS pins** available per device for SPI peripherals
- ✅ **All device models** updated with SPI capabilities
- ✅ **PoKeys56U/56E**: 55 pins, 31 CS-capable pins
- ✅ **PoKeys57U/57E**: 57 pins, 33 CS-capable pins

### Enhanced I2C Reliability (NEW)
- ✅ **Automatic Packet Fragmentation**: Handle data larger than 32-byte I2C limit
- ✅ **Intelligent Retry Logic**: Exponential backoff with jitter for failed operations
- ✅ **Configurable Validation**: Optional strict protocol validation and error detection
- ✅ **Performance Monitoring**: Real-time metrics and health diagnostics
- ✅ **Error Recovery**: Smart classification of recoverable vs. permanent errors
- ✅ **Circuit Breaker Pattern**: Prevent cascading failures in unreliable conditions

## 📚 Examples

The `examples/` directory contains focused examples demonstrating core library features:

```bash
# Basic device operations and configuration
cargo run --example physical_device_config_example
cargo run --example config_loader_example
cargo run --example step_by_step_config

# Communication protocols
cargo run --example spi_example
cargo run --example i2c_simple_test
cargo run --example i2c_comprehensive_test
cargo run --example i2c_common_devices
cargo run --example i2c_enhanced_features  # NEW: Enhanced I2C features

# Matrix keyboard support
cargo run --example matrix_keyboard_simple      # NEW: Simple matrix keyboard
cargo run --example keyboard_matrix_example     # NEW: Full matrix keyboard

# Network device support
cargo run --example network_device_test
```

## 🏗️ Architecture

This core library provides the foundation for the PoKeys ecosystem:

- **Pure Rust**: No external C dependencies, full memory safety
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Thread-Safe**: Concurrent device access with proper synchronization
- **Extensible**: Plugin architecture for custom devices and protocols
- **Performance-Optimized**: Bulk operations, caching, and intelligent retry mechanisms for maximum throughput and reliability

## 🤝 Contributing

We welcome contributions! Please ensure all tests pass:

```bash
cargo test
cargo test --features hardware-tests  # Requires actual hardware
```

## 📄 License

This project is licensed under the LGPL-2.1 License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Projects

- [PoKeys CLI](https://github.com/pokeys-toolkit/cli) - Command-line interface
- [PoKeys Thread](https://github.com/pokeys-toolkit/thread) - Threading system
- [PoKeys Model Manager](https://github.com/pokeys-toolkit/model-manager) - Device model management

---

**Note**: This is a pure Rust implementation and does not require the original PoKeysLib C library. All functionality is implemented natively in Rust for better performance, safety, and cross-platform compatibility.
