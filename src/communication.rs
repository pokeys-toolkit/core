//! Low-level communication protocol implementation

use crate::error::{PoKeysError, Result};
use crate::types::*;
use std::cell::RefCell;
use std::time::{Duration, Instant};

/// How long a warning category stays "suppressed to DEBUG" after the first
/// WARN. Chosen as 5 s — long enough to collapse a flapping connection's
/// noise into one line per second at worst, short enough that a recovered
/// device's next failure is still visible within a human-scale timeframe.
pub(crate) const WARN_DEDUP_WINDOW: Duration = Duration::from_secs(5);

/// Coarse log categories used by [`Protocol::log_warn_rate_limited`].
/// Each category gets its own timestamp so, for example, "send failed"
/// and "receive timed out" don't mask each other.
#[derive(Debug, Clone, Copy)]
pub(crate) enum WarnCategory {
    WriteError = 0,
    ReadError = 1,
    ReceiveTimeout = 2,
    Incomplete = 3,
    InvalidResponse = 4,
    SendError = 5,
}

/// Per-category last-log timestamps for WARN rate-limiting.
#[derive(Debug, Default)]
struct WarnLogGate {
    last: [Option<Instant>; 6],
}

impl WarnLogGate {
    fn last(&self, cat: WarnCategory) -> Option<Instant> {
        self.last[cat as usize]
    }

    fn set(&mut self, cat: WarnCategory, at: Instant) {
        self.last[cat as usize] = Some(at);
    }
}

/// Communication protocol implementation.
pub struct Protocol {
    request_id: u8,
    send_retries: u32,
    socket_timeout: Duration,
    /// Per-category last-logged timestamps for WARN rate-limiting. See
    /// [`Protocol::log_warn_rate_limited`].
    warn_log_gate: RefCell<WarnLogGate>,
}

impl Default for Protocol {
    fn default() -> Self {
        Self {
            request_id: 0,
            send_retries: 3,
            socket_timeout: Duration::from_millis(1000),
            warn_log_gate: RefCell::new(WarnLogGate::default()),
        }
    }
}

impl Protocol {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure send-retry count and socket receive timeout.
    ///
    /// `read_retries` is accepted but ignored — it has never been read by
    /// any retry loop in this crate. The parameter is retained only to keep
    /// the public signature of the pre-1.0 API stable. Prefer the newer
    /// setters on [`crate::PoKeysDevice`]: `set_network_timeout` and
    /// `set_network_retries`.
    pub fn set_retries_and_timeout(
        &mut self,
        send_retries: u32,
        _read_retries: u32,
        timeout: Duration,
    ) {
        self.send_retries = send_retries;
        self.socket_timeout = timeout;
    }

    /// Current socket receive timeout.
    pub(crate) fn socket_timeout(&self) -> Duration {
        self.socket_timeout
    }

    /// Current `send_request` retry count on network timeouts.
    pub(crate) fn send_retries(&self) -> u32 {
        self.send_retries
    }

    /// Log a WARN message, de-duplicating by category over a short window.
    ///
    /// During a persistent network failure the send/receive retry loops can
    /// hit the same error path tens of times per second. Logging every
    /// occurrence at WARN floods the caller's log sinks; dropping all of
    /// them hides real problems. Compromise: the first WARN in a given
    /// category within [`WARN_DEDUP_WINDOW`] stays at WARN; subsequent
    /// occurrences until the window expires log at DEBUG instead.
    ///
    /// Categories are deliberately coarse (send error / receive error /
    /// incomplete / invalid) so a flapping device produces a single WARN
    /// per window per failure mode.
    pub(crate) fn log_warn_rate_limited(&self, category: WarnCategory, args: std::fmt::Arguments) {
        let now = Instant::now();
        let mut gate = self.warn_log_gate.borrow_mut();
        let last = gate.last(category);
        let recent = last
            .map(|t| now.duration_since(t) < WARN_DEDUP_WINDOW)
            .unwrap_or(false);

        if recent {
            log::debug!("{args}");
        } else {
            log::warn!("{args}");
            gate.set(category, now);
        }
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

    /// Current socket receive timeout (used only on network connections).
    pub fn socket_timeout(&self) -> Duration {
        self.protocol.socket_timeout()
    }

    /// Current per-`send_request` retry count on network timeouts.
    pub fn send_retries(&self) -> u32 {
        self.protocol.send_retries()
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
                                        self.protocol.log_warn_rate_limited(
                                            WarnCategory::InvalidResponse,
                                            format_args!("Invalid response: {e}"),
                                        );
                                        break;
                                    }
                                }
                            }
                            Ok(_) => {
                                // No data received, continue waiting
                                wait_count += 1;
                            }
                            Err(e) => {
                                self.protocol.log_warn_rate_limited(
                                    WarnCategory::ReadError,
                                    format_args!("Read error: {e}"),
                                );
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    self.protocol.log_warn_rate_limited(
                        WarnCategory::WriteError,
                        format_args!("Write error: {e}"),
                    );
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
                                    self.protocol.log_warn_rate_limited(
                                        WarnCategory::InvalidResponse,
                                        format_args!("Invalid response: {e}"),
                                    );
                                }
                            }
                        }
                        Ok(_) => {
                            self.protocol.log_warn_rate_limited(
                                WarnCategory::Incomplete,
                                format_args!("Incomplete response received"),
                            );
                        }
                        Err(e) => {
                            self.protocol.log_warn_rate_limited(
                                WarnCategory::ReceiveTimeout,
                                format_args!("Network receive error: {e}"),
                            );
                        }
                    }
                }
                Err(e) => {
                    self.protocol.log_warn_rate_limited(
                        WarnCategory::SendError,
                        format_args!("Network send error: {e}"),
                    );
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
                                        self.protocol.log_warn_rate_limited(
                                            WarnCategory::InvalidResponse,
                                            format_args!("Invalid response: {e}"),
                                        );
                                        break;
                                    }
                                }
                            }
                            Ok(_) => {
                                // No data received, continue waiting
                                wait_count += 1;
                            }
                            Err(e) => {
                                self.protocol.log_warn_rate_limited(
                                    WarnCategory::ReadError,
                                    format_args!("Read error: {e}"),
                                );
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    self.protocol.log_warn_rate_limited(
                        WarnCategory::WriteError,
                        format_args!("Write error: {e}"),
                    );
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
                                self.protocol.log_warn_rate_limited(
                                    WarnCategory::InvalidResponse,
                                    format_args!("Invalid response: {e}"),
                                );
                            }
                        },
                        Err(e) => {
                            self.protocol.log_warn_rate_limited(
                                WarnCategory::ReceiveTimeout,
                                format_args!("Network receive error: {e}"),
                            );
                        }
                    }
                }
                Err(e) => {
                    self.protocol.log_warn_rate_limited(
                        WarnCategory::SendError,
                        format_args!("Network send error: {e}"),
                    );
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

    #[test]
    fn test_reboot_request_format() {
        // "Reboot system" command, per PoKeys protocol spec:
        //   byte 1 (header) = 0xBB
        //   byte 2 (CMD)    = 0xF3
        //   bytes 3-6       = reserved (0)
        //   byte 7          = request ID
        //   byte 8          = checksum of bytes 1-7
        let mut protocol = Protocol::new();
        let request = protocol.prepare_request(0xF3, 0, 0, 0, 0, None);

        assert_eq!(request[0], REQUEST_HEADER);
        assert_eq!(request[1], 0xF3);
        assert_eq!(request[2], 0);
        assert_eq!(request[3], 0);
        assert_eq!(request[4], 0);
        assert_eq!(request[5], 0);
        assert_eq!(request[6], 1);
        assert_eq!(request[7], Protocol::calculate_checksum(&request));

        // Payload bytes (9-64 in 1-based spec numbering) are unused.
        for i in 8..REQUEST_BUFFER_SIZE {
            assert_eq!(request[i], 0);
        }
    }

    #[test]
    fn test_protocol_defaults() {
        let protocol = Protocol::new();
        assert_eq!(protocol.send_retries(), 3);
        assert_eq!(protocol.socket_timeout(), Duration::from_millis(1000));
    }

    #[test]
    fn test_protocol_tunables_round_trip() {
        let mut protocol = Protocol::new();
        // read_retries param is intentionally ignored; pass anything.
        protocol.set_retries_and_timeout(1, 99, Duration::from_millis(50));
        assert_eq!(protocol.send_retries(), 1);
        assert_eq!(protocol.socket_timeout(), Duration::from_millis(50));
    }

    #[test]
    fn test_communication_manager_tunables_round_trip() {
        let mut mgr = CommunicationManager::new(DeviceConnectionType::NetworkDevice);
        assert_eq!(mgr.send_retries(), 3);
        assert_eq!(mgr.socket_timeout(), Duration::from_millis(1000));

        mgr.set_retries_and_timeout(2, 0, Duration::from_millis(250));
        assert_eq!(mgr.send_retries(), 2);
        assert_eq!(mgr.socket_timeout(), Duration::from_millis(250));
    }
}
