//! MAX7219 Automated Chain Test
//!
//! This program performs automated testing of MAX7219 daisy-chain configurations
//! without requiring user interaction. Useful for continuous integration and
//! automated hardware validation.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("🤖 MAX7219 Automated Chain Test");
    println!("===============================");
    println!("Running automated tests on MAX7219 daisy-chain configuration...");
    println!();

    // Configuration (modify these values for your setup)
    let device_serial = 32218u32; // Change to your device serial, or 0 for auto-detect
    let cs_pin = 24u8;
    let chain_length = 3u8; // Change to your chain length

    println!("📋 Test Configuration:");
    println!(
        "   Device Serial: {}",
        if device_serial == 0 {
            "Auto-detect".to_string()
        } else {
            device_serial.to_string()
        }
    );
    println!("   CS Pin: {}", cs_pin);
    println!("   Chain Length: {}", chain_length);
    println!();

    // Connect to device
    println!("📡 Connecting to PoKeys device...");
    let mut device = if device_serial == 0 {
        connect_to_device(0)?
    } else {
        connect_to_device_with_serial(device_serial, true, 3000)?
    };
    println!("✅ Connected successfully");

    // Create display controller
    println!("\n🔧 Creating MAX7219 chain controller...");
    let mut display = Max7219::new_chain(&mut device, cs_pin, chain_length)?;
    println!("✅ Chain controller created");

    // Run automated tests
    let mut test_results = TestResults::new();

    run_automated_tests(&mut display, &mut test_results)?;

    // Print final report
    print_test_report(&test_results);

    // Cleanup
    println!("\n🧹 Cleaning up...");
    display.clear_all()?;
    display.set_shutdown_all(true)?;
    println!("✅ Cleanup completed");

    // Exit with appropriate code
    if test_results.all_passed() {
        println!("\n🎉 ALL TESTS PASSED!");
        std::process::exit(0);
    } else {
        println!("\n❌ SOME TESTS FAILED!");
        std::process::exit(1);
    }
}

#[derive(Debug)]
struct TestResults {
    tests: Vec<TestResult>,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    duration: Duration,
    details: String,
}

impl TestResults {
    fn new() -> Self {
        Self { tests: Vec::new() }
    }

    fn add_result(&mut self, name: String, passed: bool, duration: Duration, details: String) {
        self.tests.push(TestResult {
            name,
            passed,
            duration,
            details,
        });
    }

    fn all_passed(&self) -> bool {
        self.tests.iter().all(|t| t.passed)
    }

    fn passed_count(&self) -> usize {
        self.tests.iter().filter(|t| t.passed).count()
    }

    fn total_count(&self) -> usize {
        self.tests.len()
    }
}

fn run_automated_tests(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    println!("\n🧪 Running Automated Tests");
    println!("==========================");

    // Test 1: Basic Initialization
    test_basic_initialization(display, results)?;

    // Test 2: Individual Display Addressing
    test_individual_addressing(display, results)?;

    // Test 3: Bulk Operations
    test_bulk_operations(display, results)?;

    // Test 4: Text Display Functions
    test_text_display_functions(display, results)?;

    // Test 5: Numeric Display Functions
    test_numeric_display_functions(display, results)?;

    // Test 6: Intensity Control
    test_intensity_control(display, results)?;

    // Test 7: Flash Effects
    test_flash_effects(display, results)?;

    // Test 8: Error Handling
    test_error_handling(display, results)?;

    // Test 9: Performance Benchmarks
    test_performance_benchmarks(display, results)?;

    // Test 10: Chain Integrity
    test_chain_integrity(display, results)?;

    Ok(())
}

fn test_basic_initialization(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("1️⃣  Testing Basic Initialization...");

    // Test chain length
    let expected_length = 3u8; // Adjust based on your setup
    if display.chain_length() == expected_length {
        details.push_str(&format!("Chain length: {} ✓\n", display.chain_length()));
    } else {
        details.push_str(&format!(
            "Chain length: {} (expected {}) ✗\n",
            display.chain_length(),
            expected_length
        ));
        passed = false;
    }

    // Test initial configuration
    for i in 0..display.chain_length() {
        match display.set_target_display(i) {
            Ok(_) => match display.configure_raw_segments(8) {
                Ok(_) => details.push_str(&format!("Display {} configured ✓\n", i)),
                Err(e) => {
                    details.push_str(&format!("Display {} config failed: {} ✗\n", i, e));
                    passed = false;
                }
            },
            Err(e) => {
                details.push_str(&format!("Display {} targeting failed: {} ✗\n", i, e));
                passed = false;
            }
        }
    }

    let duration = start.elapsed();
    results.add_result(
        "Basic Initialization".to_string(),
        passed,
        duration,
        details,
    );

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_individual_addressing(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("2️⃣  Testing Individual Display Addressing...");

    // Clear all displays first
    match display.clear_all() {
        Ok(_) => details.push_str("Clear all displays ✓\n"),
        Err(e) => {
            details.push_str(&format!("Clear all failed: {} ✗\n", e));
            passed = false;
        }
    }

    // Test each display individually
    for i in 0..display.chain_length() {
        let test_text = format!("D{}", i);
        match display.display_text_on(&test_text, i) {
            Ok(_) => {
                details.push_str(&format!("Display {} text '{}' ✓\n", i, test_text));
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => {
                details.push_str(&format!("Display {} text failed: {} ✗\n", i, e));
                passed = false;
            }
        }
    }

    std::thread::sleep(Duration::from_secs(1));

    let duration = start.elapsed();
    results.add_result(
        "Individual Addressing".to_string(),
        passed,
        duration,
        details,
    );

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_bulk_operations(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("3️⃣  Testing Bulk Operations...");

    // Test bulk clear
    match display.clear_all() {
        Ok(_) => details.push_str("Bulk clear ✓\n"),
        Err(e) => {
            details.push_str(&format!("Bulk clear failed: {} ✗\n", e));
            passed = false;
        }
    }

    // Test bulk intensity
    for intensity in [1, 8, 15].iter() {
        match display.set_intensity_all(*intensity) {
            Ok(_) => details.push_str(&format!("Bulk intensity {} ✓\n", intensity)),
            Err(e) => {
                details.push_str(&format!("Bulk intensity {} failed: {} ✗\n", intensity, e));
                passed = false;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    // Test bulk test mode
    match display.set_test_mode_all(true) {
        Ok(_) => {
            details.push_str("Bulk test mode on ✓\n");
            std::thread::sleep(Duration::from_millis(500));

            match display.set_test_mode_all(false) {
                Ok(_) => details.push_str("Bulk test mode off ✓\n"),
                Err(e) => {
                    details.push_str(&format!("Bulk test mode off failed: {} ✗\n", e));
                    passed = false;
                }
            }
        }
        Err(e) => {
            details.push_str(&format!("Bulk test mode on failed: {} ✗\n", e));
            passed = false;
        }
    }

    let duration = start.elapsed();
    results.add_result("Bulk Operations".to_string(), passed, duration, details);

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_text_display_functions(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("4️⃣  Testing Text Display Functions...");

    let test_texts = ["HELLO", "WORLD", "12345", "A.B.C", "hello", "MiXeD"];

    for (i, text) in test_texts.iter().enumerate() {
        let display_index = (i as u8) % display.chain_length();
        match display.display_text_on(text, display_index) {
            Ok(_) => details.push_str(&format!("Text '{}' on display {} ✓\n", text, display_index)),
            Err(e) => {
                details.push_str(&format!("Text '{}' failed: {} ✗\n", text, e));
                passed = false;
            }
        }
        std::thread::sleep(Duration::from_millis(300));
    }

    // Test justification
    let justifications = [
        TextJustification::Left,
        TextJustification::Center,
        TextJustification::Right,
    ];
    for (i, just) in justifications.iter().enumerate() {
        let display_index = (i as u8) % display.chain_length();
        match display.display_text_justified_on("HI", *just, display_index) {
            Ok(_) => details.push_str(&format!("Justified text {:?} ✓\n", just)),
            Err(e) => {
                details.push_str(&format!("Justified text {:?} failed: {} ✗\n", just, e));
                passed = false;
            }
        }
        std::thread::sleep(Duration::from_millis(300));
    }

    let duration = start.elapsed();
    results.add_result(
        "Text Display Functions".to_string(),
        passed,
        duration,
        details,
    );

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_numeric_display_functions(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("5️⃣  Testing Numeric Display Functions...");

    // Configure first display for numeric mode
    match display.set_target_display(0) {
        Ok(_) => {
            match display.configure_numeric(8) {
                Ok(_) => {
                    details.push_str("Numeric mode configuration ✓\n");

                    // Test various numbers
                    let test_numbers = [0, 123, 12345, 99999999];
                    for number in test_numbers.iter() {
                        match display.display_number(*number) {
                            Ok(_) => details.push_str(&format!("Number {} ✓\n", number)),
                            Err(e) => {
                                details.push_str(&format!("Number {} failed: {} ✗\n", number, e));
                                passed = false;
                            }
                        }
                        std::thread::sleep(Duration::from_millis(300));
                    }

                    // Switch back to raw segments
                    match display.configure_raw_segments(8) {
                        Ok(_) => details.push_str("Switch back to raw segments ✓\n"),
                        Err(e) => {
                            details.push_str(&format!("Switch to raw segments failed: {} ✗\n", e));
                            passed = false;
                        }
                    }
                }
                Err(e) => {
                    details.push_str(&format!("Numeric mode config failed: {} ✗\n", e));
                    passed = false;
                }
            }
        }
        Err(e) => {
            details.push_str(&format!("Target display 0 failed: {} ✗\n", e));
            passed = false;
        }
    }

    let duration = start.elapsed();
    results.add_result(
        "Numeric Display Functions".to_string(),
        passed,
        duration,
        details,
    );

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_intensity_control(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("6️⃣  Testing Intensity Control...");

    // Set test pattern
    for i in 0..display.chain_length() {
        match display.display_text_on("BRIGHT", i) {
            Ok(_) => {}
            Err(e) => {
                details.push_str(&format!("Set test pattern failed: {} ✗\n", e));
                passed = false;
            }
        }
    }

    // Test intensity levels
    for intensity in [0, 5, 10, 15].iter() {
        match display.set_intensity_all(*intensity) {
            Ok(_) => {
                details.push_str(&format!("Intensity {} ✓\n", intensity));
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(e) => {
                details.push_str(&format!("Intensity {} failed: {} ✗\n", intensity, e));
                passed = false;
            }
        }
    }

    // Test individual display intensity
    if display.chain_length() > 1 {
        for i in 0..display.chain_length() {
            let intensity = (i * 5) % 16;
            match display.set_target_display(i) {
                Ok(_) => match display.set_intensity(intensity) {
                    Ok(_) => details.push_str(&format!(
                        "Individual intensity {} on display {} ✓\n",
                        intensity, i
                    )),
                    Err(e) => {
                        details.push_str(&format!("Individual intensity failed: {} ✗\n", e));
                        passed = false;
                    }
                },
                Err(e) => {
                    details.push_str(&format!("Target display {} failed: {} ✗\n", i, e));
                    passed = false;
                }
            }
        }
    }

    // Reset to medium intensity
    let _ = display.set_intensity_all(8);

    let duration = start.elapsed();
    results.add_result("Intensity Control".to_string(), passed, duration, details);

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_flash_effects(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("7️⃣  Testing Flash Effects...");

    // Test flash on first display
    match display.flash_text_on("FLASH", 0, 3.0, 2.0) {
        Ok(_) => details.push_str("Flash effect ✓\n"),
        Err(e) => {
            details.push_str(&format!("Flash effect failed: {} ✗\n", e));
            passed = false;
        }
    }

    let duration = start.elapsed();
    results.add_result("Flash Effects".to_string(), passed, duration, details);

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_error_handling(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("8️⃣  Testing Error Handling...");

    // Test invalid display index
    let invalid_index = display.chain_length() + 1;
    match display.set_target_display(invalid_index) {
        Ok(_) => {
            details.push_str(&format!(
                "Invalid index {} accepted (should fail) ✗\n",
                invalid_index
            ));
            passed = false;
        }
        Err(_) => details.push_str(&format!(
            "Invalid index {} correctly rejected ✓\n",
            invalid_index
        )),
    }

    // Test boundary conditions
    match display.set_target_display(display.chain_length() - 1) {
        Ok(_) => details.push_str("Max valid index accepted ✓\n"),
        Err(e) => {
            details.push_str(&format!("Max valid index rejected: {} ✗\n", e));
            passed = false;
        }
    }

    let duration = start.elapsed();
    results.add_result("Error Handling".to_string(), passed, duration, details);

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn test_performance_benchmarks(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let passed = true; // Performance tests don't fail, just report metrics

    println!("9️⃣  Testing Performance Benchmarks...");

    // Benchmark individual updates
    let iterations = 100;
    let bench_start = Instant::now();

    for i in 0..iterations {
        let display_index = (i as u8) % display.chain_length();
        let text = format!("B{:03}", i % 1000);
        let _ = display.display_text_on(&text, display_index);
    }

    let bench_duration = bench_start.elapsed();
    let update_rate = iterations as f64 / bench_duration.as_secs_f64();

    details.push_str(&format!("Individual updates: {:.1} ops/sec\n", update_rate));

    // Benchmark bulk operations
    let bulk_iterations = 50;
    let bulk_start = Instant::now();

    for i in 0..bulk_iterations {
        let intensity = (i % 16) as u8;
        let _ = display.set_intensity_all(intensity);
    }

    let bulk_duration = bulk_start.elapsed();
    let bulk_rate = bulk_iterations as f64 / bulk_duration.as_secs_f64();

    details.push_str(&format!("Bulk operations: {:.1} ops/sec\n", bulk_rate));

    let duration = start.elapsed();
    results.add_result(
        "Performance Benchmarks".to_string(),
        passed,
        duration,
        details,
    );

    println!("   ✅ COMPLETED ({:.2}s)", duration.as_secs_f64());
    println!(
        "      Individual: {:.1} ops/sec, Bulk: {:.1} ops/sec",
        update_rate, bulk_rate
    );

    Ok(())
}

fn test_chain_integrity(display: &mut Max7219, results: &mut TestResults) -> Result<()> {
    let start = Instant::now();
    let mut details = String::new();
    let mut passed = true;

    println!("🔟 Testing Chain Integrity...");

    // Clear all displays
    match display.clear_all() {
        Ok(_) => details.push_str("Chain clear ✓\n"),
        Err(e) => {
            details.push_str(&format!("Chain clear failed: {} ✗\n", e));
            passed = false;
        }
    }

    // Send unique pattern to each display
    for i in 0..display.chain_length() {
        let pattern = format!("C{}", i);
        match display.display_text_on(&pattern, i) {
            Ok(_) => details.push_str(&format!("Chain pattern {} ✓\n", i)),
            Err(e) => {
                details.push_str(&format!("Chain pattern {} failed: {} ✗\n", i, e));
                passed = false;
            }
        }
    }

    std::thread::sleep(Duration::from_secs(1));

    // Test rapid sequential updates
    for _ in 0..10 {
        for i in 0..display.chain_length() {
            match display.display_text_on("FAST", i) {
                Ok(_) => {}
                Err(e) => {
                    details.push_str(&format!("Rapid update failed: {} ✗\n", e));
                    passed = false;
                    break;
                }
            }
        }
    }

    details.push_str("Rapid sequential updates ✓\n");

    let duration = start.elapsed();
    results.add_result("Chain Integrity".to_string(), passed, duration, details);

    if passed {
        println!("   ✅ PASSED ({:.2}s)", duration.as_secs_f64());
    } else {
        println!("   ❌ FAILED ({:.2}s)", duration.as_secs_f64());
    }

    Ok(())
}

fn print_test_report(results: &TestResults) {
    println!("\n📊 AUTOMATED TEST REPORT");
    println!("========================");

    for (i, test) in results.tests.iter().enumerate() {
        let status = if test.passed { "✅ PASS" } else { "❌ FAIL" };
        println!(
            "{:2}. {:<25} {} ({:.2}s)",
            i + 1,
            test.name,
            status,
            test.duration.as_secs_f64()
        );

        if !test.passed || !test.details.is_empty() {
            for line in test.details.lines() {
                println!("    {}", line);
            }
        }
    }

    println!("\n📈 SUMMARY");
    println!("==========");
    println!("Total Tests: {}", results.total_count());
    println!("Passed: {}", results.passed_count());
    println!("Failed: {}", results.total_count() - results.passed_count());
    println!(
        "Success Rate: {:.1}%",
        (results.passed_count() as f64 / results.total_count() as f64) * 100.0
    );

    let total_duration: Duration = results.tests.iter().map(|t| t.duration).sum();
    println!("Total Duration: {:.2}s", total_duration.as_secs_f64());
}
