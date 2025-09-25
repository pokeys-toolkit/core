//! SPI protocol implementation

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};

/// SPI protocol implementation
impl PoKeysDevice {
    /// Configure SPI interface
    ///
    /// # Arguments
    /// * `prescaler` - SPI clock prescaler value (affects SPI clock speed)
    /// * `frame_format` - SPI frame format configuration
    ///
    /// This matches the C library function: PK_SPIConfigure(device, prescaler, frameFormat)
    pub fn spi_configure(&mut self, prescaler: u8, frame_format: u8) -> Result<()> {
        // Command 0xE5, sub-command 0x01 for SPI configuration
        self.send_request(0xE5, 0x01, prescaler, frame_format, 0)?;
        Ok(())
    }

    /// Write data to SPI bus
    ///
    /// # Arguments
    /// * `buffer` - Data buffer to write (maximum 55 bytes)
    /// * `pin_cs` - Chip select pin number
    ///
    /// This matches the C library function: PK_SPIWrite(device, buffer, length, pinCS)
    pub fn spi_write(&mut self, buffer: &[u8], pin_cs: u8) -> Result<()> {
        if buffer.is_empty() {
            return Err(PoKeysError::Parameter(
                "SPI buffer cannot be empty".to_string(),
            ));
        }

        if buffer.len() > 55 {
            return Err(PoKeysError::Parameter(
                "SPI data too long (maximum 55 bytes)".to_string(),
            ));
        }

        // Send request with data payload
        let response = self.send_request_with_data(
            0xE5,               // Command
            0x10,               // Sub-command for write
            buffer.len() as u8, // Data length
            pin_cs,             // Chip select pin
            0,                  // Unused parameter
            buffer,             // Data payload
        )?;

        // Check response status (byte 3 should be 1 for success)
        if response.len() > 3 && response[3] == 1 {
            Ok(())
        } else {
            Err(PoKeysError::Protocol("SPI write failed".to_string()))
        }
    }

    /// Read data from SPI bus
    ///
    /// # Arguments
    /// * `length` - Number of bytes to read (maximum 55 bytes)
    ///
    /// # Returns
    /// Vector containing the read data
    ///
    /// This matches the C library function: PK_SPIRead(device, buffer, length)
    pub fn spi_read(&mut self, length: u8) -> Result<Vec<u8>> {
        if length == 0 {
            return Err(PoKeysError::Parameter(
                "SPI read length cannot be zero".to_string(),
            ));
        }

        if length > 55 {
            return Err(PoKeysError::Parameter(
                "SPI read length too long (maximum 55 bytes)".to_string(),
            ));
        }

        // Command 0xE5, sub-command 0x20 for SPI read
        let response = self.send_request(0xE5, 0x20, length, 0, 0)?;

        // Check response status (byte 3 should be 1 for success)
        if response.len() > 3 && response[3] == 1 {
            let mut data = Vec::new();
            // Data starts at byte 8 in the response
            if response.len() >= 8 + length as usize {
                data.extend_from_slice(&response[8..8 + length as usize]);
            }
            Ok(data)
        } else {
            Err(PoKeysError::Protocol("SPI read failed".to_string()))
        }
    }

    /// SPI transfer (write and read simultaneously) - convenience method
    ///
    /// This is a higher-level method that combines write and read operations
    /// for full-duplex SPI communication.
    pub fn spi_transfer(&mut self, write_data: &[u8], pin_cs: u8) -> Result<Vec<u8>> {
        // For full-duplex operation, we would need a different protocol command
        // For now, implement as separate write and read operations
        self.spi_write(write_data, pin_cs)?;
        self.spi_read(write_data.len() as u8)
    }
}
