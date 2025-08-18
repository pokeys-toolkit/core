//! Minimal SPI Test
//!
//! This test replicates the exact sequence that's failing in the MAX7219 raw segments example.

use pokeys_lib::*;

fn main() -> Result<()> {
    println!("Minimal SPI Test for Device 32218");
    println!("=================================");
    println!("Replicating the exact failing sequence from max7219_raw_segments");
    println!();

    // Connect to device (same as working example)
    println!("🔍 Connecting to network device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    // Find working CS pin (same as working example)
    println!("\n🔍 Finding working CS pin...");
    let candidate_pins = [8, 24, 23, 25, 22, 26];
    let mut working_cs_pin = None;

    for &pin in &candidate_pins {
        if device
            .set_pin_function(pin, PinFunction::DigitalOutput)
            .is_ok()
            && device.set_digital_output(pin, true).is_ok()
        {
            working_cs_pin = Some(pin as u8);
            println!("✅ Found working CS pin: {pin}");
            break;
        }
    }

    let cs_pin = match working_cs_pin {
        Some(pin) => pin,
        None => {
            println!("❌ No working CS pin found!");
            return Ok(());
        }
    };

    // Now try the exact sequence that fails
    println!("\n🔧 Attempting the exact failing sequence...");

    // Step 1: Configure CS pin (this should work)
    println!("   Step 1: Configure CS pin {cs_pin} as digital output");
    device.set_pin_function(cs_pin.into(), PinFunction::DigitalOutput)?;
    println!("   ✅ CS pin configured");

    // Step 2: Set CS pin HIGH (this should work)
    println!("   Step 2: Set CS pin HIGH");
    device.set_digital_output(cs_pin.into(), true)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    println!("   ✅ CS pin set HIGH");

    // Step 3: Configure SPI (this is where it likely fails)
    println!("   Step 3: Configure SPI with prescaler 0x04, format 0x00");
    println!("   Calling: device.spi_configure(0x04, 0x00)");

    match device.spi_configure(0x04, 0x00) {
        Ok(_) => {
            println!("   ✅ SPI configuration successful!");

            // If SPI config works, try a simple SPI write
            println!("   Step 4: Test SPI write");
            let test_cmd = vec![0x0C, 0x01]; // Exit shutdown mode
            match device.spi_write(&test_cmd, cs_pin) {
                Ok(_) => {
                    println!("   ✅ SPI write successful!");
                }
                Err(e) => {
                    println!("   ❌ SPI write failed: {e}");
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!("   ❌ SPI configuration failed: {e}");
            println!("   This is the exact error from the max7219_raw_segments example!");

            // Let's try to understand what's happening
            println!("\n🔍 Debugging information:");
            println!("   Error: {e:?}");

            // Try to see if it's a specific SPI parameter issue
            println!("\n🧪 Testing different SPI parameters:");

            let test_params = [
                (0x01, 0x00, "Prescaler 1, Mode 0"),
                (0x02, 0x00, "Prescaler 2, Mode 0"),
                (0x08, 0x00, "Prescaler 8, Mode 0"),
                (0x04, 0x01, "Prescaler 4, Mode 1"),
            ];

            for (prescaler, mode, description) in &test_params {
                println!("   Testing: {description} (0x{prescaler:02X}, 0x{mode:02X})");
                match device.spi_configure(*prescaler, *mode) {
                    Ok(_) => {
                        println!("   ✅ {description} worked!");
                        break;
                    }
                    Err(e) => {
                        println!("   ❌ {description} failed: {e}");
                    }
                }
            }

            return Err(e);
        }
    }

    println!("\n🎉 Minimal SPI test completed successfully!");
    println!("If this test passes, the issue might be elsewhere in the raw segments example.");

    Ok(())
}
