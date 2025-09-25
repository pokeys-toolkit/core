//! SPI Protocol Example
//!
//! This example demonstrates the SPI functionality that matches the original C library:
//! - PK_SPIConfigure() -> spi_configure()
//! - PK_SPIWrite() -> spi_write()  
//! - PK_SPIRead() -> spi_read()

use pokeys_lib::*;

fn main() -> Result<()> {
    println!("PoKeys SPI Protocol Example");
    println!("===========================");

    // Enumerate and connect to first available device
    let device_count = enumerate_usb_devices()?;
    if device_count == 0 {
        println!("No PoKeys devices found!");
        return Ok(());
    }

    println!("Found {device_count} device(s)");

    let mut device = connect_to_device(0)?;
    println!("Connected to device: {}", device.device_data.serial_number);

    // Configure SPI
    // Parameters match C library: PK_SPIConfigure(device, prescaler, frameFormat)
    let prescaler = 0x04; // SPI clock prescaler
    let frame_format = 0x00; // SPI frame format (mode 0)

    println!("\nConfiguring SPI...");
    println!("  Prescaler: 0x{prescaler:02X}");
    println!("  Frame Format: 0x{frame_format:02X}");

    device.spi_configure(prescaler, frame_format)?;
    println!("SPI configured successfully!");

    // SPI Write Example
    // Matches C library: PK_SPIWrite(device, buffer, length, pinCS)
    let write_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let chip_select_pin = 10; // Use pin 10 as chip select

    println!("\nWriting data to SPI...");
    println!("  Data: {write_data:02X?}");
    println!("  Chip Select Pin: {chip_select_pin}");

    device.spi_write(&write_data, chip_select_pin)?;
    println!("SPI write completed successfully!");

    // SPI Read Example
    // Matches C library: PK_SPIRead(device, buffer, length)
    let read_length = 8;

    println!("\nReading data from SPI...");
    println!("  Read Length: {read_length} bytes");

    let read_data = device.spi_read(read_length)?;
    println!("  Read Data: {read_data:02X?}");
    println!("SPI read completed successfully!");

    // SPI Transfer Example (convenience method)
    println!("\nPerforming SPI transfer (write + read)...");
    let transfer_data = vec![0xAA, 0xBB, 0xCC];
    println!("  Write Data: {transfer_data:02X?}");

    let response_data = device.spi_transfer(&transfer_data, chip_select_pin)?;
    println!("  Response Data: {response_data:02X?}");
    println!("SPI transfer completed successfully!");

    // Demonstrate error handling
    println!("\nTesting error conditions...");

    // Test empty buffer (should fail)
    match device.spi_write(&[], chip_select_pin) {
        Err(e) => println!("  Empty buffer correctly rejected: {e}"),
        Ok(_) => println!("  WARNING: Empty buffer was accepted!"),
    }

    // Test oversized buffer (should fail)
    let oversized_buffer = vec![0u8; 56]; // 56 bytes > 55 byte limit
    match device.spi_write(&oversized_buffer, chip_select_pin) {
        Err(e) => println!("  Oversized buffer correctly rejected: {e}"),
        Ok(_) => println!("  WARNING: Oversized buffer was accepted!"),
    }

    // Test zero-length read (should fail)
    match device.spi_read(0) {
        Err(e) => println!("  Zero-length read correctly rejected: {e}"),
        Ok(_) => println!("  WARNING: Zero-length read was accepted!"),
    }

    println!("\nSPI example completed successfully!");
    println!("\nC Library Function Mapping:");
    println!("  PK_SPIConfigure() -> device.spi_configure()");
    println!("  PK_SPIWrite()     -> device.spi_write()");
    println!("  PK_SPIRead()      -> device.spi_read()");
    println!("  [New]             -> device.spi_transfer()");

    Ok(())
}
