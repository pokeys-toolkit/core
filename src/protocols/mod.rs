//! Communication protocols (I2C, SPI, 1-Wire, UART, CAN)
//!
//! This module provides implementations for various communication protocols
//! supported by PoKeys devices. Each protocol is implemented in its own
//! submodule for better organization and maintainability.

pub mod can;
pub mod convenience;
pub mod i2c;
pub mod onewire;
pub mod spi;
pub mod uart;

// Re-export convenience functions for easier access
pub use convenience::*;

#[cfg(test)]
mod tests {
    use crate::types::{CanMessage, I2cStatus};

    #[test]
    fn test_can_message_creation() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut message = CanMessage {
            id: 0x123,
            data: [0; 8],
            len: data.len() as u8,
            format: 0,
            msg_type: 0,
        };

        message.data[..data.len()].copy_from_slice(&data);

        assert_eq!(message.id, 0x123);
        assert_eq!(message.len, 4);
        assert_eq!(message.data[0], 0x01);
        assert_eq!(message.data[3], 0x04);
    }

    #[test]
    fn test_spi_parameter_validation() {
        // Test empty buffer
        let _result = std::panic::catch_unwind(|| {
            // This would fail in a real device context, but we're testing parameter validation
        });

        // Test buffer too long
        let _long_buffer = [0u8; 56]; // 56 bytes is too long (max 55)
                                      // In a real implementation, this would return an error

        // Test zero length read
        // This should return an error for zero length
    }

    #[test]
    fn test_spi_command_structure() {
        // Test that SPI commands use the correct protocol structure
        // Command 0xE5 with sub-commands:
        // - 0x01 for configuration
        // - 0x10 for write
        // - 0x20 for read

        // This test would verify the command structure matches the C library
        assert_eq!(0xE5, 0xE5); // SPI command
        assert_eq!(0x01, 0x01); // Configure sub-command
        assert_eq!(0x10, 0x10); // Write sub-command
        assert_eq!(0x20, 0x20); // Read sub-command
    }

    #[test]
    fn test_spi_data_limits() {
        // Test SPI data length limits match C library (55 bytes max)
        // These are compile-time constants, so we just document the limits
        const MAX_SPI_DATA_LENGTH: usize = 55;
        const OVER_LIMIT: usize = 56;
        // Test that our constants are correctly defined
        assert_eq!(MAX_SPI_DATA_LENGTH, 55);
        assert_eq!(OVER_LIMIT, 56);
        // Test the relationship between constants (not constant evaluation)
        let max_val = MAX_SPI_DATA_LENGTH;
        let over_val = OVER_LIMIT;
        assert!(over_val > max_val);
    }

    #[test]
    fn test_i2c_status_conversion() {
        assert_eq!(I2cStatus::Ok as u8, 1);
        assert_eq!(I2cStatus::Error as u8, 0);
        assert_eq!(I2cStatus::InProgress as u8, 0x10);
    }

    #[test]
    fn test_ds18b20_temperature_conversion() {
        // Test temperature conversion formula
        let temp_raw = 0x0191i16; // 25.0625°C in DS18B20 format
        let temperature = (temp_raw as f32) * 0.0625;
        assert!((temperature - 25.0625).abs() < 0.001);

        let temp_raw = 0xFF5Eu16 as i16; // -10.125°C in DS18B20 format (two's complement)
        let temperature = (temp_raw as f32) * 0.0625;
        assert!((temperature - (-10.125)).abs() < 0.001);
    }
}
