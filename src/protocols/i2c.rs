//! I2C protocol implementation
//!
//! This module provides I2C communication functionality for PoKeys devices.
//! The implementation follows the PoKeys protocol specification for I2C operations.

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use crate::types::I2cStatus;

/// I2C protocol implementation
impl PoKeysDevice {
    /// Initialize I2C bus with default settings
    ///
    /// This initializes the I2C bus. Note: I2C bus is always activated on PoKeys devices.
    pub fn i2c_init(&mut self) -> Result<()> {
        // I2C bus is always activated on PoKeys devices, so just return success
        // We can optionally check the activation status
        let response = self.send_request(0xDB, 0x02, 0, 0, 0)?;
        
        // Check if I2C is activated (should always be successful)
        if response.len() > 3 && response[3] == 1 {
            Ok(())
        } else {
            Err(PoKeysError::Protocol("I2C bus not available".to_string()))
        }
    }

    /// Configure I2C bus with specific settings
    ///
    /// # Arguments
    /// * `speed_khz` - I2C bus speed in kHz (typically 100 or 400)
    /// * `options` - Additional I2C configuration options
    pub fn i2c_configure(&mut self, _speed_khz: u16, _options: u8) -> Result<()> {
        // I2C configuration is handled automatically by the device
        // Just ensure I2C is available
        self.i2c_init()
    }

    /// Write data to I2C device
    ///
    /// # Arguments
    /// * `address` - 7-bit I2C device address
    /// * `data` - Data buffer to write (maximum 32 bytes)
    ///
    /// # Returns
    /// I2C operation status
    pub fn i2c_write(&mut self, address: u8, data: &[u8]) -> Result<I2cStatus> {
        if data.is_empty() {
            return Err(PoKeysError::Parameter("I2C data cannot be empty".to_string()));
        }

        if data.len() > 32 {
            return Err(PoKeysError::Parameter("I2C data too long (maximum 32 bytes)".to_string()));
        }

        // Start I2C write operation
        // Command 0xDB, operation 0x10 - Write to I2C - start
        let response = self.send_request_with_data(
            0xDB,               // Command
            0x10,               // Operation: Write to I2C - start
            address,            // I2C device address
            data.len() as u8,   // Length of data packet
            0,                  // Number of bytes to read after write (0 for write-only)
            data,               // Data payload (bytes 9-40)
        )?;

        // Check initial response
        let initial_status = self.parse_i2c_status(&response)?;
        
        // If operation is in progress, get the result
        if initial_status == I2cStatus::InProgress {
            // Wait a bit for the operation to complete
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            // Get the result with operation 0x11 - Write to I2C - get result
            let result_response = self.send_request(0xDB, 0x11, 0, 0, 0)?;
            self.parse_i2c_status(&result_response)
        } else {
            Ok(initial_status)
        }
    }

    /// Read data from I2C device
    ///
    /// # Arguments
    /// * `address` - 7-bit I2C device address
    /// * `length` - Number of bytes to read (maximum 32 bytes)
    ///
    /// # Returns
    /// Tuple of (status, data) where data contains the read bytes
    pub fn i2c_read(&mut self, address: u8, length: u8) -> Result<(I2cStatus, Vec<u8>)> {
        if length == 0 {
            return Err(PoKeysError::Parameter("I2C read length cannot be zero".to_string()));
        }

        if length > 32 {
            return Err(PoKeysError::Parameter("I2C read length too long (maximum 32 bytes)".to_string()));
        }

        // Start I2C read operation
        // Command 0xDB, operation 0x20 - Read from I2C - start
        let response = self.send_request(
            0xDB,               // Command
            0x20,               // Operation: Read from I2C - start
            address,            // I2C device address
            length,             // Length of data packet to read
            0,                  // Reserved
        )?;

        let initial_status = self.parse_i2c_status(&response)?;
        
        // If operation is in progress, get the result
        if initial_status == I2cStatus::InProgress {
            // Wait a bit for the operation to complete
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            // Get the result with operation 0x21 - Read from I2C - get result
            let result_response = self.send_request(0xDB, 0x21, 0, 0, 0)?;
            let status = self.parse_i2c_status(&result_response)?;
            
            let mut data = Vec::new();
            if status == I2cStatus::Ok && result_response.len() > 10 {
                // Byte 10: data length, Bytes 11-42: data bytes
                let data_length = result_response[9] as usize; // Byte 10 (0-indexed as 9)
                if result_response.len() >= 10 + data_length {
                    data.extend_from_slice(&result_response[10..10 + data_length]);
                }
            }
            
            Ok((status, data))
        } else {
            Ok((initial_status, Vec::new()))
        }
    }

    /// Write to I2C device register
    ///
    /// This is a convenience method for writing to a specific register in an I2C device.
    ///
    /// # Arguments
    /// * `address` - 7-bit I2C device address
    /// * `register` - Register address
    /// * `data` - Data to write to the register
    pub fn i2c_write_register(&mut self, address: u8, register: u8, data: &[u8]) -> Result<I2cStatus> {
        if data.len() > 31 {
            return Err(PoKeysError::Parameter("I2C register data too long (maximum 31 bytes)".to_string()));
        }

        let mut write_data = Vec::with_capacity(1 + data.len());
        write_data.push(register);
        write_data.extend_from_slice(data);

        self.i2c_write(address, &write_data)
    }

    /// Read from I2C device register
    ///
    /// This is a convenience method for reading from a specific register in an I2C device.
    ///
    /// # Arguments
    /// * `address` - 7-bit I2C device address
    /// * `register` - Register address
    /// * `length` - Number of bytes to read
    pub fn i2c_read_register(&mut self, address: u8, register: u8, length: u8) -> Result<(I2cStatus, Vec<u8>)> {
        // First write the register address
        let status = self.i2c_write(address, &[register])?;
        if status != I2cStatus::Ok {
            return Ok((status, Vec::new()));
        }

        // Small delay between write and read
        std::thread::sleep(std::time::Duration::from_millis(1));

        // Then read the data
        self.i2c_read(address, length)
    }

    /// Scan I2C bus for devices
    ///
    /// This method scans the I2C bus for responding devices.
    ///
    /// # Returns
    /// Vector of addresses that responded to the scan
    pub fn i2c_scan(&mut self) -> Result<Vec<u8>> {
        // Start I2C scan operation
        // Command 0xDB, operation 0x30 - Scan I2C - start
        let response = self.send_request(0xDB, 0x30, 0, 0, 0)?;
        
        let initial_status = self.parse_i2c_status(&response)?;
        
        // If operation is in progress, get the result
        if initial_status == I2cStatus::InProgress {
            // Wait for scan to complete
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // Get the result with operation 0x31 - Scan I2C - get result
            let result_response = self.send_request(0xDB, 0x31, 0, 0, 0)?;
            let status = self.parse_i2c_status(&result_response)?;
            
            let mut found_devices = Vec::new();
            if status == I2cStatus::Ok && result_response.len() >= 25 {
                // Bytes 10-25: bit encoded result (16 bytes = 128 bits for addresses 0x00-0x7F)
                for byte_idx in 0..16 {
                    if result_response.len() > 9 + byte_idx {
                        let byte_val = result_response[9 + byte_idx];
                        for bit_idx in 0..8 {
                            if (byte_val & (1 << bit_idx)) != 0 {
                                let address = (byte_idx * 8 + bit_idx) as u8;
                                // Only include valid 7-bit I2C addresses (0x08-0x77)
                                if (0x08..=0x77).contains(&address) {
                                    found_devices.push(address);
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(found_devices)
        } else {
            Ok(Vec::new())
        }
    }

    /// Parse I2C status from response
    fn parse_i2c_status(&self, response: &[u8]) -> Result<I2cStatus> {
        if response.len() < 4 {
            return Err(PoKeysError::Protocol("Invalid I2C response length".to_string()));
        }

        let status = match response[3] {
            0 => I2cStatus::Error,
            1 => I2cStatus::Ok,
            0x10 => I2cStatus::InProgress,
            _ => I2cStatus::Error,
        };

        Ok(status)
    }

    /// Check I2C bus status
    ///
    /// This method checks the current status of the I2C bus.
    pub fn i2c_get_status(&mut self) -> Result<I2cStatus> {
        // Check I2C activation status
        let response = self.send_request(0xDB, 0x02, 0, 0, 0)?;
        self.parse_i2c_status(&response)
    }
}
