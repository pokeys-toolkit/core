//! MAX7219 Chip Select Test
//!
//! This test specifically focuses on proper CS pin handling for MAX7219 communication.
//! The MAX7219 requires:
//! - CS HIGH when idle (default state)
//! - CS LOW during SPI transmission
//! - CS HIGH after transmission to latch data

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

const CS_PIN: u8 = 24;

fn main() -> Result<()> {
    println!("MAX7219 Chip Select Test");
    println!("========================");
    println!("Testing proper CS pin handling for MAX7219 on pin {CS_PIN}");
    println!("Using network device with serial: 32218");

    // Connect to network device with serial 32218
    println!("\n🔍 Discovering network devices...");
    let network_devices = enumerate_network_devices(3000)?;
    if network_devices.is_empty() {
        println!("❌ No network devices found!");
        return Ok(());
    }

    println!("✅ Found {} network device(s)", network_devices.len());

    // Find device with serial 32218
    let target_device = network_devices
        .iter()
        .find(|dev| dev.serial_number == 32218);
    if target_device.is_none() {
        println!("❌ Network device with serial 32218 not found!");
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

    let _device_info = target_device.unwrap();
    println!("✅ Found target device - Serial: 32218");

    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!(
        "✅ Connected to network device: {}",
        device.device_data.serial_number
    );

    // Step 1: Configure CS pin as digital output and set it HIGH (idle state)
    println!("\n🔧 Step 1: Configuring CS pin...");
    println!("   Setting pin {CS_PIN} as digital output");
    device.set_pin_function(CS_PIN.into(), PinFunction::DigitalOutput)?;

    println!("   Setting CS pin HIGH (idle state)");
    device.set_digital_output(CS_PIN.into(), true)?; // HIGH = idle state for MAX7219

    std::thread::sleep(std::time::Duration::from_millis(10)); // Let pin settle
    println!("✅ CS pin configured and set to idle state");

    // Step 2: Configure SPI
    println!("\n🔧 Step 2: Configuring SPI...");
    let spi_prescaler = 0x04; // Moderate clock speed
    let spi_frame_format = 0x00; // SPI Mode 0 (CPOL=0, CPHA=0)

    device.spi_configure(spi_prescaler, spi_frame_format)?;
    println!("✅ SPI configured (prescaler: 0x{spi_prescaler:02X}, mode: 0)");

    // Step 3: Test CS pin control during SPI operations
    println!("\n🧪 Step 3: Testing CS control during SPI operations...");

    // Read current CS pin state
    device.read_device_data()?;
    let cs_state_before = device.get_digital_input(CS_PIN.into())?;
    println!(
        "   CS pin state before SPI: {}",
        if cs_state_before { "HIGH" } else { "LOW" }
    );

    // Send MAX7219 command (shutdown register = normal mode)
    println!("   Sending MAX7219 command: Shutdown = Normal Mode");
    let command = vec![0x0C, 0x01]; // Shutdown register, normal mode

    println!("   📡 SPI Write: CS should go LOW during transmission, then HIGH");
    device.spi_write(&command, CS_PIN)?;

    // Small delay to let CS settle
    std::thread::sleep(std::time::Duration::from_millis(1));

    // Read CS pin state after SPI
    device.read_device_data()?;
    let cs_state_after = device.get_digital_input(CS_PIN.into())?;
    println!(
        "   CS pin state after SPI: {}",
        if cs_state_after { "HIGH" } else { "LOW" }
    );

    // Step 4: Manual CS control test
    println!("\n🔧 Step 4: Manual CS control test...");
    println!("   This demonstrates manual CS control vs automatic SPI CS control");

    // Manual CS control
    println!("   Manual: Setting CS LOW");
    device.set_digital_output(CS_PIN.into(), false)?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    device.read_device_data()?;
    let manual_low = device.get_digital_input(CS_PIN.into())?;
    println!(
        "   Manual CS state: {}",
        if manual_low { "HIGH" } else { "LOW" }
    );

    println!("   Manual: Setting CS HIGH");
    device.set_digital_output(CS_PIN.into(), true)?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    device.read_device_data()?;
    let manual_high = device.get_digital_input(CS_PIN.into())?;
    println!(
        "   Manual CS state: {}",
        if manual_high { "HIGH" } else { "LOW" }
    );

    // Step 5: Complete MAX7219 initialization sequence
    println!("\n🚀 Step 5: Complete MAX7219 initialization...");

    // Ensure CS is HIGH before starting
    device.set_digital_output(CS_PIN.into(), true)?;
    std::thread::sleep(std::time::Duration::from_millis(1));

    let max7219_commands = [
        (0x0C, 0x01, "Exit shutdown mode"),
        (0x0F, 0x00, "Disable display test"),
        (0x09, 0xFF, "Set decode mode (BCD all digits)"),
        (0x0B, 0x07, "Set scan limit (8 digits)"),
        (0x0A, 0x08, "Set intensity (medium)"),
    ];

    for (register, value, description) in &max7219_commands {
        println!("   Sending: {description} (0x{register:02X} = 0x{value:02X})");
        let cmd = vec![*register, *value];
        device.spi_write(&cmd, CS_PIN)?;
        std::thread::sleep(std::time::Duration::from_millis(1)); // Small delay between commands
    }

    // Display test pattern
    println!("   Displaying test pattern (12345678)");
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, digit + 1]; // Digit register, value 1-8
        device.spi_write(&cmd, CS_PIN)?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    println!("✅ MAX7219 initialization completed!");

    // Step 6: CS pin analysis
    println!("\n📊 Step 6: CS Pin Analysis");
    println!("   Expected behavior:");
    println!("   - CS should be HIGH when idle");
    println!("   - CS should go LOW during SPI transmission");
    println!("   - CS should return HIGH after transmission to latch data");
    println!("   - PoKeys firmware should handle CS timing automatically");

    if cs_state_before && cs_state_after {
        println!("   ✅ CS pin behavior appears correct (HIGH before and after SPI)");
    } else {
        println!("   ⚠️  CS pin behavior may need investigation:");
        println!(
            "      Before SPI: {}",
            if cs_state_before { "HIGH" } else { "LOW" }
        );
        println!(
            "      After SPI:  {}",
            if cs_state_after { "HIGH" } else { "LOW" }
        );
    }

    // Step 7: Recommendations
    println!("\n💡 Recommendations:");
    println!("   1. Always configure CS pin as digital output before SPI operations");
    println!("   2. Set CS pin HIGH initially (MAX7219 idle state)");
    println!("   3. Let PoKeys firmware handle CS timing during SPI transactions");
    println!("   4. Add small delays between MAX7219 commands (1ms minimum)");
    println!("   5. Verify CS pin wiring and pull-up resistors if needed");

    println!("\n🎯 Test completed! Check your MAX7219 display for the pattern '12345678'");

    Ok(())
}
