//! MAX7219 Device Abstraction Example
//!
//! This example demonstrates the high-level MAX7219 device abstraction
//! that provides a clean, type-safe interface for controlling MAX7219
//! 7-segment displays.
//!
//! The device abstraction handles all the low-level SPI communication
//! and register management, providing simple methods for common operations.

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("MAX7219 Device Abstraction Example");
    println!("==================================");
    println!("Demonstrating high-level MAX7219 control");
    println!();

    // Connect to PoKeys device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    // Create MAX7219 device instance
    println!("\n🔧 Creating MAX7219 display controller...");
    let mut display = Max7219::new(&mut device, 24)?; // CS pin 24
    println!("✅ MAX7219 controller created and initialized");
    println!("   Current mode: {:?}", display.mode());
    println!("   Current intensity: {}", display.intensity());

    // Example 1: Numeric display
    println!("\n📟 Example 1: Numeric Display (Code B mode)");
    display.configure_numeric(8)?;
    println!("✅ Configured for numeric display (intensity: 8)");

    let numbers = [0, 12345, 87654321, 999];
    for &number in &numbers {
        println!("   Displaying: {number}");
        display.display_number(number)?;
        std::thread::sleep(Duration::from_secs(2));
    }

    // Example 2: Text display with decimal points
    println!("\n📟 Example 2: Text Display (Raw Segments mode)");
    display.configure_raw_segments(10)?;
    println!("✅ Configured for raw segments (intensity: 10)");

    let text_examples = ["HELLO", "POKEYS", "STATUS", "ERROR", "1.23", "12.34.56"];
    for text in &text_examples {
        println!("   Displaying: '{text}'");
        display.display_text(text)?;
        std::thread::sleep(Duration::from_secs(2));
    }

    // Example 2b: Text justification
    println!("\n📐 Example 2b: Text Justification");
    use pokeys_lib::devices::spi::TextJustification;

    let test_text = "HELLO";
    println!("   Testing justification with: '{test_text}'");

    println!("   Left justification (default):");
    display.display_text_justified(test_text, TextJustification::Left)?;
    std::thread::sleep(Duration::from_secs(2));

    println!("   Right justification:");
    display.display_text_justified(test_text, TextJustification::Right)?;
    std::thread::sleep(Duration::from_secs(2));

    println!("   Center justification:");
    display.display_text_justified(test_text, TextJustification::Center)?;
    std::thread::sleep(Duration::from_secs(2));

    // Example 3: Brightness control
    println!("\n💡 Example 3: Brightness Control");
    display.display_text("BRIGHT")?;

    println!("   Brightness sweep: 0 → 15");
    for brightness in 0..=15 {
        display.set_intensity(brightness)?;
        std::thread::sleep(Duration::from_millis(200));
    }

    println!("   Brightness sweep: 15 → 0");
    for brightness in (0..=15).rev() {
        display.set_intensity(brightness)?;
        std::thread::sleep(Duration::from_millis(200));
    }

    // Reset to medium brightness
    display.set_intensity(8)?;

    // Example 4: Display test mode
    println!("\n🧪 Example 4: Display Test Mode");
    println!("   Enabling test mode (all segments on)");
    display.set_test_mode(true)?;
    std::thread::sleep(Duration::from_secs(2));

    println!("   Disabling test mode");
    display.set_test_mode(false)?;
    std::thread::sleep(Duration::from_millis(500));

    // Example 5: Raw segment patterns
    println!("\n🎨 Example 5: Raw Segment Patterns");

    // Create custom patterns
    let patterns = [
        0b01000000, // A segment only
        0b00100000, // B segment only
        0b00010000, // C segment only
        0b00001000, // D segment only
        0b00000100, // E segment only
        0b00000010, // F segment only
        0b00000001, // G segment only
        0b10000000, // DP only
    ];

    println!("   Displaying individual segments");
    display.display_raw_patterns(&patterns)?;
    std::thread::sleep(Duration::from_secs(3));

    // Example 6: Mode switching
    println!("\n🔄 Example 6: Mode Switching");

    println!("   Switching to Code B mode");
    display.configure_numeric(8)?;
    display.display_number(88888888)?;
    println!("   Mode: {:?}", display.mode());
    std::thread::sleep(Duration::from_secs(2));

    println!("   Switching to Raw Segments mode");
    display.configure_raw_segments(8)?;
    display.display_text("RAW MODE")?;
    println!("   Mode: {:?}", display.mode());
    std::thread::sleep(Duration::from_secs(2));

    display.display_text("[]-")?;
    println!("   Displaying []-");
    std::thread::sleep(Duration::from_secs(2));

    // Example 7: Power management
    println!("\n⚡ Example 7: Power Management");

    println!("   Entering shutdown mode (power saving)");
    display.set_shutdown(true)?;
    std::thread::sleep(Duration::from_secs(2));

    println!("   Exiting shutdown mode");
    display.set_shutdown(false)?;
    display.display_text("AWAKE")?;
    std::thread::sleep(Duration::from_secs(2));

    // Example 8: Clear display
    println!("\n🧹 Example 8: Clear Display");
    println!("   Clearing display");
    display.clear()?;
    std::thread::sleep(Duration::from_secs(1));

    // Final demonstration
    println!("\n🎉 Final Demonstration");
    display.configure_numeric(12)?;
    display.display_number(12345678)?;
    println!("   Final display: 12345678 (brightness: 12)");

    println!("\n✅ MAX7219 Device Abstraction Example Complete!");
    println!();
    println!("📋 Summary of Features Demonstrated:");
    println!("   • High-level device abstraction");
    println!("   • Automatic SPI configuration");
    println!("   • Code B numeric display mode");
    println!("   • Raw segments text display mode");
    println!("   • Proper decimal point handling");
    println!("   • Brightness control");
    println!("   • Display test mode");
    println!("   • Custom segment patterns");
    println!("   • Mode switching");
    println!("   • Power management");
    println!("   • Display clearing");
    println!();
    println!("💡 Benefits of Device Abstraction:");
    println!("   • Type-safe interface");
    println!("   • Automatic register management");
    println!("   • Error handling");
    println!("   • Mode validation");
    println!("   • Simplified API");
    println!("   • No need to understand MAX7219 registers");

    Ok(())
}
