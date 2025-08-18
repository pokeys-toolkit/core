//! CAN protocol implementation

use crate::device::PoKeysDevice;
use crate::error::Result;
use crate::types::CanMessage;

/// CAN protocol implementation
impl PoKeysDevice {
    /// Configure CAN interface
    pub fn can_configure(&mut self, baud_rate: u32, mode: u8) -> Result<()> {
        self.send_request(
            0xE0,
            (baud_rate & 0xFF) as u8,
            ((baud_rate >> 8) & 0xFF) as u8,
            ((baud_rate >> 16) & 0xFF) as u8,
            mode,
        )?;
        Ok(())
    }

    /// Send CAN message
    pub fn can_send(&mut self, message: &CanMessage) -> Result<()> {
        self.send_request(
            0xE1,
            (message.id & 0xFF) as u8,
            ((message.id >> 8) & 0xFF) as u8,
            message.len,
            message.format,
        )?;

        // Send message data
        if message.len > 0 {
            self.send_request(
                0xE2,
                message.data[0],
                message.data[1],
                message.data[2],
                message.data[3],
            )?;

            if message.len > 4 {
                self.send_request(
                    0xE3,
                    message.data[4],
                    message.data[5],
                    message.data[6],
                    message.data[7],
                )?;
            }
        }

        Ok(())
    }

    /// Receive CAN message
    pub fn can_receive(&mut self) -> Result<Option<CanMessage>> {
        let response = self.send_request(0xE4, 0, 0, 0, 0)?;

        if response[8] == 0 {
            return Ok(None); // No message available
        }

        let mut message = CanMessage {
            id: u32::from_le_bytes([response[9], response[10], response[11], response[12]]),
            data: [0; 8],
            len: response[13],
            format: response[14],
            msg_type: response[15],
        };

        // Get message data if available
        if message.len > 0 && response.len() >= 24 {
            message.data[..message.len as usize]
                .copy_from_slice(&response[16..16 + message.len as usize]);
        }

        Ok(Some(message))
    }

    /// Set CAN filter
    pub fn can_set_filter(&mut self, filter_id: u32, mask: u32) -> Result<()> {
        self.send_request(
            0xE5,
            (filter_id & 0xFF) as u8,
            ((filter_id >> 8) & 0xFF) as u8,
            (mask & 0xFF) as u8,
            ((mask >> 8) & 0xFF) as u8,
        )?;
        Ok(())
    }
}
