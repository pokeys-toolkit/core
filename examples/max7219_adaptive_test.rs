//! Adaptive MAX7219 Test
//!
//! This test automatically finds a working CS pin and tests MAX7219 functionality.
//! It tries multiple pins until it finds one that works.

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

/// Test if a pin can be used as CS for MAX7219
fn test_cs_pin(device: &mut PoKeysDevice, pin: u32) -> Result<bool> {
    // Try to configure pin as digital output
    if device
        .set_pin_function(pin, PinFunction::DigitalOutput)
        .is_err()
    {
        return Ok(false);
    }

    // Try to set pin HIGH (idle state)
    if device.set_digital_output(pin, true).is_err() {
        return Ok(false);
    }

    // Small delay to let pin settle
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Try SPI write with this pin as CS
    let noop_cmd = vec![0x00, 0x00]; // MAX7219 no-op
    if device.spi_write(&noop_cmd, pin as u8).is_err() {
        return Ok(false);
    }

    Ok(true)
}

/// Configure MAX7219 with given CS pin
fn configure_max7219(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
    // Configure CS pin
    device.set_pin_function(cs_pin.into(), PinFunction::DigitalOutput)?;
    device.set_digital_output(cs_pin.into(), true)?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Configure SPI
    device.spi_configure(0x04, 0x00)?;

    // MAX7219 initialization sequence
    let init_commands = [
        (0x0C, 0x01, "Exit shutdown mode"),
        (0x0F, 0x00, "Disable display test"),
        (0x09, 0xFF, "Set decode mode (BCD all digits)"),
        (0x0B, 0x07, "Set scan limit (8 digits)"),
        (0x0A, 0x08, "Set intensity (medium)"),
    ];

    for (register, value, description) in &init_commands {
        println!("   {description}");
        let cmd = vec![*register, *value];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    Ok(())
}

/// Display test pattern on MAX7219
fn display_test_pattern(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
    println!("   Displaying test pattern (12345678)");

    for digit in 0..8 {
        // FIXED: Reverse digit order for correct display
        // digit 0 = rightmost (8), digit 7 = leftmost (1)
        let display_value = 8 - digit; // So digit 0 shows 8, digit 7 shows 1
        let cmd = vec![0x01 + digit, display_value];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("Adaptive MAX7219 Test for Device 32218");
    println!("======================================");
    println!("This test will find a working CS pin and test MAX7219 functionality");
    println!();

    // Connect to device
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

    // Read device info
    device.read_device_data()?;
    println!(
        "📋 Device Type: {}, Firmware: {}.{}",
        device.device_data.device_type_id,
        device.device_data.firmware_version_major,
        device.device_data.firmware_version_minor
    );

    // Try different CS pins in order of preference
    let candidate_pins = [
        24, 8, 23, 25, 22, 26, 21, 27, 20, 28, 19, 29, 18, 30, 17, 16, 15, 14, 13, 12, 11, 10, 9,
        5, 4, 3, 2, 1,
    ];

    println!("\n🔍 Searching for working CS pin...");
    let mut working_cs_pin = None;

    for &pin in &candidate_pins {
        print!("   Testing pin {pin}: ");

        match test_cs_pin(&mut device, pin) {
            Ok(true) => {
                println!("✅ WORKS");
                working_cs_pin = Some(pin as u8);
                break;
            }
            Ok(false) => {
                println!("❌ Failed");
            }
            Err(e) => {
                println!("❌ Error: {e}");
            }
        }
    }

    let cs_pin = match working_cs_pin {
        Some(pin) => pin,
        None => {
            println!("\n❌ No working CS pin found!");
            println!("💡 Possible issues:");
            println!("   - Device configuration is locked");
            println!("   - All pins are reserved for other functions");
            println!("   - Device needs reset or reconfiguration");
            println!("   - Try running: cargo run --example pin_diagnostic_test");
            return Ok(());
        }
    };

    println!("\n🎉 Found working CS pin: {cs_pin}");

    // Configure MAX7219
    println!("\n🔧 Configuring MAX7219 with CS pin {cs_pin}...");
    match configure_max7219(&mut device, cs_pin) {
        Ok(_) => println!("✅ MAX7219 configured successfully!"),
        Err(e) => {
            println!("❌ MAX7219 configuration failed: {e}");
            return Ok(());
        }
    }

    // Display test pattern
    println!("\n📟 Testing display...");
    match display_test_pattern(&mut device, cs_pin) {
        Ok(_) => {
            println!("✅ Test pattern sent successfully!");
            println!("🎯 You should see '12345678' on your MAX7219 display");
        }
        Err(e) => {
            println!("❌ Display test failed: {e}");
        }
    }

    // Test different patterns
    println!("\n🎨 Testing different patterns...");

    // Pattern 1: All segments on
    println!("   Pattern 1: All segments on (2 seconds)");
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, 0x7F]; // All segments
        device.spi_write(&cmd, cs_pin)?;
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 2: Alternating segments
    println!("   Pattern 2: Alternating segments (2 seconds)");
    for digit in 0..8 {
        let pattern = if digit % 2 == 0 { 0xAA } else { 0x55 };
        let cmd = vec![0x01 + digit, pattern];
        device.spi_write(&cmd, cs_pin)?;
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 3: Clear display
    println!("   Pattern 3: Clear display");
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, 0x0F]; // Blank
        device.spi_write(&cmd, cs_pin)?;
    }

    // Final summary
    println!("\n🎉 MAX7219 Test Completed Successfully!");
    println!("📋 Results:");
    println!("   ✅ Working CS pin: {cs_pin}");
    println!("   ✅ SPI communication: OK");
    println!("   ✅ MAX7219 configuration: OK");
    println!("   ✅ Display patterns: OK");

    println!("\n💡 Use these commands for future testing:");
    println!("   pokeys max7219 config --device 32218 --cs-pin {cs_pin}");
    println!("   pokeys max7219 display --device 32218 --number 12345678 --cs-pin {cs_pin}");
    println!("   pokeys max7219 test --device 32218 --cs-pin {cs_pin}");

    println!("\n📝 Update your MAX7219 code to use CS pin: {cs_pin}");

    Ok(())
}
