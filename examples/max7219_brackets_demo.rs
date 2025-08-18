//! MAX7219 Brackets and Special Characters Demo
//!
//! This example demonstrates the support for bracket characters "[", "]"
//! and dash "-" in raw segment mode for the MAX7219 device abstraction.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("MAX7219 Brackets and Special Characters Demo");
    println!("============================================");
    println!("Demonstrating '[', ']', and '-' character support");
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

    // Demonstrate individual bracket characters
    println!("\n📝 Individual Character Display");

    let special_chars = [
        ("[", "Left bracket"),
        ("]", "Right bracket"),
        ("-", "Dash/minus"),
        ("_", "Underscore"),
    ];

    for (char, description) in &special_chars {
        println!("   Displaying: '{char}' ({description})");
        display.display_text_justified(char, TextJustification::Center)?;
        std::thread::sleep(Duration::from_secs(2));
        display.clear()?;
        std::thread::sleep(Duration::from_millis(500));
    }

    // Demonstrate bracket pairs
    println!("\n🔗 Bracket Pairs");

    let bracket_examples = [
        ("[]", "Empty brackets"),
        ("[1]", "Number in brackets"),
        ("[A]", "Letter in brackets"),
        ("[OK]", "Status in brackets"),
        ("[ERR]", "Error in brackets"),
    ];

    for (text, description) in &bracket_examples {
        println!("   Displaying: '{text}' ({description})");

        // Show left-justified
        display.display_text_justified(text, TextJustification::Left)?;
        println!("     Left-justified");
        std::thread::sleep(Duration::from_secs(1));

        // Show center-justified
        display.display_text_justified(text, TextJustification::Center)?;
        println!("     Center-justified");
        std::thread::sleep(Duration::from_secs(1));

        // Show right-justified
        display.display_text_justified(text, TextJustification::Right)?;
        println!("     Right-justified");
        std::thread::sleep(Duration::from_secs(1));

        display.clear()?;
        std::thread::sleep(Duration::from_millis(500));
    }

    // Demonstrate dash usage
    println!("\n➖ Dash Character Usage");

    let dash_examples = [
        ("-", "Single dash"),
        ("--", "Double dash"),
        ("---", "Triple dash"),
        ("A-B", "Dash separator"),
        ("1-2-3", "Dash delimited"),
        ("ON-OFF", "Status with dash"),
        ("12-34", "Number with dash"),
    ];

    for (text, description) in &dash_examples {
        println!("   Displaying: '{text}' ({description})");
        display.display_text_justified(text, TextJustification::Center)?;
        std::thread::sleep(Duration::from_secs(2));
    }

    // Demonstrate mixed special characters
    println!("\n🎨 Mixed Special Characters");

    let mixed_examples = [
        ("[-]", "Dash in brackets"),
        ("[--]", "Double dash in brackets"),
        ("[-1-]", "Number with dashes in brackets"),
        ("[A-B]", "Letters with dash in brackets"),
        ("[-OK-]", "Status with dashes in brackets"),
        ("[1.23]", "Decimal number in brackets"),
        ("[-1.5]", "Negative decimal in brackets"),
    ];

    for (text, description) in &mixed_examples {
        println!("   Displaying: '{text}' ({description})");
        display.display_text_justified(text, TextJustification::Center)?;
        std::thread::sleep(Duration::from_secs(2));
    }

    // Demonstrate practical applications
    println!("\n💼 Practical Applications");

    println!("   Array/List notation:");
    let array_examples = ["[0]", "[1]", "[2]", "[3]"];
    for item in &array_examples {
        display.display_text_justified(item, TextJustification::Center)?;
        println!("     Array index: {item}");
        std::thread::sleep(Duration::from_millis(800));
    }

    println!("   Status indicators:");
    let status_examples = ["[OK]", "[ERR]", "[---]", "[RDY]"];
    for status in &status_examples {
        display.display_text_justified(status, TextJustification::Center)?;
        println!("     Status: {status}");
        std::thread::sleep(Duration::from_millis(800));
    }

    println!("   Range notation:");
    let range_examples = ["[1-5]", "[A-Z]", "[0-9]"];
    for range in &range_examples {
        display.display_text_justified(range, TextJustification::Center)?;
        println!("     Range: {range}");
        std::thread::sleep(Duration::from_millis(800));
    }

    println!("   Menu options:");
    let menu_examples = ["[1]MENU", "[2]SETUP", "[3]EXIT"];
    for menu in &menu_examples {
        display.display_text_justified(menu, TextJustification::Left)?;
        println!("     Menu: {menu}");
        std::thread::sleep(Duration::from_millis(1000));
    }

    // Demonstrate with different justifications
    println!("\n📐 Justification with Special Characters");

    let test_text = "[HELLO]";
    println!("   Testing justification with: '{test_text}'");

    println!("   Left justification:");
    display.display_text_justified(test_text, TextJustification::Left)?;
    print_visual_representation(test_text, TextJustification::Left);
    std::thread::sleep(Duration::from_secs(2));

    println!("   Right justification:");
    display.display_text_justified(test_text, TextJustification::Right)?;
    print_visual_representation(test_text, TextJustification::Right);
    std::thread::sleep(Duration::from_secs(2));

    println!("   Center justification:");
    display.display_text_justified(test_text, TextJustification::Center)?;
    print_visual_representation(test_text, TextJustification::Center);
    std::thread::sleep(Duration::from_secs(2));

    // Final demonstration
    println!("\n🎉 Final Demonstration");
    display.display_text_justified("[-DONE-]", TextJustification::Center)?;
    println!("   Final display: '[-DONE-]' (centered)");
    std::thread::sleep(Duration::from_secs(3));

    // Clear display
    display.clear()?;

    println!("\n✅ Brackets and Special Characters Demo Complete!");
    println!();
    println!("📋 Supported Special Characters:");
    println!("   • '[' - Left bracket  (segments A,D,E,F)");
    println!("   • ']' - Right bracket (segments A,B,C,D)");
    println!("   • '-' - Dash/minus    (segment G only)");
    println!("   • '_' - Underscore    (segment D only)");
    println!("   • ' ' - Space/blank   (no segments)");
    println!();
    println!("💡 Use Cases:");
    println!("   • Array indices: [0], [1], [2]");
    println!("   • Status indicators: [OK], [ERR], [RDY]");
    println!("   • Range notation: [1-5], [A-Z]");
    println!("   • Menu options: [1]MENU, [2]SETUP");
    println!("   • Negative numbers: [-123], [-1.5]");
    println!("   • Separators: A-B, 12-34");
    println!();
    println!("🔧 Technical Details:");
    println!("   • Left bracket '[': Uses segments A,D,E,F (like 'C')");
    println!("   • Right bracket ']': Uses segments A,B,C,D (like reversed 'C')");
    println!("   • Dash '-': Uses segment G only (horizontal line)");
    println!("   • Works with all justification modes (Left, Right, Center)");
    println!("   • Compatible with decimal points and other characters");

    Ok(())
}

/// Print visual representation of text with justification
fn print_visual_representation(text: &str, justification: TextJustification) {
    // Simple visual representation for verification
    let effective_length = text.len().min(8);

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

    let mut visual = ['_'; 8];
    for (i, c) in text.chars().enumerate() {
        if start_pos + i < 8 {
            visual[start_pos + i] = c;
        }
    }

    let visual_str: String = visual.iter().collect();
    println!("     Visual: '{visual_str}' (start pos: {start_pos})");
}
