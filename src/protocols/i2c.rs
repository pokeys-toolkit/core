//! I2C protocol implementation
//!
//! This module provides I2C communication functionality for PoKeys devices.
//! The implementation follows the PoKeys protocol specification for I2C operations.

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use crate::types::{I2cStatus, RetryConfig};
use std::time::Duration;

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

    /// Write data to I2C device with enhanced error handling
    ///
    /// # Arguments
    /// * `address` - 7-bit I2C device address
    /// * `data` - Data buffer to write (maximum 32 bytes)
    ///
    /// # Returns
    /// I2C operation status
    pub fn i2c_write(&mut self, address: u8, data: &[u8]) -> Result<I2cStatus> {
        if data.is_empty() {
            return Err(PoKeysError::Parameter(
                "I2C data cannot be empty".to_string(),
            ));
        }

        if data.len() > 32 {
            return Err(PoKeysError::I2cPacketTooLarge {
                size: data.len(),
                max_size: 32,
                suggestion: "Use i2c_write_fragmented() for large packets or split data manually"
                    .to_string(),
            });
        }

        // Start I2C write operation
        // Command 0xDB, operation 0x10 - Write to I2C - start
        let response = self.send_request_with_data(
            0xDB,             // Command
            0x10,             // Operation: Write to I2C - start
            address,          // I2C device address
            data.len() as u8, // Length of data packet
            0,                // Number of bytes to read after write (0 for write-only)
            data,             // Data payload (bytes 9-40)
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
            return Err(PoKeysError::Parameter(
                "I2C read length cannot be zero".to_string(),
            ));
        }

        if length > 32 {
            return Err(PoKeysError::Parameter(
                "I2C read length too long (maximum 32 bytes)".to_string(),
            ));
        }

        // Start I2C read operation
        // Command 0xDB, operation 0x20 - Read from I2C - start
        let response = self.send_request(
            0xDB,    // Command
            0x20,    // Operation: Read from I2C - start
            address, // I2C device address
            length,  // Length of data packet to read
            0,       // Reserved
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
    pub fn i2c_write_register(
        &mut self,
        address: u8,
        register: u8,
        data: &[u8],
    ) -> Result<I2cStatus> {
        if data.len() > 31 {
            return Err(PoKeysError::Parameter(
                "I2C register data too long (maximum 31 bytes)".to_string(),
            ));
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
    pub fn i2c_read_register(
        &mut self,
        address: u8,
        register: u8,
        length: u8,
    ) -> Result<(I2cStatus, Vec<u8>)> {
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
            return Err(PoKeysError::Protocol(
                "Invalid I2C response length".to_string(),
            ));
        }

        let status = match response[3] {
            0 => I2cStatus::Error,
            1 => I2cStatus::Ok,
            0x10 => I2cStatus::InProgress,
            _ => I2cStatus::Error,
        };

        Ok(status)
    }

    /// Write data to I2C device with automatic packet fragmentation
    ///
    /// This method automatically fragments large I2C packets into smaller chunks
    /// that fit within the 32-byte limit.
    ///
    /// # Arguments
    /// * `address` - 7-bit I2C device address
    /// * `data` - Data buffer to write (any size)
    ///
    /// # Returns
    /// I2C operation status
    pub fn i2c_write_fragmented(&mut self, address: u8, data: &[u8]) -> Result<I2cStatus> {
        const MAX_PACKET_SIZE: usize = 32;

        if data.len() <= MAX_PACKET_SIZE {
            return self.i2c_write(address, data);
        }

        // Fragment into multiple packets with sequence numbers
        for (seq, chunk) in data.chunks(MAX_PACKET_SIZE - 2).enumerate() {
            let mut packet = vec![0xF0 | (seq as u8 & 0x0F)]; // Fragment header
            packet.extend_from_slice(chunk);

            let status = self.i2c_write(address, &packet)?;
            if status != I2cStatus::Ok {
                return Ok(status);
            }

            // Wait for acknowledgment before sending next fragment
            std::thread::sleep(Duration::from_millis(10));
        }

        // Send end-of-transmission marker
        self.i2c_write(address, &[0xFF])
    }

    /// Write data to I2C device with retry logic
    ///
    /// # Arguments
    /// * `address` - 7-bit I2C device address
    /// * `data` - Data buffer to write
    /// * `config` - Retry configuration
    ///
    /// # Returns
    /// I2C operation status
    pub fn i2c_write_with_retry(
        &mut self,
        address: u8,
        data: &[u8],
        config: &RetryConfig,
    ) -> Result<I2cStatus> {
        let mut delay = config.base_delay_ms;

        for attempt in 0..config.max_attempts {
            match self.i2c_write(address, data) {
                Ok(status) => return Ok(status),
                Err(e) if e.is_recoverable() => {
                    if attempt < config.max_attempts - 1 {
                        let actual_delay = if config.jitter {
                            delay + (fastrand::u64(0..delay / 4))
                        } else {
                            delay
                        };

                        std::thread::sleep(Duration::from_millis(actual_delay));
                        delay = std::cmp::min(
                            (delay as f64 * config.backoff_multiplier) as u64,
                            config.max_delay_ms,
                        );
                    }
                }
                Err(e) => return Err(e), // Non-recoverable error
            }
        }

        Err(PoKeysError::MaxRetriesExceeded)
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
