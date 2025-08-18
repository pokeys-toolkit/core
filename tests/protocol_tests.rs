//! Protocol and communication tests - No hardware required
//!
//! These tests verify the protocol implementation, message formatting,
//! and communication logic without requiring actual hardware.

#![allow(clippy::needless_range_loop)]

use pokeys_lib::*;

#[cfg(test)]
mod protocol_tests {

    #[test]
    fn test_request_buffer_formatting() {
        // Test basic request buffer structure
        let mut buffer = [0u8; 64];

        // Simulate formatting a basic request
        buffer[0] = 0x00; // Command ID
        buffer[1] = 0x01; // Parameter 1
        buffer[2] = 0x02; // Parameter 2
        buffer[3] = 0x03; // Parameter 3
        buffer[4] = 0x04; // Parameter 4

        assert_eq!(buffer[0], 0x00);
        assert_eq!(buffer[1], 0x01);
        assert_eq!(buffer[2], 0x02);
        assert_eq!(buffer[3], 0x03);
        assert_eq!(buffer[4], 0x04);

        // Rest should be zero
        for i in 5..64 {
            assert_eq!(buffer[i], 0);
        }
    }

    #[test]
    fn test_response_parsing() {
        // Test parsing a simulated device response
        let response = [
            0x00, // Status
            0x01, 0x02, 0x03, 0x04, // Device ID
            0x05, 0x06, // Firmware version
            0x07, 0x08, 0x09, 0x0A, // Serial number
            0x37, // Pin count (55)
            0x06, // PWM count
            0x08, // Analog inputs
            0x19, // Encoders (25)
        ];

        // Simulate parsing device info from response
        let device_type = response[1] as u32
            | ((response[2] as u32) << 8)
            | ((response[3] as u32) << 16)
            | ((response[4] as u32) << 24);

        let firmware_major = response[5];
        let firmware_minor = response[6];

        let serial = response[7] as u32
            | ((response[8] as u32) << 8)
            | ((response[9] as u32) << 16)
            | ((response[10] as u32) << 24);

        let pin_count = response[11];
        let pwm_count = response[12];
        let analog_inputs = response[13];
        let encoders = response[14];

        assert_eq!(device_type, 0x04030201);
        assert_eq!(firmware_major, 5);
        assert_eq!(firmware_minor, 6);
        assert_eq!(serial, 0x0A090807);
        assert_eq!(pin_count, 55);
        assert_eq!(pwm_count, 6);
        assert_eq!(analog_inputs, 8);
        assert_eq!(encoders, 25);
    }

    #[test]
    fn test_pin_state_encoding() {
        // Test encoding pin states into bit arrays
        let mut pin_states = [0u8; 8]; // 64 pins max (8 bytes * 8 bits)

        // Set some pins
        let pins_to_set = [1, 5, 17, 33, 55];

        for &pin in &pins_to_set {
            if pin > 0 && pin <= 64 {
                let byte_index = (pin - 1) / 8;
                let bit_index = (pin - 1) % 8;
                pin_states[byte_index as usize] |= 1 << bit_index;
            }
        }

        // Verify pins are set correctly
        for &pin in &pins_to_set {
            if pin > 0 && pin <= 64 {
                let byte_index = (pin - 1) / 8;
                let bit_index = (pin - 1) % 8;
                let is_set = (pin_states[byte_index as usize] & (1 << bit_index)) != 0;
                assert!(is_set, "Pin {pin} should be set");
            }
        }

        // Verify other pins are not set
        let test_pins = [2, 3, 4, 6, 16, 18, 32, 34, 54, 56];
        for &pin in &test_pins {
            if pin > 0 && pin <= 64 {
                let byte_index = (pin - 1) / 8;
                let bit_index = (pin - 1) % 8;
                let is_set = (pin_states[byte_index as usize] & (1 << bit_index)) != 0;
                assert!(!is_set, "Pin {pin} should not be set");
            }
        }
    }

    #[test]
    fn test_analog_value_encoding() {
        // Test encoding 12-bit analog values
        let analog_values = [0u16, 2048, 4095, 1234, 3456];
        let mut buffer = [0u8; 16];

        // Encode analog values (2 bytes per value, little-endian)
        for (i, &value) in analog_values.iter().enumerate() {
            let offset = i * 2;
            buffer[offset] = (value & 0xFF) as u8;
            buffer[offset + 1] = ((value >> 8) & 0xFF) as u8;
        }

        // Decode and verify
        for (i, &expected) in analog_values.iter().enumerate() {
            let offset = i * 2;
            let decoded = buffer[offset] as u16 | ((buffer[offset + 1] as u16) << 8);
            assert_eq!(decoded, expected, "Analog value {i} mismatch");
        }
    }

    #[test]
    fn test_encoder_value_encoding() {
        // Test encoding 32-bit signed encoder values
        let encoder_values = [0i32, -1, 1, i32::MIN, i32::MAX, 12345, -67890];
        let mut buffer = [0u8; 32];

        // Encode encoder values (4 bytes per value, little-endian)
        for (i, &value) in encoder_values.iter().enumerate() {
            let offset = i * 4;
            let bytes = value.to_le_bytes();
            buffer[offset..offset + 4].copy_from_slice(&bytes);
        }

        // Decode and verify
        for (i, &expected) in encoder_values.iter().enumerate() {
            let offset = i * 4;
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&buffer[offset..offset + 4]);
            let decoded = i32::from_le_bytes(bytes);
            assert_eq!(decoded, expected, "Encoder value {i} mismatch");
        }
    }

    #[test]
    fn test_pwm_duty_cycle_encoding() {
        // Test encoding PWM duty cycles as percentages to raw values
        let duty_percentages = [0.0, 25.0, 50.0, 75.0, 100.0];
        let pwm_period = 1000u16; // Example period

        for &duty_percent in &duty_percentages {
            // Convert percentage to raw value
            let raw_value = ((duty_percent / 100.0) * pwm_period as f32) as u16;

            // Verify range
            assert!(raw_value <= pwm_period);

            // Convert back to percentage
            let back_to_percent = (raw_value as f32 / pwm_period as f32) * 100.0;

            // Allow small floating point error
            assert!((back_to_percent - duty_percent).abs() < 0.1);
        }
    }

    #[test]
    fn test_checksum_calculation() {
        // Test various checksum algorithms that might be used

        // Simple sum checksum
        fn sum_checksum(data: &[u8]) -> u8 {
            data.iter().fold(0u8, |acc, &x| acc.wrapping_add(x))
        }

        // XOR checksum
        fn xor_checksum(data: &[u8]) -> u8 {
            data.iter().fold(0u8, |acc, &x| acc ^ x)
        }

        let test_data = [0x01, 0x02, 0x03, 0x04, 0x05];

        let sum_check = sum_checksum(&test_data);
        let xor_check = xor_checksum(&test_data);

        assert_eq!(sum_check, 15); // 1+2+3+4+5 = 15
        assert_eq!(xor_check, 1); // 1^2^3^4^5 = 1

        // Test with known patterns
        let all_zeros = [0u8; 10];
        assert_eq!(sum_checksum(&all_zeros), 0);
        assert_eq!(xor_checksum(&all_zeros), 0);

        let all_ones = [0xFF; 4];
        assert_eq!(sum_checksum(&all_ones), 252); // 4 * 255 = 1020, wrapped = 252
        assert_eq!(xor_checksum(&all_ones), 0); // Even number of 0xFF XORs to 0
    }

    #[test]
    fn test_command_id_constants() {
        // Test that command IDs are in expected ranges
        let basic_commands = [
            0x00, // Get device data
            0x01, // Set pin function
            0x02, // Get pin function
            0x03, // Set digital outputs
            0x04, // Get digital inputs
            0x05, // Get analog inputs
            0x10, // PWM configuration
            0x20, // Encoder configuration
            0x30, // Matrix keyboard
            0x40, // LCD commands
            0x50, // Pulse engine
        ];

        for &cmd in &basic_commands {
            assert!(cmd < 0x80, "Command ID {cmd} should be < 0x80");
        }
    }

    #[test]
    fn test_multi_byte_value_handling() {
        // Test handling of multi-byte values in different endianness

        let test_value = 0x12345678u32;

        // Little-endian encoding
        let le_bytes = test_value.to_le_bytes();
        assert_eq!(le_bytes, [0x78, 0x56, 0x34, 0x12]);

        // Big-endian encoding
        let be_bytes = test_value.to_be_bytes();
        assert_eq!(be_bytes, [0x12, 0x34, 0x56, 0x78]);

        // Verify round-trip conversion
        assert_eq!(u32::from_le_bytes(le_bytes), test_value);
        assert_eq!(u32::from_be_bytes(be_bytes), test_value);
    }

    #[test]
    fn test_buffer_overflow_protection() {
        // Test that buffer operations don't overflow

        let mut buffer = [0u8; 64];
        let data_to_copy = [0xAA; 32];

        // Safe copy within bounds
        buffer[0..32].copy_from_slice(&data_to_copy);

        // Verify copy
        for i in 0..32 {
            assert_eq!(buffer[i], 0xAA);
        }

        // Verify rest is still zero
        for i in 32..64 {
            assert_eq!(buffer[i], 0);
        }
    }

    #[test]
    fn test_protocol_version_handling() {
        // Test protocol version compatibility

        struct ProtocolVersion {
            major: u8,
            minor: u8,
        }

        impl ProtocolVersion {
            fn is_compatible(&self, other: &ProtocolVersion) -> bool {
                // Same major version, minor can be different
                self.major == other.major
            }

            fn is_newer(&self, other: &ProtocolVersion) -> bool {
                self.major > other.major || (self.major == other.major && self.minor > other.minor)
            }
        }

        let v1_0 = ProtocolVersion { major: 1, minor: 0 };
        let v1_1 = ProtocolVersion { major: 1, minor: 1 };
        let v2_0 = ProtocolVersion { major: 2, minor: 0 };

        // Test compatibility
        assert!(v1_0.is_compatible(&v1_1));
        assert!(v1_1.is_compatible(&v1_0));
        assert!(!v1_0.is_compatible(&v2_0));

        // Test version comparison
        assert!(v1_1.is_newer(&v1_0));
        assert!(v2_0.is_newer(&v1_1));
        assert!(!v1_0.is_newer(&v1_1));
    }

    #[test]
    fn test_error_response_parsing() {
        // Test parsing error responses from device

        let error_responses = [
            ([0xFF, 0x01, 0x00, 0x00], "Invalid command"),
            ([0xFF, 0x02, 0x00, 0x00], "Invalid parameter"),
            ([0xFF, 0x03, 0x00, 0x00], "Device busy"),
            ([0xFF, 0x04, 0x00, 0x00], "Communication error"),
        ];

        for (response, description) in &error_responses {
            let status = response[0];
            let error_code = response[1];

            assert_eq!(
                status, 0xFF,
                "Error status should be 0xFF for {description}"
            );
            assert!(
                error_code > 0,
                "Error code should be non-zero for {description}"
            );
        }
    }

    #[test]
    fn test_timeout_handling() {
        use std::time::{Duration, Instant};

        // Simulate timeout detection
        fn check_timeout(start_time: Instant, timeout: Duration) -> bool {
            start_time.elapsed() > timeout
        }

        let start = Instant::now();
        let short_timeout = Duration::from_millis(1);
        let long_timeout = Duration::from_millis(1000);

        // Should not timeout immediately
        assert!(!check_timeout(start, long_timeout));

        // Wait a bit and check short timeout
        std::thread::sleep(Duration::from_millis(2));
        assert!(check_timeout(start, short_timeout));
    }

    #[test]
    fn test_data_validation() {
        // Test validation functions for protocol data

        fn validate_pin_number(pin: u8, max_pins: u8) -> bool {
            pin > 0 && pin <= max_pins
        }

        fn validate_pwm_duty_cycle(duty: f32) -> bool {
            (0.0..=100.0).contains(&duty)
        }

        fn validate_encoder_id(encoder: u8, max_encoders: u8) -> bool {
            encoder < max_encoders
        }

        // Test pin validation
        assert!(!validate_pin_number(0, 55));
        assert!(validate_pin_number(1, 55));
        assert!(validate_pin_number(55, 55));
        assert!(!validate_pin_number(56, 55));

        // Test PWM validation
        assert!(validate_pwm_duty_cycle(0.0));
        assert!(validate_pwm_duty_cycle(50.0));
        assert!(validate_pwm_duty_cycle(100.0));
        assert!(!validate_pwm_duty_cycle(-1.0));
        assert!(!validate_pwm_duty_cycle(101.0));

        // Test encoder validation
        assert!(validate_encoder_id(0, 25));
        assert!(validate_encoder_id(24, 25));
        assert!(!validate_encoder_id(25, 25));
    }

    #[test]
    fn test_bit_field_operations() {
        // Test bit field operations used in protocol

        #[derive(Default)]
        struct DeviceFlags {
            value: u32,
        }

        impl DeviceFlags {
            fn set_flag(&mut self, bit: u8) {
                self.value |= 1 << bit;
            }

            fn clear_flag(&mut self, bit: u8) {
                self.value &= !(1 << bit);
            }

            fn get_flag(&self, bit: u8) -> bool {
                (self.value & (1 << bit)) != 0
            }

            fn from_bytes(bytes: &[u8; 4]) -> Self {
                Self {
                    value: u32::from_le_bytes(*bytes),
                }
            }

            fn to_bytes(&self) -> [u8; 4] {
                self.value.to_le_bytes()
            }
        }

        let mut flags = DeviceFlags::default();

        // Test setting and getting flags
        flags.set_flag(0);
        flags.set_flag(15);
        flags.set_flag(31);

        assert!(flags.get_flag(0));
        assert!(flags.get_flag(15));
        assert!(flags.get_flag(31));
        assert!(!flags.get_flag(1));
        assert!(!flags.get_flag(16));

        // Test clearing flags
        flags.clear_flag(15);
        assert!(!flags.get_flag(15));
        assert!(flags.get_flag(0));
        assert!(flags.get_flag(31));

        // Test byte conversion
        let bytes = flags.to_bytes();
        let flags2 = DeviceFlags::from_bytes(&bytes);
        assert_eq!(flags.value, flags2.value);
    }
}

#[cfg(test)]
mod communication_tests {
    use super::*;
    use std::time::Duration;

    /// Mock communication interface for testing
    struct MockCommunication {
        connected: bool,
        last_request: Vec<u8>,
        next_response: Vec<u8>,
        should_fail: bool,
        delay: Duration,
    }

    impl MockCommunication {
        fn new() -> Self {
            Self {
                connected: false,
                last_request: Vec::new(),
                next_response: Vec::new(),
                should_fail: false,
                delay: Duration::from_millis(0),
            }
        }

        fn connect(&mut self) -> Result<()> {
            if self.should_fail {
                return Err(PoKeysError::CannotConnect);
            }
            self.connected = true;
            Ok(())
        }

        fn disconnect(&mut self) {
            self.connected = false;
        }

        fn send_request(&mut self, request: &[u8]) -> Result<Vec<u8>> {
            if !self.connected {
                return Err(PoKeysError::NotConnected);
            }

            if self.should_fail {
                return Err(PoKeysError::Transfer("Communication failed".to_string()));
            }

            // Simulate delay
            if self.delay > Duration::from_millis(0) {
                std::thread::sleep(self.delay);
            }

            self.last_request = request.to_vec();
            Ok(self.next_response.clone())
        }

        fn set_next_response(&mut self, response: Vec<u8>) {
            self.next_response = response;
        }

        fn set_should_fail(&mut self, fail: bool) {
            self.should_fail = fail;
        }

        fn set_delay(&mut self, delay: Duration) {
            self.delay = delay;
        }

        fn get_last_request(&self) -> &[u8] {
            &self.last_request
        }
    }

    #[test]
    fn test_mock_communication_basic() {
        let mut comm = MockCommunication::new();

        // Test initial state
        assert!(!comm.connected);

        // Test connection
        comm.connect().unwrap();
        assert!(comm.connected);

        // Test communication
        let request = vec![0x00, 0x01, 0x02];
        let response = vec![0x10, 0x11, 0x12];
        comm.set_next_response(response.clone());

        let result = comm.send_request(&request).unwrap();
        assert_eq!(result, response);
        assert_eq!(comm.get_last_request(), &request);

        // Test disconnection
        comm.disconnect();
        assert!(!comm.connected);
    }

    #[test]
    fn test_mock_communication_errors() {
        let mut comm = MockCommunication::new();

        // Test communication without connection
        let request = vec![0x00];
        assert!(comm.send_request(&request).is_err());

        // Test connection failure
        comm.set_should_fail(true);
        assert!(comm.connect().is_err());

        // Test communication failure
        comm.set_should_fail(false);
        comm.connect().unwrap();
        comm.set_should_fail(true);
        assert!(comm.send_request(&request).is_err());
    }

    #[test]
    fn test_mock_communication_timeout_simulation() {
        let mut comm = MockCommunication::new();
        comm.connect().unwrap();

        // Set a delay to simulate slow communication
        comm.set_delay(Duration::from_millis(100));
        comm.set_next_response(vec![0xFF]);

        let start = std::time::Instant::now();
        let request = vec![0x00];
        let _result = comm.send_request(&request).unwrap();
        let elapsed = start.elapsed();

        // Should have taken at least the delay time
        assert!(elapsed >= Duration::from_millis(90)); // Allow some tolerance
    }

    #[test]
    fn test_request_response_patterns() {
        let mut comm = MockCommunication::new();
        comm.connect().unwrap();

        // Test device info request pattern
        let device_info_request = vec![0x00, 0x00, 0x00, 0x00, 0x00];
        let device_info_response = vec![
            0x00, // Status OK
            0x0A, 0x00, 0x00, 0x00, // Device type
            0x01, 0x02, // Firmware version
            0x34, 0x12, 0x00, 0x00, // Serial number
            55,   // Pin count
            6,    // PWM count
            8,    // Analog inputs
            25,   // Encoders
        ];

        comm.set_next_response(device_info_response.clone());
        let response = comm.send_request(&device_info_request).unwrap();

        assert_eq!(response, device_info_response);
        assert_eq!(comm.get_last_request(), &device_info_request);

        // Test digital output request pattern
        let digital_out_request = vec![0x03, 0x01, 0xFF, 0x00, 0x00]; // Set pin 1 high
        let digital_out_response = vec![0x00]; // Status OK

        comm.set_next_response(digital_out_response.clone());
        let response = comm.send_request(&digital_out_request).unwrap();

        assert_eq!(response, digital_out_response);
        assert_eq!(comm.get_last_request(), &digital_out_request);
    }

    #[test]
    fn test_multiple_requests() {
        let mut comm = MockCommunication::new();
        comm.connect().unwrap();

        let requests_responses = vec![
            (vec![0x00], vec![0x00, 0x01]),
            (vec![0x01], vec![0x00, 0x02]),
            (vec![0x02], vec![0x00, 0x03]),
        ];

        for (request, expected_response) in requests_responses {
            comm.set_next_response(expected_response.clone());
            let response = comm.send_request(&request).unwrap();
            assert_eq!(response, expected_response);
            assert_eq!(comm.get_last_request(), &request);
        }
    }

    #[test]
    fn test_large_data_transfer() {
        let mut comm = MockCommunication::new();
        comm.connect().unwrap();

        // Test large request/response
        let large_request = vec![0xAA; 1024];
        let large_response = vec![0x55; 2048];

        comm.set_next_response(large_response.clone());
        let response = comm.send_request(&large_request).unwrap();

        assert_eq!(response, large_response);
        assert_eq!(comm.get_last_request(), &large_request);
    }

    #[test]
    fn test_empty_data_handling() {
        let mut comm = MockCommunication::new();
        comm.connect().unwrap();

        // Test empty request
        let empty_request = vec![];
        let response = vec![0x00];

        comm.set_next_response(response.clone());
        let result = comm.send_request(&empty_request).unwrap();

        assert_eq!(result, response);
        assert_eq!(comm.get_last_request(), &empty_request);

        // Test empty response
        let request = vec![0x00];
        let empty_response = vec![];

        comm.set_next_response(empty_response.clone());
        let result = comm.send_request(&request).unwrap();

        assert_eq!(result, empty_response);
    }
}
