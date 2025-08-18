//! SPI Regression Test
//!
//! This test helps identify what changed in the SPI implementation
//! that broke the MAX7219 display functionality.

use pokeys_lib::*;

fn main() -> Result<()> {
    println!("SPI Regression Test for MAX7219");
    println!("===============================");
    println!("Testing different SPI approaches to identify the regression");
    println!();

    // Connect to device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    let cs_pin = 24u8;

    // Configure CS pin and SPI
    println!("\n🔧 Setting up SPI...");
    device.set_pin_function(cs_pin.into(), PinFunction::DigitalOutput)?;
    device.set_digital_output(cs_pin.into(), true)?;
    device.spi_configure(0x04, 0x00)?;
    println!("✅ SPI setup complete");

    // Test 1: Current implementation (likely broken)
    println!("\n🧪 Test 1: Current SPI implementation");
    println!("   Sending MAX7219 exit shutdown command (0x0C, 0x01)");

    match device.spi_write(&[0x0C, 0x01], cs_pin) {
        Ok(_) => {
            println!("   ✅ SPI write completed without error");
            println!("   (But display might not show anything - this is the bug)");
        }
        Err(e) => {
            println!("   ❌ SPI write failed: {e}");
        }
    }

    // Test 2: Try without manual CS control
    println!("\n🧪 Test 2: SPI without manual CS control");
    println!("   Testing if manual CS control is the issue");

    // Try using send_request_with_data directly without CS manipulation
    match device.send_request_with_data(0xB1, 2, 0, 0, 0, &[0x0C, 0x01]) {
        Ok(_) => {
            println!("   ✅ Direct SPI command completed");
        }
        Err(e) => {
            println!("   ❌ Direct SPI command failed: {e}");
        }
    }

    // Test 3: Try different SPI command codes
    println!("\n🧪 Test 3: Testing different SPI command codes");

    let test_commands = [
        (0xB0, "SPI configure command"),
        (0xB1, "SPI write command"),
        (0xB2, "SPI read command"),
    ];

    for (cmd, desc) in &test_commands {
        println!("   Testing {desc}: 0x{cmd:02X}");
        match device.send_request(*cmd, 2, 0, 0, 0) {
            Ok(_) => println!("   ✅ Command 0x{cmd:02X} accepted"),
            Err(e) => println!("   ❌ Command 0x{cmd:02X} failed: {e}"),
        }
    }

    // Test 4: Try a complete MAX7219 initialization sequence
    println!("\n🧪 Test 4: Complete MAX7219 initialization");
    println!("   Attempting to make display show '8' on all digits");

    let init_sequence = [
        (0x0C, 0x01, "Exit shutdown"),
        (0x0F, 0x00, "Disable test mode"),
        (0x09, 0xFF, "Code B decode all digits"),
        (0x0B, 0x07, "Scan limit 8 digits"),
        (0x0A, 0x08, "Intensity medium"),
        (0x01, 0x08, "Digit 0 = 8"),
        (0x02, 0x08, "Digit 1 = 8"),
        (0x03, 0x08, "Digit 2 = 8"),
        (0x04, 0x08, "Digit 3 = 8"),
        (0x05, 0x08, "Digit 4 = 8"),
        (0x06, 0x08, "Digit 5 = 8"),
        (0x07, 0x08, "Digit 6 = 8"),
        (0x08, 0x08, "Digit 7 = 8"),
    ];

    for (reg, val, desc) in &init_sequence {
        println!("   {desc}: 0x{reg:02X} -> 0x{val:02X}");
        match device.spi_write(&[*reg, *val], cs_pin) {
            Ok(_) => println!("   ✅ Command successful"),
            Err(e) => {
                println!("   ❌ Command failed: {e}");
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("\n📋 Test Results Summary:");
    println!("   • If display shows nothing: SPI communication is broken");
    println!("   • If display shows '88888888': SPI is working");
    println!("   • Check display for any signs of life");

    println!("\n💡 Next Steps:");
    println!("   1. Check if display shows '88888888'");
    println!("   2. If not, the SPI write implementation needs fixing");
    println!("   3. The issue is likely in manual CS control or data format");

    println!("\n⏳ Waiting 5 seconds for display observation...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Clear display
    println!("\n🧹 Clearing display...");
    for digit in 1..=8 {
        device.spi_write(&[digit, 0x0F], cs_pin)?; // Blank all digits
    }

    Ok(())
}
