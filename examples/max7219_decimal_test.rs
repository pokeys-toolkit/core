//! MAX7219 Decimal Point Test
//!
//! This test verifies that decimal points work correctly in text display.

use pokeys_lib::*;

// Segment definitions (same as in raw segments example)
const SEG_A: u8 = 0b01000000; // Bit 6
const SEG_B: u8 = 0b00100000; // Bit 5
const SEG_C: u8 = 0b00010000; // Bit 4
const SEG_D: u8 = 0b00001000; // Bit 3
const SEG_E: u8 = 0b00000100; // Bit 2
const SEG_F: u8 = 0b00000010; // Bit 1
const SEG_G: u8 = 0b00000001; // Bit 0
const SEG_DP: u8 = 0b10000000; // Bit 7 (decimal point)

// Digit patterns
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

const SYMBOL_BLANK: u8 = 0b00000000;

fn char_to_raw_segments(c: char) -> u8 {
    match c {
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
        ' ' => SYMBOL_BLANK,
        _ => SEG_G, // Dash for unknown
    }
}

fn display_text_with_decimals(device: &mut PoKeysDevice, cs_pin: u8, text: &str) -> Result<()> {
    println!("📟 Displaying '{text}' with proper decimal points");

    // Parse text and handle decimal points properly
    let mut display_data = [SYMBOL_BLANK; 8];
    let mut display_pos = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() && display_pos < 8 {
        let current_char = chars[i];

        // Check if next character is a decimal point
        let has_decimal = i + 1 < chars.len() && chars[i + 1] == '.';

        if current_char == '.' {
            // Skip standalone decimal points
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

    // Send to display
    for (text_pos, &segment_pattern) in display_data.iter().enumerate() {
        let max7219_digit = 7 - text_pos;
        let cmd = vec![0x01 + max7219_digit as u8, segment_pattern];
        device.spi_write(&cmd, cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(1));

        if segment_pattern != SYMBOL_BLANK {
            let has_dp = (segment_pattern & SEG_DP) != 0;
            println!(
                "   Position {} = 0x{:02X}{}",
                text_pos,
                segment_pattern,
                if has_dp { " [with DP]" } else { "" }
            );
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("MAX7219 Decimal Point Test");
    println!("=========================");
    println!("Testing proper decimal point display");
    println!();

    // Connect to device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    let cs_pin = 24u8;

    // Configure SPI and MAX7219
    println!("\n🔧 Configuring MAX7219...");
    device.set_pin_function(cs_pin.into(), PinFunction::DigitalOutput)?;
    device.set_digital_output(cs_pin.into(), true)?;
    device.spi_configure(0x04, 0x00)?;

    // Initialize MAX7219 for raw segments
    let init_commands = [
        (0x0C, 0x01), // Exit shutdown
        (0x0F, 0x00), // Disable test
        (0x09, 0x00), // Raw segments (no decode)
        (0x0B, 0x07), // Scan limit 8 digits
        (0x0A, 0x08), // Medium intensity
    ];

    for (reg, val) in &init_commands {
        device.spi_write(&[*reg, *val], cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("✅ MAX7219 configured for raw segments");

    // Test decimal point displays
    let test_cases = ["1.1", "12.34", "1.2.3.4", "123.456", "3.14159"];

    for test_case in &test_cases {
        println!("\n🧪 Testing: '{test_case}'");
        display_text_with_decimals(&mut device, cs_pin, test_case)?;

        println!("   Check display - should show '{test_case}' with proper decimal points");
        println!("   Waiting 3 seconds...");
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Clear display
        for digit in 1..=8 {
            device.spi_write(&[digit, 0x00], cs_pin)?;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n✅ Decimal point test completed!");
    println!("💡 Results:");
    println!("   • '1.1' should show as '1' with decimal point, then '1'");
    println!("   • '12.34' should show as '1', '2' with DP, '3', '4'");
    println!("   • Multiple decimals should work correctly");

    Ok(())
}
