//! Convenience functions for common protocol operations

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use crate::types::{CanMessage, I2cStatus};

/// Simple I2C write operation
pub fn i2c_write_simple(device: &mut PoKeysDevice, address: u8, data: &[u8]) -> Result<()> {
    let status = device.i2c_write(address, data)?;
    match status {
        I2cStatus::Ok | I2cStatus::Complete => Ok(()),
        I2cStatus::Error => Err(PoKeysError::Protocol("I2C write failed".to_string())),
        I2cStatus::InProgress => Err(PoKeysError::Protocol(
            "I2C operation in progress".to_string(),
        )),
        I2cStatus::Timeout => Err(PoKeysError::Protocol("I2C write timeout".to_string())),
        I2cStatus::ChecksumError => Err(PoKeysError::Protocol("I2C checksum error".to_string())),
        I2cStatus::DeviceNotFound => Err(PoKeysError::Protocol("I2C device not found".to_string())),
        I2cStatus::PacketTooLarge => Err(PoKeysError::Protocol("I2C packet too large".to_string())),
    }
}

/// Simple I2C read operation
pub fn i2c_read_simple(device: &mut PoKeysDevice, address: u8, length: u8) -> Result<Vec<u8>> {
    let (status, data) = device.i2c_read(address, length)?;
    match status {
        I2cStatus::Ok | I2cStatus::Complete => Ok(data),
        I2cStatus::Error => Err(PoKeysError::Protocol("I2C read failed".to_string())),
        I2cStatus::InProgress => Err(PoKeysError::Protocol(
            "I2C operation in progress".to_string(),
        )),
        I2cStatus::Timeout => Err(PoKeysError::Protocol("I2C read timeout".to_string())),
        I2cStatus::ChecksumError => Err(PoKeysError::Protocol("I2C checksum error".to_string())),
        I2cStatus::DeviceNotFound => Err(PoKeysError::Protocol("I2C device not found".to_string())),
        I2cStatus::PacketTooLarge => Err(PoKeysError::Protocol("I2C packet too large".to_string())),
    }
}

/// Simple SPI write operation (convenience function matching C library usage)
pub fn spi_write_simple(device: &mut PoKeysDevice, buffer: &[u8], pin_cs: u8) -> Result<()> {
    device.spi_write(buffer, pin_cs)
}

/// Simple SPI read operation (convenience function matching C library usage)
pub fn spi_read_simple(device: &mut PoKeysDevice, length: u8) -> Result<Vec<u8>> {
    device.spi_read(length)
}

/// Configure SPI with common settings (convenience function)
pub fn spi_configure_simple(
    device: &mut PoKeysDevice,
    prescaler: u8,
    frame_format: u8,
) -> Result<()> {
    device.spi_configure(prescaler, frame_format)
}

/// Read DS18B20 temperature sensor via 1-Wire
pub fn onewire_read_ds18b20_temperature(
    device: &mut PoKeysDevice,
    device_id: &[u8; 8],
) -> Result<f32> {
    // Reset bus
    if !device.onewire_reset()? {
        return Err(PoKeysError::Protocol("No 1-Wire device found".to_string()));
    }

    // Select device
    device.onewire_write_byte(0x55)?; // MATCH ROM command
    for &byte in device_id {
        device.onewire_write_byte(byte)?;
    }

    // Start temperature conversion
    device.onewire_write_byte(0x44)?; // CONVERT T command

    // Wait for conversion (750ms typical)
    std::thread::sleep(std::time::Duration::from_millis(750));

    // Reset and select device again
    device.onewire_reset()?;
    device.onewire_write_byte(0x55)?; // MATCH ROM command
    for &byte in device_id {
        device.onewire_write_byte(byte)?;
    }

    // Read scratchpad
    device.onewire_write_byte(0xBE)?; // READ SCRATCHPAD command

    let temp_lsb = device.onewire_read_byte()?;
    let temp_msb = device.onewire_read_byte()?;

    // Convert to temperature
    let temp_raw = ((temp_msb as i16) << 8) | (temp_lsb as i16);
    let temperature = (temp_raw as f32) * 0.0625; // DS18B20 resolution

    Ok(temperature)
}

/// Send CAN message with standard ID
pub fn can_send_standard(device: &mut PoKeysDevice, id: u16, data: &[u8]) -> Result<()> {
    if data.len() > 8 {
        return Err(PoKeysError::Parameter("CAN data too long".to_string()));
    }

    let mut message = CanMessage {
        id: id as u32,
        data: [0; 8],
        len: data.len() as u8,
        format: 0,   // Standard frame
        msg_type: 0, // Data frame
    };

    message.data[..data.len()].copy_from_slice(data);
    device.can_send(&message)
}
