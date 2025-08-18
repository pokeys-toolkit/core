//! MAX7219 Chain Stress Test
//!
//! This program performs stress testing on MAX7219 daisy-chain configurations
//! to verify reliability under continuous operation and various load conditions.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("⚡ MAX7219 Chain Stress Test");
    println!("===========================");
    println!("This program stress tests MAX7219 daisy-chain reliability and performance.");
    println!();

    // Get configuration
    let (mut device, cs_pin, chain_length) = get_stress_test_config()?;

    // Create display controller
    let mut display = Max7219::new_chain(&mut device, cs_pin, chain_length)?;

    // Initialize displays
    initialize_for_stress_test(&mut display)?;

    // Run stress tests
    run_stress_tests(&mut display)?;

    // Cleanup
    display.clear_all()?;
    display.set_shutdown_all(true)?;

    println!("\n✅ Stress test completed successfully!");

    Ok(())
}

fn get_stress_test_config() -> Result<(PoKeysDevice, u8, u8)> {
    // Connect to device
    print!("Enter PoKeys device serial (or Enter for auto): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let device = if input.trim().is_empty() {
        connect_to_device(0)?
    } else {
        let serial: u32 = input.trim().parse().unwrap_or(0);
        connect_to_device_with_serial(serial, true, 3000)?
    };

    // Get CS pin
    print!("Enter CS pin (default 24): ");
    io::stdout().flush().unwrap();
    let mut cs_input = String::new();
    io::stdin().read_line(&mut cs_input).unwrap();
    let cs_pin = cs_input.trim().parse().unwrap_or(24);

    // Get chain length
    print!("Enter chain length (1-8): ");
    io::stdout().flush().unwrap();
    let mut chain_input = String::new();
    io::stdin().read_line(&mut chain_input).unwrap();
    let chain_length: u8 = chain_input.trim().parse().unwrap_or(1);

    Ok((device, cs_pin, chain_length))
}

fn initialize_for_stress_test(display: &mut Max7219) -> Result<()> {
    println!("🔧 Initializing displays for stress testing...");

    // Configure all displays
    for i in 0..display.chain_length() {
        display.set_target_display(i)?;
        display.configure_raw_segments(8)?;
    }

    // Initial test pattern
    for i in 0..display.chain_length() {
        display.display_text_on("READY", i)?;
    }

    println!("✅ All displays initialized and ready");
    std::thread::sleep(Duration::from_secs(2));

    Ok(())
}

fn run_stress_tests(display: &mut Max7219) -> Result<()> {
    println!("\n🚀 Starting Stress Tests");
    println!("========================");

    // Test 1: Rapid Update Stress Test
    println!("\n1️⃣  Rapid Update Stress Test");
    rapid_update_stress_test(display)?;

    // Test 2: Pattern Cycling Stress Test
    println!("\n2️⃣  Pattern Cycling Stress Test");
    pattern_cycling_stress_test(display)?;

    // Test 3: Intensity Cycling Stress Test
    println!("\n3️⃣  Intensity Cycling Stress Test");
    intensity_cycling_stress_test(display)?;

    // Test 4: Flash Stress Test
    println!("\n4️⃣  Flash Stress Test");
    flash_stress_test(display)?;

    // Test 5: Mixed Operation Stress Test
    println!("\n5️⃣  Mixed Operation Stress Test");
    mixed_operation_stress_test(display)?;

    // Test 6: Endurance Test
    println!("\n6️⃣  Endurance Test");
    endurance_test(display)?;

    // Test 7: Thermal Stress Test
    println!("\n7️⃣  Thermal Stress Test");
    thermal_stress_test(display)?;

    Ok(())
}

fn rapid_update_stress_test(display: &mut Max7219) -> Result<()> {
    println!("   Testing rapid display updates...");

    let iterations = 1000;
    let start = Instant::now();
    let mut error_count = 0;

    println!("   Performing {} rapid updates...", iterations);

    for i in 0..iterations {
        let display_index = (i as u8) % display.chain_length();
        let pattern = format!("R{:03}", i % 1000);

        match display.display_text_on(&pattern, display_index) {
            Ok(_) => {}
            Err(e) => {
                error_count += 1;
                if error_count <= 5 {
                    // Only print first 5 errors
                    println!("   ❌ Error at iteration {}: {}", i, e);
                }
            }
        }

        // Progress indicator
        if i % 100 == 0 {
            print!(".");
            io::stdout().flush().unwrap();
        }
    }

    let duration = start.elapsed();
    let rate = iterations as f64 / duration.as_secs_f64();

    println!(
        "\n   Completed {} updates in {:.2}s",
        iterations,
        duration.as_secs_f64()
    );
    println!("   Update rate: {:.1} updates/second", rate);
    println!("   Error count: {}", error_count);

    if error_count == 0 {
        println!("   ✅ Rapid update stress test PASSED");
    } else {
        println!(
            "   ⚠️  Rapid update stress test completed with {} errors",
            error_count
        );
    }

    Ok(())
}

fn pattern_cycling_stress_test(display: &mut Max7219) -> Result<()> {
    println!("   Testing pattern cycling stress...");

    let patterns = [
        "AAAAAAAA", "BBBBBBBB", "CCCCCCCC", "DDDDDDDD", "11111111", "22222222", "33333333",
        "44444444", "........", "--------", "________", "        ", "ABCDEFGH", "12345678",
        "abcdefgh", "!@#$%^&*",
    ];

    let cycles = 50;
    let start = Instant::now();

    println!(
        "   Cycling through {} patterns for {} cycles...",
        patterns.len(),
        cycles
    );

    for cycle in 0..cycles {
        for (pattern_idx, pattern) in patterns.iter().enumerate() {
            // Display pattern on all displays
            for display_idx in 0..display.chain_length() {
                display.display_text_on(pattern, display_idx)?;
            }

            // Brief pause to let pattern settle
            std::thread::sleep(Duration::from_millis(50));

            // Progress indicator
            if cycle % 10 == 0 && pattern_idx == 0 {
                print!(".");
                io::stdout().flush().unwrap();
            }
        }
    }

    let duration = start.elapsed();
    let total_updates = cycles * patterns.len() * display.chain_length() as usize;

    println!(
        "\n   Completed {} pattern updates in {:.2}s",
        total_updates,
        duration.as_secs_f64()
    );
    println!("   ✅ Pattern cycling stress test PASSED");

    Ok(())
}

fn intensity_cycling_stress_test(display: &mut Max7219) -> Result<()> {
    println!("   Testing intensity cycling stress...");

    // Set test pattern
    for i in 0..display.chain_length() {
        display.display_text_on("BRIGHT", i)?;
    }

    let cycles = 100;
    let start = Instant::now();

    println!("   Cycling intensity for {} cycles...", cycles);

    for cycle in 0..cycles {
        // Cycle through all intensity levels
        for intensity in 0..=15 {
            display.set_intensity_all(intensity)?;
            std::thread::sleep(Duration::from_millis(20));
        }

        // Reverse cycle
        for intensity in (0..=15).rev() {
            display.set_intensity_all(intensity)?;
            std::thread::sleep(Duration::from_millis(20));
        }

        if cycle % 10 == 0 {
            print!(".");
            io::stdout().flush().unwrap();
        }
    }

    let duration = start.elapsed();

    // Reset to medium intensity
    display.set_intensity_all(8)?;

    println!(
        "\n   Completed intensity cycling in {:.2}s",
        duration.as_secs_f64()
    );
    println!("   ✅ Intensity cycling stress test PASSED");

    Ok(())
}

fn flash_stress_test(display: &mut Max7219) -> Result<()> {
    println!("   Testing flash stress...");

    let flash_duration = 2.0; // seconds per flash test
    let frequencies = [0.5, 1.0, 2.0, 5.0, 10.0];

    println!("   Testing flash at different frequencies...");

    for freq in frequencies.iter() {
        println!(
            "   Flashing at {:.1} Hz for {:.1}s...",
            freq, flash_duration
        );

        // Flash each display in sequence
        for display_idx in 0..display.chain_length() {
            let text = format!("F{:.1}", freq);
            display.flash_text_on(&text, display_idx, *freq, flash_duration)?;
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    println!("   ✅ Flash stress test PASSED");

    Ok(())
}

fn mixed_operation_stress_test(display: &mut Max7219) -> Result<()> {
    println!("   Testing mixed operations stress...");

    let iterations = 200;
    let start = Instant::now();

    println!("   Performing {} mixed operations...", iterations);

    for i in 0..iterations {
        let operation = i % 6;
        let display_idx = (i as u8) % display.chain_length();

        match operation {
            0 => {
                // Text display
                let text = format!("M{:03}", i % 1000);
                display.display_text_on(&text, display_idx)?;
            }
            1 => {
                // Clear display
                display.set_target_display(display_idx)?;
                display.clear()?;
            }
            2 => {
                // Intensity change
                let intensity = (i % 16) as u8;
                display.set_target_display(display_idx)?;
                display.set_intensity(intensity)?;
            }
            3 => {
                // Bulk clear
                display.clear_all()?;
            }
            4 => {
                // Bulk intensity
                let intensity = ((i / 4) % 16) as u8;
                display.set_intensity_all(intensity)?;
            }
            5 => {
                // Justified text
                let justification = match i % 3 {
                    0 => TextJustification::Left,
                    1 => TextJustification::Center,
                    _ => TextJustification::Right,
                };
                display.display_text_justified_on("MIX", justification, display_idx)?;
            }
            _ => unreachable!(),
        }

        // Small delay between operations
        std::thread::sleep(Duration::from_millis(10));

        if i % 20 == 0 {
            print!(".");
            io::stdout().flush().unwrap();
        }
    }

    let duration = start.elapsed();

    println!(
        "\n   Completed mixed operations in {:.2}s",
        duration.as_secs_f64()
    );
    println!("   ✅ Mixed operation stress test PASSED");

    Ok(())
}

fn endurance_test(display: &mut Max7219) -> Result<()> {
    println!("   Testing endurance (long-running operations)...");

    print!("   Enter endurance test duration in seconds (default 60): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let duration_secs: u64 = input.trim().parse().unwrap_or(60);

    println!("   Running endurance test for {} seconds...", duration_secs);

    let start = Instant::now();
    let mut iteration = 0u32;
    let mut last_report = Instant::now();

    while start.elapsed().as_secs() < duration_secs {
        // Cycle through different operations
        let operation = iteration % 4;
        let display_idx = (iteration as u8) % display.chain_length();

        match operation {
            0 => {
                let text = format!("E{:04}", iteration % 10000);
                display.display_text_on(&text, display_idx)?;
            }
            1 => {
                display.clear_all()?;
            }
            2 => {
                let intensity = ((iteration / 10) % 16) as u8;
                display.set_intensity_all(intensity)?;
            }
            3 => {
                for i in 0..display.chain_length() {
                    let text = format!("T{}", i);
                    display.display_text_on(&text, i)?;
                }
            }
            _ => unreachable!(),
        }

        iteration += 1;

        // Progress report every 10 seconds
        if last_report.elapsed().as_secs() >= 10 {
            let elapsed = start.elapsed().as_secs();
            let remaining = duration_secs - elapsed;
            println!(
                "   Progress: {}s elapsed, {}s remaining, {} operations completed",
                elapsed, remaining, iteration
            );
            last_report = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    let total_duration = start.elapsed();
    let rate = iteration as f64 / total_duration.as_secs_f64();

    println!("   Endurance test completed:");
    println!("   Duration: {:.2}s", total_duration.as_secs_f64());
    println!("   Operations: {}", iteration);
    println!("   Average rate: {:.1} operations/second", rate);
    println!("   ✅ Endurance test PASSED");

    Ok(())
}

fn thermal_stress_test(display: &mut Max7219) -> Result<()> {
    println!("   Testing thermal stress (high intensity operations)...");

    println!("   This test runs displays at maximum intensity with all segments on");
    println!("   to generate thermal stress. Monitor display temperatures.");

    print!("   Continue with thermal stress test? (y/n): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() != "y" {
        println!("   Thermal stress test skipped");
        return Ok(());
    }

    // Set maximum intensity
    display.set_intensity_all(15)?;

    // Turn on all segments
    let all_segments_pattern = [0xFF; 8]; // All segments + decimal points
    for i in 0..display.chain_length() {
        display.set_target_display(i)?;
        display.display_raw_patterns(&all_segments_pattern)?;
    }

    println!("   All displays at maximum intensity with all segments on");
    println!("   Running thermal stress for 30 seconds...");
    println!("   Monitor displays for overheating, flickering, or dimming");

    // Run for 30 seconds with status updates
    for i in 0..30 {
        std::thread::sleep(Duration::from_secs(1));
        if (i + 1) % 5 == 0 {
            println!("   Thermal stress: {}s elapsed", i + 1);
        }
    }

    // Cool down period
    println!("   Starting cool-down period...");
    display.set_intensity_all(1)?; // Minimum intensity
    display.clear_all()?;

    std::thread::sleep(Duration::from_secs(10));

    // Return to normal
    display.set_intensity_all(8)?;
    for i in 0..display.chain_length() {
        display.display_text_on("COOL", i)?;
    }

    println!("   ✅ Thermal stress test PASSED");
    println!("   Note: Check displays for any signs of thermal damage");

    Ok(())
}
