//! MAX7219 Text Justification Example
//!
//! This example demonstrates the text justification features of the MAX7219
//! device abstraction, showing left, right, and center justification options.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("MAX7219 Text Justification Example");
    println!("==================================");
    println!("Demonstrating left, right, and center text justification");
    println!();

    // Connect to PoKeys device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    // Create MAX7219 device instance
    println!("\n🔧 Creating MAX7219 display controller...");
    let mut display = Max7219::new(&mut device, 24)?; // CS pin 24
    display.configure_raw_segments(10)?; // Medium-high brightness
    println!("✅ MAX7219 configured for raw segments mode");

    // Test texts of different lengths
    let test_texts = [
        ("A", "Single character"),
        ("HI", "Two characters"),
        ("HELLO", "Five characters"),
        ("STATUS", "Six characters"),
        ("DISPLAY", "Seven characters"),
        ("12345678", "Eight characters (full)"),
        ("1.23", "With decimal point"),
        ("12.34.56", "Multiple decimal points"),
        ("[OK]", "Status in brackets"),
        ("[-1]", "Negative number in brackets"),
        ("A-B", "Dash separator"),
    ];

    for (text, description) in &test_texts {
        println!("\n📝 Testing: '{text}' ({description})");

        // Left justification
        println!("   Left justification:");
        display.display_text_justified(text, TextJustification::Left)?;
        print_expected_result(text, TextJustification::Left);
        std::thread::sleep(Duration::from_secs(2));

        // Right justification
        println!("   Right justification:");
        display.display_text_justified(text, TextJustification::Right)?;
        print_expected_result(text, TextJustification::Right);
        std::thread::sleep(Duration::from_secs(2));

        // Center justification
        println!("   Center justification:");
        display.display_text_justified(text, TextJustification::Center)?;
        print_expected_result(text, TextJustification::Center);
        std::thread::sleep(Duration::from_secs(2));

        // Clear between tests
        display.clear()?;
        std::thread::sleep(Duration::from_millis(500));
    }

    // Demonstrate backward compatibility
    println!("\n🔄 Backward Compatibility Test");
    println!("Using original display_text method (should be left-justified):");
    display.display_text("COMPAT")?;
    println!("   Expected: 'COMPAT  ' (left-justified by default)");
    std::thread::sleep(Duration::from_secs(3));

    // Demonstrate practical use cases
    println!("\n💡 Practical Use Cases");

    // Status messages (left-justified)
    println!("   Status messages (left-justified):");
    let status_messages = ["READY", "ERROR", "BUSY", "DONE"];
    for status in &status_messages {
        display.display_text_justified(status, TextJustification::Left)?;
        println!("     Status: '{status}'");
        std::thread::sleep(Duration::from_millis(1000));
    }

    // Numbers (right-justified like calculators)
    println!("   Numbers (right-justified like calculators):");
    let numbers = ["1", "12", "123", "1234", "12345"];
    for number in &numbers {
        display.display_text_justified(number, TextJustification::Right)?;
        println!("     Number: '{number}'");
        std::thread::sleep(Duration::from_millis(1000));
    }

    // Centered titles
    println!("   Centered titles:");
    let titles = ["MENU", "SETUP", "INFO"];
    for title in &titles {
        display.display_text_justified(title, TextJustification::Center)?;
        println!("     Title: '{title}'");
        std::thread::sleep(Duration::from_millis(1500));
    }

    // Final demonstration with decimal numbers
    println!("\n🔢 Decimal Number Formatting");
    let decimal_numbers = ["1.0", "12.5", "123.45", "1.23.45"];

    for number in &decimal_numbers {
        println!("   Number: '{number}'");

        // Show left, right, and center for each
        display.display_text_justified(number, TextJustification::Left)?;
        println!("     Left: Display shows left-aligned");
        std::thread::sleep(Duration::from_millis(1000));

        display.display_text_justified(number, TextJustification::Right)?;
        println!("     Right: Display shows right-aligned");
        std::thread::sleep(Duration::from_millis(1000));

        display.display_text_justified(number, TextJustification::Center)?;
        println!("     Center: Display shows center-aligned");
        std::thread::sleep(Duration::from_millis(1000));
    }

    // Clear display
    display.clear()?;

    println!("\n✅ Text Justification Example Complete!");
    println!();
    println!("📋 Summary of Justification Options:");
    println!("   • Left   - Text starts from leftmost position (default)");
    println!("   • Right  - Text ends at rightmost position");
    println!("   • Center - Text is centered on display");
    println!();
    println!("💡 Use Cases:");
    println!("   • Left   - Status messages, labels, general text");
    println!("   • Right  - Numbers, calculator-style display");
    println!("   • Center - Titles, headings, emphasis");
    println!();
    println!("🔧 Backward Compatibility:");
    println!("   • display_text() still works (left-justified)");
    println!("   • display_text_justified() provides full control");
    println!("   • No breaking changes to existing code");

    Ok(())
}

/// Print expected result for visual verification
fn print_expected_result(text: &str, justification: TextJustification) {
    // Calculate effective length (accounting for decimal points)
    let mut effective_length = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() && effective_length < 8 {
        let current_char = chars[i];

        if current_char == '.' {
            i += 1;
            continue;
        }

        // Check if next character is a decimal point
        let has_decimal = i + 1 < chars.len() && chars[i + 1] == '.';

        if has_decimal {
            i += 1; // Skip the decimal point
        }

        effective_length += 1;
        i += 1;
    }

    // Calculate start position
    let start_pos = match justification {
        TextJustification::Left => 0,
        TextJustification::Right => 8_usize.saturating_sub(effective_length),
        TextJustification::Center => {
            if effective_length >= 8 {
                0
            } else {
                (8 - effective_length) / 2
            }
        }
    };

    // Create visual representation
    let mut visual = ['_'; 8];
    let display_text = if text.len() > 8 { &text[..8] } else { text };

    for (i, c) in display_text.chars().enumerate() {
        if c != '.' && start_pos + i < 8 {
            visual[start_pos + i] = c;
        }
    }

    let visual_str: String = visual.iter().collect();
    println!("     Expected: '{visual_str}' (start pos: {start_pos})");
}
