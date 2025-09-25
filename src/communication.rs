//! Low-level communication protocol implementation

use crate::error::{PoKeysError, Result};
use crate::types::*;
use std::time::Duration;

/// Communication protocol implementation
pub struct Protocol {
    request_id: u8,
    send_retries: u32,
    read_retries: u32,
    socket_timeout: Duration,
}

impl Default for Protocol {
    fn default() -> Self {
        Self {
            request_id: 0,
            send_retries: 3,
            read_retries: 3,
            socket_timeout: Duration::from_millis(1000),
        }
    }
}

impl Protocol {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_retries_and_timeout(
        &mut self,
        send_retries: u32,
        read_retries: u32,
        timeout: Duration,
    ) {
        self.send_retries = send_retries;
        self.read_retries = read_retries;
        self.socket_timeout = timeout;
    }

    /// Calculate checksum for protocol data
    pub fn calculate_checksum(data: &[u8]) -> u8 {
        data.iter()
            .take(CHECKSUM_LENGTH)
            .fold(0u8, |acc, &x| acc.wrapping_add(x))
    }

    /// Prepare request packet
    pub fn prepare_request(
        &mut self,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
        display: Option<bool>,
    ) -> [u8; REQUEST_BUFFER_SIZE] {
        let mut request = [0u8; REQUEST_BUFFER_SIZE];

        request[0] = REQUEST_HEADER; // 0xBB
        request[1] = request_type;
        request[2] = param1;
        request[3] = param2;
        request[4] = param3;
        request[5] = param4;
        request[6] = self.next_request_id();
        request[7] = Self::calculate_checksum(&request);

        if display.unwrap_or(false) {
            println!("request: {request:02X?}");
        }

        request
    }

    /// Validate response packet
    pub fn validate_response(&self, response: &[u8], expected_request_id: u8) -> Result<()> {
        if response.len() < 8 {
            return Err(PoKeysError::Protocol("Response too short".to_string()));
        }

        if response[0] != RESPONSE_HEADER {
            return Err(PoKeysError::Protocol("Invalid response header".to_string()));
        }

        if response[6] != expected_request_id {
            return Err(PoKeysError::Protocol("Request ID mismatch".to_string()));
        }

        let expected_checksum = Self::calculate_checksum(response);
        if response[7] != expected_checksum {
            return Err(PoKeysError::InvalidChecksum);
        }

        Ok(())
    }

    fn next_request_id(&mut self) -> u8 {
        self.request_id = self.request_id.wrapping_add(1);
        self.request_id
    }
}

/// USB HID communication interface
pub trait UsbHidInterface {
    fn write(&mut self, data: &[u8]) -> Result<usize>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize>;
    fn read_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<usize>;
}

impl<T: UsbHidInterface + ?Sized> UsbHidInterface for Box<T> {
    fn write(&mut self, data: &[u8]) -> Result<usize> {
        (**self).write(data)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        (**self).read(buffer)
    }

    fn read_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<usize> {
        (**self).read_timeout(buffer, timeout)
    }
}

/// Network communication interface
pub trait NetworkInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize>;
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize>;
    fn receive_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<usize>;
}

impl<T: NetworkInterface + ?Sized> NetworkInterface for Box<T> {
    fn send(&mut self, data: &[u8]) -> Result<usize> {
        (**self).send(data)
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize> {
        (**self).receive(buffer)
    }

    fn receive_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<usize> {
        (**self).receive_timeout(buffer, timeout)
    }
}

/// Communication manager that handles different connection types
#[allow(dead_code)]
pub struct CommunicationManager {
    protocol: Protocol,
    connection_type: DeviceConnectionType,
}

impl CommunicationManager {
    pub fn new(connection_type: DeviceConnectionType) -> Self {
        Self {
            protocol: Protocol::new(),
            connection_type,
        }
    }

    pub fn set_retries_and_timeout(
        &mut self,
        send_retries: u32,
        read_retries: u32,
        timeout: Duration,
    ) {
        self.protocol
            .set_retries_and_timeout(send_retries, read_retries, timeout);
    }

    /// Get the next request ID for manual packet construction
    pub fn get_next_request_id(&mut self) -> u8 {
        self.protocol.next_request_id()
    }

    /// Prepare a request with optional data payload
    pub fn prepare_request_with_data(
        &mut self,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
        data: Option<&[u8]>,
    ) -> [u8; REQUEST_BUFFER_SIZE] {
        let mut request =
            self.protocol
                .prepare_request(request_type, param1, param2, param3, param4, None);

        // Add data payload if provided (starting at byte 8, which is protocol byte 9)
        if let Some(payload) = data {
            let data_len = std::cmp::min(payload.len(), 56); // Max 56 bytes of data (64 - 8 header bytes)
            request[8..8 + data_len].copy_from_slice(&payload[0..data_len]);

            // Recalculate checksum after adding data
            request[7] = Protocol::calculate_checksum(&request);
        }

        request
    }

    /// Validate response packet
    pub fn validate_response(&self, response: &[u8], expected_request_id: u8) -> Result<()> {
        self.protocol
            .validate_response(response, expected_request_id)
    }

    /// Send request via USB HID interface
    pub fn send_usb_request<T: UsbHidInterface + ?Sized>(
        &mut self,
        interface: &mut T,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        let request =
            self.protocol
                .prepare_request(request_type, param1, param2, param3, param4, None);
        let request_id = request[6];

        let mut retries = 0;
        while retries < self.protocol.send_retries {
            // Prepare HID packet (add report ID byte at the beginning)
            let mut hid_packet = [0u8; 65];
            hid_packet[1..65].copy_from_slice(&request[..64]);

            // Send request
            match interface.write(&hid_packet) {
                Ok(_) => {
                    // Try to receive response
                    let mut response = [0u8; RESPONSE_BUFFER_SIZE];
                    let mut wait_count = 0;

                    while wait_count < 50 {
                        match interface.read_timeout(&mut response, Duration::from_millis(20)) {
                            Ok(bytes_read) if bytes_read > 0 => {
                                // Validate response
                                match self.protocol.validate_response(&response, request_id) {
                                    Ok(_) => return Ok(response),
                                    Err(e) => {
                                        log::warn!("Invalid response: {e}");
                                        break;
                                    }
                                }
                            }
                            Ok(_) => {
                                // No data received, continue waiting
                                wait_count += 1;
                            }
                            Err(e) => {
                                log::warn!("Read error: {e}");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Write error: {e}");
                }
            }

            retries += 1;
        }

        Err(PoKeysError::Transfer(
            "Failed to send USB request".to_string(),
        ))
    }

    /// Send request via network interface
    pub fn send_network_request<T: NetworkInterface + ?Sized>(
        &mut self,
        interface: &mut T,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        let request =
            self.protocol
                .prepare_request(request_type, param1, param2, param3, param4, None);
        let request_id = request[6];

        // println!("request: {request:02X?}");

        let mut retries = 0;
        while retries < self.protocol.send_retries {
            // Send request
            match interface.send(&request[..64]) {
                Ok(_) => {
                    // Try to receive response
                    let mut response = [0u8; RESPONSE_BUFFER_SIZE];

                    match interface.receive_timeout(&mut response, self.protocol.socket_timeout) {
                        Ok(bytes_read) if bytes_read >= 8 => {
                            // Validate response
                            match self.protocol.validate_response(&response, request_id) {
                                Ok(_) => return Ok(response),
                                Err(e) => {
                                    log::warn!("Invalid response: {e}");
                                }
                            }
                        }
                        Ok(_) => {
                            log::warn!("Incomplete response received");
                        }
                        Err(e) => {
                            log::warn!("Network receive error: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Network send error: {e}");
                }
            }

            retries += 1;
        }

        Err(PoKeysError::Transfer(
            "Failed to send network request".to_string(),
        ))
    }

    /// Send request without expecting a response
    pub fn send_request_no_response<T: UsbHidInterface + ?Sized>(
        &mut self,
        interface: &mut T,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    ) -> Result<()> {
        let request =
            self.protocol
                .prepare_request(request_type, param1, param2, param3, param4, None);

        // Prepare HID packet
        let mut hid_packet = [0u8; 65];
        hid_packet[1..65].copy_from_slice(&request[..64]);

        interface.write(&hid_packet)?;
        Ok(())
    }

    /// Send multi-part request for large data transfers
    pub fn send_multipart_request<T: UsbHidInterface + ?Sized>(
        &mut self,
        interface: &mut T,
        request_type: u8,
        data: &[u8],
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        // Implementation for multi-part data transfer
        // This would be used for large data transfers like motion buffer updates

        let request = self
            .protocol
            .prepare_request(request_type, 0, 0, 0, 0, None);
        let request_id = request[6];

        // Send initial request
        let mut hid_packet = [0u8; 65];
        hid_packet[1..65].copy_from_slice(&request[..64]);
        interface.write(&hid_packet)?;

        // Send data in chunks
        for chunk in data.chunks(64) {
            let mut data_packet = [0u8; 65];
            data_packet[1..chunk.len() + 1].copy_from_slice(chunk);
            interface.write(&data_packet)?;
        }

        // Receive response
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        interface.read_timeout(&mut response, self.protocol.socket_timeout)?;

        self.protocol.validate_response(&response, request_id)?;
        Ok(response)
    }

    /// Send raw request via USB HID interface (for requests with data payloads)
    pub fn send_usb_request_raw<T: UsbHidInterface + ?Sized>(
        &mut self,
        interface: &mut T,
        request: &[u8; REQUEST_BUFFER_SIZE],
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        let request_id = request[6];

        let mut retries = 0;
        while retries < self.protocol.send_retries {
            // Prepare HID packet (add report ID byte at the beginning)
            let mut hid_packet = [0u8; 65];
            hid_packet[1..65].copy_from_slice(&request[..64]);

            // Send request
            match interface.write(&hid_packet) {
                Ok(_) => {
                    // Try to receive response
                    let mut response = [0u8; RESPONSE_BUFFER_SIZE];
                    let mut wait_count = 0;

                    while wait_count < 50 {
                        match interface.read_timeout(&mut response, Duration::from_millis(20)) {
                            Ok(bytes_read) if bytes_read > 0 => {
                                // Validate response
                                match self.protocol.validate_response(&response, request_id) {
                                    Ok(_) => return Ok(response),
                                    Err(e) => {
                                        log::warn!("Invalid response: {e}");
                                        break;
                                    }
                                }
                            }
                            Ok(_) => {
                                // No data received, continue waiting
                                wait_count += 1;
                            }
                            Err(e) => {
                                log::warn!("Read error: {e}");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Write error: {e}");
                }
            }

            retries += 1;
        }

        Err(PoKeysError::Transfer(
            "Failed to send USB request".to_string(),
        ))
    }

    /// Send raw request via network interface (for requests with data payloads)
    pub fn send_network_request_raw<T: NetworkInterface + ?Sized>(
        &mut self,
        interface: &mut T,
        request: &[u8; REQUEST_BUFFER_SIZE],
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        let request_id = request[6];

        let mut retries = 0;
        while retries < self.protocol.send_retries {
            match interface.send(&request[..64]) {
                Ok(_) => {
                    let mut response = [0u8; RESPONSE_BUFFER_SIZE];
                    match interface.receive(&mut response) {
                        Ok(_) => match self.protocol.validate_response(&response, request_id) {
                            Ok(_) => return Ok(response),
                            Err(e) => {
                                log::warn!("Invalid response: {e}");
                            }
                        },
                        Err(e) => {
                            log::warn!("Network receive error: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Network send error: {e}");
                }
            }

            retries += 1;
        }

        Err(PoKeysError::Transfer(
            "Failed to send network request".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        let data = [0xBB, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let checksum = Protocol::calculate_checksum(&data);
        let expected = 0xBB + 0x01 + 0x02 + 0x03 + 0x04 + 0x05 + 0x06;
        assert_eq!(checksum, expected as u8);
    }

    #[test]
    fn test_request_preparation() {
        let mut protocol = Protocol::new();
        let request = protocol.prepare_request(0x10, 0x20, 0x30, 0x40, 0x50, None);

        assert_eq!(request[0], REQUEST_HEADER);
        assert_eq!(request[1], 0x10);
        assert_eq!(request[2], 0x20);
        assert_eq!(request[3], 0x30);
        assert_eq!(request[4], 0x40);
        assert_eq!(request[5], 0x50);
        assert_eq!(request[6], 1); // First request ID

        let expected_checksum = Protocol::calculate_checksum(&request);
        assert_eq!(request[7], expected_checksum);
    }

    #[test]
    fn test_response_validation() {
        let protocol = Protocol::new();
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[0] = RESPONSE_HEADER;
        response[6] = 1; // Request ID
        response[7] = Protocol::calculate_checksum(&response);

        assert!(protocol.validate_response(&response, 1).is_ok());
        assert!(protocol.validate_response(&response, 2).is_err()); // Wrong request ID

        response[7] = 0xFF; // Wrong checksum
        assert!(protocol.validate_response(&response, 1).is_err());
    }
}
