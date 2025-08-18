//! MAX7219 Daisy Chain Example
//!
//! This example demonstrates how to use multiple MAX7219 displays
//! connected in a daisy-chain configuration. Up to 8 displays
//! can be chained together and controlled individually.

use pokeys_lib::devices::spi::{Max7219, TextJustification};
use pokeys_lib::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("MAX7219 Daisy Chain Example");
    println!("===========================");
    println!("Demonstrating control of multiple MAX7219 displays in daisy chain");
    println!();

    // Connect to PoKeys device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    // Create MAX7219 chain controller for 3 displays
    println!("\n🔧 Creating MAX7219 daisy chain controller...");
    let chain_length = 3;
    let mut display = Max7219::new_chain(&mut device, 24, chain_length)?; // CS pin 24, 3 displays
    println!(
        "✅ MAX7219 chain created with {} displays",
        display.chain_length()
    );
    println!("   Current target display: {}", display.target_display());

    // Configure all displays for raw segments mode
    println!("\n⚙️  Configuring all displays...");
    for i in 0..chain_length {
        display.set_target_display(i)?;
        display.configure_raw_segments(8)?;
        println!("   Display {i} configured for raw segments");
    }

    // Example 1: Individual display control
    println!("\n📟 Example 1: Individual Display Control");

    let messages = ["HELLO", "WORLD", "CHAIN"];
    for (i, message) in messages.iter().enumerate() {
        display.set_target_display(i as u8)?;
        display.display_text(message)?;
        println!("   Display {i}: '{message}'");
        std::thread::sleep(Duration::from_millis(500));
    }

    std::thread::sleep(Duration::from_secs(2));

    // Example 2: Convenience methods
    println!("\n🚀 Example 2: Convenience Methods");

    display.display_text_on("ONE", 0)?;
    display.display_text_on("TWO", 1)?;
    display.display_text_on("THREE", 2)?;
    println!("   Used display_text_on() for direct targeting");

    std::thread::sleep(Duration::from_secs(2));

    // Example 3: Justified text on different displays
    println!("\n📐 Example 3: Text Justification Across Chain");

    let test_text = "TEST";
    display.display_text_justified_on(test_text, TextJustification::Left, 0)?;
    display.display_text_justified_on(test_text, TextJustification::Center, 1)?;
    display.display_text_justified_on(test_text, TextJustification::Right, 2)?;

    println!("   Display 0: '{test_text}' (left-justified)");
    println!("   Display 1: '{test_text}' (center-justified)");
    println!("   Display 2: '{test_text}' (right-justified)");

    std::thread::sleep(Duration::from_secs(3));

    // Example 4: Chain-wide operations
    println!("\n🌐 Example 4: Chain-Wide Operations");

    println!("   Testing all displays (all segments on)...");
    display.set_test_mode_all(true)?;
    std::thread::sleep(Duration::from_secs(2));
    display.set_test_mode_all(false)?;

    println!("   Clearing all displays...");
    display.clear_all()?;
    std::thread::sleep(Duration::from_secs(1));

    println!("   Setting brightness on all displays...");
    display.set_intensity_all(15)?; // Maximum brightness

    // Show something to see the brightness
    for i in 0..chain_length {
        display.display_text_on("BRIGHT", i)?;
    }
    std::thread::sleep(Duration::from_secs(2));

    display.set_intensity_all(3)?; // Dim
    println!("   Dimmed all displays");
    std::thread::sleep(Duration::from_secs(2));

    // Example 5: Counter across displays
    println!("\n🔢 Example 5: Counter Across Displays");

    display.set_intensity_all(8)?; // Medium brightness

    for count in 0..=999 {
        // Split number across displays (right-to-left)
        let hundreds = count / 100;
        let tens = (count / 10) % 10;
        let ones = count % 10;

        // Display on chain (display 0 = hundreds, 1 = tens, 2 = ones)
        if hundreds > 0 {
            display.display_text_on(&hundreds.to_string(), 0)?;
        } else {
            display.display_text_on(" ", 0)?; // Blank leading zero
        }

        if count >= 10 {
            display.display_text_on(&tens.to_string(), 1)?;
        } else {
            display.display_text_on(" ", 1)?; // Blank leading zero
        }

        display.display_text_on(&ones.to_string(), 2)?;

        if count % 100 == 0 {
            println!("   Counter: {count}");
        }

        std::thread::sleep(Duration::from_millis(50));

        if count >= 50 {
            // Don't count all the way to 999 for demo
            break;
        }
    }

    // Example 6: Scrolling text
    println!("\n📜 Example 6: Scrolling Text Effect");

    let scroll_text = "HELLO WORLD FROM DAISY CHAIN";
    let scroll_chars: Vec<char> = scroll_text.chars().collect();

    for start_pos in 0..=(scroll_chars.len().saturating_sub(chain_length as usize)) {
        for display_idx in 0..chain_length {
            let char_idx = start_pos + display_idx as usize;
            if char_idx < scroll_chars.len() {
                let ch = scroll_chars[char_idx].to_string();
                display.display_text_on(&ch, display_idx)?;
            } else {
                display.display_text_on(" ", display_idx)?;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    // Example 7: Status indicators with brackets
    println!("\n📊 Example 7: Status Indicators");

    let statuses = ["[OK]", "[ERR]", "[RDY]"];
    for (i, status) in statuses.iter().enumerate() {
        display.display_text_justified_on(status, TextJustification::Center, i as u8)?;
        println!("   Display {i}: {status}");
    }

    std::thread::sleep(Duration::from_secs(3));

    // Example 8: Array indices
    println!("\n📋 Example 8: Array Index Display");

    for i in 0..chain_length {
        let index_text = format!("[{i}]");
        display.display_text_justified_on(&index_text, TextJustification::Center, i)?;
        println!("   Display {i}: {index_text}");
    }

    std::thread::sleep(Duration::from_secs(3));

    // Example 9: Power management
    println!("\n⚡ Example 9: Power Management");

    println!("   Shutting down all displays...");
    display.set_shutdown_all(true)?;
    std::thread::sleep(Duration::from_secs(2));

    println!("   Waking up all displays...");
    display.set_shutdown_all(false)?;

    // Show they're awake
    for i in 0..chain_length {
        display.display_text_on("AWAKE", i)?;
    }
    std::thread::sleep(Duration::from_secs(2));

    // Final demonstration
    println!("\n🎉 Final Demonstration");

    display.display_text_on("CHAIN", 0)?;
    display.display_text_on("DEMO", 1)?;
    display.display_text_on("DONE", 2)?;

    println!("   Final display:");
    println!("   Display 0: 'CHAIN'");
    println!("   Display 1: 'DEMO'");
    println!("   Display 2: 'DONE'");

    std::thread::sleep(Duration::from_secs(3));

    // Clear all displays
    display.clear_all()?;

    println!("\n✅ MAX7219 Daisy Chain Example Complete!");
    println!();
    println!("📋 Summary of Chain Features:");
    println!("   • Chain length: {} displays", display.chain_length());
    println!("   • Individual display targeting");
    println!("   • Convenience methods for direct control");
    println!("   • Chain-wide operations (clear, intensity, test, shutdown)");
    println!("   • Text justification on each display");
    println!("   • Scrolling text effects");
    println!("   • Status and array index displays");
    println!("   • Power management");
    println!();
    println!("💡 Use Cases:");
    println!("   • Multi-digit counters and timers");
    println!("   • Status displays across multiple zones");
    println!("   • Scrolling message displays");
    println!("   • Array or list index indicators");
    println!("   • Multi-channel monitoring systems");
    println!();
    println!("🔧 Technical Details:");
    println!("   • SPI data flows through chain: Display 0 → 1 → 2");
    println!("   • Each display receives 2 bytes per command");
    println!("   • Non-target displays receive NO-OP commands");
    println!("   • Chain length: 1-8 displays supported");
    println!("   • Backward compatible with single displays");

    Ok(())
}
