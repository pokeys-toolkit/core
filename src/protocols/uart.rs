//! UART protocol implementation

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};

/// UART protocol implementation
impl PoKeysDevice {
    /// Configure UART interface
    pub fn uart_configure(
        &mut self,
        baud_rate: u32,
        data_bits: u8,
        stop_bits: u8,
        parity: u8,
    ) -> Result<()> {
        self.send_request(
            0xD0,
            (baud_rate & 0xFF) as u8,
            ((baud_rate >> 8) & 0xFF) as u8,
            data_bits,
            stop_bits,
        )?;

        self.send_request(0xD1, parity, 0, 0, 0)?;
        Ok(())
    }

    /// Write data to UART
    pub fn uart_write(&mut self, data: &[u8]) -> Result<()> {
        if data.len() > 60 {
            return Err(PoKeysError::Parameter("UART data too long".to_string()));
        }

        self.send_request(0xD2, data.len() as u8, 0, 0, 0)?;
        // Implementation would send data in subsequent request
        Ok(())
    }

    /// Read data from UART
    pub fn uart_read(&mut self) -> Result<Vec<u8>> {
        let response = self.send_request(0xD3, 0, 0, 0, 0)?;

        let mut data = Vec::new();
        if response.len() > 8 {
            let data_len = response[8] as usize;
            if response.len() >= 9 + data_len {
                data.extend_from_slice(&response[9..9 + data_len]);
            }
        }

        Ok(data)
    }

    /// Check UART status
    pub fn uart_status(&mut self) -> Result<(u8, u8)> {
        let response = self.send_request(0xD4, 0, 0, 0, 0)?;
        Ok((response[8], response[9])) // (tx_buffer_free, rx_bytes_available)
    }
}
