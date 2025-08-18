//! Multi-Display Manager
//!
//! Manages multiple MAX7219 displays using individual CS pins
//! instead of daisy chaining (due to PoKeys 2-byte SPI limit).

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;
use std::time::Duration;

/// Multi-display manager for individual CS pin approach
struct MultiDisplayManager<'a> {
    device: &'a mut PoKeysDevice,
    displays: Vec<DisplayInfo>,
}

struct DisplayInfo {
    cs_pin: u8,
    display_id: u8,
}

impl<'a> MultiDisplayManager<'a> {
    fn new(device: &'a mut PoKeysDevice) -> Self {
        Self {
            device,
            displays: Vec::new(),
        }
    }

    fn add_display(&mut self, display_id: u8, cs_pin: u8) {
        self.displays.push(DisplayInfo { cs_pin, display_id });
        println!("📱 Added Display {display_id} on CS pin {cs_pin}");
    }

    fn initialize_all(&mut self) -> Result<()> {
        println!("🔧 Initializing all displays...");

        for display_info in &self.displays {
            println!(
                "   Initializing Display {} (CS pin {})...",
                display_info.display_id, display_info.cs_pin
            );

            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.configure_raw_segments(8)?;
            display.clear()?;

            println!("   ✅ Display {} initialized", display_info.display_id);
        }

        println!("✅ All displays initialized");
        Ok(())
    }

    fn display_text_on(&mut self, text: &str, display_id: u8) -> Result<()> {
        if let Some(display_info) = self.displays.iter().find(|d| d.display_id == display_id) {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.configure_raw_segments(8)?;
            display.display_text(text)?;
            Ok(())
        } else {
            Err(PoKeysError::Parameter(format!(
                "Display {display_id} not found"
            )))
        }
    }

    fn display_number_on(&mut self, number: u32, display_id: u8) -> Result<()> {
        if let Some(display_info) = self.displays.iter().find(|d| d.display_id == display_id) {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.configure_numeric(8)?;
            display.display_number(number)?;
            Ok(())
        } else {
            Err(PoKeysError::Parameter(format!(
                "Display {display_id} not found"
            )))
        }
    }

    #[allow(dead_code)]
    fn set_intensity_on(&mut self, intensity: u8, display_id: u8) -> Result<()> {
        if let Some(display_info) = self.displays.iter().find(|d| d.display_id == display_id) {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.set_intensity(intensity)?;
            Ok(())
        } else {
            Err(PoKeysError::Parameter(format!(
                "Display {display_id} not found"
            )))
        }
    }

    #[allow(dead_code)]
    fn set_test_mode_on(&mut self, enable: bool, display_id: u8) -> Result<()> {
        if let Some(display_info) = self.displays.iter().find(|d| d.display_id == display_id) {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.set_test_mode(enable)?;
            Ok(())
        } else {
            Err(PoKeysError::Parameter(format!(
                "Display {display_id} not found"
            )))
        }
    }

    fn clear_display(&mut self, display_id: u8) -> Result<()> {
        if let Some(display_info) = self.displays.iter().find(|d| d.display_id == display_id) {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.clear()?;
            Ok(())
        } else {
            Err(PoKeysError::Parameter(format!(
                "Display {display_id} not found"
            )))
        }
    }

    fn clear_all(&mut self) -> Result<()> {
        println!("🧹 Clearing all displays...");
        for display_info in &self.displays {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.clear()?;
        }
        Ok(())
    }

    fn set_test_mode_all(&mut self, enable: bool) -> Result<()> {
        let mode_str = if enable { "ON" } else { "OFF" };
        println!("💡 Setting test mode {mode_str} on all displays...");

        for display_info in &self.displays {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.set_test_mode(enable)?;
        }
        Ok(())
    }

    fn set_intensity_all(&mut self, intensity: u8) -> Result<()> {
        println!("💡 Setting intensity {intensity} on all displays...");

        for display_info in &self.displays {
            let mut display = Max7219::new(self.device, display_info.cs_pin)?;
            display.set_intensity(intensity)?;
        }
        Ok(())
    }

    fn display_count(&self) -> usize {
        self.displays.len()
    }
}

fn main() -> Result<()> {
    println!("Multi-Display Manager Test");
    println!("==========================");
    println!("Testing multiple MAX7219 displays with individual CS pins");
    println!();
    println!("Hardware setup:");
    println!("  Display 0: CS pin 24");
    println!("  Display 1: CS pin 26");
    println!("  Both displays: shared MOSI and CLK");
    println!("  No DOUT→DIN connection (not daisy chained)");
    println!();

    // Connect to device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected successfully");

    // Create multi-display manager
    println!("\n🔧 Creating multi-display manager...");
    let mut manager = MultiDisplayManager::new(&mut device);

    // Add displays
    manager.add_display(0, 24); // Display 0 on CS pin 24
    manager.add_display(1, 26); // Display 1 on CS pin 26

    println!(
        "✅ Manager created with {} displays",
        manager.display_count()
    );

    // Test 1: Initialize all displays
    println!("\n🧪 Test 1: Initialize all displays");
    manager.initialize_all()?;

    // Test 2: Test mode verification
    println!("\n🧪 Test 2: Test mode verification");
    manager.set_test_mode_all(true)?;
    println!("   Both displays should show all segments lit");
    println!("   Do both displays show all segments? (y/n)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let test_mode_works = input.trim().to_lowercase() == "y";

    manager.set_test_mode_all(false)?;
    std::thread::sleep(Duration::from_secs(1));

    if !test_mode_works {
        println!("   ❌ Test mode failed - check hardware connections");
        return Ok(());
    }
    println!("   ✅ Test mode works - both displays respond");

    // Test 3: Individual display control
    println!("\n🧪 Test 3: Individual display control");
    manager.display_text_on("HELLO", 0)?;
    manager.display_text_on("WORLD", 1)?;

    println!("   Display 0 should show 'HELLO'");
    println!("   Display 1 should show 'WORLD'");
    println!("   Do both displays show correct text? (y/n)");

    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let individual_control_works = input.trim().to_lowercase() == "y";

    if individual_control_works {
        println!("   ✅ Individual display control works perfectly!");
    } else {
        println!("   ❌ Individual display control has issues");
    }

    std::thread::sleep(Duration::from_secs(2));

    // Test 4: Number display
    println!("\n🧪 Test 4: Number display");
    manager.display_number_on(12345, 0)?;
    manager.display_number_on(67890, 1)?;

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

    // Test 5: Intensity control
    println!("\n🧪 Test 5: Intensity control");
    manager.display_text_on("BRIGHT", 0)?;
    manager.display_text_on("BRIGHT", 1)?;

    let intensities = [1, 5, 10, 15];
    for intensity in intensities {
        manager.set_intensity_all(intensity)?;
        println!("   Intensity: {intensity}/15");
        std::thread::sleep(Duration::from_secs(1));
    }

    println!("   Did both displays change brightness correctly? (y/n)");
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let intensity_works = input.trim().to_lowercase() == "y";

    if intensity_works {
        println!("   ✅ Intensity control works correctly");
    } else {
        println!("   ❌ Intensity control has issues");
    }

    // Test 6: Individual display operations
    println!("\n🧪 Test 6: Individual display operations");

    // Clear all first
    manager.clear_all()?;
    std::thread::sleep(Duration::from_secs(1));

    // Test individual operations
    manager.display_text_on("LEFT", 0)?;
    println!("   Display 0 should show 'LEFT', Display 1 should be blank");
    std::thread::sleep(Duration::from_secs(2));

    manager.display_text_on("RIGHT", 1)?;
    println!("   Display 0 should show 'LEFT', Display 1 should show 'RIGHT'");
    std::thread::sleep(Duration::from_secs(2));

    manager.clear_display(0)?;
    println!("   Display 0 should be blank, Display 1 should show 'RIGHT'");
    std::thread::sleep(Duration::from_secs(2));

    manager.clear_display(1)?;
    println!("   Both displays should be blank");
    std::thread::sleep(Duration::from_secs(1));

    // Final test: Alternating display
    println!("\n🧪 Test 7: Alternating display test");
    for i in 0..10 {
        manager.display_number_on(i, 0)?;
        manager.display_number_on(9 - i, 1)?;
        println!("   Display 0: {}, Display 1: {}", i, 9 - i);
        std::thread::sleep(Duration::from_millis(500));
    }

    manager.clear_all()?;

    println!("\n📋 Multi-Display Manager Test Results:");
    println!("======================================");

    if test_mode_works && individual_control_works && numbers_work && intensity_works {
        println!("✅ ALL TESTS PASSED!");
        println!("✅ Individual CS pin solution works perfectly");
        println!("✅ Both displays respond independently");
        println!("✅ No more partial segments issue");
        println!("✅ Full control over each display");
        println!();
        println!("🎉 PROBLEM SOLVED!");
        println!("   The 'partial segments on Display 1' issue is completely resolved");
        println!("   by using individual CS pins instead of daisy chaining.");
        println!();
        println!("💡 Key insights:");
        println!("   • PoKeys has a 2-byte SPI command limit");
        println!("   • Traditional MAX7219 daisy chaining requires 4-byte commands");
        println!("   • Individual CS pins work perfectly with 2-byte commands");
        println!("   • Each display gets full, independent control");
    } else {
        println!("❌ Some tests failed:");
        if !test_mode_works {
            println!("   • Test mode failed");
        }
        if !individual_control_works {
            println!("   • Individual control failed");
        }
        if !numbers_work {
            println!("   • Number display failed");
        }
        if !intensity_works {
            println!("   • Intensity control failed");
        }
        println!("   Check hardware connections and CS pin assignments");
    }

    Ok(())
}
