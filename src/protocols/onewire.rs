//! 1-Wire protocol implementation

use crate::device::PoKeysDevice;
use crate::error::Result;

/// 1-Wire protocol implementation
impl PoKeysDevice {
    /// Initialize 1-Wire bus
    pub fn onewire_init(&mut self) -> Result<()> {
        self.send_request(0xC0, 0, 0, 0, 0)?;
        Ok(())
    }

    /// Reset 1-Wire bus
    pub fn onewire_reset(&mut self) -> Result<bool> {
        let response = self.send_request(0xC1, 0, 0, 0, 0)?;
        Ok(response[8] != 0) // Device presence detected
    }

    /// Write byte to 1-Wire bus
    pub fn onewire_write_byte(&mut self, data: u8) -> Result<()> {
        self.send_request(0xC2, data, 0, 0, 0)?;
        Ok(())
    }

    /// Read byte from 1-Wire bus
    pub fn onewire_read_byte(&mut self) -> Result<u8> {
        let response = self.send_request(0xC3, 0, 0, 0, 0)?;
        Ok(response[8])
    }

    /// Search for 1-Wire devices
    pub fn onewire_search(&mut self) -> Result<Vec<[u8; 8]>> {
        let response = self.send_request(0xC4, 0, 0, 0, 0)?;

        let mut devices = Vec::new();
        let device_count = response[8] as usize;

        for i in 0..device_count {
            let start_idx = 9 + (i * 8);
            if start_idx + 7 < response.len() {
                let mut device_id = [0u8; 8];
                device_id.copy_from_slice(&response[start_idx..start_idx + 8]);
                devices.push(device_id);
            }
        }

        Ok(devices)
    }
}
