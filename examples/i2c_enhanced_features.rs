//! Enhanced I2C Features Example
//!
//! This example demonstrates the new enhanced I2C features including:
//! - Automatic packet fragmentation for large data
//! - Intelligent retry mechanisms with exponential backoff
//! - Configurable validation levels
//! - Performance metrics and health monitoring
//!
//! Run with: cargo run --example i2c_enhanced_features

#![allow(clippy::uninlined_format_args)]

use pokeys_lib::*;
use std::collections::HashSet;

fn main() -> Result<()> {
    println!("🚀 PoKeys Enhanced I2C Features Demo");
    println!("=====================================");

    // Try to connect to a network device first, then USB
    let mut device = match connect_to_first_available_device() {
        Ok(device) => device,
        Err(e) => {
            println!("❌ No PoKeys device found: {}", e);
            println!("💡 Please connect a PoKeys device to run this example");
            return Ok(());
        }
    };

    println!(
        "✅ Connected to device (Serial: {})",
        device.device_data.serial_number
    );

    // Initialize I2C
    device.i2c_init()?;
    println!("✅ I2C initialized");

    // Demonstrate enhanced I2C features
    demonstrate_packet_fragmentation(&mut device)?;
    demonstrate_retry_mechanisms(&mut device)?;
    demonstrate_validation_levels(&mut device)?;
    demonstrate_health_monitoring(&mut device)?;

    println!("\n🎉 Enhanced I2C features demonstration completed!");
    Ok(())
}

fn connect_to_first_available_device() -> Result<PoKeysDevice> {
    // Try network devices first
    println!("🔍 Searching for network devices...");
    match enumerate_network_devices(2000) {
        Ok(network_devices) if !network_devices.is_empty() => {
            println!("📡 Found {} network device(s)", network_devices.len());
            return connect_to_network_device(&network_devices[0]);
        }
        _ => println!("📡 No network devices found"),
    }

    // Try USB devices
    println!("🔍 Searching for USB devices...");
    match enumerate_usb_devices() {
        Ok(count) if count > 0 => {
            println!("🔌 Found {} USB device(s)", count);
            connect_to_device(0)
        }
        _ => Err(PoKeysError::DeviceNotFound),
    }
}

fn demonstrate_packet_fragmentation(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n📦 Demonstrating Packet Fragmentation");
    println!("-------------------------------------");

    // Create a large data packet (larger than 32 bytes)
    let large_data = vec![0x42; 100]; // 100 bytes of data
    let test_address = 0x50; // Example I2C address

    println!("📏 Attempting to send {} bytes of data", large_data.len());

    // First, show what happens with regular i2c_write (should fail)
    match device.i2c_write(test_address, &large_data) {
        Err(PoKeysError::I2cPacketTooLarge {
            size,
            max_size,
            suggestion,
        }) => {
            println!("❌ Regular i2c_write failed as expected:");
            println!("   📊 Packet size: {} bytes", size);
            println!("   📏 Maximum size: {} bytes", max_size);
            println!("   💡 Suggestion: {}", suggestion);
        }
        _ => println!("⚠️  Unexpected result from regular i2c_write"),
    }

    // Now demonstrate automatic fragmentation
    println!("\n🔧 Using automatic packet fragmentation...");
    match device.i2c_write_fragmented(test_address, &large_data) {
        Ok(status) => {
            println!("✅ Fragmented write completed with status: {:?}", status);
            println!("   📦 {} bytes sent in fragments", large_data.len());
        }
        Err(e) => {
            println!("⚠️  Fragmented write failed: {}", e);
            println!(
                "   (This is expected if no device is connected at address 0x{:02X})",
                test_address
            );
        }
    }

    Ok(())
}

fn demonstrate_retry_mechanisms(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n🔄 Demonstrating Retry Mechanisms");
    println!("---------------------------------");

    // Configure retry settings
    let retry_config = RetryConfig {
        max_attempts: 3,
        base_delay_ms: 50,
        max_delay_ms: 500,
        backoff_multiplier: 2.0,
        jitter: true,
    };

    println!("⚙️  Retry configuration:");
    println!("   🔢 Max attempts: {}", retry_config.max_attempts);
    println!("   ⏱️  Base delay: {}ms", retry_config.base_delay_ms);
    println!("   ⏱️  Max delay: {}ms", retry_config.max_delay_ms);
    println!(
        "   📈 Backoff multiplier: {}",
        retry_config.backoff_multiplier
    );
    println!("   🎲 Jitter enabled: {}", retry_config.jitter);

    // Try to write to a non-existent device (will likely fail and retry)
    let test_data = vec![0x01, 0x02, 0x03];
    let non_existent_address = 0x77; // Address unlikely to have a device

    println!(
        "\n🎯 Attempting write with retry to address 0x{:02X}...",
        non_existent_address
    );

    let start_time = std::time::Instant::now();
    match device.i2c_write_with_retry(non_existent_address, &test_data, &retry_config) {
        Ok(status) => {
            println!("✅ Write with retry succeeded: {:?}", status);
        }
        Err(PoKeysError::MaxRetriesExceeded) => {
            let elapsed = start_time.elapsed();
            println!(
                "❌ Write failed after {} attempts",
                retry_config.max_attempts
            );
            println!("   ⏱️  Total time: {:?}", elapsed);
            println!("   (This is expected behavior when no device responds)");
        }
        Err(e) => {
            println!("❌ Write failed with error: {}", e);
        }
    }

    Ok(())
}

fn demonstrate_validation_levels(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n🔍 Demonstrating Validation Levels");
    println!("----------------------------------");

    // Show current validation level
    println!("📋 Current validation level: {:?}", device.validation_level);

    // Set strict validation
    device.set_validation_level(ValidationLevel::Strict);
    println!("🔒 Set validation level to Strict");

    // Set custom validation
    let mut valid_commands = HashSet::new();
    valid_commands.insert(0xDB); // I2C command
    valid_commands.insert(0x00); // Device info command

    let custom_config = ValidationConfig {
        validate_checksums: true,
        validate_command_ids: true,
        validate_device_ids: true,
        validate_packet_structure: true,
        max_device_id: 10,
        valid_commands,
    };

    device.set_validation_level(ValidationLevel::Custom(custom_config));
    println!("⚙️  Set validation level to Custom with specific rules");

    // Reset to none for normal operation
    device.set_validation_level(ValidationLevel::None);
    println!("🔓 Reset validation level to None for normal operation");

    Ok(())
}

fn demonstrate_health_monitoring(device: &mut PoKeysDevice) -> Result<()> {
    println!("\n🏥 Demonstrating Health Monitoring");
    println!("----------------------------------");

    // Get current I2C metrics
    let metrics = device.get_i2c_metrics();
    println!("📊 Current I2C Metrics:");
    println!("   📈 Total commands: {}", metrics.total_commands);
    println!("   ✅ Successful commands: {}", metrics.successful_commands);
    println!("   ❌ Failed commands: {}", metrics.failed_commands);
    println!(
        "   ⏱️  Average response time: {:?}",
        metrics.average_response_time
    );

    // Perform health check
    println!("\n🔍 Performing health check...");
    let health = device.health_check();

    println!("🏥 Health Status:");
    match health.connectivity {
        ConnectivityStatus::Healthy => println!("   🟢 Connectivity: Healthy"),
        ConnectivityStatus::Degraded(msg) => println!("   🟡 Connectivity: Degraded ({})", msg),
        ConnectivityStatus::Failed(msg) => println!("   🔴 Connectivity: Failed ({})", msg),
    }

    match health.i2c_health {
        I2cHealthStatus::Healthy => println!("   🟢 I2C Health: Healthy"),
        I2cHealthStatus::Degraded(msg) => println!("   🟡 I2C Health: Degraded ({})", msg),
        I2cHealthStatus::Failed(msg) => println!("   🔴 I2C Health: Failed ({})", msg),
    }

    println!("   📊 Error rate: {:.2}%", health.error_rate * 100.0);
    println!("   ⚡ Performance:");
    println!(
        "      ⏱️  Avg response time: {:.2}ms",
        health.performance.avg_response_time_ms
    );
    println!(
        "      ✅ Success rate: {:.2}%",
        health.performance.success_rate * 100.0
    );

    // Demonstrate I2C configuration
    println!("\n⚙️  I2C Configuration:");
    let config = device.get_i2c_config();
    println!("   📏 Max packet size: {} bytes", config.max_packet_size);
    println!("   📦 Auto fragment: {}", config.auto_fragment);
    println!("   ⏱️  Fragment delay: {}ms", config.fragment_delay_ms);
    println!("   🔍 Validation level: {:?}", config.validation_level);

    // Show how to modify I2C configuration
    let new_config = I2cConfig {
        max_packet_size: 32,
        timeout_ms: 1000,
        retry_attempts: 3,
        auto_fragment: true,
        fragment_delay_ms: 5,
        validation_level: ValidationLevel::Basic,
        performance_monitoring: false,
    };

    device.set_i2c_config(new_config);
    println!("✅ Updated I2C configuration with auto-fragmentation enabled");

    Ok(())
}
