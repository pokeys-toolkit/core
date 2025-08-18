//! MAX7219 Case Sensitivity Test
//!
//! This example demonstrates that MAX7219 text display is case-sensitive,
//! showing different visual patterns for uppercase and lowercase letters.

use pokeys_lib::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    println!("🔤 MAX7219 Case Sensitivity Test");
    println!("================================");
    println!("This example demonstrates case-sensitive text display on MAX7219.");
    println!("Different cases produce different visual patterns on 7-segment displays.\n");

    // Connect to device
    let device_count = enumerate_usb_devices()?;
    if device_count == 0 {
        println!("❌ No PoKeys devices found!");
        println!("   Please connect a PoKeys device and try again.");
        return Ok(());
    }

    let mut device = connect_to_device(0)?;
    println!("✅ Connected to PoKeys device");

    // Get device data
    device.get_device_data()?;
    println!("   Device: {}", device.device_data.device_name());
    println!("   Serial: {}", device.device_data.serial_number);

    // Configure SPI
    device.spi_configure(0x04, 0x00)?;
    println!("✅ SPI configured");

    // Create MAX7219 display
    let mut display = Max7219::new(&mut device, 24)?;
    display.configure_raw_segments(8)?;
    display.set_intensity(8)?;

    println!("✅ MAX7219 display configured (CS pin 24)");
    println!("\n🎮 Starting case sensitivity demonstration...");
    println!("   Watch the display to see different patterns for upper/lower case");

    // Test cases to demonstrate case sensitivity
    let test_cases = [
        ("HELLO", "Uppercase letters - bold, clear patterns"),
        ("hello", "Lowercase letters - different visual style"),
        ("Hello", "Mixed case - 'H' + 'ello'"),
        ("WORLD", "Another uppercase example"),
        ("world", "Same word in lowercase - notice differences"),
        ("World", "Mixed case variation"),
        ("ABC", "Uppercase A, B, C"),
        ("abc", "Lowercase a, b, c - visually distinct"),
        ("123", "Numbers are case-independent"),
        ("Test", "Mixed: 'T' + 'est'"),
        ("test", "All lowercase version"),
        ("TEST", "All uppercase version"),
    ];

    for (i, (text, description)) in test_cases.iter().enumerate() {
        println!("\n📺 Test {}: \"{}\"", i + 1, text);
        println!("   {description}");

        display.display_text(text)?;

        // Wait for user to observe the display
        thread::sleep(Duration::from_secs(3));
    }

    // Demonstrate character-by-character differences
    println!("\n🔍 Character-by-Character Comparison");
    println!("====================================");

    let char_pairs = [
        ('A', 'a', "Letter A: uppercase vs lowercase"),
        ('B', 'b', "Letter B: uppercase vs lowercase"),
        ('C', 'c', "Letter C: uppercase vs lowercase"),
        ('D', 'd', "Letter D: uppercase vs lowercase"),
        ('H', 'h', "Letter H: uppercase vs lowercase"),
        ('O', 'o', "Letter O: uppercase vs lowercase"),
        ('P', 'p', "Letter P: uppercase vs lowercase"),
        ('U', 'u', "Letter U: uppercase vs lowercase"),
    ];

    for (upper, lower, description) in char_pairs {
        println!("\n🔤 {description}");

        // Show uppercase
        println!("   Displaying: '{upper}'");
        display.display_text(&upper.to_string())?;
        thread::sleep(Duration::from_secs(2));

        // Show lowercase
        println!("   Displaying: '{lower}'");
        display.display_text(&lower.to_string())?;
        thread::sleep(Duration::from_secs(2));
    }

    // Show practical examples
    println!("\n💡 Practical Examples");
    println!("=====================");

    let practical_examples = [
        ("Status", "Mixed case status message"),
        ("ERROR", "Uppercase error message - more prominent"),
        ("error", "Lowercase error message - less prominent"),
        ("Ready", "Mixed case ready message"),
        ("STOP", "Uppercase stop - emergency style"),
        ("stop", "Lowercase stop - normal operation"),
        ("Go", "Mixed case go command"),
        ("PASS", "Uppercase pass - clear indication"),
        ("pass", "Lowercase pass - subtle indication"),
    ];

    for (text, description) in practical_examples {
        println!("\n📋 \"{text}\" - {description}");
        display.display_text(text)?;
        thread::sleep(Duration::from_secs(2));
    }

    // Final message
    display.display_text("Done")?;

    println!("\n🎉 Case Sensitivity Test Complete!");
    println!("==================================");
    println!("✅ Demonstrated uppercase vs lowercase differences");
    println!("✅ Showed mixed case capabilities");
    println!("✅ Provided practical usage examples");

    println!("\n💡 Key Takeaways:");
    println!("   🔤 MAX7219 text display is case-sensitive");
    println!("   📺 Uppercase letters are typically bolder and clearer");
    println!("   📝 Lowercase letters provide different visual styles");
    println!("   🎯 Mixed case allows for varied emphasis");
    println!("   ⚠️  Always consider case when designing display messages");

    println!("\n📚 Usage Guidelines:");
    println!("   • Use UPPERCASE for alerts, errors, and emphasis");
    println!("   • Use lowercase for subtle information");
    println!("   • Use Mixed Case for normal status messages");
    println!("   • Test your specific text on actual hardware");
    println!("   • Consider readability in your application context");

    Ok(())
}
