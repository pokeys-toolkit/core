//! MAX7219 Flash Text Demonstration
//!
//! This example demonstrates the text flashing functionality of the MAX7219 display.
//! It shows various flash patterns, frequencies, and use cases for alerts and notifications.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("🔥 MAX7219 Flash Text Demonstration");
    println!("===================================");

    // Connect to device
    println!("\n📡 Connecting to PoKeys device...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device");

    // Create display
    let mut display = Max7219::new(&mut device, 24)?;
    display.configure_raw_segments(8)?;
    display.set_intensity(10)?;

    println!("\n🎯 Flash Text Demo Scenarios");
    println!("============================");

    // Demo 1: Basic flash
    println!("\n1️⃣  Basic Flash - 'ERROR' at 2 Hz for 5 seconds");
    wait_for_user("Press Enter to start basic flash demo...")?;
    display.flash_text("ERROR", 2.0, 5.0)?;

    // Demo 2: Fast flash for urgent alerts
    println!("\n2️⃣  Fast Flash - 'ALERT' at 5 Hz for 3 seconds");
    wait_for_user("Press Enter to start fast flash demo...")?;
    display.flash_text("ALERT", 5.0, 3.0)?;

    // Demo 3: Slow flash for status
    println!("\n3️⃣  Slow Flash - 'STATUS' at 1 Hz for 4 seconds");
    wait_for_user("Press Enter to start slow flash demo...")?;
    display.flash_text("STATUS", 1.0, 4.0)?;

    // Demo 4: Very fast flash for critical alerts
    println!("\n4️⃣  Critical Flash - 'STOP' at 8 Hz for 2 seconds");
    wait_for_user("Press Enter to start critical flash demo...")?;
    display.flash_text("STOP", 8.0, 2.0)?;

    // Demo 5: Justified flash text
    println!("\n5️⃣  Justified Flash - 'WARN' centered at 3 Hz for 4 seconds");
    wait_for_user("Press Enter to start justified flash demo...")?;
    display.flash_text_justified("WARN", TextJustification::Center, 3.0, 4.0)?;

    // Demo 6: Right-aligned flash
    println!("\n6️⃣  Right-Aligned Flash - 'END' at 2 Hz for 3 seconds");
    wait_for_user("Press Enter to start right-aligned flash demo...")?;
    display.flash_text_justified("END", TextJustification::Right, 2.0, 3.0)?;

    // Demo 7: Character mapping demonstration
    println!("\n7️⃣  Character Mapping - Shows how special characters display");
    wait_for_user("Press Enter to start character mapping demo...")?;

    let char_demos = [
        ("ERROR", "Shows 'ErrOr' (R→r mapping)"),
        ("DATA", "Shows 'dAtA' (D→d, a→A mapping)"),
        ("TEST", "Shows 'tESt' (T→t mapping)"),
        ("ready", "Shows 'reAdy' (a→A mapping)"),
    ];

    for (text, description) in &char_demos {
        println!("   Flashing '{text}' - {description}");
        display.flash_text(text, 2.0, 2.0)?;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Demo 8: Frequency comparison
    println!("\n8️⃣  Frequency Comparison - Same text at different frequencies");
    wait_for_user("Press Enter to start frequency comparison...")?;

    let frequencies = [
        (0.5, "Very Slow"),
        (1.0, "Slow"),
        (2.0, "Normal"),
        (4.0, "Fast"),
        (8.0, "Very Fast"),
    ];

    for (freq, description) in &frequencies {
        println!("   {description} flash ({freq} Hz)");
        display.flash_text("FREQ", *freq, 2.0)?;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Demo 9: Use case scenarios
    println!("\n9️⃣  Real-World Use Cases");
    wait_for_user("Press Enter to start use case demos...")?;

    let use_cases = [
        ("ERROR", 3.0, 3.0, "System Error Alert"),
        ("READY", 1.0, 2.0, "System Ready Status"),
        ("STOP", 5.0, 2.0, "Emergency Stop"),
        ("PASS", 2.0, 2.0, "Test Passed"),
        ("FAIL", 4.0, 3.0, "Test Failed"),
    ];

    for (text, freq, duration, description) in &use_cases {
        println!("   {description} - {text}");
        display.flash_text(text, *freq, *duration)?;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Final cleanup
    display.clear()?;
    println!("\n✅ Flash text demonstration completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Variable flash frequencies (0.5 - 8 Hz)");
    println!("   • Configurable duration");
    println!("   • Text justification (left, center, right)");
    println!("   • Smart character mapping for readability");
    println!("   • Real-world use case scenarios");
    println!("   • Automatic display clearing after flash");

    Ok(())
}

fn wait_for_user(prompt: &str) -> Result<()> {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    Ok(())
}
