//! OEM parameter read/write support (command 0xFD)
//!
//! PoKeys57-series devices provide 62 non-volatile 32-bit OEM parameters (indices 0–61).
//! This module exposes a general-purpose interface for reading and writing those parameters,
//! and a dedicated helper pair for a device *location* value stored at
//! [`LOCATION_PARAMETER_INDEX`].

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};

/// Maximum valid OEM parameter index (the device stores 62 parameters, 0–61).
pub const OEM_PARAMETER_MAX_INDEX: u8 = 61;

/// OEM parameter index used to store the device location.
pub const LOCATION_PARAMETER_INDEX: u8 = 0;

// Protocol constants for command 0xFD
const OEM_PARAM_CMD: u8 = 0xFD;
const OEM_PARAM_READ: u8 = 0x00;
const OEM_PARAM_WRITE: u8 = 0x01;
const OEM_PARAM_CLEAR: u8 = 0x02;

// Response byte offsets (0-based)
const RESP_SUBCMD: usize = 2; // echoed sub-command (0x00 or 0x01)
const RESP_STATUS: usize = 5; // read response: bit-mapped set-status
const RESP_VALUE_START: usize = 8; // first 32-bit parameter value (LE)

impl PoKeysDevice {
    /// Read a single OEM parameter from non-volatile storage.
    ///
    /// Returns `None` if the parameter slot has never been written (the device
    /// clears the corresponding status bit on a factory reset).
    ///
    /// # Errors
    ///
    /// Returns [`PoKeysError::Parameter`] when `index` > [`OEM_PARAMETER_MAX_INDEX`],
    /// or a communication error if the exchange fails.
    pub fn read_oem_parameter(&mut self, index: u8) -> Result<Option<i32>> {
        validate_index(index)?;

        // 0xFD / 0x00 – read parameters
        // byte layout: CMD=0xFD, sub-cmd=0x00, param-index, count=1, reserved=0
        let response = self.send_request(OEM_PARAM_CMD, OEM_PARAM_READ, index, 1, 0)?;

        if response[RESP_SUBCMD] != OEM_PARAM_READ {
            return Err(PoKeysError::Protocol(format!(
                "OEM read: unexpected response sub-command 0x{:02X}",
                response[RESP_SUBCMD]
            )));
        }

        // Bit 0 of the status byte indicates whether the first returned parameter has been set.
        if response[RESP_STATUS] & 0x01 == 0 {
            return Ok(None);
        }

        let value = i32::from_le_bytes([
            response[RESP_VALUE_START],
            response[RESP_VALUE_START + 1],
            response[RESP_VALUE_START + 2],
            response[RESP_VALUE_START + 3],
        ]);
        Ok(Some(value))
    }

    /// Write a single OEM parameter to non-volatile storage.
    ///
    /// # Errors
    ///
    /// Returns [`PoKeysError::Parameter`] when `index` > [`OEM_PARAMETER_MAX_INDEX`],
    /// or a communication error if the exchange fails.
    pub fn write_oem_parameter(&mut self, index: u8, value: i32) -> Result<()> {
        validate_index(index)?;

        // 0xFD / 0x01 – set parameter
        // byte layout: CMD=0xFD, sub-cmd=0x01, param-index, reserved, reserved
        // data payload (bytes 9-12 / 0-based 8-11): parameter value as little-endian u32
        let value_bytes = value.to_le_bytes();
        let response =
            self.send_request_with_data(OEM_PARAM_CMD, OEM_PARAM_WRITE, index, 0, 0, &value_bytes)?;

        if response[RESP_SUBCMD] != OEM_PARAM_WRITE {
            return Err(PoKeysError::Protocol(format!(
                "OEM write: unexpected response sub-command 0x{:02X}",
                response[RESP_SUBCMD]
            )));
        }

        // The device echoes the written value back at bytes 9-12; verify it matches.
        let echoed = i32::from_le_bytes([
            response[RESP_VALUE_START],
            response[RESP_VALUE_START + 1],
            response[RESP_VALUE_START + 2],
            response[RESP_VALUE_START + 3],
        ]);
        if echoed != value {
            return Err(PoKeysError::Protocol(format!(
                "OEM write: echoed value {} does not match written value {}",
                echoed, value
            )));
        }

        Ok(())
    }

    /// Read the device location from OEM parameter storage.
    ///
    /// The location is persisted at OEM parameter index [`LOCATION_PARAMETER_INDEX`].
    /// Returns `None` if the location has never been set.
    pub fn get_location(&mut self) -> Result<Option<i32>> {
        self.read_oem_parameter(LOCATION_PARAMETER_INDEX)
    }

    /// Write the device location to OEM parameter storage.
    ///
    /// The location is a user-defined integer stored in the device's non-volatile
    /// OEM parameter slot at index [`LOCATION_PARAMETER_INDEX`].
    pub fn set_location(&mut self, location: i32) -> Result<()> {
        self.write_oem_parameter(LOCATION_PARAMETER_INDEX, location)
    }

    /// Clear the device location from OEM parameter storage.
    ///
    /// After this call [`get_location`](PoKeysDevice::get_location) will return `None`.
    pub fn clear_location(&mut self) -> Result<()> {
        self.clear_oem_parameter(LOCATION_PARAMETER_INDEX)
    }

    /// Clear a single OEM parameter, marking it as unset in non-volatile storage.
    ///
    /// After clearing, [`read_oem_parameter`](PoKeysDevice::read_oem_parameter) returns `None`
    /// for this index until a new value is written.
    ///
    /// # Errors
    ///
    /// Returns [`PoKeysError::Parameter`] when `index` > [`OEM_PARAMETER_MAX_INDEX`],
    /// or a communication error if the exchange fails.
    pub fn clear_oem_parameter(&mut self, index: u8) -> Result<()> {
        validate_index(index)?;

        // 0xFD / 0x02 – clear parameter
        // byte layout: CMD=0xFD, sub-cmd=0x02, param-index, reserved, reserved
        let response = self.send_request(OEM_PARAM_CMD, OEM_PARAM_CLEAR, index, 0, 0)?;

        if response[RESP_SUBCMD] != OEM_PARAM_CLEAR {
            return Err(PoKeysError::Protocol(format!(
                "OEM clear: unexpected response sub-command 0x{:02X}",
                response[RESP_SUBCMD]
            )));
        }

        Ok(())
    }
}

fn validate_index(index: u8) -> Result<()> {
    if index > OEM_PARAMETER_MAX_INDEX {
        return Err(PoKeysError::Parameter(format!(
            "OEM parameter index {} out of range (valid range 0–{})",
            index, OEM_PARAMETER_MAX_INDEX
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::Protocol;
    use crate::types::*;

    // ── index validation ────────────────────────────────────────────────────

    #[test]
    fn validate_index_accepts_boundary_values() {
        assert!(validate_index(0).is_ok());
        assert!(validate_index(61).is_ok());
    }

    #[test]
    fn validate_index_rejects_out_of_range() {
        let err = validate_index(62).unwrap_err();
        assert!(matches!(err, PoKeysError::Parameter(_)));
        assert!(err.to_string().contains("62"));
    }

    // ── read packet construction ────────────────────────────────────────────

    #[test]
    fn read_request_packet_layout() {
        let mut protocol = Protocol::new();
        // CMD=0xFD, sub-cmd=0x00, index=5, count=1, reserved=0
        let pkt = protocol.prepare_request(OEM_PARAM_CMD, OEM_PARAM_READ, 5, 1, 0, None);

        assert_eq!(pkt[0], REQUEST_HEADER); // 0xBB
        assert_eq!(pkt[1], OEM_PARAM_CMD); // 0xFD
        assert_eq!(pkt[2], OEM_PARAM_READ); // 0x00
        assert_eq!(pkt[3], 5); // parameter index
        assert_eq!(pkt[4], 1); // count
        assert_eq!(pkt[5], 0); // reserved
        // checksum covers bytes 0-6
        assert_eq!(pkt[7], Protocol::calculate_checksum(&pkt));
    }

    // ── write packet construction ───────────────────────────────────────────

    #[test]
    fn write_request_packet_layout() {
        let mut protocol = Protocol::new();
        // CMD=0xFD, sub-cmd=0x01, index=3, reserved, reserved
        let mut pkt = protocol.prepare_request(OEM_PARAM_CMD, OEM_PARAM_WRITE, 3, 0, 0, None);

        // Embed value 0xDEAD_BEEF at bytes 8-11
        let value: i32 = 0x0102_0304;
        let value_bytes = value.to_le_bytes();
        pkt[8..12].copy_from_slice(&value_bytes);
        pkt[7] = Protocol::calculate_checksum(&pkt);

        assert_eq!(pkt[1], OEM_PARAM_CMD);
        assert_eq!(pkt[2], OEM_PARAM_WRITE);
        assert_eq!(pkt[3], 3); // parameter index
        assert_eq!(&pkt[8..12], &value_bytes);
        // checksum is still over bytes 0-6 only
        assert_eq!(pkt[7], Protocol::calculate_checksum(&pkt));
    }

    // ── response parsing ────────────────────────────────────────────────────

    #[test]
    fn read_response_parses_set_parameter() {
        // Craft a synthetic read response
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[0] = RESPONSE_HEADER; // 0xAA
        response[1] = OEM_PARAM_CMD; // 0xFD
        response[2] = OEM_PARAM_READ; // 0x00
        response[3] = LOCATION_PARAMETER_INDEX; // index
        response[4] = 1; // count
        response[5] = 0x01; // status: bit 0 set → parameter is set
        response[6] = 1; // request ID
        let value: i32 = 42;
        response[8..12].copy_from_slice(&value.to_le_bytes());
        response[7] = Protocol::calculate_checksum(&response);

        // Verify parsing logic directly
        assert_eq!(response[RESP_SUBCMD], OEM_PARAM_READ);
        assert_ne!(response[RESP_STATUS] & 0x01, 0);
        let parsed = i32::from_le_bytes([
            response[RESP_VALUE_START],
            response[RESP_VALUE_START + 1],
            response[RESP_VALUE_START + 2],
            response[RESP_VALUE_START + 3],
        ]);
        assert_eq!(parsed, 42);
    }

    #[test]
    fn read_response_detects_unset_parameter() {
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[0] = RESPONSE_HEADER;
        response[1] = OEM_PARAM_CMD;
        response[2] = OEM_PARAM_READ;
        response[5] = 0x00; // status: bit 0 clear → parameter not set
        response[6] = 1;
        response[7] = Protocol::calculate_checksum(&response);

        assert_eq!(response[RESP_STATUS] & 0x01, 0);
    }

    #[test]
    fn write_response_echo_check() {
        let value: i32 = -99;
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[0] = RESPONSE_HEADER;
        response[1] = OEM_PARAM_CMD;
        response[2] = OEM_PARAM_WRITE;
        response[3] = LOCATION_PARAMETER_INDEX;
        response[6] = 1;
        response[8..12].copy_from_slice(&value.to_le_bytes());
        response[7] = Protocol::calculate_checksum(&response);

        let echoed = i32::from_le_bytes([
            response[RESP_VALUE_START],
            response[RESP_VALUE_START + 1],
            response[RESP_VALUE_START + 2],
            response[RESP_VALUE_START + 3],
        ]);
        assert_eq!(echoed, value);
    }

    // ── clear packet construction ───────────────────────────────────────────

    #[test]
    fn clear_request_packet_layout() {
        let mut protocol = Protocol::new();
        let pkt = protocol.prepare_request(OEM_PARAM_CMD, OEM_PARAM_CLEAR, 7, 0, 0, None);

        assert_eq!(pkt[0], REQUEST_HEADER);
        assert_eq!(pkt[1], OEM_PARAM_CMD);
        assert_eq!(pkt[2], OEM_PARAM_CLEAR);
        assert_eq!(pkt[3], 7); // parameter index
        assert_eq!(pkt[4], 0); // reserved
        assert_eq!(pkt[5], 0); // reserved
        assert_eq!(pkt[7], Protocol::calculate_checksum(&pkt));
    }

    #[test]
    fn clear_response_subcmd_check() {
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[0] = RESPONSE_HEADER;
        response[1] = OEM_PARAM_CMD;
        response[2] = OEM_PARAM_CLEAR;
        response[3] = 7; // parameter index echoed
        response[6] = 1;
        response[7] = Protocol::calculate_checksum(&response);

        assert_eq!(response[RESP_SUBCMD], OEM_PARAM_CLEAR);
    }

    // ── constants ───────────────────────────────────────────────────────────

    #[test]
    fn location_index_is_within_range() {
        assert!(LOCATION_PARAMETER_INDEX <= OEM_PARAMETER_MAX_INDEX);
    }
}
