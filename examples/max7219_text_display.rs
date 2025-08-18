//! MAX7219 Text Display Examples
//!
//! This example demonstrates various ways to display text on MAX7219
//! using both CLI and programmatic approaches.

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

/// Convert character to MAX7219 code (Code B decode mode)
fn char_to_max7219_code(c: char) -> u8 {
    match c.to_ascii_uppercase() {
        '0' => 0x00,
        '1' => 0x01,
        '2' => 0x02,
        '3' => 0x03,
        '4' => 0x04,
        '5' => 0x05,
        '6' => 0x06,
        '7' => 0x07,
        '8' => 0x08,
        '9' => 0x09,
        '-' => 0x0A,
        'E' => 0x0B,
        'H' => 0x0C,
        'L' => 0x0D,
        'P' => 0x0E,
        ' ' => 0x0F, // Blank
        _ => 0x0A,   // Default to dash for unknown characters
    }
}

/// Display text on MAX7219
fn display_text_on_max7219(device: &mut PoKeysDevice, cs_pin: u8, text: &str) -> Result<()> {
    println!("📟 Displaying: '{text}'");

    // Convert text to character codes
    let mut display_data = [0x0F; 8]; // Start with all blanks
    for (i, c) in text.chars().enumerate() {
        if i < 8 {
            display_data[i] = char_to_max7219_code(c);
        }
    }

    // Send to display (FIXED: reverse order for correct left-to-right display)
    // MAX7219 digit 0 = rightmost, digit 7 = leftmost
    for (text_pos, &digit_value) in display_data.iter().enumerate() {
        let max7219_digit = 7 - text_pos; // Reverse the digit position
        let cmd = vec![0x01 + max7219_digit as u8, digit_value];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    Ok(())
}

/// Configure MAX7219 for text display
fn configure_max7219_for_text(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
    // Configure CS pin
    device.set_pin_function(cs_pin.into(), PinFunction::DigitalOutput)?;
    device.set_digital_output(cs_pin.into(), true)?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Configure SPI
    device.spi_configure(0x04, 0x00)?;

    // Initialize MAX7219
    let init_commands = [
        (0x0C, 0x01, "Exit shutdown mode"),
        (0x0F, 0x00, "Disable display test"),
        (0x09, 0xFF, "Set decode mode (Code B for all digits)"),
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

fn main() -> Result<()> {
    println!("MAX7219 Text Display Examples");
    println!("=============================");
    println!("Device: 32218");
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

    // Find working CS pin
    println!("\n🔍 Finding working CS pin...");
    let candidate_pins = [8, 24, 23, 25, 22, 26, 21, 27];
    let mut working_cs_pin = None;

    for &pin in &candidate_pins {
        if device
            .set_pin_function(pin, PinFunction::DigitalOutput)
            .is_ok()
            && device.set_digital_output(pin, true).is_ok()
        {
            working_cs_pin = Some(pin as u8);
            println!("✅ Found working CS pin: {pin}");
            break;
        }
    }

    let cs_pin = match working_cs_pin {
        Some(pin) => pin,
        None => {
            println!("❌ No working CS pin found!");
            return Ok(());
        }
    };

    // Configure MAX7219
    println!("\n🔧 Configuring MAX7219...");
    configure_max7219_for_text(&mut device, cs_pin)?;
    println!("✅ MAX7219 configured for text display");

    // Text display examples
    println!("\n📟 Text Display Examples:");

    let text_examples = [
        ("HELLO", "Basic greeting"),
        ("12345678", "Numbers"),
        ("ERROR", "Error message"),
        ("HELP", "Help message"),
        ("LED", "LED abbreviation"),
        ("1-2-3", "Numbers with dashes"),
        ("E-HELP", "Mixed characters"),
        ("        ", "Clear display (spaces)"),
    ];

    for (text, description) in &text_examples {
        println!("\n   Example: {description} ({text})");
        display_text_on_max7219(&mut device, cs_pin, text)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }

    // CLI examples
    println!("\n💡 CLI Command Examples:");
    println!("You can also use these CLI commands:");
    println!();

    let cli_examples = [
        ("HELLO", "Display greeting"),
        ("ERROR", "Display error message"),
        ("12345", "Display number as text"),
        ("LED-ON", "Display status message"),
        ("HELP", "Display help"),
    ];

    for (text, description) in &cli_examples {
        println!("   # {description}");
        println!("   ./target/debug/pokeys max7219 display --device 32218 --text \"{text}\" --cs-pin {cs_pin}");
        println!();
    }

    // Character support information
    println!("📋 Supported Characters (Code B decode mode):");
    println!("   Numbers: 0-9");
    println!("   Letters: E, H, L, P");
    println!("   Symbols: - (dash), (space)");
    println!("   Note: Other characters display as dash (-)");

    println!("\n🎯 Advanced Usage:");
    println!("   1. Use spaces to clear specific positions");
    println!("   2. Mix numbers and supported letters");
    println!("   3. Use dashes for separators or unknown chars");
    println!("   4. Text is left-aligned, up to 8 characters");

    println!("\n✅ Text display examples completed!");
    println!("💡 Try the CLI commands above to display your own text");

    Ok(())
}
