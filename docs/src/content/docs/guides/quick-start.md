---
title: Quick Start
description: Get started with the PoKeys Core Library
---

# Quick Start

## Basic Device Control

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

## Encoder Monitoring

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

## Basic Communication

```rust
use pokeys_lib::*;

fn main() -> Result<()> {
    let mut device = connect_to_device(0)?;

    // Configure I2C for sensor communication
    device.configure_i2c(100000)?; // 100kHz I2C

    // Scan for I2C devices
    let devices = device.scan_i2c_devices()?;
    println!("Found I2C devices: {:?}", devices);

    Ok(())
}
```

## Next Steps

- Explore the [API Reference](/reference/) for detailed documentation
- Check out more [Examples](/examples/) for advanced usage
- Learn about [device models and capabilities](/guides/device-models/)
