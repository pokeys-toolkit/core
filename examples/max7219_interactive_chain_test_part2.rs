//! MAX7219 Interactive Chain Test - Part 2
//!
//! This file contains the remaining test functions for the interactive test.
//! Copy these functions into the main test file.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::io::{self, Write};
use std::time::{Duration, Instant};

fn test_chain_validation(display: &mut Max7219) -> Result<()> {
    println!("\n🔍 Testing Chain Validation");
    println!("===========================");

    // Test each display responds
    println!("1. Testing individual display responsiveness:");
    for i in 0..display.chain_length() {
        println!("   Testing Display {}...", i);

        // Clear display
        display.set_target_display(i)?;
        display.clear()?;
        std::thread::sleep(Duration::from_millis(100));

        // Show test pattern
        display.display_text("TEST")?;
        std::thread::sleep(Duration::from_millis(500));

        // Clear again
        display.clear()?;
        std::thread::sleep(Duration::from_millis(100));

        println!("   ✅ Display {} responsive", i);
    }

    // Test chain integrity with sequential pattern
    println!("\n2. Testing chain integrity with sequential pattern:");
    for i in 0..display.chain_length() {
        let pattern = format!("SEQ{}", i);
        display.display_text_on(&pattern, i)?;
        println!("   Display {}: '{}'", i, pattern);
        std::thread::sleep(Duration::from_millis(200));
    }

    std::thread::sleep(Duration::from_secs(2));

    // Test reverse pattern to verify no cross-talk
    println!("\n3. Testing reverse pattern (checking for cross-talk):");
    for i in (0..display.chain_length()).rev() {
        display.set_target_display(i)?;
        display.clear()?;
        let pattern = format!("REV{}", i);
        display.display_text(&pattern)?;
        println!("   Display {}: '{}'", i, pattern);
        std::thread::sleep(Duration::from_millis(200));
    }

    std::thread::sleep(Duration::from_secs(2));

    // Test simultaneous update
    println!("\n4. Testing simultaneous update:");
    display.clear_all()?;
    std::thread::sleep(Duration::from_millis(500));

    for i in 0..display.chain_length() {
        display.display_text_on("SYNC", i)?;
    }
    println!("   All displays should show 'SYNC' simultaneously");

    std::thread::sleep(Duration::from_secs(2));

    Ok(())
}

fn test_performance(display: &mut Max7219) -> Result<()> {
    println!("\n⚡ Testing Performance");
    println!("=====================");

    let iterations = 100;

    // Test individual display update speed
    println!("1. Testing individual display update speed:");
    let start = Instant::now();

    for i in 0..iterations {
        let display_index = (i as u8) % display.chain_length();
        let text = format!("T{:03}", i % 1000);
        display.display_text_on(&text, display_index)?;
    }

    let individual_duration = start.elapsed();
    let individual_rate = iterations as f64 / individual_duration.as_secs_f64();

    println!(
        "   {} updates in {:.2}s = {:.1} updates/sec",
        iterations,
        individual_duration.as_secs_f64(),
        individual_rate
    );

    // Test bulk operation speed
    println!("\n2. Testing bulk operation speed:");
    let start = Instant::now();

    for i in 0..iterations {
        let intensity = (i % 16) as u8;
        display.set_intensity_all(intensity)?;
    }

    let bulk_duration = start.elapsed();
    let bulk_rate = iterations as f64 / bulk_duration.as_secs_f64();

    println!(
        "   {} bulk updates in {:.2}s = {:.1} updates/sec",
        iterations,
        bulk_duration.as_secs_f64(),
        bulk_rate
    );

    // Test clear operation speed
    println!("\n3. Testing clear operation speed:");
    let start = Instant::now();

    for _ in 0..iterations {
        display.clear_all()?;
    }

    let clear_duration = start.elapsed();
    let clear_rate = iterations as f64 / clear_duration.as_secs_f64();

    println!(
        "   {} clear operations in {:.2}s = {:.1} clears/sec",
        iterations,
        clear_duration.as_secs_f64(),
        clear_rate
    );

    // Performance summary
    println!("\n📊 Performance Summary:");
    println!("   Individual updates: {:.1} ops/sec", individual_rate);
    println!("   Bulk operations:    {:.1} ops/sec", bulk_rate);
    println!("   Clear operations:   {:.1} ops/sec", clear_rate);

    // Reset displays
    display.set_intensity_all(8)?;
    display.clear_all()?;

    Ok(())
}

fn test_error_conditions(display: &mut Max7219) -> Result<()> {
    println!("\n❌ Testing Error Conditions");
    println!("===========================");

    println!("1. Testing invalid display index:");
    let invalid_index = display.chain_length() + 1;
    match display.set_target_display(invalid_index) {
        Ok(_) => println!(
            "   ❌ ERROR: Should have failed for index {}",
            invalid_index
        ),
        Err(e) => println!(
            "   ✅ Correctly rejected invalid index {}: {}",
            invalid_index, e
        ),
    }

    println!("\n2. Testing mode mismatch:");
    // Configure for numeric mode
    display.set_target_display(0)?;
    display.configure_numeric(8)?;

    // Try to display text (should work in numeric mode, but let's test the behavior)
    match display.display_text("HELLO") {
        Ok(_) => {
            println!("   ⚠️  Text display in numeric mode succeeded (this is expected behavior)")
        }
        Err(e) => println!("   ✅ Text display in numeric mode failed: {}", e),
    }

    // Switch back to raw segments
    display.configure_raw_segments(8)?;

    // Try to display number (should fail in raw segments mode)
    match display.display_number(12345) {
        Ok(_) => println!("   ❌ ERROR: Number display in raw segments mode should have failed"),
        Err(e) => println!(
            "   ✅ Correctly rejected number display in raw segments mode: {}",
            e
        ),
    }

    println!("\n3. Testing boundary conditions:");

    // Test maximum intensity
    match display.set_intensity(255) {
        Ok(_) => {
            println!("   ✅ High intensity value accepted (should be clamped to 15)");
            println!("      Current intensity: {}", display.intensity());
        }
        Err(e) => println!("   ❌ High intensity value rejected: {}", e),
    }

    // Test very long text
    let long_text = "VERYLONGTEXT";
    match display.display_text(long_text) {
        Ok(_) => println!("   ✅ Long text accepted (should be truncated to 8 chars)"),
        Err(e) => println!("   ❌ Long text rejected: {}", e),
    }

    // Reset to normal state
    display.set_intensity(8)?;
    display.clear()?;

    println!("\n✅ Error condition testing completed");

    Ok(())
}

fn custom_test_mode(display: &mut Max7219) -> Result<()> {
    println!("\n🛠️  Custom Test Mode");
    println!("===================");

    loop {
        println!("\nCustom Test Options:");
        println!("1. Display custom text on specific display");
        println!("2. Set custom intensity on specific display");
        println!("3. Test custom flash pattern");
        println!("4. Display raw segment pattern");
        println!("5. Chain communication test");
        println!("6. Return to main menu");

        print!("Select option (1-6): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => custom_text_test(display)?,
            "2" => custom_intensity_test(display)?,
            "3" => custom_flash_test(display)?,
            "4" => custom_raw_pattern_test(display)?,
            "5" => custom_chain_test(display)?,
            "6" => break,
            _ => println!("❌ Invalid option. Please select 1-6."),
        }
    }

    Ok(())
}

fn custom_text_test(display: &mut Max7219) -> Result<()> {
    print!("Enter display index (0-{}): ", display.chain_length() - 1);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let display_index: u8 = input.trim().parse().unwrap_or(0);

    if display_index >= display.chain_length() {
        println!("❌ Invalid display index");
        return Ok(());
    }

    print!("Enter text to display: ");
    io::stdout().flush().unwrap();
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let text = text.trim();

    println!("Select justification:");
    println!("1. Left");
    println!("2. Center");
    println!("3. Right");
    print!("Choice (1-3): ");
    io::stdout().flush().unwrap();

    let mut just_input = String::new();
    io::stdin().read_line(&mut just_input).unwrap();

    let justification = match just_input.trim() {
        "2" => TextJustification::Center,
        "3" => TextJustification::Right,
        _ => TextJustification::Left,
    };

    display.display_text_justified_on(text, justification, display_index)?;
    println!(
        "✅ Displayed '{}' on display {} with {:?} justification",
        text, display_index, justification
    );

    Ok(())
}

fn custom_intensity_test(display: &mut Max7219) -> Result<()> {
    print!(
        "Enter display index (0-{}, or 'all' for all displays): ",
        display.chain_length() - 1
    );
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    print!("Enter intensity (0-15): ");
    io::stdout().flush().unwrap();
    let mut intensity_input = String::new();
    io::stdin().read_line(&mut intensity_input).unwrap();
    let intensity: u8 = intensity_input.trim().parse().unwrap_or(8);

    if input == "all" {
        display.set_intensity_all(intensity)?;
        println!("✅ Set intensity {} on all displays", intensity);
    } else {
        let display_index: u8 = input.parse().unwrap_or(0);
        if display_index >= display.chain_length() {
            println!("❌ Invalid display index");
            return Ok(());
        }

        display.set_target_display(display_index)?;
        display.set_intensity(intensity)?;
        println!(
            "✅ Set intensity {} on display {}",
            intensity, display_index
        );
    }

    Ok(())
}

fn custom_flash_test(display: &mut Max7219) -> Result<()> {
    print!("Enter display index (0-{}): ", display.chain_length() - 1);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let display_index: u8 = input.trim().parse().unwrap_or(0);

    if display_index >= display.chain_length() {
        println!("❌ Invalid display index");
        return Ok(());
    }

    print!("Enter text to flash: ");
    io::stdout().flush().unwrap();
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let text = text.trim();

    print!("Enter flash frequency (Hz): ");
    io::stdout().flush().unwrap();
    let mut freq_input = String::new();
    io::stdin().read_line(&mut freq_input).unwrap();
    let frequency: f32 = freq_input.trim().parse().unwrap_or(2.0);

    print!("Enter duration (seconds): ");
    io::stdout().flush().unwrap();
    let mut dur_input = String::new();
    io::stdin().read_line(&mut dur_input).unwrap();
    let duration: f32 = dur_input.trim().parse().unwrap_or(3.0);

    println!(
        "🔥 Flashing '{}' on display {} at {:.1} Hz for {:.1} seconds...",
        text, display_index, frequency, duration
    );

    display.flash_text_on(text, display_index, frequency, duration)?;

    println!("✅ Flash test completed");

    Ok(())
}

fn custom_raw_pattern_test(display: &mut Max7219) -> Result<()> {
    print!("Enter display index (0-{}): ", display.chain_length() - 1);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let display_index: u8 = input.trim().parse().unwrap_or(0);

    if display_index >= display.chain_length() {
        println!("❌ Invalid display index");
        return Ok(());
    }

    println!("Enter 8 hex values for segment patterns (e.g., FF 00 7E 30 6D 79 33 5B):");
    print!("Pattern: ");
    io::stdout().flush().unwrap();

    let mut pattern_input = String::new();
    io::stdin().read_line(&mut pattern_input).unwrap();

    let hex_values: Vec<&str> = pattern_input.trim().split_whitespace().collect();
    if hex_values.len() != 8 {
        println!("❌ Please enter exactly 8 hex values");
        return Ok(());
    }

    let mut patterns = [0u8; 8];
    for (i, hex_str) in hex_values.iter().enumerate() {
        match u8::from_str_radix(hex_str, 16) {
            Ok(value) => patterns[i] = value,
            Err(_) => {
                println!("❌ Invalid hex value: {}", hex_str);
                return Ok(());
            }
        }
    }

    display.set_target_display(display_index)?;
    display.display_raw_patterns(&patterns)?;

    println!("✅ Raw pattern displayed on display {}", display_index);
    println!("   Pattern: {:02X?}", patterns);

    Ok(())
}

fn custom_chain_test(display: &mut Max7219) -> Result<()> {
    println!("🔗 Chain Communication Test");
    println!("This test verifies that each display in the chain receives the correct data");

    // Clear all displays
    display.clear_all()?;
    std::thread::sleep(Duration::from_millis(500));

    // Send unique pattern to each display
    println!("\n1. Sending unique patterns to each display:");
    for i in 0..display.chain_length() {
        let pattern = format!("CH{}", i);
        display.display_text_on(&pattern, i)?;
        println!("   Display {}: '{}'", i, pattern);
        std::thread::sleep(Duration::from_millis(200));
    }

    std::thread::sleep(Duration::from_secs(2));

    // Test cross-communication (verify no interference)
    println!("\n2. Testing for cross-communication interference:");
    for i in 0..display.chain_length() {
        // Clear all first
        display.clear_all()?;
        std::thread::sleep(Duration::from_millis(100));

        // Light up only one display
        display.display_text_on("ONLY", i)?;
        println!("   Only display {} should show 'ONLY'", i);

        print!("   Press Enter to continue to next display...");
        io::stdout().flush().unwrap();
        let mut _dummy = String::new();
        io::stdin().read_line(&mut _dummy).unwrap();
    }

    // Final verification
    println!("\n3. Final chain verification:");
    display.clear_all()?;
    std::thread::sleep(Duration::from_millis(500));

    for i in 0..display.chain_length() {
        display.display_text_on("OK", i)?;
    }

    println!("   All displays should show 'OK'");
    println!("✅ Chain communication test completed");

    Ok(())
}

fn main() -> Result<()> {
    println!("This is part 2 of the interactive chain test");
    println!("Run max7219_interactive_chain_test instead");
    Ok(())
}
