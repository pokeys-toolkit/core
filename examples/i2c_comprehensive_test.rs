//! Comprehensive I2C Test Example
//!
//! This example demonstrates various I2C operations including:
//! - I2C bus initialization and configuration
//! - Device scanning
//! - Basic read/write operations
//! - Register-based operations
//! - Error handling
//!
//! Usage: cargo run --example i2c_comprehensive_test

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::useless_vec)]

use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("🔧 PoKeys I2C Comprehensive Test");
    println!("================================");

    // Connect to the first available device
    let device_count = enumerate_usb_devices()?;
    if device_count == 0 {
        println!("❌ No PoKeys devices found!");
        return Ok(());
    }

    println!("📱 Found {} PoKeys device(s)", device_count);
    let mut device = connect_to_device(0)?;

    // Get device info
    device.get_device_data()?;
    let device_name = String::from_utf8_lossy(&device.device_data.device_name);
    println!(
        "🔗 Connected to: {} (Serial: {})",
        device_name.trim_end_matches('\0'),
        device.device_data.serial_number
    );

    // Test 1: Initialize I2C bus
    println!("\n🚀 Test 1: I2C Initialization");
    println!("-----------------------------");

    match device.i2c_init() {
        Ok(()) => println!("✅ I2C bus initialized successfully"),
        Err(e) => {
            println!("❌ I2C initialization failed: {}", e);
            return Ok(());
        }
    }

    // Test 2: Configure I2C with different speeds
    println!("\n⚙️  Test 2: I2C Configuration");
    println!("-----------------------------");

    // Test standard speed (100kHz)
    match device.i2c_configure(100, 0) {
        Ok(()) => println!("✅ I2C configured for 100kHz (standard speed)"),
        Err(e) => println!("❌ I2C 100kHz configuration failed: {}", e),
    }

    // Test fast speed (400kHz)
    match device.i2c_configure(400, 0) {
        Ok(()) => println!("✅ I2C configured for 400kHz (fast speed)"),
        Err(e) => println!("❌ I2C 400kHz configuration failed: {}", e),
    }

    // Test 3: I2C Bus Scan
    println!("\n🔍 Test 3: I2C Bus Scan");
    println!("-----------------------");

    match device.i2c_scan() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("📭 No I2C devices found on the bus");
            } else {
                println!("📡 Found {} I2C device(s):", devices.len());
                for addr in &devices {
                    println!("   - Device at address 0x{:02X}", addr);
                }
            }
        }
        Err(e) => println!("❌ I2C bus scan failed: {}", e),
    }

    // Test 4: Basic Write Operations
    println!("\n✍️  Test 4: Basic Write Operations");
    println!("----------------------------------");

    // Test writing to a common I2C address (0x50 - typical EEPROM address)
    let test_address = 0x50;
    let test_data = vec![0x00, 0x01, 0x02, 0x03]; // Test data

    match device.i2c_write(test_address, &test_data) {
        Ok(status) => {
            println!("📤 Write to 0x{:02X}: {:?}", test_address, status);
            match status {
                I2cStatus::Ok => println!("✅ Write completed successfully"),
                I2cStatus::Complete => println!("✅ Write operation complete"),
                I2cStatus::InProgress => println!("⏳ Write operation in progress"),
                I2cStatus::Error => println!("❌ Write operation failed"),
                I2cStatus::Timeout => println!("⏰ Write operation timed out"),
                I2cStatus::ChecksumError => println!("🔍 Write checksum error"),
                I2cStatus::DeviceNotFound => println!("📭 Device not found for write"),
                I2cStatus::PacketTooLarge => println!("📦 Write packet too large"),
            }
        }
        Err(e) => println!("❌ Write operation error: {}", e),
    }

    // Test 5: Basic Read Operations
    println!("\n📖 Test 5: Basic Read Operations");
    println!("---------------------------------");

    match device.i2c_read(test_address, 4) {
        Ok((status, data)) => {
            println!("📥 Read from 0x{:02X}: {:?}", test_address, status);
            match status {
                I2cStatus::Ok => {
                    println!("✅ Read completed successfully");
                    println!("📊 Data received: {:02X?}", data);
                }
                I2cStatus::Complete => {
                    println!("✅ Read operation complete");
                    println!("📊 Data received: {:02X?}", data);
                }
                I2cStatus::InProgress => println!("⏳ Read operation in progress"),
                I2cStatus::Error => println!("❌ Read operation failed"),
                I2cStatus::Timeout => println!("⏰ Read operation timed out"),
                I2cStatus::ChecksumError => println!("🔍 Read checksum error"),
                I2cStatus::DeviceNotFound => println!("📭 Device not found for read"),
                I2cStatus::PacketTooLarge => println!("📦 Read packet too large"),
            }
        }
        Err(e) => println!("❌ Read operation error: {}", e),
    }

    // Test 6: Register-based Operations
    println!("\n📋 Test 6: Register-based Operations");
    println!("------------------------------------");

    let register_addr = 0x00;
    let register_data = vec![0xAA, 0xBB];

    // Write to register
    match device.i2c_write_register(test_address, register_addr, &register_data) {
        Ok(status) => {
            println!(
                "📝 Register write to 0x{:02X}[0x{:02X}]: {:?}",
                test_address, register_addr, status
            );
        }
        Err(e) => println!("❌ Register write error: {}", e),
    }

    // Small delay for device processing
    thread::sleep(Duration::from_millis(10));

    // Read from register
    match device.i2c_read_register(test_address, register_addr, 2) {
        Ok((status, data)) => {
            println!(
                "📖 Register read from 0x{:02X}[0x{:02X}]: {:?}",
                test_address, register_addr, status
            );
            if status == I2cStatus::Ok {
                println!("📊 Register data: {:02X?}", data);
            }
        }
        Err(e) => println!("❌ Register read error: {}", e),
    }

    // Test 7: Error Handling
    println!("\n🚨 Test 7: Error Handling");
    println!("-------------------------");

    // Test with invalid parameters
    match device.i2c_write(0x50, &vec![0; 100]) {
        Ok(_) => println!("❌ Should have failed with data too long"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }

    match device.i2c_read(0x50, 0) {
        Ok(_) => println!("❌ Should have failed with zero length"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }

    // Test 8: Status Check
    println!("\n📊 Test 8: I2C Status Check");
    println!("---------------------------");

    match device.i2c_get_status() {
        Ok(status) => println!("📈 Current I2C status: {:?}", status),
        Err(e) => println!("❌ Status check failed: {}", e),
    }

    // Test 9: Performance Test
    println!("\n⚡ Test 9: Performance Test");
    println!("---------------------------");

    let start_time = std::time::Instant::now();
    let mut successful_operations = 0;
    let total_operations = 10;

    for i in 0..total_operations {
        let test_byte = vec![i as u8];
        if device.i2c_write(test_address, &test_byte).is_ok() {
            successful_operations += 1;
        }
        thread::sleep(Duration::from_millis(1)); // Small delay between operations
    }

    let elapsed = start_time.elapsed();
    println!(
        "⏱️  Completed {} operations in {:?}",
        total_operations, elapsed
    );
    println!(
        "✅ Successful operations: {}/{}",
        successful_operations, total_operations
    );
    println!(
        "📊 Average time per operation: {:?}",
        elapsed / total_operations
    );

    println!("\n🎉 I2C Comprehensive Test Complete!");
    println!("===================================");

    Ok(())
}
