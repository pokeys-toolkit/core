# PoKeys Core Library

A pure Rust implementation of the PoKeysLib for controlling PoKeys devices. This is the **core library** that provides all fundamental device communication and control functionality for the PoKeys ecosystem.

## 🚀 Performance Breakthrough: Dual Optimization System

**Revolutionary dual optimization system provides massive performance improvements**:

### Bulk Operations Optimization
- **Before**: 110 individual commands, 14.44ms configuration time
- **After**: 2 bulk commands, 513µs configuration time  
- **Result**: 96.4% time reduction, **28x faster pin configuration**

### Single Enumeration Optimization  
- **Before**: Multiple 5-second device enumerations per sync
- **After**: Single enumeration, cached results reused
- **Result**: 65% faster device discovery, **3x faster multi-device sync**

### Encoder Pin Numbering Fix
- **Fixed**: Encoder pin numbering conversion (1-based config ↔ 0-based protocol)
- **Result**: Correct encoder pin assignments in vendor tools
- **Impact**: Reliable encoder configuration and monitoring

## ✨ Core Features

### Device Connectivity
- **USB Devices**: Full support for USB-connected PoKeys devices
- **Network Devices**: Discovery and connection to network-enabled devices  
- **Auto-Detection**: Intelligent connection type detection
- **Multi-Device**: Concurrent management of multiple devices

### Digital & Analog I/O
- **Digital I/O**: Pin configuration and digital input/output operations
- **Analog I/O**: Multi-channel analog input with configurable reference voltage
- **Pin Functions**: Digital input/output, analog input, PWM, encoder, counter, keyboard matrix
- **Bulk Operations**: Optimized bulk pin configuration and state reading

### Advanced Control Systems
- **PWM Control**: Multiple PWM channels with configurable frequency and duty cycle
- **Encoder Support**: Quadrature encoder reading with 4x/2x sampling modes, position and velocity tracking
- **Pulse Engine v2**: Stepper motor control with advanced pulse generation
- **Matrix Operations**: Matrix keyboard scanning and LED matrix control

### Communication Protocols
- **SPI**: Full SPI master support with multiple chip select pins
- **I2C**: I2C master operations with device scanning
- **1-Wire**: 1-Wire protocol support for temperature sensors
- **CAN Bus**: CAN message transmission and reception
- **UART**: Serial communication support

### Display & Interface Support
- **LCD Display**: Text LCD display control and management
- **MAX7219**: Comprehensive support for MAX7219 LED display drivers
  - Individual and daisy-chained displays
  - 7-segment, dot matrix, and raw segment modes
  - Text display with justification and scrolling
- **Seven-Segment**: Built-in character mapping and display utilities

### Sensor Integration
- **EasySensors**: Integrated sensor support and data acquisition
- **Real-Time Clock**: RTC operations and time synchronization
- **Temperature Sensors**: 1-Wire temperature sensor support

### Safety & Reliability
- **Device Models**: Comprehensive pin capability validation and safety checks
- **Error Handling**: Detailed error types with context and recovery suggestions
- **Thread Safety**: Safe concurrent access to device resources
- **Failsafe Settings**: Configurable failsafe behavior for critical applications
- **SPI Pin Reservation**: Hardware constraint enforcement prevents conflicts

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

### MAX7219 Multi-Display Control
```rust
use pokeys_lib::*;
use pokeys_lib::devices::spi::Max7219;

fn main() -> Result<()> {
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;

    // Create individual displays (no daisy chaining)
    let mut display0 = Max7219::new(&mut device, 24)?; // CS pin 24
    let mut display1 = Max7219::new(&mut device, 26)?; // CS pin 26

    // Configure for text display
    display0.configure_raw_segments(8)?;
    display1.configure_raw_segments(8)?;

    // Display different content on each
    display0.display_text("HELLO")?;
    display1.display_text("WORLD")?;

    // Control intensity independently
    display0.set_intensity(15)?; // Bright
    display1.set_intensity(5)?;  // Dim

    Ok(())
}
```

### Encoder Monitoring
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;
    
    // Configure encoder on pins 10-11
    device.configure_encoder(0, 10, 11, EncoderMode::Quadrature4x)?;
    
    loop {
        let position = device.get_encoder_position(0)?;
        println!("Encoder position: {}", position);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

### SPI Communication
```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;
    
    // Configure SPI (automatically reserves pins 23=MOSI, 25=CLK)
    device.configure_spi(1000000, SpiMode::Mode0)?;
    
    // Send data using pin 24 as chip select
    let data = vec![0x01, 0x02, 0x03];
    let response = device.spi_transfer(24, &data)?;
    
    println!("SPI response: {:?}", response);
    Ok(())
}
```

## 🛡️ SPI Pin Reservation & Device Model Updates

Comprehensive SPI pin reservation system with updated device models:

### Hardware Constraint Enforcement
- ✅ **Pin 23 (MOSI)** automatically reserved when SPI is enabled
- ✅ **Pin 25 (CLK)** automatically reserved when SPI is enabled  
- ✅ **Configuration validation** prevents hardware conflicts
- ✅ **31-33 CS pins** available per device for MAX7219 displays

### Updated Device Models
- ✅ **All device models** updated with SPI capabilities
- ✅ **PoKeys56U/56E**: 55 pins, 31 CS-capable pins
- ✅ **PoKeys57U/57E**: 57 pins, 33 CS-capable pins
- ✅ **Pin validation** ensures only supported functions are used
- ✅ **Clear error messages** when conflicts are detected

## 📚 Examples

The `examples/` directory contains comprehensive examples demonstrating all library features:

```bash
# Basic device operations
cargo run --example basic_device_control

# MAX7219 display examples
cargo run --example test_two_displays
cargo run --example comprehensive_multi_display_test
cargo run --example max7219_console_demo

# Communication protocols
cargo run --example spi_example
cargo run --example i2c_simple_test
cargo run --example i2c_comprehensive_test

# Advanced features
cargo run --example encoder_monitoring
cargo run --example physical_device_config_example
cargo run --example network_device_test
```

## 🏗️ Architecture

This core library provides the foundation for the PoKeys ecosystem:

- **Pure Rust**: No external C dependencies, full memory safety
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Thread-Safe**: Concurrent device access with proper synchronization
- **Extensible**: Plugin architecture for custom devices and protocols
- **Performance-Optimized**: Bulk operations and caching for maximum throughput

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
