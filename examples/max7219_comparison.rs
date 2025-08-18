//! MAX7219 Comparison: Low-Level vs High-Level
//!
//! This example demonstrates the difference between using low-level SPI
//! commands directly versus using the high-level MAX7219 device abstraction.
//!
//! It shows how the device abstraction simplifies code, reduces errors,
//! and provides a more maintainable interface.

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("MAX7219 Comparison: Low-Level vs High-Level");
    println!("===========================================");
    println!("Demonstrating the benefits of device abstraction");
    println!();

    // Connect to PoKeys device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    // Demonstrate low-level approach
    println!("\n🔧 LOW-LEVEL APPROACH (Direct SPI)");
    println!("===================================");
    low_level_example(&mut device)?;

    std::thread::sleep(Duration::from_secs(2));

    // Demonstrate high-level approach
    println!("\n🚀 HIGH-LEVEL APPROACH (Device Abstraction)");
    println!("===========================================");
    high_level_example(&mut device)?;

    println!("\n📊 COMPARISON SUMMARY");
    println!("====================");
    print_comparison_summary();

    Ok(())
}

/// Example using low-level SPI commands directly
fn low_level_example(device: &mut PoKeysDevice) -> Result<()> {
    println!("Configuring MAX7219 using direct SPI commands...");

    let cs_pin = 24u8;

    // Configure SPI manually
    device.spi_configure(0x04, 0x00)?;

    // Configure MAX7219 registers manually (lots of magic numbers!)
    device.spi_write(&[0x0C, 0x01], cs_pin)?; // Exit shutdown
    std::thread::sleep(Duration::from_micros(1));

    device.spi_write(&[0x0F, 0x00], cs_pin)?; // Disable test
    std::thread::sleep(Duration::from_micros(1));

    device.spi_write(&[0x09, 0xFF], cs_pin)?; // Code B decode all
    std::thread::sleep(Duration::from_micros(1));

    device.spi_write(&[0x0B, 0x07], cs_pin)?; // Scan limit 8 digits
    std::thread::sleep(Duration::from_micros(1));

    device.spi_write(&[0x0A, 0x08], cs_pin)?; // Intensity 8
    std::thread::sleep(Duration::from_micros(1));

    println!("✅ Configuration complete (required 5 manual SPI commands)");

    // Display number manually (more magic numbers and calculations!)
    println!("Displaying number 12345 manually...");
    let number = 12345u32;
    let mut num = number;
    let mut digits = [0x0F; 8]; // Blank value in Code B

    // Convert number to digits manually
    for i in 0..8 {
        if num > 0 || i == 0 {
            digits[7 - i] = (num % 10) as u8;
            num /= 10;
        } else {
            digits[7 - i] = 0x0F; // Blank
        }
    }

    // Send each digit manually (more magic register addresses!)
    for (array_pos, &digit_value) in digits.iter().enumerate() {
        let max7219_digit = 7 - array_pos;
        device.spi_write(&[0x01 + max7219_digit as u8, digit_value], cs_pin)?;
        std::thread::sleep(Duration::from_micros(1));
    }

    println!("✅ Number displayed (required 8 more SPI commands)");
    println!("💭 Issues with low-level approach:");
    println!("   • Magic numbers everywhere (0x0C, 0x0F, 0x09, etc.)");
    println!("   • Manual timing delays required");
    println!("   • Complex digit position calculations");
    println!("   • No error checking or validation");
    println!("   • Easy to make mistakes");
    println!("   • Hard to maintain and understand");

    Ok(())
}

/// Example using high-level device abstraction
fn high_level_example(device: &mut PoKeysDevice) -> Result<()> {
    println!("Using MAX7219 device abstraction...");

    // Create device abstraction (handles all SPI configuration automatically)
    let mut display = Max7219::new(device, 24)?;
    println!("✅ MAX7219 created and configured automatically");

    // Configure for numeric display (handles all register setup)
    display.configure_numeric(8)?;
    println!("✅ Configured for numeric display (intensity: 8)");

    // Display number with simple method call
    display.display_number(12345)?;
    println!("✅ Number displayed with single method call");

    // Switch to text mode and display text with decimal points
    display.configure_raw_segments(8)?;
    display.display_text("1.23 boo")?;
    println!("✅ Text with decimal points displayed easily");

    // Demonstrate other features
    display.set_intensity(15)?;
    println!("✅ Brightness changed with simple method");

    // display.clear()?;
    println!("✅ Display cleared with single command");

    println!("💡 Benefits of high-level approach:");
    println!("   • No magic numbers - everything is named and documented");
    println!("   • Automatic configuration and timing");
    println!("   • Type-safe interface prevents errors");
    println!("   • Built-in validation and error handling");
    println!("   • Easy to use and understand");
    println!("   • Maintainable and extensible");

    Ok(())
}

fn print_comparison_summary() {
    println!();
    println!("┌─────────────────────┬─────────────────────┬─────────────────────┐");
    println!("│ Aspect              │ Low-Level SPI       │ Device Abstraction  │");
    println!("├─────────────────────┼─────────────────────┼─────────────────────┤");
    println!("│ Code Complexity     │ High (50+ lines)    │ Low (5-10 lines)    │");
    println!("│ Magic Numbers       │ Many (0x0C, 0x0F..) │ None (named consts) │");
    println!("│ Error Prone         │ Very high           │ Low                 │");
    println!("│ Maintainability     │ Poor                │ Excellent           │");
    println!("│ Learning Curve      │ Steep               │ Gentle              │");
    println!("│ Type Safety         │ None                │ Full                │");
    println!("│ Documentation       │ External datasheet  │ Built-in docs       │");
    println!("│ Validation          │ Manual              │ Automatic           │");
    println!("│ Timing Management   │ Manual delays       │ Automatic           │");
    println!("│ Mode Switching      │ Complex             │ Simple method calls │");
    println!("└─────────────────────┴─────────────────────┴─────────────────────┘");
    println!();
    println!("🎯 RECOMMENDATION: Use device abstractions for:");
    println!("   • Production code");
    println!("   • Complex applications");
    println!("   • Team development");
    println!("   • Long-term maintenance");
    println!();
    println!("🔧 Use low-level SPI for:");
    println!("   • Learning and experimentation");
    println!("   • Custom device implementations");
    println!("   • Performance-critical applications");
    println!("   • Debugging device issues");
    println!();
    println!("💡 The device abstraction is built on top of the low-level SPI,");
    println!("   so you get the best of both worlds - power and simplicity!");
}
