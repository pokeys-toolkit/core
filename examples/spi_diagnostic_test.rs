//! SPI Diagnostic Test
//!
//! This test isolates the SPI communication issue by testing each step individually.

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

fn main() -> Result<()> {
    println!("SPI Diagnostic Test for Device 32218");
    println!("====================================");
    println!("This test isolates the SPI communication issue step by step");
    println!();

    // Step 1: Connect to device
    println!("🔍 Step 1: Connecting to network device 32218...");
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
    println!("✅ Step 1: Connected to device 32218");

    // Step 2: Read device data
    println!("\n🔍 Step 2: Reading device data...");
    match device.read_device_data() {
        Ok(_) => {
            println!("✅ Step 2: Device data read successfully");
            println!("   Device Type: {}", device.device_data.device_type_id);
            println!(
                "   Firmware: {}.{}",
                device.device_data.firmware_version_major,
                device.device_data.firmware_version_minor
            );
        }
        Err(e) => {
            println!("❌ Step 2: Failed to read device data: {e}");
            return Err(e);
        }
    }

    // Step 3: Test pin configuration
    println!("\n🔍 Step 3: Testing pin configuration (pin 24)...");
    match device.set_pin_function(24, PinFunction::DigitalOutput) {
        Ok(_) => {
            println!("✅ Step 3a: Pin 24 configured as digital output");

            match device.set_digital_output(24, true) {
                Ok(_) => {
                    println!("✅ Step 3b: Pin 24 set HIGH");
                }
                Err(e) => {
                    println!("❌ Step 3b: Failed to set pin 24 HIGH: {e}");
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!("❌ Step 3a: Failed to configure pin 24: {e}");
            return Err(e);
        }
    }

    // Step 4: Test SPI configuration (this is likely where it fails)
    println!("\n🔍 Step 4: Testing SPI configuration...");
    println!("   Attempting: device.spi_configure(0x04, 0x00)");
    match device.spi_configure(0x04, 0x00) {
        Ok(_) => {
            println!("✅ Step 4: SPI configured successfully");
        }
        Err(e) => {
            println!("❌ Step 4: SPI configuration failed: {e}");
            println!("   This is likely the source of the 'Failed to send network request' error");

            // Try to get more details about the error
            println!("\n🔍 Error Analysis:");
            println!("   Error type: {e:?}");

            // Try a simpler SPI configuration
            println!("\n🔍 Step 4b: Trying alternative SPI configuration...");
            match device.spi_configure(0x01, 0x00) {
                Ok(_) => {
                    println!("✅ Step 4b: Alternative SPI config worked");
                }
                Err(e2) => {
                    println!("❌ Step 4b: Alternative SPI config also failed: {e2}");
                }
            }

            return Err(e);
        }
    }

    // Step 5: Test simple SPI write
    println!("\n🔍 Step 5: Testing simple SPI write...");
    let test_data = vec![0x00, 0x00]; // No-op command
    match device.spi_write(&test_data, 24) {
        Ok(_) => {
            println!("✅ Step 5: SPI write successful");
        }
        Err(e) => {
            println!("❌ Step 5: SPI write failed: {e}");
            return Err(e);
        }
    }

    // Step 6: Test MAX7219 initialization commands
    println!("\n🔍 Step 6: Testing MAX7219 initialization commands...");
    let init_commands = [
        (0x0C, 0x01, "Exit shutdown mode"),
        (0x0F, 0x00, "Disable display test"),
        (0x09, 0x00, "Set decode mode (raw segments)"),
        (0x0B, 0x07, "Set scan limit (8 digits)"),
        (0x0A, 0x08, "Set intensity (medium)"),
    ];

    for (i, (register, value, description)) in init_commands.iter().enumerate() {
        println!("   Step 6.{}: {}", i + 1, description);
        let cmd = vec![*register, *value];
        match device.spi_write(&cmd, 24) {
            Ok(_) => {
                println!("   ✅ Command successful");
            }
            Err(e) => {
                println!("   ❌ Command failed: {e}");
                return Err(e);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    println!("\n🎉 All SPI diagnostic tests passed!");
    println!("📋 Summary:");
    println!("   ✅ Device connection: OK");
    println!("   ✅ Device data read: OK");
    println!("   ✅ Pin configuration: OK");
    println!("   ✅ SPI configuration: OK");
    println!("   ✅ SPI write operations: OK");
    println!("   ✅ MAX7219 initialization: OK");

    println!("\n💡 If this test passes but the raw segments example fails,");
    println!("   the issue might be elsewhere in the example code.");

    Ok(())
}
