//! Test Character Mappings
//!
//! This example tests the character mappings to ensure they display correctly.

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;

fn main() -> Result<()> {
    println!("🔤 Testing MAX7219 Character Mappings");
    println!("====================================");

    // Connect to device
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device");

    // Create display
    let mut display = Max7219::new(&mut device, 24)?;
    display.configure_raw_segments(8)?;
    display.set_intensity(10)?;
    println!("✅ Display configured");

    // Test special character mappings
    let test_cases = [
        ("ERROR", "Shows 'ErrOr' (R→r mapping)"),
        ("DATA", "Shows 'dAtA' (D→d, a→A mapping)"),
        ("TEST", "Shows 'tESt' (T→t mapping)"),
        ("ready", "Shows 'reAdy' (a→A mapping)"),
        ("Status", "Shows 'StAtus' (mixed case)"),
        ("HELLO", "Shows 'HELLO' (normal uppercase)"),
        ("hello", "Shows 'hello' (normal lowercase)"),
    ];

    for (text, description) in &test_cases {
        println!("\n🔤 Testing: '{text}' - {description}");
        display.display_text(text)?;

        // Wait for user to see the result
        println!("   Press Enter to continue...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }

    // Clear display
    display.clear()?;
    println!("\n✅ Character mapping test completed!");

    println!("\n📋 Summary of Special Mappings:");
    println!("   D (uppercase) → displays as lowercase d");
    println!("   R (uppercase) → displays as lowercase r");
    println!("   T (uppercase) → displays as lowercase t");
    println!("   a (lowercase) → displays as uppercase A");
    println!("   All other characters display normally");

    Ok(())
}
