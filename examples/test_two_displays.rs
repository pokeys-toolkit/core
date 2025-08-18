//! Simple Two Display Test
//!
//! Basic test for two MAX7219 displays using individual CS pins.
//! Display 0: CS pin 24, Display 1: CS pin 26

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("Simple Two Display Test");
    println!("=======================");
    println!("Display 0: CS pin 24");
    println!("Display 1: CS pin 26");
    println!();

    // Connect to device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected successfully");

    // Test 1: Display 0 only
    println!("\n🧪 Test 1: Display 0 only (CS pin 24)");
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.configure_raw_segments(8)?;
        display0.display_text("DISPLAY0")?;

        println!("   Display 0 should show 'DISPLAY0'");
        println!("   Does Display 0 show 'DISPLAY0'? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let display0_works = input.trim().to_lowercase() == "y";

        if display0_works {
            println!("   ✅ Display 0 works correctly");
        } else {
            println!("   ❌ Display 0 has issues");
            return Ok(());
        }

        display0.clear()?;
    }

    std::thread::sleep(Duration::from_secs(1));

    // Test 2: Display 1 only
    println!("\n🧪 Test 2: Display 1 only (CS pin 26)");
    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.configure_raw_segments(8)?;
        display1.display_text("DISPLAY1")?;

        println!("   Display 1 should show 'DISPLAY1'");
        println!("   Does Display 1 show 'DISPLAY1'? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let display1_works = input.trim().to_lowercase() == "y";

        if display1_works {
            println!("   ✅ Display 1 works correctly");
        } else {
            println!("   ❌ Display 1 has issues");
            return Ok(());
        }

        display1.clear()?;
    }

    std::thread::sleep(Duration::from_secs(1));

    // Test 3: Both displays with different content
    println!("\n🧪 Test 3: Both displays with different content");
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.configure_raw_segments(8)?;
        display0.display_text("LEFT")?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.configure_raw_segments(8)?;
        display1.display_text("RIGHT")?;
    }

    println!("   Display 0 should show 'LEFT'");
    println!("   Display 1 should show 'RIGHT'");
    println!("   Do both displays show correct text? (y/n)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let both_work = input.trim().to_lowercase() == "y";

    if both_work {
        println!("   ✅ Both displays work independently!");
    } else {
        println!("   ❌ Issue with independent operation");
    }

    std::thread::sleep(Duration::from_secs(2));

    // Test 4: Numbers
    println!("\n🧪 Test 4: Number display");
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.configure_numeric(8)?;
        display0.display_number(12345)?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.configure_numeric(8)?;
        display1.display_number(67890)?;
    }

    println!("   Display 0 should show '12345'");
    println!("   Display 1 should show '67890'");
    println!("   Do both displays show correct numbers? (y/n)");

    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let numbers_work = input.trim().to_lowercase() == "y";

    if numbers_work {
        println!("   ✅ Number display works correctly");
    } else {
        println!("   ❌ Number display has issues");
    }

    std::thread::sleep(Duration::from_secs(2));

    // Test 5: Test mode
    println!("\n🧪 Test 5: Test mode");
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.set_test_mode(true)?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.set_test_mode(true)?;
    }

    println!("   Both displays should show all segments lit");
    println!("   Do both displays show all segments? (y/n)");

    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let test_mode_works = input.trim().to_lowercase() == "y";

    // Turn off test mode
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.set_test_mode(false)?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.set_test_mode(false)?;
    }

    if test_mode_works {
        println!("   ✅ Test mode works on both displays");
    } else {
        println!("   ❌ Test mode has issues");
    }

    std::thread::sleep(Duration::from_secs(1));

    // Test 6: Intensity control
    println!("\n🧪 Test 6: Intensity control");
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

    let intensities = [1, 5, 10, 15];
    for intensity in intensities {
        {
            let mut display0 = Max7219::new(&mut device, 24)?;
            display0.set_intensity(intensity)?;
        }

        {
            let mut display1 = Max7219::new(&mut device, 26)?;
            display1.set_intensity(intensity)?;
        }

        println!("   Intensity: {intensity}/15");
        std::thread::sleep(Duration::from_secs(1));
    }

    println!("   Did both displays change brightness? (y/n)");
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let intensity_works = input.trim().to_lowercase() == "y";

    if intensity_works {
        println!("   ✅ Intensity control works");
    } else {
        println!("   ❌ Intensity control has issues");
    }

    // Clear both displays
    {
        let mut display0 = Max7219::new(&mut device, 24)?;
        display0.clear()?;
    }

    {
        let mut display1 = Max7219::new(&mut device, 26)?;
        display1.clear()?;
    }

    println!("\n📋 Two Display Test Results:");
    println!("============================");

    if both_work && numbers_work && test_mode_works && intensity_works {
        println!("🎉 SUCCESS! Both displays work perfectly!");
        println!("✅ Display 0 (CS pin 24) works correctly");
        println!("✅ Display 1 (CS pin 26) works correctly");
        println!("✅ Independent control of each display");
        println!("✅ No partial segments issue");
        println!("✅ Full functionality on both displays");
        println!();
        println!("🔧 The solution:");
        println!("   • Use individual CS pins instead of daisy chaining");
        println!("   • Display 0: CS pin 24");
        println!("   • Display 1: CS pin 26");
        println!("   • Create separate Max7219 instances for each display");
        println!("   • No DOUT→DIN connection needed");
    } else {
        println!("❌ Some issues detected:");
        if !both_work {
            println!("   • Independent operation failed");
        }
        if !numbers_work {
            println!("   • Number display failed");
        }
        if !test_mode_works {
            println!("   • Test mode failed");
        }
        if !intensity_works {
            println!("   • Intensity control failed");
        }
    }

    Ok(())
}
