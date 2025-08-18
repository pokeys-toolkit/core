//! Network Device Connection Test
//!
//! This test verifies connection to network device with serial 32218
//! and basic functionality before running SPI tests.

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

fn main() -> Result<()> {
    println!("Network Device Connection Test");
    println!("==============================");
    println!("Target device serial: 32218");
    println!();

    // Step 1: Discover network devices
    println!("🔍 Step 1: Discovering network devices...");
    let network_devices = enumerate_network_devices(5000)?; // 5 second timeout

    if network_devices.is_empty() {
        println!("❌ No network devices found!");
        println!("💡 Make sure:");
        println!("   - Device is powered on");
        println!("   - Device is connected to network");
        println!("   - Device and computer are on same network");
        println!("   - No firewall blocking UDP discovery");
        return Ok(());
    }

    println!("✅ Found {} network device(s):", network_devices.len());
    for (i, device) in network_devices.iter().enumerate() {
        println!(
            "   {}. Serial: {}, IP: {}, FW: {}.{}",
            i + 1,
            device.serial_number,
            format_ip(device.ip_address),
            device.firmware_version_major,
            device.firmware_version_minor
        );
    }

    // Step 2: Find target device
    println!("\n🎯 Step 2: Looking for device with serial 32218...");
    let target_device = network_devices
        .iter()
        .find(|dev| dev.serial_number == 32218);

    if target_device.is_none() {
        println!("❌ Device with serial 32218 not found!");
        println!("Available devices:");
        for device in &network_devices {
            println!(
                "   Serial: {}, IP: {}",
                device.serial_number,
                format_ip(device.ip_address)
            );
        }
        return Ok(());
    }

    let device_info = target_device.unwrap();
    println!("✅ Found target device:");
    println!("   Serial: {}", device_info.serial_number);
    println!("   IP: {}", format_ip(device_info.ip_address));
    println!(
        "   Firmware: {}.{}",
        device_info.firmware_version_major, device_info.firmware_version_minor
    );

    // Step 3: Connect to device
    println!("\n🔗 Step 3: Connecting to device...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected successfully!");

    // Step 4: Read device information
    println!("\n📋 Step 4: Reading device information...");
    device.read_device_data()?;

    println!("Device Details:");
    println!("   Serial Number: {}", device.device_data.serial_number);
    println!("   Device Type: {}", device.device_data.device_type_id);
    println!(
        "   Firmware Version: {}.{}",
        device.device_data.firmware_version_major, device.device_data.firmware_version_minor
    );

    // Convert device name from [u8; 30] to string
    let device_name = String::from_utf8_lossy(&device.device_data.device_name)
        .trim_end_matches('\0')
        .to_string();
    println!("   Device Name: {device_name}");

    // Step 5: Test basic I/O
    println!("\n🧪 Step 5: Testing basic I/O...");

    // Test pin 24 (our CS pin) configuration
    println!("   Testing pin 24 configuration...");
    device.set_pin_function(24, PinFunction::DigitalOutput)?;
    println!("   ✅ Pin 24 configured as digital output");

    device.set_digital_output(24, true)?;
    println!("   ✅ Pin 24 set HIGH");

    std::thread::sleep(std::time::Duration::from_millis(100));

    device.read_device_data()?;
    let pin_state = device.get_digital_input(24)?;
    println!(
        "   📊 Pin 24 state: {}",
        if pin_state { "HIGH" } else { "LOW" }
    );

    // Step 6: Test SPI configuration
    println!("\n⚙️  Step 6: Testing SPI configuration...");
    device.spi_configure(0x04, 0x00)?; // Prescaler 4, Mode 0
    println!("   ✅ SPI configured (prescaler: 0x04, mode: 0)");

    // Step 7: Test SPI communication (no-op command)
    println!("\n📡 Step 7: Testing SPI communication...");
    let noop_command = vec![0x00, 0x00]; // Generic no-op command
    device.spi_write(&noop_command, 24)?;
    println!("   ✅ SPI write successful (no-op command sent)");

    // Step 8: Summary
    println!("\n🎉 Step 8: Test Summary");
    println!("✅ Network device discovery: PASSED");
    println!("✅ Device connection: PASSED");
    println!("✅ Device information read: PASSED");
    println!("✅ Pin configuration: PASSED");
    println!("✅ SPI configuration: PASSED");
    println!("✅ SPI communication: PASSED");

    println!("\n🚀 Device 32218 is ready for SPI device testing!");
    println!("💡 You can now run SPI-based examples with this device");

    Ok(())
}
