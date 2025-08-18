//! MAX7219 Digit Order Verification Test
//!
//! This test verifies that digits display in the correct order
//! by showing specific test patterns.

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

/// Configure MAX7219 for testing
fn configure_max7219(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
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
        (0x09, 0xFF, "Set decode mode (Code B all digits)"),
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

/// Display number with correct digit order
fn display_number_correct_order(device: &mut PoKeysDevice, cs_pin: u8, number: u32) -> Result<()> {
    println!("📟 Displaying number: {number}");

    // Convert number to digits (right-aligned)
    let mut num = number;
    let mut digits = [0x0F; 8]; // Start with all blanks

    // Fill digits from right to left (normal number display)
    for i in 0..8 {
        if num > 0 || i == 0 {
            digits[7 - i] = (num % 10) as u8;
            num /= 10;
        } else {
            digits[7 - i] = 0x0F; // Blank
        }
    }

    // Send digits to display with correct mapping
    // digits[0] = leftmost position -> MAX7219 digit 7
    // digits[7] = rightmost position -> MAX7219 digit 0
    for (array_pos, &digit_value) in digits.iter().enumerate() {
        let max7219_digit = 7 - array_pos; // Convert array position to MAX7219 digit number
        let cmd = vec![0x01 + max7219_digit as u8, digit_value];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(1));

        if digit_value != 0x0F {
            println!("   Array[{array_pos}] -> MAX7219 Digit {max7219_digit} = {digit_value}");
        }
    }

    Ok(())
}

/// Display text with correct digit order
fn display_text_correct_order(device: &mut PoKeysDevice, cs_pin: u8, text: &str) -> Result<()> {
    println!("📟 Displaying text: '{text}'");

    // Convert text to character codes
    let mut display_data = [0x0F; 8]; // Start with all blanks
    for (i, c) in text.chars().enumerate() {
        if i < 8 {
            display_data[i] = match c.to_ascii_uppercase() {
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
                ' ' => 0x0F,
                _ => 0x0A, // Dash for unknown chars
            };
        }
    }

    // Send to display with correct mapping
    // display_data[0] = leftmost character -> MAX7219 digit 7
    // display_data[7] = rightmost character -> MAX7219 digit 0
    for (text_pos, &digit_value) in display_data.iter().enumerate() {
        let max7219_digit = 7 - text_pos; // Reverse the digit position
        let cmd = vec![0x01 + max7219_digit as u8, digit_value];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(1));

        if digit_value != 0x0F {
            println!(
                "   Text[{}] -> MAX7219 Digit {} = '{}' (0x{:02X})",
                text_pos,
                max7219_digit,
                match digit_value {
                    0x00..=0x09 => (b'0' + digit_value) as char,
                    0x0A => '-',
                    0x0B => 'E',
                    0x0C => 'H',
                    0x0D => 'L',
                    0x0E => 'P',
                    0x0F => ' ',
                    _ => '?',
                },
                digit_value
            );
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("MAX7219 Digit Order Verification Test");
    println!("=====================================");
    println!("This test verifies that digits display in the correct order");
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
    let candidate_pins = [8, 24, 23, 25, 22, 26];
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
    configure_max7219(&mut device, cs_pin)?;
    println!("✅ MAX7219 configured");

    // Test 1: Display simple number
    println!("\n🧪 Test 1: Number Display Order");
    println!("Expected: 12345 (should read left-to-right)");
    display_number_correct_order(&mut device, cs_pin, 12345)?;
    std::thread::sleep(std::time::Duration::from_millis(3000));

    // Test 2: Display sequential digits
    println!("\n🧪 Test 2: Sequential Digits");
    println!("Expected: 12345678 (should read left-to-right)");
    display_number_correct_order(&mut device, cs_pin, 12345678)?;
    std::thread::sleep(std::time::Duration::from_millis(3000));

    // Test 3: Display text
    println!("\n🧪 Test 3: Text Display Order");
    println!("Expected: HELLO (H-E-L-L-O--- from left to right)");
    display_text_correct_order(&mut device, cs_pin, "HELLO")?;
    std::thread::sleep(std::time::Duration::from_millis(3000));

    // Test 4: Display mixed content
    println!("\n🧪 Test 4: Mixed Content");
    println!("Expected: E-123 (E-dash-1-2-3 from left to right)");
    display_text_correct_order(&mut device, cs_pin, "E-123")?;
    std::thread::sleep(std::time::Duration::from_millis(3000));

    // Test 5: Clear display
    println!("\n🧪 Test 5: Clear Display");
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, 0x0F]; // Blank all digits
        device.spi_write(&cmd, cs_pin)?;
    }

    println!("\n✅ Digit Order Tests Completed!");
    println!("📋 Verification:");
    println!("   ✅ Numbers should display left-to-right (12345 not 54321)");
    println!("   ✅ Text should display left-to-right (HELLO not OLLEH)");
    println!("   ✅ Mixed content should be readable left-to-right");

    println!("\n💡 If the display order was correct, you can now use:");
    println!(
        "   ./target/debug/pokeys max7219 display --device 32218 --number 12345 --cs-pin {cs_pin}"
    );
    println!("   ./target/debug/pokeys max7219 display --device 32218 --text \"HELLO\" --cs-pin {cs_pin}");

    Ok(())
}
