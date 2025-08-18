//! MAX7219 SPI Configuration Test
//!
//! This example demonstrates configuring a MAX7219 display controller
//! using the SPI functionality. The MAX7219 is commonly used for:
//! - 7-segment displays
//! - LED dot matrix displays
//! - LED bar graphs
//!
//! Hardware Setup:
//! - MAX7219 CS (Chip Select) connected to PoKeys pin 24
//! - MAX7219 CLK connected to SPI clock pin
//! - MAX7219 DIN connected to SPI MOSI pin

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

// MAX7219 Register Addresses
#[allow(dead_code)]
const MAX7219_REG_NOOP: u8 = 0x00;
const MAX7219_REG_DIGIT0: u8 = 0x01;
#[allow(dead_code)]
const MAX7219_REG_DIGIT1: u8 = 0x02;
#[allow(dead_code)]
const MAX7219_REG_DIGIT2: u8 = 0x03;
#[allow(dead_code)]
const MAX7219_REG_DIGIT3: u8 = 0x04;
#[allow(dead_code)]
const MAX7219_REG_DIGIT4: u8 = 0x05;
#[allow(dead_code)]
const MAX7219_REG_DIGIT5: u8 = 0x06;
#[allow(dead_code)]
const MAX7219_REG_DIGIT6: u8 = 0x07;
#[allow(dead_code)]
const MAX7219_REG_DIGIT7: u8 = 0x08;
const MAX7219_REG_DECODE_MODE: u8 = 0x09;
const MAX7219_REG_INTENSITY: u8 = 0x0A;
const MAX7219_REG_SCAN_LIMIT: u8 = 0x0B;
const MAX7219_REG_SHUTDOWN: u8 = 0x0C;
const MAX7219_REG_DISPLAY_TEST: u8 = 0x0F;

// MAX7219 Configuration Values
#[allow(dead_code)]
const MAX7219_DECODE_NONE: u8 = 0x00; // No decode for digits
#[allow(dead_code)]
const MAX7219_DECODE_B_DIGIT0: u8 = 0x01; // Code B decode for digit 0
const MAX7219_DECODE_B_ALL: u8 = 0xFF; // Code B decode for all digits

#[allow(dead_code)]
const MAX7219_INTENSITY_MIN: u8 = 0x00; // Minimum intensity (1/32)
#[allow(dead_code)]
const MAX7219_INTENSITY_MAX: u8 = 0x0F; // Maximum intensity (31/32)

#[allow(dead_code)]
const MAX7219_SCAN_LIMIT_1: u8 = 0x00; // Display digit 0 only
const MAX7219_SCAN_LIMIT_8: u8 = 0x07; // Display digits 0-7

const MAX7219_SHUTDOWN_MODE: u8 = 0x00; // Shutdown mode
const MAX7219_NORMAL_MODE: u8 = 0x01; // Normal operation

const MAX7219_TEST_OFF: u8 = 0x00; // Normal operation
const MAX7219_TEST_ON: u8 = 0x01; // Display test mode

// Hardware Configuration
const CS_PIN: u8 = 24; // Chip Select on pin 24

fn main() -> Result<()> {
    println!("MAX7219 SPI Configuration Test");
    println!("==============================");
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

    let device_info = target_device.unwrap();
    println!(
        "✅ Found target device - Serial: {}, IP: {}",
        device_info.serial_number,
        format_ip(device_info.ip_address)
    );

    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!(
        "✅ Connected to network device: {}",
        device.device_data.serial_number
    );

    // Configure SPI for MAX7219
    // MAX7219 supports up to 10MHz clock, uses SPI Mode 0 (CPOL=0, CPHA=0)
    let spi_prescaler = 0x04; // Moderate clock speed
    let spi_frame_format = 0x00; // SPI Mode 0

    println!("\n🔧 Configuring SPI for MAX7219...");
    println!("   Prescaler: 0x{spi_prescaler:02X}");
    println!("   Frame Format: 0x{spi_frame_format:02X} (Mode 0)");
    println!("   Chip Select Pin: {CS_PIN}");

    device.spi_configure(spi_prescaler, spi_frame_format)?;
    println!("✅ SPI configured successfully!");

    // MAX7219 Configuration Sequence
    println!("\n📡 Configuring MAX7219...");

    // Step 1: Exit shutdown mode
    println!("   1. Exiting shutdown mode...");
    max7219_write_register(&mut device, MAX7219_REG_SHUTDOWN, MAX7219_NORMAL_MODE)?;

    // Step 2: Disable display test
    println!("   2. Disabling display test...");
    max7219_write_register(&mut device, MAX7219_REG_DISPLAY_TEST, MAX7219_TEST_OFF)?;

    // Step 3: Set decode mode (Code B for all digits for easy number display)
    println!("   3. Setting decode mode (Code B for all digits)...");
    max7219_write_register(&mut device, MAX7219_REG_DECODE_MODE, MAX7219_DECODE_B_ALL)?;

    // Step 4: Set scan limit (display all 8 digits)
    println!("   4. Setting scan limit (8 digits)...");
    max7219_write_register(&mut device, MAX7219_REG_SCAN_LIMIT, MAX7219_SCAN_LIMIT_8)?;

    // Step 5: Set intensity (medium brightness)
    let intensity = 0x08; // Medium brightness
    println!("   5. Setting intensity (0x{intensity:02X})...");
    max7219_write_register(&mut device, MAX7219_REG_INTENSITY, intensity)?;

    println!("✅ MAX7219 configured successfully!");

    // Test Pattern 1: Display "12345678"
    println!("\n🎯 Test Pattern 1: Displaying '12345678'...");
    for digit in 0..8 {
        let value = digit + 1; // Display 1-8
        max7219_write_register(&mut device, MAX7219_REG_DIGIT0 + digit, value)?;
        println!("   Digit {digit} = {value}");
    }

    println!("✅ Pattern displayed! You should see '12345678' on the display.");
    println!("   Press Enter to continue to next test...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Test Pattern 2: Display "87654321" (reverse)
    println!("\n🎯 Test Pattern 2: Displaying '87654321'...");
    for digit in 0..8 {
        let value = 8 - digit; // Display 8-1
        max7219_write_register(&mut device, MAX7219_REG_DIGIT0 + digit, value)?;
        println!("   Digit {digit} = {value}");
    }

    println!("✅ Pattern displayed! You should see '87654321' on the display.");
    println!("   Press Enter to continue to brightness test...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Test Pattern 3: Brightness sweep
    println!("\n🎯 Test Pattern 3: Brightness sweep...");
    for brightness in 0..=15 {
        println!("   Setting brightness to {brightness} (0x{brightness:02X})");
        max7219_write_register(&mut device, MAX7219_REG_INTENSITY, brightness)?;
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    println!("✅ Brightness sweep completed!");
    println!("   Press Enter to continue to display test...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Test Pattern 4: Display test mode
    println!("\n🎯 Test Pattern 4: Display test mode...");
    println!("   Enabling display test (all segments on)...");
    max7219_write_register(&mut device, MAX7219_REG_DISPLAY_TEST, MAX7219_TEST_ON)?;

    std::thread::sleep(std::time::Duration::from_millis(2000));

    println!("   Disabling display test...");
    max7219_write_register(&mut device, MAX7219_REG_DISPLAY_TEST, MAX7219_TEST_OFF)?;

    println!("✅ Display test completed!");

    // Test Pattern 5: Clear display
    println!("\n🎯 Test Pattern 5: Clearing display...");
    for digit in 0..8 {
        max7219_write_register(&mut device, MAX7219_REG_DIGIT0 + digit, 0x0F)?; // Blank
    }

    println!("✅ Display cleared!");
    println!("   Press Enter to shutdown...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Shutdown
    println!("\n🔌 Shutting down MAX7219...");
    max7219_write_register(&mut device, MAX7219_REG_SHUTDOWN, MAX7219_SHUTDOWN_MODE)?;

    println!("✅ MAX7219 test completed successfully!");
    println!("\n📋 Test Summary:");
    println!("   ✅ SPI configuration");
    println!("   ✅ MAX7219 initialization");
    println!("   ✅ Number display (12345678)");
    println!("   ✅ Reverse display (87654321)");
    println!("   ✅ Brightness control");
    println!("   ✅ Display test mode");
    println!("   ✅ Display clear");
    println!("   ✅ Shutdown");

    Ok(())
}

/// Write a register value to the MAX7219
fn max7219_write_register(device: &mut PoKeysDevice, register: u8, value: u8) -> Result<()> {
    // MAX7219 expects 16-bit commands: [register][value]
    let command = vec![register, value];

    // Send via SPI with CS on pin 24
    device.spi_write(&command, CS_PIN)?;

    // Small delay to ensure command is processed
    std::thread::sleep(std::time::Duration::from_micros(1));

    Ok(())
}

/// Display a number on the MAX7219 (0-99999999)
#[allow(dead_code)]
fn max7219_display_number(device: &mut PoKeysDevice, number: u32) -> Result<()> {
    let mut num = number;
    let mut digits = [0x0F; 8]; // Start with all blanks

    // Convert number to individual digits (right-aligned)
    for i in 0..8 {
        if num > 0 || i == 0 {
            digits[7 - i] = (num % 10) as u8;
            num /= 10;
        } else {
            digits[7 - i] = 0x0F; // Blank
        }
    }

    // Send digits to display
    for (digit_pos, digit_value) in digits.iter().enumerate() {
        max7219_write_register(device, MAX7219_REG_DIGIT0 + digit_pos as u8, *digit_value)?;
    }

    Ok(())
}

/// Set MAX7219 brightness (0-15)
#[allow(dead_code)]
fn max7219_set_brightness(device: &mut PoKeysDevice, brightness: u8) -> Result<()> {
    let brightness = brightness.min(15); // Clamp to valid range
    max7219_write_register(device, MAX7219_REG_INTENSITY, brightness)
}

/// Clear the MAX7219 display
#[allow(dead_code)]
fn max7219_clear(device: &mut PoKeysDevice) -> Result<()> {
    for digit in 0..8 {
        max7219_write_register(device, MAX7219_REG_DIGIT0 + digit, 0x0F)?; // Blank
    }
    Ok(())
}
