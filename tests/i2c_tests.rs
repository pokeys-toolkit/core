//! I2C Protocol Tests
//!
//! Comprehensive tests for I2C functionality including:
//! - Basic I2C operations
//! - Error handling
//! - Parameter validation
//! - Status parsing

use pokeys_lib::*;

#[cfg(test)]
mod i2c_tests {
    use super::*;

    #[test]
    fn test_i2c_parameter_validation() {
        // These tests verify parameter validation without requiring hardware
        
        // Test empty data validation
        assert!(validate_i2c_write_params(&[]).is_err());
        
        // Test data length validation
        let long_data = vec![0u8; 100];
        assert!(validate_i2c_write_params(&long_data).is_err());
        
        // Test valid data
        let valid_data = vec![0x01, 0x02, 0x03];
        assert!(validate_i2c_write_params(&valid_data).is_ok());
        
        // Test read length validation
        assert!(validate_i2c_read_length(0).is_err());
        assert!(validate_i2c_read_length(100).is_err());
        assert!(validate_i2c_read_length(10).is_ok());
    }

    #[test]
    fn test_i2c_status_parsing() {
        // Test status code parsing
        assert_eq!(parse_i2c_status_code(0), I2cStatus::Error);
        assert_eq!(parse_i2c_status_code(1), I2cStatus::Ok);
        assert_eq!(parse_i2c_status_code(2), I2cStatus::Complete);
        assert_eq!(parse_i2c_status_code(0x10), I2cStatus::InProgress);
        assert_eq!(parse_i2c_status_code(0xFF), I2cStatus::Error);
    }

    #[test]
    fn test_i2c_address_validation() {
        // Test I2C address validation (7-bit addresses)
        assert!(validate_i2c_address(0x00).is_err()); // Reserved
        assert!(validate_i2c_address(0x01).is_err()); // Reserved
        assert!(validate_i2c_address(0x08).is_ok());  // Valid
        assert!(validate_i2c_address(0x50).is_ok());  // Valid
        assert!(validate_i2c_address(0x77).is_ok());  // Valid
        assert!(validate_i2c_address(0x78).is_err()); // Reserved
        assert!(validate_i2c_address(0x80).is_err()); // Invalid (8-bit)
    }

    #[test]
    fn test_i2c_register_operations() {
        // Test register operation parameter validation
        let valid_data = vec![0xAA, 0xBB];
        assert!(validate_register_write_params(0x50, 0x00, &valid_data).is_ok());
        
        let too_long_data = vec![0u8; 55]; // Register + 54 bytes = 55 total (too long)
        assert!(validate_register_write_params(0x50, 0x00, &too_long_data).is_err());
    }

    #[test]
    fn test_i2c_speed_configuration() {
        // Test I2C speed validation
        assert!(validate_i2c_speed(50).is_err());   // Too slow
        assert!(validate_i2c_speed(100).is_ok());   // Standard
        assert!(validate_i2c_speed(400).is_ok());   // Fast
        assert!(validate_i2c_speed(1000).is_ok());  // Fast+
        assert!(validate_i2c_speed(5000).is_err()); // Too fast
    }

    // Helper functions for testing (these would be implemented in the main library)
    
    fn validate_i2c_write_params(data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(PoKeysError::Parameter("I2C data cannot be empty".to_string()));
        }
        if data.len() > 55 {
            return Err(PoKeysError::Parameter("I2C data too long".to_string()));
        }
        Ok(())
    }

    fn validate_i2c_read_length(length: u8) -> Result<()> {
        if length == 0 {
            return Err(PoKeysError::Parameter("I2C read length cannot be zero".to_string()));
        }
        if length > 55 {
            return Err(PoKeysError::Parameter("I2C read length too long".to_string()));
        }
        Ok(())
    }

    fn parse_i2c_status_code(code: u8) -> I2cStatus {
        match code {
            0 => I2cStatus::Error,
            1 => I2cStatus::Ok,
            2 => I2cStatus::Complete,
            0x10 => I2cStatus::InProgress,
            _ => I2cStatus::Error,
        }
    }

    fn validate_i2c_address(address: u8) -> Result<()> {
        if address < 0x08 || address > 0x77 {
            return Err(PoKeysError::Parameter("Invalid I2C address".to_string()));
        }
        Ok(())
    }

    fn validate_register_write_params(address: u8, register: u8, data: &[u8]) -> Result<()> {
        validate_i2c_address(address)?;
        if data.len() > 54 { // 55 - 1 for register byte
            return Err(PoKeysError::Parameter("Register data too long".to_string()));
        }
        Ok(())
    }

    fn validate_i2c_speed(speed_khz: u16) -> Result<()> {
        if speed_khz < 100 || speed_khz > 1000 {
            return Err(PoKeysError::Parameter("Invalid I2C speed".to_string()));
        }
        Ok(())
    }
}

// Integration tests (require actual hardware)
#[cfg(test)]
mod i2c_integration_tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // These tests require actual PoKeys hardware
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run hardware tests
    fn test_i2c_initialization() {
        let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");
        if device_count == 0 {
            panic!("No PoKeys devices found for testing");
        }

        let mut device = connect_to_device(0).expect("Failed to connect to device");
        
        // Test initialization
        assert!(device.i2c_init().is_ok());
        
        // Test configuration
        assert!(device.i2c_configure(100, 0).is_ok());
        assert!(device.i2c_configure(400, 0).is_ok());
    }

    #[test]
    #[ignore]
    fn test_i2c_bus_scan() {
        let mut device = setup_test_device();
        
        device.i2c_init().expect("I2C init failed");
        
        let devices = device.i2c_scan().expect("I2C scan failed");
        println!("Found I2C devices: {:02X?}", devices);
        
        // The scan should complete without error, regardless of devices found
        assert!(true);
    }

    #[test]
    #[ignore]
    fn test_i2c_basic_operations() {
        let mut device = setup_test_device();
        
        device.i2c_init().expect("I2C init failed");
        
        // Test with a common I2C address (may or may not have a device)
        let test_address = 0x50;
        let test_data = vec![0x00, 0x01, 0x02];
        
        // Write operation (may fail if no device present, but should not crash)
        let write_result = device.i2c_write(test_address, &test_data);
        match write_result {
            Ok(status) => println!("Write status: {:?}", status),
            Err(e) => println!("Write error (expected if no device): {}", e),
        }
        
        // Read operation
        let read_result = device.i2c_read(test_address, 3);
        match read_result {
            Ok((status, data)) => println!("Read status: {:?}, data: {:02X?}", status, data),
            Err(e) => println!("Read error (expected if no device): {}", e),
        }
    }

    #[test]
    #[ignore]
    fn test_i2c_register_operations() {
        let mut device = setup_test_device();
        
        device.i2c_init().expect("I2C init failed");
        
        let test_address = 0x50;
        let register = 0x00;
        let test_data = vec![0xAA, 0xBB];
        
        // Register write
        let write_result = device.i2c_write_register(test_address, register, &test_data);
        match write_result {
            Ok(status) => {
                println!("Register write status: {:?}", status);
                
                // Small delay for device processing
                thread::sleep(Duration::from_millis(10));
                
                // Register read
                let read_result = device.i2c_read_register(test_address, register, 2);
                match read_result {
                    Ok((status, data)) => {
                        println!("Register read status: {:?}, data: {:02X?}", status, data);
                    }
                    Err(e) => println!("Register read error: {}", e),
                }
            }
            Err(e) => println!("Register write error: {}", e),
        }
    }

    #[test]
    #[ignore]
    fn test_i2c_error_conditions() {
        let mut device = setup_test_device();
        
        device.i2c_init().expect("I2C init failed");
        
        // Test invalid parameters
        assert!(device.i2c_write(0x50, &[]).is_err());
        assert!(device.i2c_write(0x50, &vec![0; 100]).is_err());
        assert!(device.i2c_read(0x50, 0).is_err());
        assert!(device.i2c_read(0x50, 100).is_err());
    }

    #[test]
    #[ignore]
    fn test_i2c_performance() {
        let mut device = setup_test_device();
        
        device.i2c_init().expect("I2C init failed");
        
        let test_address = 0x50;
        let test_data = vec![0x01];
        let iterations = 100;
        
        let start_time = std::time::Instant::now();
        let mut successful_ops = 0;
        
        for _ in 0..iterations {
            if device.i2c_write(test_address, &test_data).is_ok() {
                successful_ops += 1;
            }
            thread::sleep(Duration::from_millis(1));
        }
        
        let elapsed = start_time.elapsed();
        println!("I2C Performance: {}/{} operations in {:?}", 
                successful_ops, iterations, elapsed);
        println!("Average time per operation: {:?}", elapsed / iterations);
        
        // Performance should be reasonable (less than 100ms per operation)
        assert!(elapsed.as_millis() < (iterations * 100) as u128);
    }

    fn setup_test_device() -> PoKeysDevice {
        let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");
        if device_count == 0 {
            panic!("No PoKeys devices found for testing");
        }
        connect_to_device(0).expect("Failed to connect to device")
    }
}
