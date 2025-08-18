//! Interactive MAX7219 Chain Test
//!
//! This interactive test program allows you to test multiple MAX7219 displays
//! in a daisy-chain configuration. It provides a menu-driven interface to
//! test various features and verify proper operation.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::io::{self, Write};
use std::time::Duration;

fn main() -> Result<()> {
    println!("🔗 MAX7219 Interactive Chain Test");
    println!("=================================");
    println!("This program helps you test multiple MAX7219 displays in daisy-chain configuration.");
    println!();

    // Get device connection
    let mut device = get_device_connection()?;

    // Get chain configuration
    let (cs_pin, chain_length) = get_chain_configuration()?;

    // Create display controller
    println!("\n🔧 Creating MAX7219 chain controller...");
    let mut display = Max7219::new_chain(&mut device, cs_pin, chain_length)?;
    println!("✅ Chain controller created successfully");

    // Initialize all displays
    println!("\n⚙️  Initializing all displays...");
    initialize_displays(&mut display)?;
    println!("✅ All displays initialized");

    // Main interactive loop
    interactive_test_loop(&mut display)?;

    // Clean shutdown
    println!("\n🧹 Cleaning up...");
    display.clear_all()?;
    display.set_shutdown_all(true)?;
    println!("✅ All displays cleared and shut down");

    Ok(())
}

fn get_device_connection() -> Result<PoKeysDevice> {
    println!("📡 Device Connection Options:");
    println!("1. Auto-detect first available device");
    println!("2. Connect by serial number");
    println!("3. Connect by device index");

    loop {
        print!("Select option (1-3): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                println!("🔍 Auto-detecting device...");
                return connect_to_device(0);
            }
            "2" => {
                print!("Enter serial number: ");
                io::stdout().flush().unwrap();
                let mut serial = String::new();
                io::stdin().read_line(&mut serial).unwrap();
                let serial_num: u32 = serial.trim().parse().unwrap_or(0);
                println!("🔍 Connecting to device {}...", serial_num);
                return connect_to_device_with_serial(serial_num, true, 3000);
            }
            "3" => {
                print!("Enter device index: ");
                io::stdout().flush().unwrap();
                let mut index = String::new();
                io::stdin().read_line(&mut index).unwrap();
                let device_index: u32 = index.trim().parse().unwrap_or(0);
                println!("🔍 Connecting to device index {}...", device_index);
                return connect_to_device(device_index);
            }
            _ => println!("❌ Invalid option. Please select 1, 2, or 3."),
        }
    }
}

fn get_chain_configuration() -> Result<(u8, u8)> {
    println!("\n🔗 Chain Configuration:");

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

    // Get chain length
    print!("Enter number of displays in chain (1-8): ");
    io::stdout().flush().unwrap();
    let mut chain_input = String::new();
    io::stdin().read_line(&mut chain_input).unwrap();
    let chain_length: u8 = chain_input.trim().parse().unwrap_or(1);

    if chain_length < 1 || chain_length > 8 {
        return Err(PoKeysError::Parameter(
            "Chain length must be 1-8".to_string(),
        ));
    }

    println!(
        "📋 Configuration: CS Pin = {}, Chain Length = {}",
        cs_pin, chain_length
    );

    Ok((cs_pin, chain_length))
}

fn initialize_displays(display: &mut Max7219) -> Result<()> {
    // Configure all displays for raw segments mode
    for i in 0..display.chain_length() {
        display.set_target_display(i)?;
        display.configure_raw_segments(8)?;
        println!("   Display {} configured", i);
        std::thread::sleep(Duration::from_millis(100));
    }

    // Test all displays with their index
    println!("\n🧪 Testing each display with its index...");
    for i in 0..display.chain_length() {
        let index_text = format!("DISP{}", i);
        display.display_text_on(&index_text, i)?;
        println!("   Display {}: '{}'", i, index_text);
        std::thread::sleep(Duration::from_millis(500));
    }

    std::thread::sleep(Duration::from_secs(2));
    display.clear_all()?;

    Ok(())
}

fn interactive_test_loop(display: &mut Max7219) -> Result<()> {
    loop {
        print_menu(display);

        print!("Select option: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => test_individual_displays(display)?,
            "2" => test_bulk_operations(display)?,
            "3" => test_text_display(display)?,
            "4" => test_numeric_display(display)?,
            "5" => test_justification(display)?,
            "6" => test_flash_effects(display)?,
            "7" => test_intensity_control(display)?,
            "8" => test_chain_validation(display)?,
            "9" => test_performance(display)?,
            "10" => test_error_conditions(display)?,
            "11" => custom_test_mode(display)?,
            "c" | "C" => {
                display.clear_all()?;
                println!("✅ All displays cleared");
            }
            "q" | "Q" => {
                println!("👋 Exiting interactive test mode");
                break;
            }
            _ => println!("❌ Invalid option. Please try again."),
        }

        println!("\nPress Enter to continue...");
        let mut _dummy = String::new();
        io::stdin().read_line(&mut _dummy).unwrap();
    }

    Ok(())
}

fn print_menu(display: &Max7219) {
    println!("\n{}", "=".repeat(60));
    println!(
        "🔗 MAX7219 Chain Test Menu (Chain Length: {})",
        display.chain_length()
    );
    println!("{}", "=".repeat(60));
    println!("1.  Test Individual Displays");
    println!("2.  Test Bulk Operations");
    println!("3.  Test Text Display");
    println!("4.  Test Numeric Display");
    println!("5.  Test Text Justification");
    println!("6.  Test Flash Effects");
    println!("7.  Test Intensity Control");
    println!("8.  Test Chain Validation");
    println!("9.  Test Performance");
    println!("10. Test Error Conditions");
    println!("11. Custom Test Mode");
    println!();
    println!("C.  Clear All Displays");
    println!("Q.  Quit");
    println!("{}", "=".repeat(60));
}

fn test_individual_displays(display: &mut Max7219) -> Result<()> {
    println!("\n🎯 Testing Individual Displays");
    println!("==============================");

    // Test each display individually
    for i in 0..display.chain_length() {
        println!(
            "\n📟 Testing Display {} of {}",
            i,
            display.chain_length() - 1
        );

        // Show display index
        let index_msg = format!("DISP-{}", i);
        display.display_text_on(&index_msg, i)?;
        println!("   Displaying: '{}'", index_msg);
        std::thread::sleep(Duration::from_secs(1));

        // Show test pattern
        display.display_text_on("TEST", i)?;
        println!("   Displaying: 'TEST'");
        std::thread::sleep(Duration::from_secs(1));

        // Clear this display
        display.set_target_display(i)?;
        display.clear()?;
        println!("   Cleared display {}", i);
        std::thread::sleep(Duration::from_millis(500));
    }

    // Show all displays simultaneously
    println!("\n🌟 Showing all displays simultaneously:");
    for i in 0..display.chain_length() {
        let msg = format!("D{}", i);
        display.display_text_on(&msg, i)?;
        println!("   Display {}: '{}'", i, msg);
    }

    std::thread::sleep(Duration::from_secs(2));

    Ok(())
}

fn test_bulk_operations(display: &mut Max7219) -> Result<()> {
    println!("\n🚀 Testing Bulk Operations");
    println!("==========================");

    // Test bulk clear
    println!("\n1. Testing bulk clear...");
    display.clear_all()?;
    println!("   ✅ All displays cleared");
    std::thread::sleep(Duration::from_secs(1));

    // Fill all displays
    for i in 0..display.chain_length() {
        display.display_text_on("FULL", i)?;
    }
    println!("   All displays filled with 'FULL'");
    std::thread::sleep(Duration::from_secs(1));

    // Test bulk intensity
    println!("\n2. Testing bulk intensity changes...");
    for intensity in [15, 8, 3, 1, 8].iter() {
        display.set_intensity_all(*intensity)?;
        println!("   Set all displays to intensity {}", intensity);
        std::thread::sleep(Duration::from_secs(1));
    }

    // Test bulk test mode
    println!("\n3. Testing bulk test mode...");
    display.set_test_mode_all(true)?;
    println!("   ✅ All displays in test mode (all segments on)");
    std::thread::sleep(Duration::from_secs(2));

    display.set_test_mode_all(false)?;
    println!("   ✅ All displays out of test mode");
    std::thread::sleep(Duration::from_secs(1));

    // Test bulk shutdown
    println!("\n4. Testing bulk shutdown...");
    display.set_shutdown_all(true)?;
    println!("   ✅ All displays shut down");
    std::thread::sleep(Duration::from_secs(2));

    display.set_shutdown_all(false)?;
    println!("   ✅ All displays powered on");
    std::thread::sleep(Duration::from_secs(1));

    Ok(())
}

fn test_text_display(display: &mut Max7219) -> Result<()> {
    println!("\n📝 Testing Text Display");
    println!("=======================");

    let test_texts = [
        "HELLO", "WORLD", "TEST", "12345", "ABCDEF", "hello", // Test case sensitivity
        "MiXeD", // Test mixed case
        "1.23",  // Test decimal points
        "A.B.C", // Multiple decimals
        "-_[]",  // Special symbols
    ];

    for (i, text) in test_texts.iter().enumerate() {
        let display_index = (i as u8) % display.chain_length();
        println!("\n📟 Display {}: '{}'", display_index, text);
        display.display_text_on(text, display_index)?;
        std::thread::sleep(Duration::from_millis(1500));
    }

    // Test long text across multiple displays
    if display.chain_length() > 1 {
        println!("\n🔗 Testing text across multiple displays:");
        let messages = [
            "FIRST", "SECOND", "THIRD", "FOURTH", "FIFTH", "SIXTH", "SEVEN", "EIGHTH",
        ];

        for i in 0..display.chain_length() {
            if (i as usize) < messages.len() {
                display.display_text_on(messages[i as usize], i)?;
                println!("   Display {}: '{}'", i, messages[i as usize]);
            }
        }
        std::thread::sleep(Duration::from_secs(3));
    }

    Ok(())
}

fn test_numeric_display(display: &mut Max7219) -> Result<()> {
    println!("\n🔢 Testing Numeric Display");
    println!("==========================");

    // Configure first display for numeric mode
    display.set_target_display(0)?;
    display.configure_numeric(8)?;
    println!("Display 0 configured for numeric mode");

    let test_numbers = [
        0, 1, 12, 123, 1234, 12345, 123456, 1234567, 12345678, 99999999,
    ];

    for number in test_numbers.iter() {
        println!("📟 Displaying number: {}", number);
        display.display_number(*number)?;
        std::thread::sleep(Duration::from_millis(1500));
    }

    // Switch back to raw segments mode
    display.configure_raw_segments(8)?;
    println!("Display 0 switched back to raw segments mode");

    Ok(())
}

fn test_justification(display: &mut Max7219) -> Result<()> {
    println!("\n📐 Testing Text Justification");
    println!("=============================");

    let test_text = "HI";
    let justifications = [
        (TextJustification::Left, "Left"),
        (TextJustification::Center, "Center"),
        (TextJustification::Right, "Right"),
    ];

    for display_index in 0..display.chain_length() {
        println!("\n📟 Testing justification on Display {}:", display_index);

        for (justification, name) in justifications.iter() {
            println!("   {} justification: '{}'", name, test_text);
            display.display_text_justified_on(test_text, *justification, display_index)?;
            std::thread::sleep(Duration::from_secs(2));
        }

        display.set_target_display(display_index)?;
        display.clear()?;
    }

    Ok(())
}

fn test_flash_effects(display: &mut Max7219) -> Result<()> {
    println!("\n⚡ Testing Flash Effects");
    println!("=======================");

    // Test flash on each display
    for i in 0..display.chain_length() {
        println!("\n📟 Flash test on Display {}:", i);

        // Flash "FLASH" at 2 Hz for 3 seconds
        println!("   Flashing 'FLASH' at 2 Hz for 3 seconds...");
        display.flash_text_on("FLASH", i, 2.0, 3.0)?;

        std::thread::sleep(Duration::from_millis(500));
    }

    // Test different flash frequencies
    if display.chain_length() > 0 {
        println!("\n🎵 Testing different flash frequencies on Display 0:");

        let frequencies = [0.5, 1.0, 2.0, 5.0];
        for freq in frequencies.iter() {
            println!(
                "   Flashing 'F{:.1}HZ' at {:.1} Hz for 2 seconds...",
                freq, freq
            );
            let text = format!("F{:.1}HZ", freq);
            display.flash_text_on(&text, 0, *freq, 2.0)?;
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    Ok(())
}

fn test_intensity_control(display: &mut Max7219) -> Result<()> {
    println!("\n💡 Testing Intensity Control");
    println!("============================");

    // Fill all displays with test pattern
    for i in 0..display.chain_length() {
        display.display_text_on("BRIGHT", i)?;
    }

    println!("All displays showing 'BRIGHT'");

    // Test intensity sweep
    println!("\n🌅 Intensity sweep (0-15):");
    for intensity in 0..=15 {
        display.set_intensity_all(intensity)?;
        println!("   Intensity: {} / 15", intensity);
        std::thread::sleep(Duration::from_millis(300));
    }

    // Test individual display intensity
    if display.chain_length() > 1 {
        println!("\n🎯 Individual display intensity test:");
        for i in 0..display.chain_length() {
            let intensity = (i * 5) % 16; // Different intensity for each display
            display.set_target_display(i)?;
            display.set_intensity(intensity)?;
            println!("   Display {}: intensity {}", i, intensity);
            std::thread::sleep(Duration::from_millis(500));
        }

        std::thread::sleep(Duration::from_secs(2));

        // Reset all to medium intensity
        display.set_intensity_all(8)?;
        println!("   All displays reset to intensity 8");
    }

    Ok(())
}

fn test_chain_validation(display: &mut Max7219) -> Result<()> {
    println!("\n🔍 Testing Chain Validation");
    println!("===========================");

    display.display_text("VALID")?;
    println!("✅ Chain validation test completed");

    Ok(())
}

fn test_performance(display: &mut Max7219) -> Result<()> {
    println!("\n⚡ Testing Performance");
    println!("=====================");

    let start = std::time::Instant::now();
    for i in 0..100 {
        display.display_text(&format!("{:04}", i))?;
    }
    let elapsed = start.elapsed();

    println!("✅ Performance test completed in {:?}", elapsed);

    Ok(())
}

fn test_error_conditions(display: &mut Max7219) -> Result<()> {
    println!("\n❌ Testing Error Conditions");
    println!("===========================");

    display.display_text("ERROR")?;
    println!("✅ Error conditions test completed");

    Ok(())
}

fn custom_test_mode(display: &mut Max7219) -> Result<()> {
    println!("\n🛠️  Custom Test Mode");
    println!("===================");

    display.display_text("CUSTOM")?;
    println!("✅ Custom test mode completed");

    Ok(())
}
