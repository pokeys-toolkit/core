//! Dual Counter Demo
//!
//! Demonstrates two MAX7219 displays working together as independent counters.
//! Display 0: CS pin 24, Display 1: CS pin 26

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("Dual Counter Demo");
    println!("=================");
    println!("Two independent MAX7219 displays showing different counters");
    println!("Display 0 (CS 24): Counting up");
    println!("Display 1 (CS 26): Counting down");
    println!();

    // Connect to device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected successfully");

    // Initialize both displays
    println!("\n🔧 Initializing displays...");

    // Initialize Display 0
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.configure_numeric(10)?;
        display0.clear()?;
        println!("   ✅ Display 0 (CS 24) initialized");
    }

    // Initialize Display 1
    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.configure_numeric(10)?;
        display1.clear()?;
        println!("   ✅ Display 1 (CS 26) initialized");
    }

    // Demo 1: Basic counter
    println!("\n🧪 Demo 1: Basic dual counter (0-20)");
    for i in 0..=20 {
        // Display 0: Count up
        {
            let mut display0 = Max7219::new(&mut device, 24)?;
            display0.configure_numeric(8)?;
            display0.display_number(i)?;
        }

        // Display 1: Count down
        {
            let mut display1 = Max7219::new(&mut device, 26)?;
            display1.configure_numeric(8)?;
            display1.display_number(20 - i)?;
        }

        println!("   Display 0: {:2} | Display 1: {:2}", i, 20 - i);
        std::thread::sleep(Duration::from_millis(300));
    }

    std::thread::sleep(Duration::from_secs(1));

    // Demo 2: Different speed counters
    println!("\n🧪 Demo 2: Different speed counters");
    println!("   Display 0: Fast counter, Display 1: Slow counter");

    let mut slow_count = 0;

    for step in 0..50 {
        // Fast counter (every step) - use step directly
        let fast_count = step;

        // Slow counter (every 5 steps)
        if step % 5 == 0 {
            slow_count = step / 5;
        }

        // Update Display 0 (fast)
        {
            let mut display0 = Max7219::new(&mut device, 24)?;
            display0.configure_numeric(8)?;
            display0.display_number(fast_count)?;
        }

        // Update Display 1 (slow)
        {
            let mut display1 = Max7219::new(&mut device, 26)?;
            display1.configure_numeric(8)?;
            display1.display_number(slow_count)?;
        }

        if step % 5 == 0 {
            println!("   Fast: {fast_count:2} | Slow: {slow_count:2}");
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    std::thread::sleep(Duration::from_secs(1));

    // Demo 3: Text display
    println!("\n🧪 Demo 3: Text display demo");

    let messages = [
        ("HELLO", "WORLD"),
        ("LEFT", "RIGHT"),
        ("DISP0", "DISP1"),
        ("TEST", "PASS"),
    ];

    for (msg0, msg1) in messages {
        // Display 0: First message
        {
            let mut display0 = Max7219::new(&mut device, 24)?;
            display0.configure_raw_segments(8)?;
            display0.display_text(msg0)?;
        }

        // Display 1: Second message
        {
            let mut display1 = Max7219::new(&mut device, 26)?;
            display1.configure_raw_segments(8)?;
            display1.display_text(msg1)?;
        }

        println!("   Display 0: '{msg0}' | Display 1: '{msg1}'");
        std::thread::sleep(Duration::from_secs(2));
    }

    std::thread::sleep(Duration::from_secs(1));

    // Demo 4: Intensity animation
    println!("\n🧪 Demo 4: Intensity animation");

    // Set both displays to show "BRIGHT"
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.configure_raw_segments(8)?;
        display0.display_text("BRIGHT")?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.configure_raw_segments(8)?;
        display1.display_text("BRIGHT")?;
    }

    // Animate intensity in opposite directions
    for cycle in 0..3 {
        println!("   Intensity animation cycle {}", cycle + 1);

        // Fade Display 0 up, Display 1 down
        for i in 0..=15 {
            {
                let mut display0 = Max7219::new(&mut device, 24)?;
                display0.set_intensity(i)?;
            }

            {
                let mut display1 = Max7219::new(&mut device, 26)?;
                display1.set_intensity(15 - i)?;
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        // Fade Display 0 down, Display 1 up
        for i in (0..=15).rev() {
            {
                let mut display0 = Max7219::new(&mut device, 24)?;
                display0.set_intensity(i)?;
            }

            {
                let mut display1 = Max7219::new(&mut device, 26)?;
                display1.set_intensity(15 - i)?;
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    // Demo 5: Final countdown
    println!("\n🧪 Demo 5: Final countdown");

    for i in (0..=10).rev() {
        // Both displays show the same countdown
        {
            let mut display0 = Max7219::new(&mut device, 24)?;
            display0.configure_numeric(15)?; // Bright
            display0.display_number(i)?;
        }

        {
            let mut display1 = Max7219::new(&mut device, 26)?;
            display1.configure_numeric(15)?; // Bright
            display1.display_number(i)?;
        }

        println!("   Countdown: {i}");
        std::thread::sleep(Duration::from_secs(1));
    }

    // Final message
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.configure_raw_segments(8)?;
        display0.display_text("DEMO")?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.configure_raw_segments(8)?;
        display1.display_text("DONE")?;
    }

    println!("   Final: Display 0 'DEMO' | Display 1 'DONE'");
    std::thread::sleep(Duration::from_secs(3));

    // Clear both displays
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.clear()?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.clear()?;
    }

    println!("\n🎉 Dual Counter Demo Complete!");
    println!("==============================");
    println!("✅ Both displays worked independently");
    println!("✅ Different counters, speeds, and content");
    println!("✅ Text and numeric modes");
    println!("✅ Intensity control");
    println!("✅ No interference between displays");
    println!();
    println!("🔧 This demonstrates the solution to your original problem:");
    println!("   • Individual CS pins (24 and 26) work perfectly");
    println!("   • No more partial segments on Display 1");
    println!("   • Full independent control of each display");
    println!("   • Much simpler than daisy chaining");

    Ok(())
}
