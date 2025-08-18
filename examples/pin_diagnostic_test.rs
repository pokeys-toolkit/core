//! Pin Diagnostic Test
//!
//! This test helps identify which pins are available for digital output
//! on your specific PoKeys device and finds a suitable CS pin for MAX7219.

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

fn main() -> Result<()> {
    println!("Pin Diagnostic Test for PoKeys Device 32218");
    println!("===========================================");
    println!("This test will help identify available pins for MAX7219 CS");
    println!();

    // Connect to network device 32218
    println!("🔍 Connecting to network device 32218...");
    let network_devices = enumerate_network_devices(3000)?;

    let target_device = network_devices
        .iter()
        .find(|dev| dev.serial_number == 32218);
    if target_device.is_none() {
        println!("❌ Device 32218 not found!");
        println!("Available devices:");
        for dev in &network_devices {
            println!(
                "   Serial: {}, IP: {}",
                dev.serial_number,
                format_ip(dev.ip_address)
            );
        }
        return Ok(());
    }

    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    // Read device information
    device.read_device_data()?;
    println!("📋 Device Info:");
    println!("   Type ID: {}", device.device_data.device_type_id);
    println!(
        "   Firmware: {}.{}",
        device.device_data.firmware_version_major, device.device_data.firmware_version_minor
    );

    // Test common pins that are typically available for digital output
    let test_pins = [
        1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        27, 28, 29, 30,
    ];

    println!("\n🧪 Testing pins for digital output capability...");
    println!("Pin | Status | Test Result");
    println!("----|--------|------------");

    let mut available_pins = Vec::new();

    for pin in test_pins {
        print!("{pin:3} | ");

        // Try to configure pin as digital output
        match device.set_pin_function(pin, PinFunction::DigitalOutput) {
            Ok(_) => {
                print!("✅ OK   | ");

                // Try to set pin high
                match device.set_digital_output(pin, true) {
                    Ok(_) => {
                        println!("Can set HIGH");
                        available_pins.push(pin);
                    }
                    Err(e) => {
                        println!("Cannot set HIGH: {e}");
                    }
                }
            }
            Err(e) => {
                println!("❌ FAIL | {e}");
            }
        }
    }

    println!("\n📊 Summary:");
    if available_pins.is_empty() {
        println!("❌ No pins available for digital output!");
        println!("💡 This might indicate:");
        println!("   - Device configuration is locked");
        println!("   - All pins are reserved for other functions");
        println!("   - Device needs to be reset or reconfigured");
    } else {
        println!("✅ Available pins for digital output: {available_pins:?}");

        // Recommend best CS pin
        let recommended_cs = if available_pins.contains(&24) {
            24
        } else if available_pins.contains(&8) {
            8
        } else {
            available_pins[0]
        };

        println!("💡 Recommended CS pin for MAX7219: {recommended_cs}");

        // Test the recommended pin more thoroughly
        println!("\n🔬 Testing recommended CS pin {recommended_cs}...");

        // Configure as digital output
        device.set_pin_function(recommended_cs, PinFunction::DigitalOutput)?;
        println!("   ✅ Configured as digital output");

        // Test HIGH/LOW states
        device.set_digital_output(recommended_cs, true)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        device.read_device_data()?;
        let state_high = device.get_digital_input(recommended_cs)?;
        println!(
            "   📊 Set HIGH, read: {}",
            if state_high { "HIGH" } else { "LOW" }
        );

        device.set_digital_output(recommended_cs, false)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        device.read_device_data()?;
        let state_low = device.get_digital_input(recommended_cs)?;
        println!(
            "   📊 Set LOW, read: {}",
            if state_low { "HIGH" } else { "LOW" }
        );

        // Test SPI configuration with this pin
        println!("\n📡 Testing SPI with CS pin {recommended_cs}...");
        device.spi_configure(0x04, 0x00)?;
        println!("   ✅ SPI configured");

        // Test SPI write (no-op command)
        let noop_cmd = vec![0x00, 0x00];
        device.spi_write(&noop_cmd, recommended_cs as u8)?;
        println!("   ✅ SPI write successful");

        println!("\n🎉 Pin {recommended_cs} is ready for MAX7219 CS!");
        println!("💡 Update your MAX7219 tests to use CS pin: {recommended_cs}");

        // Generate updated command examples
        println!("\n📝 Updated CLI commands:");
        println!("   pokeys max7219 config --device 32218 --cs-pin {recommended_cs}");
        println!(
            "   pokeys max7219 display --device 32218 --number 12345678 --cs-pin {recommended_cs}"
        );
        println!("   pokeys max7219 test --device 32218 --cs-pin {recommended_cs}");
    }

    // Additional device-specific information
    println!("\n🔍 Device-Specific Information:");
    println!(
        "Device Type ID {} typically has these characteristics:",
        device.device_data.device_type_id
    );

    match device.device_data.device_type_id {
        10 => println!("   - PoKeys55 v3: 55 pins, USB/Network"),
        11 => println!("   - PoKeys56U: 56 pins, USB only"),
        12 => println!("   - PoKeys57U: 57 pins, USB, advanced features"),
        13 => println!("   - PoKeys58EU: 58 pins, Ethernet, industrial"),
        14 => println!("   - PoKeys57E: 57 pins, Ethernet"),
        _ => println!("   - Unknown device type, check manual"),
    }

    println!("\n💡 Troubleshooting Tips:");
    println!("   1. If no pins are available, try resetting device configuration");
    println!("   2. Check if device is in 'safe mode' or configuration locked");
    println!("   3. Some pins may be reserved for specific functions (analog, PWM, etc.)");
    println!("   4. Consult device manual for pin assignment restrictions");
    println!("   5. Try different pins if recommended pin doesn't work");

    Ok(())
}
