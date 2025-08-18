//! Test Device 32218 Connection
//!
//! Simple test to verify we can connect to device 32218 using the same
//! logic as the CLI commands.

use pokeys_lib::*;

fn connect_to_device_by_id(device_id: &str) -> Result<PoKeysDevice> {
    // Try to parse as serial number first (more common for network devices)
    if let Ok(serial) = device_id.parse::<u32>() {
        // For serial numbers, try network connection first
        return connect_to_device_with_serial(serial, true, 3000);
    }

    // Try to parse as index (for USB devices)
    if let Ok(index) = device_id.parse::<usize>() {
        return connect_to_device(index.try_into().unwrap());
    }

    Err(PoKeysError::Parameter(format!(
        "Invalid device identifier: {device_id}"
    )))
}

fn main() -> Result<()> {
    println!("Testing Device 32218 Connection");
    println!("===============================");
    println!("This test uses the same connection logic as the CLI commands");
    println!();

    // Test connection using device ID "32218" (as string, like CLI)
    println!("🔗 Connecting to device '32218'...");

    match connect_to_device_by_id("32218") {
        Ok(mut device) => {
            println!("✅ Connection successful!");

            // Read device data to verify connection
            device.read_device_data()?;
            println!("📋 Device Info:");
            println!("   Serial: {}", device.device_data.serial_number);
            println!("   Type: {}", device.device_data.device_type_id);
            println!(
                "   Firmware: {}.{}",
                device.device_data.firmware_version_major,
                device.device_data.firmware_version_minor
            );

            // Test basic pin configuration (like MAX7219 CLI does)
            println!("\n🧪 Testing pin configuration...");

            // Try pin 8 first (common alternative to pin 24)
            match device.set_pin_function(8, PinFunction::DigitalOutput) {
                Ok(_) => {
                    println!("✅ Pin 8 configured as digital output");

                    match device.set_digital_output(8, true) {
                        Ok(_) => println!("✅ Pin 8 set HIGH"),
                        Err(e) => println!("❌ Failed to set pin 8 HIGH: {e}"),
                    }
                }
                Err(e) => println!("❌ Failed to configure pin 8: {e}"),
            }

            // Test SPI configuration
            println!("\n📡 Testing SPI configuration...");
            match device.spi_configure(0x04, 0x00) {
                Ok(_) => println!("✅ SPI configured successfully"),
                Err(e) => println!("❌ SPI configuration failed: {e}"),
            }

            println!("\n🎉 Device 32218 is ready for CLI commands!");
            println!("💡 You can now run:");
            println!("   ./target/debug/pokeys max7219 config --device 32218 --cs-pin 8");
        }
        Err(e) => {
            println!("❌ Connection failed: {e}");
            println!("\n🔍 Troubleshooting:");
            println!("   1. Check device 32218 is powered on");
            println!("   2. Verify network connectivity");
            println!("   3. Ensure device is on same network");
            println!("   4. Try running: cargo run --example network_device_test");
        }
    }

    Ok(())
}
