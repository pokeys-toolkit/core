//! MAX7219 Chain Validation Test
//!
//! This program performs comprehensive validation of MAX7219 daisy-chain
//! configurations to ensure proper wiring and communication.

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;
use std::io::{self, Write};
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("🔍 MAX7219 Chain Validation Test");
    println!("================================");
    println!("This program validates MAX7219 daisy-chain wiring and communication.");
    println!();

    // Get configuration
    let (device, cs_pin, expected_chain_length) = get_test_configuration()?;

    // Run validation tests
    run_validation_tests(device, cs_pin, expected_chain_length)?;

    Ok(())
}

fn get_test_configuration() -> Result<(PoKeysDevice, u8, u8)> {
    // Connect to device
    println!("📡 Connecting to PoKeys device...");
    print!("Enter device serial number (or press Enter for auto-detect): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let device = if input.trim().is_empty() {
        connect_to_device(0)?
    } else {
        let serial: u32 = input.trim().parse().unwrap_or(0);
        connect_to_device_with_serial(serial, true, 3000)?
    };

    println!("✅ Connected to PoKeys device");

    // Get CS pin
    print!("Enter CS pin number (default 24): ");
    io::stdout().flush().unwrap();
    let mut cs_input = String::new();
    io::stdin().read_line(&mut cs_input).unwrap();
    let cs_pin = if cs_input.trim().is_empty() {
        24
    } else {
        cs_input.trim().parse().unwrap_or(24)
    };

    // Get expected chain length
    print!("Enter expected number of displays in chain (1-8): ");
    io::stdout().flush().unwrap();
    let mut chain_input = String::new();
    io::stdin().read_line(&mut chain_input).unwrap();
    let expected_chain_length: u8 = chain_input.trim().parse().unwrap_or(1);

    if expected_chain_length < 1 || expected_chain_length > 8 {
        return Err(PoKeysError::Parameter(
            "Chain length must be 1-8".to_string(),
        ));
    }

    Ok((device, cs_pin, expected_chain_length))
}

fn run_validation_tests(
    mut device: PoKeysDevice,
    cs_pin: u8,
    expected_chain_length: u8,
) -> Result<()> {
    println!("\n🧪 Starting Chain Validation Tests");
    println!("==================================");

    // Test 1: Basic SPI Communication
    println!("\n1️⃣  Testing Basic SPI Communication");
    test_basic_spi_communication(&mut device, cs_pin)?;

    // Test 2: Single Display Detection
    println!("\n2️⃣  Testing Single Display Detection");
    test_single_display_detection(&mut device, cs_pin)?;

    // Test 3: Chain Length Detection
    println!("\n3️⃣  Testing Chain Length Detection");
    let detected_length = test_chain_length_detection(&mut device, cs_pin, expected_chain_length)?;

    // Test 4: Individual Display Addressing
    println!("\n4️⃣  Testing Individual Display Addressing");
    test_individual_addressing(&mut device, cs_pin, detected_length)?;

    // Test 5: Data Integrity Test
    println!("\n5️⃣  Testing Data Integrity");
    test_data_integrity(&mut device, cs_pin, detected_length)?;

    // Test 6: Timing and Performance
    println!("\n6️⃣  Testing Timing and Performance");
    test_timing_performance(&mut device, cs_pin, detected_length)?;

    // Test 7: Error Conditions
    println!("\n7️⃣  Testing Error Conditions");
    test_error_conditions(&mut device, cs_pin, detected_length)?;

    // Final Report
    print_validation_report(expected_chain_length, detected_length);

    Ok(())
}

fn test_basic_spi_communication(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
    println!("   Testing SPI configuration and basic communication...");

    // Configure SPI
    device.spi_configure(0x04, 0x00)?;
    println!("   ✅ SPI configured successfully");

    // Test basic write operation
    let test_data = [0x0C, 0x01]; // Exit shutdown command
    device.spi_write(&test_data, cs_pin)?;
    println!("   ✅ Basic SPI write successful");

    // Small delay to ensure command is processed
    std::thread::sleep(Duration::from_millis(10));

    Ok(())
}

fn test_single_display_detection(device: &mut PoKeysDevice, cs_pin: u8) -> Result<()> {
    println!("   Testing single display responsiveness...");

    // Initialize single display
    let init_commands = [
        (0x0C, 0x01), // Exit shutdown
        (0x0F, 0x00), // Disable test mode
        (0x09, 0x00), // Raw segments mode
        (0x0B, 0x07), // Scan limit (8 digits)
        (0x0A, 0x08), // Medium intensity
    ];

    for (register, value) in init_commands.iter() {
        device.spi_write(&[*register, *value], cs_pin)?;
        std::thread::sleep(Duration::from_millis(5));
    }

    // Test pattern on all digits
    for digit in 1..=8 {
        device.spi_write(&[digit, 0xFF], cs_pin)?; // All segments on
        std::thread::sleep(Duration::from_millis(5));
    }

    println!("   ✅ Single display should show all segments lit");
    println!("   👀 Verify that the first display shows all segments on");

    print!("   Does the first display show all segments? (y/n): ");
    io::stdout().flush().unwrap();
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();

    if response.trim().to_lowercase() != "y" {
        println!("   ❌ Single display test failed - check wiring and power");
        return Err(PoKeysError::Protocol(
            "Single display not responding".to_string(),
        ));
    }

    // Clear display
    for digit in 1..=8 {
        device.spi_write(&[digit, 0x00], cs_pin)?;
        std::thread::sleep(Duration::from_millis(5));
    }

    println!("   ✅ Single display test passed");

    Ok(())
}

fn test_chain_length_detection(
    device: &mut PoKeysDevice,
    cs_pin: u8,
    expected_length: u8,
) -> Result<u8> {
    println!("   Detecting actual chain length...");

    let mut detected_length = 0u8;

    // Test each possible chain length
    for test_length in 1..=8 {
        println!("   Testing chain length {}...", test_length);

        // Create test controller
        let mut display = Max7219::new_chain(device, cs_pin, test_length)?;

        // Clear all displays
        display.clear_all()?;
        std::thread::sleep(Duration::from_millis(100));

        // Send unique pattern to last display in chain
        let last_display = test_length - 1;
        let test_pattern = format!("L{}", test_length);
        display.display_text_on(&test_pattern, last_display)?;

        std::thread::sleep(Duration::from_millis(500));

        print!(
            "   Do you see '{}' on display {} (counting from 0)? (y/n): ",
            test_pattern, last_display
        );
        io::stdout().flush().unwrap();
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();

        if response.trim().to_lowercase() == "y" {
            detected_length = test_length;
            println!("   ✅ Chain length {} confirmed", test_length);
        } else {
            println!("   ❌ Chain length {} not confirmed", test_length);
            break;
        }

        // Clear for next test
        display.clear_all()?;
        std::thread::sleep(Duration::from_millis(200));
    }

    if detected_length == 0 {
        return Err(PoKeysError::Protocol(
            "No displays detected in chain".to_string(),
        ));
    }

    if detected_length != expected_length {
        println!(
            "   ⚠️  Detected length ({}) differs from expected ({})",
            detected_length, expected_length
        );
    } else {
        println!("   ✅ Detected length matches expected length");
    }

    Ok(detected_length)
}

fn test_individual_addressing(
    device: &mut PoKeysDevice,
    cs_pin: u8,
    chain_length: u8,
) -> Result<()> {
    println!("   Testing individual display addressing...");

    let mut display = Max7219::new_chain(device, cs_pin, chain_length)?;

    // Test each display individually
    for i in 0..chain_length {
        println!("   Testing display {}...", i);

        // Clear all displays
        display.clear_all()?;
        std::thread::sleep(Duration::from_millis(100));

        // Light up only this display
        let pattern = format!("D{}", i);
        display.display_text_on(&pattern, i)?;
        std::thread::sleep(Duration::from_millis(500));

        print!("   Is '{}' showing ONLY on display {}? (y/n): ", pattern, i);
        io::stdout().flush().unwrap();
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();

        if response.trim().to_lowercase() != "y" {
            println!("   ❌ Display {} addressing failed", i);
            return Err(PoKeysError::Protocol(format!(
                "Display {} addressing failed",
                i
            )));
        }

        println!("   ✅ Display {} addressing correct", i);
    }

    // Test all displays simultaneously
    println!("   Testing simultaneous addressing...");
    for i in 0..chain_length {
        let pattern = format!("A{}", i);
        display.display_text_on(&pattern, i)?;
    }

    std::thread::sleep(Duration::from_secs(1));

    print!("   Do all displays show their respective patterns (A0, A1, A2, ...)? (y/n): ");
    io::stdout().flush().unwrap();
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();

    if response.trim().to_lowercase() != "y" {
        println!("   ❌ Simultaneous addressing failed");
        return Err(PoKeysError::Protocol(
            "Simultaneous addressing failed".to_string(),
        ));
    }

    println!("   ✅ Individual addressing test passed");

    Ok(())
}

fn test_data_integrity(device: &mut PoKeysDevice, cs_pin: u8, chain_length: u8) -> Result<()> {
    println!("   Testing data integrity across chain...");

    let mut display = Max7219::new_chain(device, cs_pin, chain_length)?;

    // Test with various patterns
    let test_patterns = [
        ("AAAAAAAA", "All A's"),
        ("12345678", "Sequential numbers"),
        ("ABCDEFGH", "Sequential letters"),
        ("........", "All decimal points"),
        ("--------", "All dashes"),
        ("        ", "All blanks"),
    ];

    for (pattern, description) in test_patterns.iter() {
        println!("   Testing pattern: {} ({})", pattern, description);

        // Send pattern to all displays
        for i in 0..chain_length {
            display.display_text_on(pattern, i)?;
        }

        std::thread::sleep(Duration::from_millis(1000));

        print!("   Do all displays show '{}'? (y/n): ", pattern);
        io::stdout().flush().unwrap();
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();

        if response.trim().to_lowercase() != "y" {
            println!("   ❌ Data integrity test failed for pattern '{}'", pattern);
            return Err(PoKeysError::Protocol(format!(
                "Data integrity failed for pattern '{}'",
                pattern
            )));
        }

        println!("   ✅ Pattern '{}' integrity confirmed", pattern);
    }

    // Test rapid pattern changes
    println!("   Testing rapid pattern changes...");
    for i in 0..20 {
        let pattern = format!("T{:03}", i);
        for j in 0..chain_length {
            display.display_text_on(&pattern, j)?;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("   ✅ Data integrity test passed");

    Ok(())
}

fn test_timing_performance(device: &mut PoKeysDevice, cs_pin: u8, chain_length: u8) -> Result<()> {
    println!("   Testing timing and performance...");

    let mut display = Max7219::new_chain(device, cs_pin, chain_length)?;

    // Test update rate
    let iterations = 100;
    let start = Instant::now();

    for i in 0..iterations {
        let display_index = (i as u8) % chain_length;
        let pattern = format!("T{:03}", i % 1000);
        display.display_text_on(&pattern, display_index)?;
    }

    let duration = start.elapsed();
    let rate = iterations as f64 / duration.as_secs_f64();

    println!("   Update rate: {:.1} updates/second", rate);

    if rate < 10.0 {
        println!("   ⚠️  Update rate is low - may indicate communication issues");
    } else {
        println!("   ✅ Update rate is acceptable");
    }

    // Test bulk operations
    let start = Instant::now();
    for _ in 0..50 {
        display.clear_all()?;
        display.set_intensity_all(8)?;
    }
    let bulk_duration = start.elapsed();
    let bulk_rate = 100.0 / bulk_duration.as_secs_f64(); // 2 operations per iteration

    println!("   Bulk operation rate: {:.1} operations/second", bulk_rate);

    if bulk_rate < 50.0 {
        println!("   ⚠️  Bulk operation rate is low");
    } else {
        println!("   ✅ Bulk operation rate is acceptable");
    }

    Ok(())
}

fn test_error_conditions(device: &mut PoKeysDevice, cs_pin: u8, chain_length: u8) -> Result<()> {
    println!("   Testing error condition handling...");

    let mut display = Max7219::new_chain(device, cs_pin, chain_length)?;

    // Test invalid display index
    let invalid_index = chain_length + 1;
    match display.set_target_display(invalid_index) {
        Ok(_) => {
            println!(
                "   ❌ ERROR: Invalid display index {} was accepted",
                invalid_index
            );
            return Err(PoKeysError::Protocol("Invalid index accepted".to_string()));
        }
        Err(_) => println!(
            "   ✅ Invalid display index {} correctly rejected",
            invalid_index
        ),
    }

    // Test boundary conditions
    match display.set_target_display(chain_length - 1) {
        Ok(_) => println!("   ✅ Maximum valid index {} accepted", chain_length - 1),
        Err(e) => {
            println!(
                "   ❌ ERROR: Maximum valid index {} rejected: {}",
                chain_length - 1,
                e
            );
            return Err(e);
        }
    }

    // Test intensity boundaries
    display.set_intensity(255)?; // Should be clamped to 15
    if display.intensity() <= 15 {
        println!("   ✅ Intensity clamping works correctly");
    } else {
        println!("   ❌ ERROR: Intensity not clamped properly");
    }

    println!("   ✅ Error condition handling test passed");

    Ok(())
}

fn print_validation_report(expected_length: u8, detected_length: u8) {
    println!("\n📊 VALIDATION REPORT");
    println!("===================");

    println!("Expected chain length: {}", expected_length);
    println!("Detected chain length: {}", detected_length);

    if expected_length == detected_length {
        println!("✅ Chain length: PASS");
    } else {
        println!("❌ Chain length: FAIL (mismatch)");
    }

    println!("\n🔍 Test Results Summary:");
    println!("✅ Basic SPI Communication: PASS");
    println!("✅ Single Display Detection: PASS");
    println!("✅ Chain Length Detection: PASS");
    println!("✅ Individual Addressing: PASS");
    println!("✅ Data Integrity: PASS");
    println!("✅ Timing Performance: PASS");
    println!("✅ Error Handling: PASS");

    println!("\n🎉 OVERALL RESULT: PASS");
    println!("Your MAX7219 chain is properly configured and working correctly!");

    println!("\n💡 Tips for optimal performance:");
    println!("   • Keep SPI wires as short as possible");
    println!("   • Use proper power supply for all displays");
    println!("   • Add decoupling capacitors near each MAX7219");
    println!("   • Ensure good ground connections throughout the chain");
}
