//! Unit tests for PoKeys library - No hardware required
//!
//! These tests can be run without any PoKeys hardware connected.
//! They test the library's internal logic, data structures, and error handling.
//!

#![allow(clippy::assertions_on_constants)]
//! Run with: cargo test

use pokeys_lib::encoders::{EncoderData, EncoderOptions};
use pokeys_lib::io::{PinCapability, PinData, PinFunction};
use pokeys_lib::keyboard_matrix::MatrixKeyboard;
use pokeys_lib::lcd::LcdData;
use pokeys_lib::pulse_engine::PulseEngineV2;
use pokeys_lib::pwm::PwmData;
use pokeys_lib::sensors::EasySensor;
use pokeys_lib::*;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_library_version() {
        let version = version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));

        // Check version components
        assert_eq!(VERSION_MAJOR, 0);
        assert_eq!(VERSION_MINOR, 3);
        assert_eq!(VERSION_PATCH, 0);

        assert_eq!(version, "0.3.0");
    }

    #[test]
    fn test_device_types() {
        use DeviceTypeId::*;

        // Test device type enum values
        assert_eq!(Device55v1 as u8, 0);
        assert_eq!(Device55v2 as u8, 1);
        assert_eq!(Device56U as u8, 10);
        assert_eq!(Device57U as u8, 30);
        assert_eq!(Device58EU as u8, 40);

        // Test bootloader types
        assert_eq!(Bootloader55 as u8, 3);
        assert_eq!(Bootloader56U as u8, 15);
        assert_eq!(Bootloader58 as u8, 41);
    }

    #[test]
    fn test_connection_types() {
        use ConnectionParam::*;
        use DeviceConnectionType::*;

        assert_eq!(UsbDevice as u8, 0);
        assert_eq!(NetworkDevice as u8, 1);
        assert_eq!(FastUsbDevice as u8, 2);

        assert_eq!(Tcp as u8, 0);
        assert_eq!(Udp as u8, 1);
    }

    #[test]
    fn test_pin_functions() {
        // Test all pin function variants exist
        let _functions = [
            PinFunction::PinRestricted,
            PinFunction::Reserved,
            PinFunction::DigitalInput,
            PinFunction::DigitalOutput,
            PinFunction::AnalogInput,
            PinFunction::AnalogOutput,
            PinFunction::TriggeredInput,
            PinFunction::DigitalCounter,
            PinFunction::InvertPin,
        ];

        // Ensure they can be compared
        assert_ne!(PinFunction::DigitalInput, PinFunction::DigitalOutput);
        assert_eq!(PinFunction::DigitalInput, PinFunction::DigitalInput);
    }

    #[test]
    fn test_pin_capabilities() {
        let caps = [
            PinCapability::DigitalInput,
            PinCapability::DigitalOutput,
            PinCapability::AnalogInput,
            PinCapability::AnalogOutput,
            PinCapability::KeyboardMapping,
            PinCapability::TriggeredInput,
        ];

        // Test that capabilities can be used in collections
        let cap_vec: Vec<_> = caps.iter().collect();
        assert_eq!(cap_vec.len(), caps.len());
    }

    #[test]
    fn test_device_info_default() {
        let info = DeviceInfo::default();

        // Test default values are reasonable
        assert_eq!(info.pin_count, 0);
        assert_eq!(info.pwm_count, 0);
        assert_eq!(info.analog_inputs, 0);
        assert_eq!(info.encoders_count, 0);
        assert_eq!(info.fast_encoders, 0);
        assert_eq!(info.ultra_fast_encoders, 0);
        assert_eq!(info.pwm_internal_frequency, 0);
        assert_eq!(info.prot_i2c, 0);
        assert_eq!(info.prot_1wire, 0);
        assert_eq!(info.additional_options, 0);
    }

    #[test]
    fn test_device_data_default() {
        let data = DeviceData::default();

        assert_eq!(data.device_type_id, 0);
        assert_eq!(data.firmware_version_major, 0);
        assert_eq!(data.firmware_version_minor, 0);
        assert_eq!(data.serial_number, 0);
        assert_eq!(data.device_name(), "");
        assert_eq!(data.user_id, 0);
        assert!(!data.device_locked());
        assert_eq!(data.device_features(), 0);
    }

    #[test]
    fn test_encoder_options() {
        let mut options = EncoderOptions::new();

        // Test default values
        assert!(!options.enabled);
        assert!(!options.sampling_4x);
        assert!(!options.sampling_2x);
        assert!(!options.direct_key_mapping_a);
        assert!(!options.macro_mapping_a);
        assert!(!options.direct_key_mapping_b);
        assert!(!options.macro_mapping_b);

        // Test setting options
        options.enabled = true;
        options.sampling_4x = true;
        options.sampling_2x = true;

        assert!(options.enabled);
        assert!(options.sampling_4x);
        assert!(options.sampling_2x);
    }

    #[test]
    fn test_pwm_data() {
        let pwm_data = PwmData::new();

        // Test initial state - use actual field names and types
        assert_eq!(pwm_data.pwm_period, 0);
        assert_eq!(pwm_data.pwm_values.len(), 6); // Fixed array of 6 channels
        assert_eq!(pwm_data.enabled_channels, 0); // No channels enabled initially

        // Test all channels are initially disabled
        for i in 0..6 {
            assert!(!pwm_data.is_channel_enabled(i));
            assert_eq!(pwm_data.get_duty_cycle(i).unwrap(), 0);
        }
    }

    #[test]
    fn test_pwm_pin_mapping() {
        // Test pin to channel mapping
        assert_eq!(PwmData::pin_to_channel(22).unwrap(), 0); // PWM1
        assert_eq!(PwmData::pin_to_channel(21).unwrap(), 1); // PWM2
        assert_eq!(PwmData::pin_to_channel(20).unwrap(), 2); // PWM3
        assert_eq!(PwmData::pin_to_channel(19).unwrap(), 3); // PWM4
        assert_eq!(PwmData::pin_to_channel(18).unwrap(), 4); // PWM5
        assert_eq!(PwmData::pin_to_channel(17).unwrap(), 5); // PWM6

        // Test invalid pins
        assert!(PwmData::pin_to_channel(16).is_err());
        assert!(PwmData::pin_to_channel(23).is_err());

        // Test channel to pin mapping
        assert_eq!(PwmData::channel_to_pin(0).unwrap(), 22);
        assert_eq!(PwmData::channel_to_pin(1).unwrap(), 21);
        assert_eq!(PwmData::channel_to_pin(2).unwrap(), 20);
        assert_eq!(PwmData::channel_to_pin(3).unwrap(), 19);
        assert_eq!(PwmData::channel_to_pin(4).unwrap(), 18);
        assert_eq!(PwmData::channel_to_pin(5).unwrap(), 17);

        // Test invalid channels
        assert!(PwmData::channel_to_pin(6).is_err());
    }

    #[test]
    fn test_pwm_channel_operations() {
        let mut pwm_data = PwmData::new();

        // Test enabling channels
        pwm_data.set_channel_enabled(0, true).unwrap();
        pwm_data.set_channel_enabled(2, true).unwrap();

        assert!(pwm_data.is_channel_enabled(0));
        assert!(!pwm_data.is_channel_enabled(1));
        assert!(pwm_data.is_channel_enabled(2));
        assert_eq!(pwm_data.enabled_channels, 0b00000101); // Bits 0 and 2 set

        // Test disabling channels
        pwm_data.set_channel_enabled(0, false).unwrap();
        assert!(!pwm_data.is_channel_enabled(0));
        assert_eq!(pwm_data.enabled_channels, 0b00000100); // Only bit 2 set

        // Test duty cycle operations
        pwm_data.set_duty_cycle(1, 1500).unwrap();
        assert_eq!(pwm_data.get_duty_cycle(1).unwrap(), 1500);

        // Test invalid channel operations
        assert!(pwm_data.set_channel_enabled(6, true).is_err());
        assert!(pwm_data.set_duty_cycle(6, 1000).is_err());
        assert!(pwm_data.get_duty_cycle(6).is_err());
    }

    #[test]
    fn test_matrix_keyboard() {
        let keyboard = MatrixKeyboard::new();

        // Test initial state - use actual field names
        assert_eq!(keyboard.configuration, 0);
        assert_eq!(keyboard.width, 0);
        assert_eq!(keyboard.height, 0);
        assert_eq!(keyboard.scanning_decimation, 0);
        assert_eq!(keyboard.column_pins.len(), 8); // Fixed array size
        assert_eq!(keyboard.row_pins.len(), 16); // Fixed array size
    }

    #[test]
    fn test_lcd_data() {
        let lcd = LcdData::new();

        // Test initial state - use actual field names
        assert_eq!(lcd.configuration, 0);
        assert_eq!(lcd.rows, 0);
        assert_eq!(lcd.columns, 0);
        assert_eq!(lcd.row_refresh_flags, 0);
        assert_eq!(lcd.line1.len(), 20); // Fixed array size
        assert_eq!(lcd.line2.len(), 20); // Fixed array size
    }

    #[test]
    fn test_pulse_engine_v2() {
        let pe = PulseEngineV2::new();

        // Test initial state - use actual field names
        // Note: pe.info is a struct, not a simple integer
        assert_eq!(pe.axes_state.len(), 8); // Default number of axes
        assert_eq!(pe.axes_config.len(), 8);
        assert_eq!(pe.emergency_switch_polarity, 0);
    }

    #[test]
    fn test_real_time_clock() {
        let rtc = RealTimeClock::default();

        // Test default values - use actual field names
        assert_eq!(rtc.year, 0);
        assert_eq!(rtc.month, 0);
        assert_eq!(rtc.dom, 0); // day of month
        assert_eq!(rtc.hour, 0);
        assert_eq!(rtc.min, 0); // minute
        assert_eq!(rtc.sec, 0); // second
        assert_eq!(rtc.dow, 0); // day of week
    }

    #[test]
    fn test_error_types() {
        use PoKeysError::*;

        // Test error creation and display
        let errors = [
            DeviceNotFound,
            ConnectionFailed,
            InvalidParameter,
            CommunicationError,
            NotConnected,
            UnsupportedOperation,
            InternalError("test".to_string()),
        ];

        for error in &errors {
            let error_string = format!("{error}");
            assert!(!error_string.is_empty());

            // Test that errors can be compared
            assert_eq!(error, error);
        }
    }

    #[test]
    fn test_error_from_conversions() {
        // Test std::io::Error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let pokeys_error: PoKeysError = io_error.into();

        match pokeys_error {
            PoKeysError::Io(_) => {}
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_buffer_constants() {
        // Test that buffer sizes are reasonable
        assert!(REQUEST_BUFFER_SIZE > 0);
        assert!(RESPONSE_BUFFER_SIZE > 0);
        assert!(REQUEST_BUFFER_SIZE <= 1024);
        assert!(RESPONSE_BUFFER_SIZE <= 1024);

        // Common buffer sizes in PoKeys protocol
        assert!(REQUEST_BUFFER_SIZE >= 64);
        assert!(RESPONSE_BUFFER_SIZE >= 64);
    }

    #[test]
    fn test_pin_data_validation() {
        // Test pin number validation logic
        fn is_valid_pin(pin: u8, max_pins: u8) -> bool {
            pin > 0 && pin <= max_pins
        }

        // Test various scenarios
        assert!(!is_valid_pin(0, 55)); // Pin 0 is invalid
        assert!(is_valid_pin(1, 55)); // Pin 1 is valid
        assert!(is_valid_pin(55, 55)); // Last pin is valid
        assert!(!is_valid_pin(56, 55)); // Beyond max is invalid
    }

    #[test]
    fn test_pwm_duty_cycle_validation() {
        fn is_valid_duty_cycle(duty: f32) -> bool {
            (0.0..=100.0).contains(&duty)
        }

        assert!(is_valid_duty_cycle(0.0));
        assert!(is_valid_duty_cycle(50.0));
        assert!(is_valid_duty_cycle(100.0));
        assert!(!is_valid_duty_cycle(-1.0));
        assert!(!is_valid_duty_cycle(101.0));
    }

    #[test]
    fn test_encoder_value_ranges() {
        // Test encoder value handling
        let max_encoder_value = i32::MAX;
        let min_encoder_value = i32::MIN;

        // Test that encoder values can handle full range
        assert!(max_encoder_value > 0);
        assert!(min_encoder_value < 0);

        // Test overflow behavior
        let test_value = max_encoder_value;
        let wrapped = test_value.wrapping_add(1);
        assert_eq!(wrapped, min_encoder_value);
    }

    #[test]
    fn test_network_device_info() {
        let net_info = NetworkDeviceInfo::default();

        // Test default values - use actual method calls
        assert_eq!(net_info.ip_address(), [0, 0, 0, 0]);
        assert_eq!(net_info.subnet_mask, [0, 0, 0, 0]);
        assert_eq!(net_info.gateway(), [0, 0, 0, 0]);
        assert_eq!(net_info.dns_server(), [0, 0, 0, 0]);
        assert_eq!(net_info.mac_address(), [0; 6]);
        assert_eq!(net_info.device_name(), "");
        assert_eq!(net_info.http_port(), 80);
        assert_eq!(net_info.tcp_port(), 20055);
        assert_eq!(net_info.udp_port(), 20055);
        assert!(!net_info.dhcp_enabled());

        // Note: Cannot test setting values as they are methods, not fields
        // This is a limitation of the current API design
    }

    #[test]
    fn test_easy_sensor() {
        let sensor = EasySensor::new();

        // Test initial state - use actual field names and types
        assert_eq!(sensor.sensor_id, [0; 8]); // sensor_id is an array
        assert_eq!(sensor.sensor_type, 0);
        assert_eq!(sensor.sensor_value, 0); // i32, not f32
        assert_eq!(sensor.sensor_refresh_period, 0); // actual field name
    }

    #[test]
    fn test_data_structure_sizes() {
        use std::mem::size_of;

        // Test that key structures have reasonable sizes
        assert!(size_of::<DeviceInfo>() > 0);
        assert!(size_of::<DeviceData>() > 0);
        assert!(size_of::<PinData>() > 0);
        assert!(size_of::<EncoderData>() > 0);

        // Structures shouldn't be excessively large
        assert!(size_of::<DeviceInfo>() < 1024);
        assert!(size_of::<DeviceData>() < 1024);
    }

    #[test]
    fn test_string_conversions() {
        // Test device name handling
        let device_name = "PoKeys57U";
        assert!(!device_name.is_empty());
        assert!(device_name.len() < 256); // Reasonable length limit

        // Test that device names can contain alphanumeric characters
        assert!(device_name.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_bit_operations() {
        // Test bit manipulation functions that might be used internally
        fn set_bit(value: u8, bit: u8) -> u8 {
            value | (1 << bit)
        }

        fn clear_bit(value: u8, bit: u8) -> u8 {
            value & !(1 << bit)
        }

        fn test_bit(value: u8, bit: u8) -> bool {
            (value & (1 << bit)) != 0
        }

        let mut value = 0u8;

        // Test setting bits
        value = set_bit(value, 0);
        assert!(test_bit(value, 0));
        assert_eq!(value, 1);

        value = set_bit(value, 7);
        assert!(test_bit(value, 7));
        assert_eq!(value, 129); // 0b10000001

        // Test clearing bits
        value = clear_bit(value, 0);
        assert!(!test_bit(value, 0));
        assert_eq!(value, 128); // 0b10000000
    }

    #[test]
    fn test_checksum_calculation() {
        // Test checksum calculation that might be used in protocol
        fn calculate_checksum(data: &[u8]) -> u8 {
            data.iter().fold(0u8, |acc, &x| acc.wrapping_add(x))
        }

        let test_data = [0x01, 0x02, 0x03, 0x04];
        let checksum = calculate_checksum(&test_data);
        assert_eq!(checksum, 10);

        // Test empty data
        let empty_checksum = calculate_checksum(&[]);
        assert_eq!(empty_checksum, 0);

        // Test overflow behavior
        let overflow_data = [0xFF, 0xFF];
        let overflow_checksum = calculate_checksum(&overflow_data);
        assert_eq!(overflow_checksum, 254); // 0xFF + 0xFF = 0x1FE -> 0xFE
    }

    #[test]
    fn test_array_bounds() {
        // Test array access patterns used in the library
        let test_array = [1, 2, 3, 4, 5];

        // Test valid indices
        assert_eq!(test_array[0], 1);
        assert_eq!(test_array[4], 5);

        // Test slice operations
        let slice = &test_array[1..4];
        assert_eq!(slice, &[2, 3, 4]);

        // Test iterator
        let sum: i32 = test_array.iter().sum();
        assert_eq!(sum, 15);
    }
}

#[cfg(test)]
mod network_config_tests {
    use pokeys_lib::{NetworkDeviceConfig, NetworkDeviceInfo};

    #[test]
    fn test_network_device_config_defaults() {
        let cfg = NetworkDeviceConfig::new();
        assert_eq!(cfg.device_info.subnet_mask, [255, 255, 255, 0]);
        assert_eq!(cfg.device_info.tcp_timeout, 1000);
        assert_eq!(cfg.device_info.dhcp, 0);
        assert_eq!(cfg.device_info.additional_network_options, 0xA0);
        assert_eq!(cfg.device_info.ip_address_setup, [0, 0, 0, 0]);
        assert_eq!(cfg.device_info.gateway_ip, [0, 0, 0, 0]);
    }

    #[test]
    fn test_network_device_config_builder() {
        let mut cfg = NetworkDeviceConfig::new();
        cfg.set_ip_address([192, 168, 1, 50]);
        cfg.set_subnet_mask([255, 255, 255, 0]);
        cfg.set_default_gateway([192, 168, 1, 1]);
        cfg.set_dhcp(false);
        cfg.set_tcp_timeout(2000);
        cfg.set_network_options(false, false, false);

        assert_eq!(cfg.device_info.ip_address_setup, [192, 168, 1, 50]);
        assert_eq!(cfg.device_info.subnet_mask, [255, 255, 255, 0]);
        assert_eq!(cfg.device_info.gateway_ip, [192, 168, 1, 1]);
        assert_eq!(cfg.device_info.dhcp, 0);
        assert_eq!(cfg.device_info.tcp_timeout, 2000);
    }

    #[test]
    fn test_dhcp_toggle() {
        let mut cfg = NetworkDeviceConfig::new();
        cfg.set_dhcp(true);
        assert_eq!(cfg.device_info.dhcp, 1);
        cfg.set_dhcp(false);
        assert_eq!(cfg.device_info.dhcp, 0);
    }

    #[test]
    fn test_network_options_flags() {
        let mut cfg = NetworkDeviceConfig::new();

        cfg.set_network_options(true, false, false);
        assert_eq!(cfg.device_info.additional_network_options, 0xA0 | 0x01);

        cfg.set_network_options(false, true, false);
        assert_eq!(cfg.device_info.additional_network_options, 0xA0 | 0x02);

        cfg.set_network_options(false, false, true);
        assert_eq!(cfg.device_info.additional_network_options, 0xA0 | 0x04);

        cfg.set_network_options(true, true, true);
        assert_eq!(cfg.device_info.additional_network_options, 0xA0 | 0x07);

        cfg.set_network_options(false, false, false);
        assert_eq!(cfg.device_info.additional_network_options, 0xA0);
    }

    #[test]
    fn test_tcp_timeout_unit_conversion() {
        // Wire encoding: ms / 100, minimum 1
        let cases: &[(u16, u16)] = &[
            (1000, 10),
            (100, 1),
            (500, 5),
            (0, 0), // saturating_mul(100) when reading back handles the 0 edge
        ];
        for &(input_ms, expected_units) in cases {
            let units = (input_ms / 100).max(if input_ms == 0 { 0 } else { 1 });
            assert_eq!(units, expected_units, "input_ms={input_ms}");
        }
    }

    #[test]
    fn test_gateway_subnet_set_flag() {
        // Non-zero gateway → flag should be 1
        let info = NetworkDeviceInfo {
            gateway_ip: [192, 168, 1, 1],
            subnet_mask: [0, 0, 0, 0],
            ..Default::default()
        };
        let flag = if info.gateway_ip != [0, 0, 0, 0] || info.subnet_mask != [0, 0, 0, 0] {
            1u8
        } else {
            0u8
        };
        assert_eq!(flag, 1);

        // All-zero → flag should be 0
        let info2 = NetworkDeviceInfo::default();
        let flag2 = if info2.gateway_ip != [0, 0, 0, 0] || info2.subnet_mask != [0, 0, 0, 0] {
            1u8
        } else {
            0u8
        };
        assert_eq!(flag2, 0);
    }

    #[test]
    fn test_additional_options_nibble() {
        // Upper nibble must always be 0xA regardless of lower-nibble input
        for lower in 0u8..=0x0F {
            let options = (lower & 0x0F) | 0xA0;
            assert_eq!(
                options >> 4,
                0xA,
                "upper nibble not 0xA for lower={lower:#x}"
            );
            assert_eq!(options & 0x0F, lower);
        }
    }

    #[test]
    fn test_device_name_truncation() {
        let long_name = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; // 26 chars, max is 20
        let truncated = if long_name.len() > 20 {
            &long_name[..20]
        } else {
            long_name
        };
        assert_eq!(truncated.len(), 20);
        assert_eq!(truncated, "ABCDEFGHIJKLMNOPQRST");

        let short_name = "MyDevice"; // 8 chars, no truncation
        let not_truncated = if short_name.len() > 20 {
            &short_name[..20]
        } else {
            short_name
        };
        assert_eq!(not_truncated, "MyDevice");
    }

    #[test]
    fn test_network_device_info_dhcp_enabled() {
        let mut info = NetworkDeviceInfo::default();
        assert!(!info.dhcp_enabled());
        info.dhcp = 1;
        assert!(info.dhcp_enabled());
        info.dhcp = 0;
        assert!(!info.dhcp_enabled());
    }
}

/// Mock tests for testing without hardware
#[cfg(test)]
mod mock_tests {
    use super::*;

    /// Mock device for testing
    struct MockDevice {
        connected: bool,
        pin_states: std::collections::HashMap<u8, bool>,
        analog_values: std::collections::HashMap<u8, u16>,
        encoder_values: std::collections::HashMap<u8, i32>,
    }

    impl MockDevice {
        fn new() -> Self {
            Self {
                connected: false,
                pin_states: std::collections::HashMap::new(),
                analog_values: std::collections::HashMap::new(),
                encoder_values: std::collections::HashMap::new(),
            }
        }

        fn connect(&mut self) -> Result<()> {
            self.connected = true;
            Ok(())
        }

        fn disconnect(&mut self) {
            self.connected = false;
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn set_digital_output(&mut self, pin: u8, state: bool) -> Result<()> {
            if !self.connected {
                return Err(PoKeysError::NotConnected);
            }
            if pin == 0 || pin > 55 {
                return Err(PoKeysError::InvalidParameter);
            }
            self.pin_states.insert(pin, state);
            Ok(())
        }

        fn get_digital_input(&self, pin: u8) -> Result<bool> {
            if !self.connected {
                return Err(PoKeysError::NotConnected);
            }
            if pin == 0 || pin > 55 {
                return Err(PoKeysError::InvalidParameter);
            }
            Ok(self.pin_states.get(&pin).copied().unwrap_or(false))
        }

        fn get_analog_input(&self, pin: u8) -> Result<u16> {
            if !self.connected {
                return Err(PoKeysError::NotConnected);
            }
            if pin == 0 || pin > 8 {
                return Err(PoKeysError::InvalidParameter);
            }
            Ok(self.analog_values.get(&pin).copied().unwrap_or(0))
        }

        fn set_encoder_value(&mut self, encoder: u8, value: i32) {
            self.encoder_values.insert(encoder, value);
        }

        fn get_encoder_value(&self, encoder: u8) -> Result<i32> {
            if !self.connected {
                return Err(PoKeysError::NotConnected);
            }
            if encoder >= 25 {
                return Err(PoKeysError::InvalidParameter);
            }
            Ok(self.encoder_values.get(&encoder).copied().unwrap_or(0))
        }
    }

    #[test]
    fn test_mock_device_connection() {
        let mut device = MockDevice::new();

        assert!(!device.is_connected());

        device.connect().unwrap();
        assert!(device.is_connected());

        device.disconnect();
        assert!(!device.is_connected());
    }

    #[test]
    fn test_mock_digital_io() {
        let mut device = MockDevice::new();
        device.connect().unwrap();

        // Test setting and reading digital outputs
        device.set_digital_output(1, true).unwrap();
        assert!(device.get_digital_input(1).unwrap());

        device.set_digital_output(1, false).unwrap();
        assert!(!device.get_digital_input(1).unwrap());

        // Test multiple pins
        device.set_digital_output(5, true).unwrap();
        device.set_digital_output(10, false).unwrap();

        assert!(device.get_digital_input(5).unwrap());
        assert!(!device.get_digital_input(10).unwrap());
    }

    #[test]
    fn test_mock_error_handling() {
        let mut device = MockDevice::new();

        // Test operations on disconnected device
        assert!(device.set_digital_output(1, true).is_err());
        assert!(device.get_digital_input(1).is_err());
        assert!(device.get_analog_input(1).is_err());
        assert!(device.get_encoder_value(0).is_err());

        device.connect().unwrap();

        // Test invalid parameters
        assert!(device.set_digital_output(0, true).is_err());
        assert!(device.set_digital_output(56, true).is_err());
        assert!(device.get_digital_input(0).is_err());
        assert!(device.get_digital_input(56).is_err());
        assert!(device.get_analog_input(0).is_err());
        assert!(device.get_analog_input(9).is_err());
        assert!(device.get_encoder_value(25).is_err());
    }

    #[test]
    fn test_mock_analog_inputs() {
        let mut device = MockDevice::new();
        device.connect().unwrap();

        // Test default analog values
        for pin in 1..=8 {
            assert_eq!(device.get_analog_input(pin).unwrap(), 0);
        }

        // Test setting analog values (simulating external input)
        device.analog_values.insert(1, 2048); // Mid-scale
        device.analog_values.insert(2, 4095); // Full-scale

        assert_eq!(device.get_analog_input(1).unwrap(), 2048);
        assert_eq!(device.get_analog_input(2).unwrap(), 4095);
    }

    #[test]
    fn test_mock_encoders() {
        let mut device = MockDevice::new();
        device.connect().unwrap();

        // Test default encoder values
        for encoder in 0..25 {
            assert_eq!(device.get_encoder_value(encoder).unwrap(), 0);
        }

        // Test setting encoder values
        device.set_encoder_value(0, 100);
        device.set_encoder_value(1, -50);
        device.set_encoder_value(2, i32::MAX);
        device.set_encoder_value(3, i32::MIN);

        assert_eq!(device.get_encoder_value(0).unwrap(), 100);
        assert_eq!(device.get_encoder_value(1).unwrap(), -50);
        assert_eq!(device.get_encoder_value(2).unwrap(), i32::MAX);
        assert_eq!(device.get_encoder_value(3).unwrap(), i32::MIN);
    }

    #[test]
    fn test_mock_state_persistence() {
        let mut device = MockDevice::new();
        device.connect().unwrap();

        // Set various states
        device.set_digital_output(1, true).unwrap();
        device.set_digital_output(2, false).unwrap();
        device.analog_values.insert(1, 1234);
        device.set_encoder_value(0, 567);

        // Verify states persist
        assert!(device.get_digital_input(1).unwrap());
        assert!(!device.get_digital_input(2).unwrap());
        assert_eq!(device.get_analog_input(1).unwrap(), 1234);
        assert_eq!(device.get_encoder_value(0).unwrap(), 567);

        // Disconnect and reconnect
        device.disconnect();
        device.connect().unwrap();

        // States should still be there (in this mock implementation)
        assert!(device.get_digital_input(1).unwrap());
        assert!(!device.get_digital_input(2).unwrap());
        assert_eq!(device.get_analog_input(1).unwrap(), 1234);
        assert_eq!(device.get_encoder_value(0).unwrap(), 567);
    }

    #[test]
    fn test_servo_types() {
        // Test 180-degree servo
        let servo_180 = ServoConfig::one_eighty(22, 25000, 50000);
        assert_eq!(servo_180.pin, 22);
        match servo_180.servo_type {
            ServoType::OneEighty { pos_0, pos_180 } => {
                assert_eq!(pos_0, 25000);
                assert_eq!(pos_180, 50000);
            }
            _ => panic!("Expected OneEighty servo type"),
        }

        // Test 360-degree position servo
        let servo_360_pos = ServoConfig::three_sixty_position(21, 30000, 60000);
        assert_eq!(servo_360_pos.pin, 21);
        match servo_360_pos.servo_type {
            ServoType::ThreeSixtyPosition { pos_0, pos_360 } => {
                assert_eq!(pos_0, 30000);
                assert_eq!(pos_360, 60000);
            }
            _ => panic!("Expected ThreeSixtyPosition servo type"),
        }

        // Test 360-degree speed servo
        let servo_360_speed = ServoConfig::three_sixty_speed(20, 37500, 50000, 25000);
        assert_eq!(servo_360_speed.pin, 20);
        match servo_360_speed.servo_type {
            ServoType::ThreeSixtySpeed {
                stop,
                clockwise,
                anti_clockwise,
            } => {
                assert_eq!(stop, 37500);
                assert_eq!(clockwise, 50000);
                assert_eq!(anti_clockwise, 25000);
            }
            _ => panic!("Expected ThreeSixtySpeed servo type"),
        }
    }

    #[test]
    fn test_servo_angle_calculations() {
        // Test 180-degree servo angle validation
        let servo_180 = ServoConfig::one_eighty(22, 25000, 50000);

        // Test angle range calculations
        match servo_180.servo_type {
            ServoType::OneEighty { pos_0, pos_180 } => {
                let range = pos_180 as f32 - pos_0 as f32;
                let angle_90 = (pos_0 as f32 + (90.0 / 180.0) * range) as u32;
                assert_eq!(angle_90, 37500); // Should be midpoint
            }
            _ => panic!("Expected OneEighty servo type"),
        }

        // Test 360-degree position servo calculations
        let servo_360 = ServoConfig::three_sixty_position(21, 30000, 60000);

        match servo_360.servo_type {
            ServoType::ThreeSixtyPosition { pos_0, pos_360 } => {
                let range = pos_360 as f32 - pos_0 as f32;
                let angle_180 = (pos_0 as f32 + (180.0 / 360.0) * range) as u32;
                assert_eq!(angle_180, 45000); // Should be midpoint
            }
            _ => panic!("Expected ThreeSixtyPosition servo type"),
        }
    }

    #[test]
    fn test_servo_speed_calculations() {
        let servo_speed = ServoConfig::three_sixty_speed(20, 37500, 50000, 25000);

        match servo_speed.servo_type {
            ServoType::ThreeSixtySpeed {
                stop,
                clockwise,
                anti_clockwise,
            } => {
                // Test speed calculations
                let cw_range = clockwise as f32 - stop as f32;
                let acw_range = anti_clockwise as f32 - stop as f32;

                // 50% clockwise should be halfway between stop and clockwise
                let speed_50_cw = (stop as f32 + (50.0 / 100.0) * cw_range) as u32;
                assert_eq!(speed_50_cw, 43750);

                // 50% anti-clockwise should be halfway between stop and anti_clockwise
                let speed_50_acw = (stop as f32 + (50.0 / 100.0) * acw_range) as u32;
                assert_eq!(speed_50_acw, 31250);
            }
            _ => panic!("Expected ThreeSixtySpeed servo type"),
        }
    }

    #[test]
    fn test_servo_type_validation() {
        // Test that servo types are correctly identified
        let servo_180 = ServoConfig::one_eighty(22, 25000, 50000);
        let servo_360_pos = ServoConfig::three_sixty_position(21, 30000, 60000);
        let servo_360_speed = ServoConfig::three_sixty_speed(20, 37500, 50000, 25000);

        // Verify servo types
        assert!(matches!(servo_180.servo_type, ServoType::OneEighty { .. }));
        assert!(matches!(
            servo_360_pos.servo_type,
            ServoType::ThreeSixtyPosition { .. }
        ));
        assert!(matches!(
            servo_360_speed.servo_type,
            ServoType::ThreeSixtySpeed { .. }
        ));

        // Verify pins
        assert_eq!(servo_180.pin, 22);
        assert_eq!(servo_360_pos.pin, 21);
        assert_eq!(servo_360_speed.pin, 20);
    }
}
