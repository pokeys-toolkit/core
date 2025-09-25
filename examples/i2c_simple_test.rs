//! Simple I2C Test Example
//!
//! This is a basic I2C test that demonstrates:
//! - Direct network connection to device by serial number
//! - I2C initialization
//! - Basic read/write operations
//!
//! Usage: cargo run --example i2c_simple_test

#![allow(clippy::uninlined_format_args)]

use pokeys_lib::*;

fn main() -> Result<()> {
    println!("🔧 Simple I2C Test (Network Connection)");
    println!("=======================================");

    // Connect directly to device by serial number over network
    // Replace 32218 with your actual device serial number
    let device_serial = 32223;
    let timeout_ms = 3000;

    println!("🌐 Connecting to device {} over network...", device_serial);

    let mut device = match connect_to_device_with_serial(device_serial, true, timeout_ms) {
        Ok(device) => {
            println!("✅ Successfully connected to device {}", device_serial);
            device
        }
        Err(e) => {
            println!("❌ Failed to connect to device {}: {}", device_serial, e);
            println!("💡 Make sure:");
            println!(
                "   - Device {} is powered on and connected to network",
                device_serial
            );
            println!("   - Device is accessible from this computer");
            println!("   - Serial number {} is correct", device_serial);
            return Ok(());
        }
    };

    // Get device info
    device.get_device_data()?;
    let device_name = String::from_utf8_lossy(&device.device_data.device_name);
    println!(
        "🔗 Connected to: {} (Serial: {})",
        device_name.trim_end_matches('\0'),
        device.device_data.serial_number
    );

    // Initialize I2C
    println!("\n🚀 Initializing I2C...");
    match device.i2c_init() {
        Ok(()) => println!("✅ I2C initialized successfully"),
        Err(e) => {
            println!("❌ I2C initialization failed: {}", e);
            return Ok(());
        }
    }

    // Test basic operations with a common I2C address
    let test_address = 0x50; // Common EEPROM address
    println!("\n💡 Testing with EEPROM address 0x50");
    println!("   Connect an I2C EEPROM (like 24LC256) to see full functionality");

    test_basic_operations(&mut device, test_address)?;

    println!("\n🎉 Simple I2C Test Complete!");
    println!("💡 To test with different devices:");
    println!("   - 0x50-0x57: EEPROM devices");
    println!("   - 0x68: DS1307 Real-Time Clock");
    println!("   - 0x48-0x4F: Temperature sensors");
    Ok(())
}

fn test_basic_operations(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!(
        "\n🧪 Testing basic operations with device at 0x{:02X}",
        address
    );

    // Test write
    let test_data = vec![0x00, 0x01, 0x02];
    println!("📤 Writing test data: {:02X?}", test_data);

    match device.i2c_write(address, &test_data) {
        Ok(I2cStatus::Ok) => println!("✅ Write successful"),
        Ok(status) => println!("⚠️  Write status: {:?} (device may not be present)", status),
        Err(e) => println!("❌ Write failed: {} (device may not be present)", e),
    }

    // Small delay
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Test read
    println!("📥 Reading 3 bytes...");
    match device.i2c_read(address, 3) {
        Ok((I2cStatus::Ok, data)) => {
            println!("✅ Read successful: {:02X?}", data);
        }
        Ok((status, data)) => {
            println!(
                "⚠️  Read status: {:?}, data: {:02X?} (device may not be present)",
                status, data
            );
        }
        Err(e) => println!("❌ Read failed: {} (device may not be present)", e),
    }

    // Test register operations if this looks like an EEPROM
    if (0x50..=0x57).contains(&address) {
        println!("🔍 Testing EEPROM register operations...");
        test_eeprom_operations(device, address)?;
    }

    Ok(())
}

fn test_eeprom_operations(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    let register = 0x00;
    let test_data = b"Hi!";

    println!(
        "📝 Writing '{}' to EEPROM address 0x{:04X}",
        String::from_utf8_lossy(test_data),
        register
    );

    // For EEPROM, we need to write register address + data
    let mut write_data = Vec::new();
    write_data.push(register);
    write_data.extend_from_slice(test_data);

    match device.i2c_write(address, &write_data) {
        Ok(I2cStatus::Ok) => {
            println!("✅ EEPROM write successful");

            // Wait for EEPROM write cycle
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Set read address
            match device.i2c_write(address, &[register]) {
                Ok(I2cStatus::Ok) => {
                    // Read back the data
                    match device.i2c_read(address, test_data.len() as u8) {
                        Ok((I2cStatus::Ok, data)) => {
                            let read_str = String::from_utf8_lossy(&data);
                            println!("📖 EEPROM read: '{}'", read_str);

                            if data == test_data {
                                println!("✅ EEPROM test PASSED!");
                            } else {
                                println!(
                                    "⚠️  Data mismatch - wrote: {:02X?}, read: {:02X?}",
                                    test_data, data
                                );
                                println!(
                                    "   This could be normal if EEPROM is write-protected or different data was already stored"
                                );
                            }
                        }
                        Ok((status, data)) => {
                            println!("⚠️  EEPROM read status: {:?}, data: {:02X?}", status, data)
                        }
                        Err(e) => println!("❌ EEPROM read failed: {}", e),
                    }
                }
                Ok(status) => println!("⚠️  EEPROM address set status: {:?}", status),
                Err(e) => println!("❌ EEPROM address set failed: {}", e),
            }
        }
        Ok(status) => println!(
            "⚠️  EEPROM write status: {:?} (device may not be present)",
            status
        ),
        Err(e) => println!("❌ EEPROM write failed: {} (device may not be present)", e),
    }

    Ok(())
}
