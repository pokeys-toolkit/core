//! I2C Common Devices Example
//!
//! This example demonstrates communication with common I2C devices:
//! - 24LC256 EEPROM (or similar)
//! - DS1307 Real-Time Clock
//! - Generic temperature sensors
//!
//! Usage: cargo run --example i2c_common_devices

#![allow(clippy::uninlined_format_args)]

use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("🔧 PoKeys I2C Common Devices Example");
    println!("====================================");

    // Connect to device
    let device_count = enumerate_usb_devices()?;
    if device_count == 0 {
        println!("❌ No PoKeys devices found!");
        return Ok(());
    }

    let mut device = connect_to_device(0)?;
    println!("🔗 Connected to PoKeys device");

    // Initialize I2C
    device.i2c_init()?;
    println!("✅ I2C initialized");

    // Scan for devices
    println!("\n🔍 Scanning I2C bus...");
    let found_devices = device.i2c_scan()?;

    if found_devices.is_empty() {
        println!("📭 No I2C devices found. Connect some I2C devices and try again.");
        return Ok(());
    }

    println!("📡 Found devices at addresses: {:02X?}", found_devices);

    // Test each found device
    for &addr in &found_devices {
        println!("\n🔍 Testing device at address 0x{:02X}", addr);
        test_device_at_address(&mut device, addr)?;
    }

    Ok(())
}

fn test_device_at_address(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    match address {
        0x50..=0x57 => test_eeprom(device, address),
        0x68 => test_ds1307_rtc(device, address),
        0x48..=0x4F => test_temperature_sensor(device, address),
        _ => test_generic_device(device, address),
    }
}

/// Test EEPROM devices (24LC series)
fn test_eeprom(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("💾 Testing EEPROM at 0x{:02X}", address);

    // EEPROM write: address (2 bytes) + data
    let memory_addr = 0x0000u16;
    let test_data = b"Hello I2C!";

    // Prepare write data: high byte, low byte, then data
    let mut write_data = Vec::new();
    write_data.push((memory_addr >> 8) as u8); // High byte
    write_data.push((memory_addr & 0xFF) as u8); // Low byte
    write_data.extend_from_slice(test_data);

    println!(
        "📝 Writing '{}' to EEPROM address 0x{:04X}",
        String::from_utf8_lossy(test_data),
        memory_addr
    );

    match device.i2c_write(address, &write_data) {
        Ok(I2cStatus::Ok) => println!("✅ EEPROM write successful"),
        Ok(status) => println!("⚠️  EEPROM write status: {:?}", status),
        Err(e) => println!("❌ EEPROM write failed: {}", e),
    }

    // Wait for write cycle to complete (EEPROMs need time)
    thread::sleep(Duration::from_millis(10));

    // Read back the data
    // First, set the read address
    let addr_data = vec![(memory_addr >> 8) as u8, (memory_addr & 0xFF) as u8];
    match device.i2c_write(address, &addr_data) {
        Ok(I2cStatus::Ok) => {
            // Now read the data
            match device.i2c_read(address, test_data.len() as u8) {
                Ok((I2cStatus::Ok, data)) => {
                    println!("📖 Read from EEPROM: '{}'", String::from_utf8_lossy(&data));
                    if data == test_data {
                        println!("✅ EEPROM read/write test PASSED");
                    } else {
                        println!("❌ EEPROM data mismatch");
                    }
                }
                Ok((status, _)) => println!("⚠️  EEPROM read status: {:?}", status),
                Err(e) => println!("❌ EEPROM read failed: {}", e),
            }
        }
        Ok(status) => println!("⚠️  EEPROM address set status: {:?}", status),
        Err(e) => println!("❌ EEPROM address set failed: {}", e),
    }

    Ok(())
}

/// Test DS1307 Real-Time Clock
fn test_ds1307_rtc(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("🕐 Testing DS1307 RTC at 0x{:02X}", address);

    // Read the current time (registers 0x00-0x06)
    match device.i2c_read_register(address, 0x00, 7) {
        Ok((I2cStatus::Ok, data)) => {
            if data.len() >= 7 {
                // Decode BCD time format
                let seconds = bcd_to_decimal(data[0] & 0x7F);
                let minutes = bcd_to_decimal(data[1]);
                let hours = bcd_to_decimal(data[2] & 0x3F);
                let day = bcd_to_decimal(data[3]);
                let date = bcd_to_decimal(data[4]);
                let month = bcd_to_decimal(data[5]);
                let year = bcd_to_decimal(data[6]);

                println!(
                    "📅 RTC Time: 20{:02}:{:02}:{:02} {:02}:{:02}:{:02} (Day {})",
                    year, month, date, hours, minutes, seconds, day
                );

                // Check if clock is running (bit 7 of seconds register)
                if data[0] & 0x80 != 0 {
                    println!("⚠️  RTC clock is stopped!");
                } else {
                    println!("✅ RTC clock is running");
                }
            }
        }
        Ok((status, _)) => println!("⚠️  RTC read status: {:?}", status),
        Err(e) => println!("❌ RTC read failed: {}", e),
    }

    // Test writing to RTC (set seconds to 0)
    let seconds_bcd = decimal_to_bcd(0);
    match device.i2c_write_register(address, 0x00, &[seconds_bcd]) {
        Ok(I2cStatus::Ok) => println!("✅ RTC seconds reset to 0"),
        Ok(status) => println!("⚠️  RTC write status: {:?}", status),
        Err(e) => println!("❌ RTC write failed: {}", e),
    }

    Ok(())
}

/// Test temperature sensors (like LM75, DS18B20 with I2C interface)
fn test_temperature_sensor(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("🌡️  Testing temperature sensor at 0x{:02X}", address);

    // Try to read temperature (typically register 0x00)
    match device.i2c_read_register(address, 0x00, 2) {
        Ok((I2cStatus::Ok, data)) => {
            if data.len() >= 2 {
                // Assume LM75-style 16-bit temperature
                let temp_raw = ((data[0] as u16) << 8) | (data[1] as u16);
                let temperature = (temp_raw as i16) as f32 / 256.0;
                println!("🌡️  Temperature: {:.2}°C", temperature);
                println!("✅ Temperature sensor read successful");
            }
        }
        Ok((status, _)) => println!("⚠️  Temperature sensor status: {:?}", status),
        Err(e) => println!("❌ Temperature sensor read failed: {}", e),
    }

    Ok(())
}

/// Test generic I2C device
fn test_generic_device(device: &mut PoKeysDevice, address: u8) -> Result<()> {
    println!("🔧 Testing generic device at 0x{:02X}", address);

    // Try to read a few bytes from register 0x00
    match device.i2c_read_register(address, 0x00, 4) {
        Ok((I2cStatus::Ok, data)) => {
            println!("📊 Device data: {:02X?}", data);
            println!("✅ Generic device read successful");
        }
        Ok((status, _)) => println!("⚠️  Generic device status: {:?}", status),
        Err(e) => println!("❌ Generic device read failed: {}", e),
    }

    // Try a simple write test
    let test_data = vec![0x00, 0x01];
    match device.i2c_write(address, &test_data) {
        Ok(I2cStatus::Ok) => println!("✅ Generic device write successful"),
        Ok(status) => println!("⚠️  Generic device write status: {:?}", status),
        Err(e) => println!("❌ Generic device write failed: {}", e),
    }

    Ok(())
}

/// Convert BCD to decimal
fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}

/// Convert decimal to BCD
fn decimal_to_bcd(decimal: u8) -> u8 {
    ((decimal / 10) << 4) | (decimal % 10)
}
