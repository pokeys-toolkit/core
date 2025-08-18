//! Simple Flash Text Test
//!
//! This example tests the basic flash_text functionality.

use pokeys_lib::devices::spi::Max7219;
use pokeys_lib::*;

fn main() -> Result<()> {
    println!("🔥 Testing MAX7219 Flash Text");

    // Connect to device
    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    println!("✅ Connected to device");

    // Create display
    let mut display = Max7219::new(&mut device, 24)?;
    display.configure_raw_segments(8)?;
    display.set_intensity(10)?;
    println!("✅ Display configured");

    // Test basic flash
    println!("🔥 Flashing 'TEST' at 2 Hz for 3 seconds...");
    display.flash_text("TEST", 2.0, 3.0)?;

    println!("✅ Flash test completed!");
    Ok(())
}
