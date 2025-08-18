//! MAX7219 SPI Protocol Test
//!
//! This test validates the SPI protocol implementation by testing
//! communication with a MAX7219 display controller.

// MAX7219 Register Addresses
const MAX7219_REG_SHUTDOWN: u8 = 0x0C;
const MAX7219_REG_DISPLAY_TEST: u8 = 0x0F;
const MAX7219_REG_DECODE_MODE: u8 = 0x09;
const MAX7219_REG_INTENSITY: u8 = 0x0A;
const MAX7219_REG_SCAN_LIMIT: u8 = 0x0B;
const MAX7219_REG_DIGIT0: u8 = 0x01;

// Configuration values
const MAX7219_NORMAL_MODE: u8 = 0x01;
const MAX7219_TEST_OFF: u8 = 0x00;
const MAX7219_DECODE_B_ALL: u8 = 0xFF;

#[allow(dead_code)]
const MAX7219_SCAN_LIMIT_8: u8 = 0x07;

const CS_PIN: u8 = 24;

/// Helper function to create MAX7219 commands
fn create_max7219_command(register: u8, value: u8) -> Vec<u8> {
    vec![register, value]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max7219_spi_protocol_structure() {
        // Test that MAX7219 commands have correct structure

        // MAX7219 uses 16-bit commands: [register][data]
        let shutdown_command = [MAX7219_REG_SHUTDOWN, MAX7219_NORMAL_MODE];
        assert_eq!(shutdown_command.len(), 2);
        assert_eq!(shutdown_command[0], 0x0C); // Shutdown register
        assert_eq!(shutdown_command[1], 0x01); // Normal mode

        let test_command = [MAX7219_REG_DISPLAY_TEST, MAX7219_TEST_OFF];
        assert_eq!(test_command.len(), 2);
        assert_eq!(test_command[0], 0x0F); // Display test register
        assert_eq!(test_command[1], 0x00); // Test off
    }

    #[test]
    fn test_max7219_register_addresses() {
        // Verify MAX7219 register addresses are correct
        assert_eq!(MAX7219_REG_DIGIT0, 0x01);
        assert_eq!(MAX7219_REG_DECODE_MODE, 0x09);
        assert_eq!(MAX7219_REG_INTENSITY, 0x0A);
        assert_eq!(MAX7219_REG_SCAN_LIMIT, 0x0B);
        assert_eq!(MAX7219_REG_SHUTDOWN, 0x0C);
        assert_eq!(MAX7219_REG_DISPLAY_TEST, 0x0F);
    }

    #[test]
    fn test_max7219_configuration_values() {
        // Verify configuration values are within valid ranges
        assert_eq!(MAX7219_DECODE_B_ALL, 0xFF);
    }

    #[test]
    fn test_max7219_command_generation() {
        // Test command generation for various operations

        // Shutdown command
        let cmd = create_max7219_command(MAX7219_REG_SHUTDOWN, MAX7219_NORMAL_MODE);
        assert_eq!(cmd, vec![0x0C, 0x01]);

        // Intensity command
        let cmd = create_max7219_command(MAX7219_REG_INTENSITY, 0x08);
        assert_eq!(cmd, vec![0x0A, 0x08]);

        // Digit display command
        let cmd = create_max7219_command(MAX7219_REG_DIGIT0, 5);
        assert_eq!(cmd, vec![0x01, 0x05]);
    }

    #[test]
    fn test_spi_data_length_for_max7219() {
        // MAX7219 commands are always 2 bytes
        let command = [MAX7219_REG_SHUTDOWN, MAX7219_NORMAL_MODE];

        // Verify it's within SPI limits (max 55 bytes)
        assert!(command.len() <= 55);
        assert!(command.len() == 2); // MAX7219 specific
        assert!(!command.is_empty()); // Not empty
    }

    #[test]
    fn test_chip_select_pin_validity() {
        // Test that CS pin 24 is a valid pin number
        assert_eq!(CS_PIN, 24); // Specific to our hardware setup
    }
}

/// Integration test that would run with actual hardware
#[cfg(feature = "hardware-tests")]
#[test]
fn test_max7219_spi_communication() -> Result<()> {
    // This test requires actual hardware and would be run with:
    // cargo test --features hardware_tests

    println!("🔍 Discovering network devices...");
    let network_devices = enumerate_network_devices(3000)?;
    if network_devices.is_empty() {
        println!("Skipping hardware test - no network devices found");
        return Ok(());
    }

    // Find device with serial 32218
    let target_device = network_devices
        .iter()
        .find(|dev| dev.serial_number == 32218);
    if target_device.is_none() {
        println!("Skipping hardware test - device 32218 not found");
        return Ok(());
    }

    let mut device = connect_to_device_with_serial(32218, true, 3000)?;

    // Configure SPI for MAX7219
    device.spi_configure(0x04, 0x00)?;

    // Test basic MAX7219 communication
    let shutdown_cmd = create_max7219_command(MAX7219_REG_SHUTDOWN, MAX7219_NORMAL_MODE);
    device.spi_write(&shutdown_cmd, CS_PIN)?;

    let test_cmd = create_max7219_command(MAX7219_REG_DISPLAY_TEST, MAX7219_TEST_OFF);
    device.spi_write(&test_cmd, CS_PIN)?;

    let decode_cmd = create_max7219_command(MAX7219_REG_DECODE_MODE, MAX7219_DECODE_B_ALL);
    device.spi_write(&decode_cmd, CS_PIN)?;

    let scan_cmd = create_max7219_command(MAX7219_REG_SCAN_LIMIT, MAX7219_SCAN_LIMIT_8);
    device.spi_write(&scan_cmd, CS_PIN)?;

    let intensity_cmd = create_max7219_command(MAX7219_REG_INTENSITY, 0x08);
    device.spi_write(&intensity_cmd, CS_PIN)?;

    // Display test pattern
    for digit in 0..8 {
        let digit_cmd = create_max7219_command(MAX7219_REG_DIGIT0 + digit, digit + 1);
        device.spi_write(&digit_cmd, CS_PIN)?;
    }

    println!("MAX7219 configuration completed successfully!");
    Ok(())
}

/// Benchmark test for SPI communication speed
#[cfg(feature = "hardware-tests")]
#[test]
fn benchmark_max7219_spi_speed() -> Result<()> {
    use std::time::Instant;

    println!("🔍 Discovering network devices...");
    let network_devices = enumerate_network_devices(3000)?;
    if network_devices.is_empty() {
        println!("Skipping benchmark test - no network devices found");
        return Ok(());
    }

    // Find device with serial 32218
    let target_device = network_devices
        .iter()
        .find(|dev| dev.serial_number == 32218);
    if target_device.is_none() {
        println!("Skipping benchmark test - device 32218 not found");
        return Ok(());
    }

    let mut device = connect_to_device_with_serial(32218, true, 3000)?;
    device.spi_configure(0x04, 0x00)?;

    let command = create_max7219_command(MAX7219_REG_DIGIT0, 1);

    // Benchmark 1000 SPI writes
    let start = Instant::now();
    for _ in 0..1000 {
        device.spi_write(&command, CS_PIN)?;
    }
    let duration = start.elapsed();

    println!("1000 SPI writes took: {duration:?}");
    println!("Average per write: {:?}", duration / 1000);

    // Should be fast enough for real-time display updates
    assert!(duration.as_millis() < 5000); // Less than 5 seconds for 1000 writes

    Ok(())
}
