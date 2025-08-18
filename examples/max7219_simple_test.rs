//! Simple MAX7219 Test
//!
//! This test verifies that the SPI fix works by displaying a simple pattern.

use pokeys_lib::*;

fn main() -> Result<()> {
    println!("Simple MAX7219 Test - Verifying SPI Fix");
    println!("======================================");
    println!("This should display '12345678' on the MAX7219");
    println!();

    // Connect to device
    println!("🔍 Connecting to device 32218...");
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device 32218");

    let cs_pin = 24u8;

    // Configure SPI
    println!("\n🔧 Configuring SPI...");
    device.set_pin_function(cs_pin.into(), PinFunction::DigitalOutput)?;
    device.set_digital_output(cs_pin.into(), true)?;
    device.spi_configure(0x04, 0x00)?;
    println!("✅ SPI configured");

    // Initialize MAX7219
    println!("\n🔧 Initializing MAX7219...");
    let init_commands = [
        (0x0C, 0x01, "Exit shutdown mode"),
        (0x0F, 0x00, "Disable display test"),
        (0x09, 0xFF, "Code B decode for all digits"),
        (0x0B, 0x07, "Set scan limit (8 digits)"),
        (0x0A, 0x08, "Set intensity (medium)"),
    ];

    for (register, value, description) in &init_commands {
        println!("   {description}");
        device.spi_write(&[*register, *value], cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Display digits 1-8
    println!("\n📟 Displaying '12345678'...");
    for digit in 1..=8 {
        let digit_register = digit;
        let digit_value = digit; // In Code B mode, 1-8 display as digits 1-8
        println!("   Digit {digit} = {digit_value}");
        device.spi_write(&[digit_register, digit_value], cs_pin)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("\n✅ Test completed!");
    println!("💡 Check your MAX7219 display:");
    println!("   • Should show: 12345678");
    println!("   • If blank: SPI communication still broken");
    println!("   • If showing digits: SPI fix successful!");

    println!("\n⏳ Display will remain on for 10 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Clear display
    println!("\n🧹 Clearing display...");
    for digit in 1..=8 {
        device.spi_write(&[digit, 0x0F], cs_pin)?; // 0x0F = blank in Code B mode
    }

    println!("✅ Display cleared");

    Ok(())
}
