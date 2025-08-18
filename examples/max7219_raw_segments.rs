//! MAX7219 Raw Segment Control
//!
//! This example demonstrates raw segment control instead of Code B decode mode.
//! You have complete control over individual segments for custom characters,
//! animations, and graphics.

use pokeys_lib::*;

/// Format IP address from [u8; 4] to string
fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

/// MAX7219 segment bit positions
///
/// ```
///   AAA
///  F   B
///  F   B
///   GGG
///  E   C
///  E   C
///   DDD  DP
/// ```
///
/// Bit:  7  6  5  4  3  2  1  0
/// Seg:  DP A  B  C  D  E  F  G
const SEG_A: u8 = 0b01000000; // Bit 6
const SEG_B: u8 = 0b00100000; // Bit 5
const SEG_C: u8 = 0b00010000; // Bit 4
const SEG_D: u8 = 0b00001000; // Bit 3
const SEG_E: u8 = 0b00000100; // Bit 2
const SEG_F: u8 = 0b00000010; // Bit 1
const SEG_G: u8 = 0b00000001; // Bit 0
const SEG_DP: u8 = 0b10000000; // Bit 7 (decimal point)

/// Raw segment patterns for digits 0-9
const DIGIT_PATTERNS: [u8; 10] = [
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F, // 0
    SEG_B | SEG_C,                                 // 1
    SEG_A | SEG_B | SEG_G | SEG_E | SEG_D,         // 2
    SEG_A | SEG_B | SEG_G | SEG_C | SEG_D,         // 3
    SEG_F | SEG_G | SEG_B | SEG_C,                 // 4
    SEG_A | SEG_F | SEG_G | SEG_C | SEG_D,         // 5
    SEG_A | SEG_F | SEG_G | SEG_E | SEG_D | SEG_C, // 6
    SEG_A | SEG_B | SEG_C,                         // 7
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F | SEG_G, // 8
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_F | SEG_G, // 9
];

/// Raw segment patterns for letters
const LETTER_A: u8 = SEG_A | SEG_B | SEG_C | SEG_E | SEG_F | SEG_G;
const LETTER_B: u8 = SEG_C | SEG_D | SEG_E | SEG_F | SEG_G; // lowercase b
const LETTER_C: u8 = SEG_A | SEG_D | SEG_E | SEG_F;
const LETTER_D: u8 = SEG_B | SEG_C | SEG_D | SEG_E | SEG_G; // lowercase d
const LETTER_E: u8 = SEG_A | SEG_D | SEG_E | SEG_F | SEG_G;
const LETTER_F: u8 = SEG_A | SEG_E | SEG_F | SEG_G;
const LETTER_G: u8 = SEG_A | SEG_C | SEG_D | SEG_E | SEG_F;
const LETTER_H: u8 = SEG_B | SEG_C | SEG_E | SEG_F | SEG_G;
const LETTER_I: u8 = SEG_E | SEG_F; // lowercase i
const LETTER_J: u8 = SEG_B | SEG_C | SEG_D | SEG_E;
const LETTER_L: u8 = SEG_D | SEG_E | SEG_F;
const LETTER_N: u8 = SEG_C | SEG_E | SEG_G; // lowercase n
const LETTER_O: u8 = SEG_C | SEG_D | SEG_E | SEG_G; // lowercase o
const LETTER_P: u8 = SEG_A | SEG_B | SEG_E | SEG_F | SEG_G;
const LETTER_R: u8 = SEG_E | SEG_G; // lowercase r
const LETTER_S: u8 = SEG_A | SEG_C | SEG_D | SEG_F | SEG_G;
const LETTER_T: u8 = SEG_D | SEG_E | SEG_F | SEG_G; // lowercase t
const LETTER_U: u8 = SEG_B | SEG_C | SEG_D | SEG_E | SEG_F;
const LETTER_Y: u8 = SEG_B | SEG_C | SEG_D | SEG_F | SEG_G;

/// Special symbols
const SYMBOL_DASH: u8 = SEG_G;
const SYMBOL_UNDERSCORE: u8 = SEG_D;
const SYMBOL_EQUALS: u8 = SEG_D | SEG_G;
const SYMBOL_DEGREE: u8 = SEG_A | SEG_B | SEG_F | SEG_G;
const SYMBOL_BLANK: u8 = 0x00;

/// Convert character to raw segment pattern
fn char_to_raw_segments(c: char) -> u8 {
    match c.to_ascii_uppercase() {
        '0' => DIGIT_PATTERNS[0],
        '1' => DIGIT_PATTERNS[1],
        '2' => DIGIT_PATTERNS[2],
        '3' => DIGIT_PATTERNS[3],
        '4' => DIGIT_PATTERNS[4],
        '5' => DIGIT_PATTERNS[5],
        '6' => DIGIT_PATTERNS[6],
        '7' => DIGIT_PATTERNS[7],
        '8' => DIGIT_PATTERNS[8],
        '9' => DIGIT_PATTERNS[9],
        'A' => LETTER_A,
        'B' => LETTER_B,
        'C' => LETTER_C,
        'D' => LETTER_D,
        'E' => LETTER_E,
        'F' => LETTER_F,
        'G' => LETTER_G,
        'H' => LETTER_H,
        'I' => LETTER_I,
        'J' => LETTER_J,
        'L' => LETTER_L,
        'N' => LETTER_N,
        'O' => LETTER_O,
        'P' => LETTER_P,
        'R' => LETTER_R,
        'S' => LETTER_S,
        'T' => LETTER_T,
        'U' => LETTER_U,
        'Y' => LETTER_Y,
        '-' => SYMBOL_DASH,
        '_' => SYMBOL_UNDERSCORE,
        '=' => SYMBOL_EQUALS,
        '°' => SYMBOL_DEGREE,
        ' ' => SYMBOL_BLANK,
        _ => SYMBOL_DASH, // Default to dash for unknown characters
    }
}

/// Configure MAX7219 for raw segment mode
fn configure_max7219_raw(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
    // Configure CS pin
    device.set_pin_function(cs_pin.into(), PinFunction::DigitalOutput)?;
    device.set_digital_output(cs_pin.into(), true)?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Configure SPI
    device.spi_configure(0x04, 0x00)?;

    // Initialize MAX7219 for RAW SEGMENT mode
    let init_commands = [
        (0x0C, 0x01, "Exit shutdown mode"),
        (0x0F, 0x00, "Disable display test"),
        (0x09, 0x00, "Set decode mode (NO DECODE - raw segments)"), // KEY CHANGE!
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

/// Display text using raw segments with proper decimal point handling
fn display_text_raw(device: &mut PoKeysDevice, cs_pin: u8, text: &str) -> Result<()> {
    println!("📟 Displaying text '{text}' using raw segments");

    // Parse text and handle decimal points properly
    let mut display_data = [SYMBOL_BLANK; 8]; // Start with all blanks
    let mut display_pos = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() && display_pos < 8 {
        let current_char = chars[i];

        // Check if next character is a decimal point
        let has_decimal = i + 1 < chars.len() && chars[i + 1] == '.';

        if current_char == '.' {
            // Skip standalone decimal points (they should be combined with previous digit)
            i += 1;
            continue;
        }

        // Get base pattern for current character
        let mut pattern = char_to_raw_segments(current_char);

        // Add decimal point if next character is '.'
        if has_decimal {
            pattern |= SEG_DP;
            i += 1; // Skip the decimal point character
        }

        display_data[display_pos] = pattern;
        display_pos += 1;
        i += 1;
    }

    // Send to display with correct digit order
    for (text_pos, &segment_pattern) in display_data.iter().enumerate() {
        let max7219_digit = 7 - text_pos; // Reverse for correct left-to-right display
        let cmd = vec![0x01 + max7219_digit as u8, segment_pattern];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(1));

        if segment_pattern != SYMBOL_BLANK {
            let has_dp = (segment_pattern & SEG_DP) != 0;
            println!(
                "   Position {} (digit {}) = 0b{:08b} (0x{:02X}){}",
                text_pos,
                max7219_digit,
                segment_pattern,
                segment_pattern,
                if has_dp { " [with DP]" } else { "" }
            );
        }
    }

    Ok(())
}

/// Display number using raw segments
fn display_number_raw(device: &mut PoKeysDevice, cs_pin: u8, number: u32) -> Result<()> {
    println!("📟 Displaying number {number} using raw segments");

    // Convert number to digits
    let mut num = number;
    let mut digits = [SYMBOL_BLANK; 8]; // Start with all blanks

    // Fill digits from right to left
    for i in 0..8 {
        if num > 0 || i == 0 {
            let digit = (num % 10) as usize;
            digits[7 - i] = DIGIT_PATTERNS[digit];
            num /= 10;
        } else {
            digits[7 - i] = SYMBOL_BLANK;
        }
    }

    // Send to display with correct digit order
    for (array_pos, &segment_pattern) in digits.iter().enumerate() {
        let max7219_digit = 7 - array_pos;
        let cmd = vec![0x01 + max7219_digit as u8, segment_pattern];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(1));

        if segment_pattern != SYMBOL_BLANK {
            println!("   Position {array_pos} (digit {max7219_digit}) = 0b{segment_pattern:08b} (0x{segment_pattern:02X})");
        }
    }

    Ok(())
}

/// Display custom patterns
fn display_custom_patterns(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
    println!("🎨 Displaying custom patterns...");

    // Pattern 1: All segments on
    println!("   Pattern 1: All segments on");
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, 0x7F]; // All segments except DP
        device.spi_write(&cmd, cs_pin)?;
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 2: Only horizontal segments
    println!("   Pattern 2: Horizontal segments only");
    let horizontal = SEG_A | SEG_G | SEG_D;
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, horizontal];
        device.spi_write(&cmd, cs_pin)?;
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 3: Only vertical segments
    println!("   Pattern 3: Vertical segments only");
    let vertical = SEG_B | SEG_C | SEG_E | SEG_F;
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, vertical];
        device.spi_write(&cmd, cs_pin)?;
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 4: Decimal points only
    println!("   Pattern 4: Decimal points only");
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, SEG_DP];
        device.spi_write(&cmd, cs_pin)?;
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Pattern 5: Animation - rotating segments
    println!("   Pattern 5: Rotating animation");
    let segments = [SEG_A, SEG_B, SEG_C, SEG_D, SEG_E, SEG_F];
    for _ in 0..3 {
        for &seg in &segments {
            for digit in 0..8 {
                let cmd = vec![0x01 + digit, seg];
                device.spi_write(&cmd, cs_pin)?;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("MAX7219 Raw Segment Control");
    println!("===========================");
    println!("Complete control over individual segments");
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
    let candidate_pins = [24];
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

    // Configure MAX7219 for raw segments
    println!("\n🔧 Configuring MAX7219 for raw segment mode...");
    configure_max7219_raw(&mut device, cs_pin)?;
    println!("✅ MAX7219 configured for raw segments (decode mode = 0x00)");

    // Example 1: Display numbers using raw segments
    println!("\n📟 Example 1: Numbers with raw segments");
    display_number_raw(&mut device, cs_pin, 12345678)?;
    std::thread::sleep(std::time::Duration::from_millis(3000));

    // Example 2: Display text using raw segments
    println!("\n📟 Example 2: Text with raw segments");
    let text_examples = ["HELLO", "ABCDEF", "STATUS", "ERROR"];
    for text in &text_examples {
        display_text_raw(&mut device, cs_pin, text)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }

    // Example 3: Custom patterns and animations
    display_custom_patterns(&mut device, cs_pin)?;

    // Example 4: Mixed content with decimal points
    println!("\n📟 Example 4: Numbers with decimal points");
    display_text_raw(&mut device, cs_pin, "12.34.56")?;

    // Add decimal points manually
    for digit_pos in [1, 3, 5] {
        // Add DP to positions 1, 3, 5
        let max7219_digit = 7 - digit_pos;
        // Read current pattern and add DP
        let current_pattern = char_to_raw_segments(match digit_pos {
            1 => '2',
            3 => '4',
            5 => '6',
            _ => ' ',
        });
        let with_dp = current_pattern | SEG_DP;
        let cmd = vec![0x01 + max7219_digit as u8, with_dp];
        device.spi_write(&cmd, cs_pin)?;
    }

    std::thread::sleep(std::time::Duration::from_millis(3000));

    // Clear display
    println!("\n🧹 Clearing display...");
    for digit in 0..8 {
        let cmd = vec![0x01 + digit, SYMBOL_BLANK];
        device.spi_write(&cmd, cs_pin)?;
    }

    println!("\n✅ Raw Segment Examples Completed!");
    println!("📋 What you learned:");
    println!("   ✅ Raw segment control (decode mode 0x00)");
    println!("   ✅ Custom character patterns");
    println!("   ✅ Individual segment control");
    println!("   ✅ Decimal point control");
    println!("   ✅ Custom animations");

    println!("\n💡 Advantages of raw segments:");
    println!("   • Complete control over display");
    println!("   • Custom characters and symbols");
    println!("   • Animation possibilities");
    println!("   • Decimal point control");
    println!("   • Mixed content flexibility");

    println!("\n🔧 Use CS pin {cs_pin} for future raw segment projects");

    Ok(())
}
