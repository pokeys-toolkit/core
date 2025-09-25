//! Integration tests - No hardware required
//!
//! These tests verify the integration between different modules
//! and components of the library without requiring hardware.

use pokeys_lib::encoders::EncoderOptions;
use pokeys_lib::*;
use std::collections::HashMap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Mock device manager for integration testing
    struct MockDeviceManager {
        devices: HashMap<u32, MockDeviceInfo>,
        next_device_id: u32,
    }

    #[derive(Clone)]
    struct MockDeviceInfo {
        device_type: DeviceTypeId,
        serial_number: u32,
        firmware_major: u8,
        firmware_minor: u8,
        pin_count: u8,
        pwm_count: u8,
        analog_inputs: u8,
        encoders_count: u8,
        connected: bool,
    }

    impl MockDeviceManager {
        fn new() -> Self {
            Self {
                devices: HashMap::new(),
                next_device_id: 1,
            }
        }

        fn add_device(&mut self, device_type: DeviceTypeId) -> u32 {
            let device_id = self.next_device_id;
            self.next_device_id += 1;

            let (pin_count, pwm_count, analog_inputs, encoders_count) = match device_type {
                DeviceTypeId::Device55v3 => (55, 6, 8, 25),
                DeviceTypeId::Device56U => (55, 6, 8, 25),
                DeviceTypeId::Device57U => (55, 6, 8, 25),
                DeviceTypeId::Device58EU => (55, 6, 8, 25),
                _ => (55, 6, 8, 25), // Default values
            };

            let device_info = MockDeviceInfo {
                device_type,
                serial_number: 1000000 + device_id,
                firmware_major: 4,
                firmware_minor: 1,
                pin_count,
                pwm_count,
                analog_inputs,
                encoders_count,
                connected: false,
            };

            self.devices.insert(device_id, device_info);
            device_id
        }

        fn enumerate_devices(&self) -> Vec<u32> {
            self.devices.keys().cloned().collect()
        }

        fn connect_device(&mut self, device_id: u32) -> Result<()> {
            if let Some(device) = self.devices.get_mut(&device_id) {
                device.connected = true;
                Ok(())
            } else {
                Err(PoKeysError::DeviceNotFound)
            }
        }

        fn disconnect_device(&mut self, device_id: u32) -> Result<()> {
            if let Some(device) = self.devices.get_mut(&device_id) {
                device.connected = false;
                Ok(())
            } else {
                Err(PoKeysError::DeviceNotFound)
            }
        }

        fn get_device_info(&self, device_id: u32) -> Result<MockDeviceInfo> {
            self.devices
                .get(&device_id)
                .cloned()
                .ok_or(PoKeysError::DeviceNotFound)
        }

        fn is_device_connected(&self, device_id: u32) -> bool {
            self.devices
                .get(&device_id)
                .map(|d| d.connected)
                .unwrap_or(false)
        }
    }

    #[test]
    fn test_device_enumeration_integration() {
        let mut manager = MockDeviceManager::new();

        // Initially no devices
        assert_eq!(manager.enumerate_devices().len(), 0);

        // Add some devices
        let device1 = manager.add_device(DeviceTypeId::Device57U);
        let device2 = manager.add_device(DeviceTypeId::Device56U);
        let device3 = manager.add_device(DeviceTypeId::Device58EU);

        let devices = manager.enumerate_devices();
        assert_eq!(devices.len(), 3);
        assert!(devices.contains(&device1));
        assert!(devices.contains(&device2));
        assert!(devices.contains(&device3));

        // Test all fields of MockDeviceInfo
        let info1 = manager.get_device_info(device1).unwrap();
        assert_eq!(info1.device_type, DeviceTypeId::Device57U);
        assert_eq!(info1.serial_number, 1000000 + device1); // Test serial_number field
        assert_eq!(info1.firmware_major, 4);
        assert_eq!(info1.firmware_minor, 1);
        assert_eq!(info1.pin_count, 55);
        assert_eq!(info1.pwm_count, 6);
        assert_eq!(info1.analog_inputs, 8);
        assert_eq!(info1.encoders_count, 25);
        assert!(!info1.connected); // Initially not connected

        let info2 = manager.get_device_info(device2).unwrap();
        assert_eq!(info2.device_type, DeviceTypeId::Device56U);
        assert_eq!(info2.serial_number, 1000000 + device2); // Test serial_number field

        let info3 = manager.get_device_info(device3).unwrap();
        assert_eq!(info3.device_type, DeviceTypeId::Device58EU);
        assert_eq!(info3.serial_number, 1000000 + device3); // Test serial_number field
    }

    #[test]
    fn test_device_connection_lifecycle() {
        let mut manager = MockDeviceManager::new();
        let device_id = manager.add_device(DeviceTypeId::Device57U);

        // Initially disconnected
        assert!(!manager.is_device_connected(device_id));

        // Connect
        manager.connect_device(device_id).unwrap();
        assert!(manager.is_device_connected(device_id));

        // Disconnect
        manager.disconnect_device(device_id).unwrap();
        assert!(!manager.is_device_connected(device_id));

        // Test connecting non-existent device
        assert!(manager.connect_device(999).is_err());
    }

    #[test]
    fn test_device_info_retrieval() {
        let mut manager = MockDeviceManager::new();
        let device_id = manager.add_device(DeviceTypeId::Device57U);

        let info = manager.get_device_info(device_id).unwrap();
        assert_eq!(info.device_type, DeviceTypeId::Device57U);
        assert_eq!(info.pin_count, 55);
        assert_eq!(info.pwm_count, 6);
        assert_eq!(info.analog_inputs, 8);
        assert_eq!(info.encoders_count, 25);
        assert_eq!(info.firmware_major, 4);
        assert_eq!(info.firmware_minor, 1);

        // Test getting info for non-existent device
        assert!(manager.get_device_info(999).is_err());
    }

    #[test]
    fn test_multiple_device_management() {
        let mut manager = MockDeviceManager::new();

        // Add different device types
        let device_types = [
            DeviceTypeId::Device55v3,
            DeviceTypeId::Device56U,
            DeviceTypeId::Device57U,
            DeviceTypeId::Device58EU,
        ];

        let mut device_ids = Vec::new();
        for &device_type in &device_types {
            let id = manager.add_device(device_type);
            device_ids.push(id);
        }

        // Connect all devices
        for &id in &device_ids {
            manager.connect_device(id).unwrap();
            assert!(manager.is_device_connected(id));
        }

        // Verify all devices are connected
        for &id in &device_ids {
            assert!(manager.is_device_connected(id));
        }

        // Disconnect specific devices
        manager.disconnect_device(device_ids[0]).unwrap();
        manager.disconnect_device(device_ids[2]).unwrap();

        assert!(!manager.is_device_connected(device_ids[0]));
        assert!(manager.is_device_connected(device_ids[1]));
        assert!(!manager.is_device_connected(device_ids[2]));
        assert!(manager.is_device_connected(device_ids[3]));
    }

    /// Mock I/O manager for testing I/O operations
    struct MockIOManager {
        pin_functions: HashMap<u8, PinFunction>,
        digital_outputs: HashMap<u8, bool>,
        digital_inputs: HashMap<u8, bool>,
        analog_inputs: HashMap<u8, u16>,
        pin_capabilities: HashMap<u8, Vec<PinCapability>>,
        max_pins: u8,
    }

    impl MockIOManager {
        fn new(max_pins: u8) -> Self {
            let mut manager = Self {
                pin_functions: HashMap::new(),
                digital_outputs: HashMap::new(),
                digital_inputs: HashMap::new(),
                analog_inputs: HashMap::new(),
                pin_capabilities: HashMap::new(),
                max_pins,
            };

            // Initialize pin capabilities
            for pin in 1..=max_pins {
                let mut caps = vec![PinCapability::DigitalInput, PinCapability::DigitalOutput];

                // First 8 pins can be analog inputs
                if pin <= 8 {
                    caps.push(PinCapability::AnalogInput);
                }

                manager.pin_capabilities.insert(pin, caps);
            }

            manager
        }

        fn set_pin_function(&mut self, pin: u8, function: PinFunction) -> Result<()> {
            if pin == 0 || pin > self.max_pins {
                return Err(PoKeysError::InvalidParameter);
            }

            // Check if pin supports this function
            let caps = self.pin_capabilities.get(&pin).unwrap();
            let required_cap = match function {
                PinFunction::DigitalInput => PinCapability::DigitalInput,
                PinFunction::DigitalOutput => PinCapability::DigitalOutput,
                PinFunction::AnalogInput => PinCapability::AnalogInput,
                PinFunction::AnalogOutput => PinCapability::AnalogOutput,
                PinFunction::TriggeredInput => PinCapability::TriggeredInput,
                _ => return Ok(()), // Other functions always allowed
            };

            if !caps.contains(&required_cap) {
                return Err(PoKeysError::UnsupportedOperation);
            }

            self.pin_functions.insert(pin, function);
            Ok(())
        }

        fn get_pin_function(&self, pin: u8) -> Result<PinFunction> {
            if pin == 0 || pin > self.max_pins {
                return Err(PoKeysError::InvalidParameter);
            }

            Ok(self
                .pin_functions
                .get(&pin)
                .copied()
                .unwrap_or(PinFunction::PinRestricted))
        }

        fn set_digital_output(&mut self, pin: u8, state: bool) -> Result<()> {
            if pin == 0 || pin > self.max_pins {
                return Err(PoKeysError::InvalidParameter);
            }

            let function = self.get_pin_function(pin)?;
            if function != PinFunction::DigitalOutput {
                return Err(PoKeysError::UnsupportedOperation);
            }

            self.digital_outputs.insert(pin, state);
            Ok(())
        }

        fn get_digital_input(&self, pin: u8) -> Result<bool> {
            if pin == 0 || pin > self.max_pins {
                return Err(PoKeysError::InvalidParameter);
            }

            let function = self.get_pin_function(pin)?;
            if function != PinFunction::DigitalInput {
                return Err(PoKeysError::UnsupportedOperation);
            }

            Ok(self.digital_inputs.get(&pin).copied().unwrap_or(false))
        }

        fn get_analog_input(&self, pin: u8) -> Result<u16> {
            if pin == 0 || pin > 8 {
                return Err(PoKeysError::InvalidParameter);
            }

            let function = self.get_pin_function(pin)?;
            if function != PinFunction::AnalogInput {
                return Err(PoKeysError::UnsupportedOperation);
            }

            Ok(self.analog_inputs.get(&pin).copied().unwrap_or(0))
        }

        fn simulate_digital_input(&mut self, pin: u8, state: bool) {
            self.digital_inputs.insert(pin, state);
        }

        fn simulate_analog_input(&mut self, pin: u8, value: u16) {
            self.analog_inputs.insert(pin, value);
        }
    }

    #[test]
    fn test_io_manager_pin_configuration() {
        let mut io = MockIOManager::new(55);

        // Test setting pin functions
        io.set_pin_function(1, PinFunction::DigitalOutput).unwrap();
        io.set_pin_function(2, PinFunction::DigitalInput).unwrap();
        io.set_pin_function(3, PinFunction::AnalogInput).unwrap();

        assert_eq!(io.get_pin_function(1).unwrap(), PinFunction::DigitalOutput);
        assert_eq!(io.get_pin_function(2).unwrap(), PinFunction::DigitalInput);
        assert_eq!(io.get_pin_function(3).unwrap(), PinFunction::AnalogInput);

        // Test invalid pin numbers
        assert!(io.set_pin_function(0, PinFunction::DigitalOutput).is_err());
        assert!(io.set_pin_function(56, PinFunction::DigitalOutput).is_err());

        // Test unsupported function (analog input on pin > 8)
        assert!(io.set_pin_function(10, PinFunction::AnalogInput).is_err());
    }

    #[test]
    fn test_io_manager_digital_operations() {
        let mut io = MockIOManager::new(55);

        // Configure pins
        io.set_pin_function(1, PinFunction::DigitalOutput).unwrap();
        io.set_pin_function(2, PinFunction::DigitalInput).unwrap();

        // Test digital output
        io.set_digital_output(1, true).unwrap();
        io.set_digital_output(1, false).unwrap();

        // Test digital input
        io.simulate_digital_input(2, true);
        assert!(io.get_digital_input(2).unwrap());

        io.simulate_digital_input(2, false);
        assert!(!io.get_digital_input(2).unwrap());

        // Test operations on wrong pin types
        assert!(io.set_digital_output(2, true).is_err()); // Input pin
        assert!(io.get_digital_input(1).is_err()); // Output pin
    }

    #[test]
    fn test_io_manager_analog_operations() {
        let mut io = MockIOManager::new(55);

        // Configure analog input pins
        for pin in 1..=8 {
            io.set_pin_function(pin, PinFunction::AnalogInput).unwrap();
        }

        // Test analog inputs
        let test_values = [0, 1024, 2048, 3072, 4095];
        for (i, &value) in test_values.iter().enumerate() {
            let pin = (i + 1) as u8;
            io.simulate_analog_input(pin, value);
            assert_eq!(io.get_analog_input(pin).unwrap(), value);
        }

        // Test invalid analog pin
        assert!(io.get_analog_input(9).is_err());

        // Test analog operation on non-analog pin
        io.set_pin_function(1, PinFunction::DigitalOutput).unwrap();
        assert!(io.get_analog_input(1).is_err());
    }

    /// Mock PWM manager for testing PWM operations
    struct MockPWMManager {
        channels: HashMap<u8, PWMChannel>,
        frequency: u32,
        max_channels: u8,
    }

    #[derive(Clone)]
    struct PWMChannel {
        enabled: bool,
        duty_cycle: f32,
        pin: u8,
    }

    impl MockPWMManager {
        fn new(max_channels: u8) -> Self {
            Self {
                channels: HashMap::new(),
                frequency: 1000, // Default 1kHz
                max_channels,
            }
        }

        fn set_frequency(&mut self, freq: u32) -> Result<()> {
            if freq == 0 || freq > 100000 {
                return Err(PoKeysError::InvalidParameter);
            }
            self.frequency = freq;
            Ok(())
        }

        fn get_frequency(&self) -> u32 {
            self.frequency
        }

        fn configure_channel(&mut self, channel: u8, pin: u8) -> Result<()> {
            if channel >= self.max_channels {
                return Err(PoKeysError::InvalidParameter);
            }

            let pwm_channel = PWMChannel {
                enabled: false,
                duty_cycle: 0.0,
                pin,
            };

            self.channels.insert(channel, pwm_channel);
            Ok(())
        }

        fn set_duty_cycle(&mut self, channel: u8, duty: f32) -> Result<()> {
            if channel >= self.max_channels {
                return Err(PoKeysError::InvalidParameter);
            }

            if !(0.0..=100.0).contains(&duty) {
                return Err(PoKeysError::InvalidParameter);
            }

            if let Some(ch) = self.channels.get_mut(&channel) {
                ch.duty_cycle = duty;
                Ok(())
            } else {
                Err(PoKeysError::UnsupportedOperation)
            }
        }

        fn enable_channel(&mut self, channel: u8, enabled: bool) -> Result<()> {
            if channel >= self.max_channels {
                return Err(PoKeysError::InvalidParameter);
            }

            if let Some(ch) = self.channels.get_mut(&channel) {
                ch.enabled = enabled;
                Ok(())
            } else {
                Err(PoKeysError::UnsupportedOperation)
            }
        }

        fn get_channel_info(&self, channel: u8) -> Result<PWMChannel> {
            if channel >= self.max_channels {
                return Err(PoKeysError::InvalidParameter);
            }

            self.channels
                .get(&channel)
                .cloned()
                .ok_or(PoKeysError::UnsupportedOperation)
        }

        fn get_channel_pin(&self, channel: u8) -> Result<u8> {
            let channel_info = self.get_channel_info(channel)?;
            Ok(channel_info.pin)
        }

        fn has_pin_conflict(&self, pin1: u8, pin2: u8) -> bool {
            pin1 != pin2
        }
    }

    #[test]
    fn test_pwm_manager_basic_operations() {
        let mut pwm = MockPWMManager::new(6);

        // Test frequency setting
        pwm.set_frequency(2000).unwrap();
        assert_eq!(pwm.get_frequency(), 2000);

        // Test invalid frequencies
        assert!(pwm.set_frequency(0).is_err());
        assert!(pwm.set_frequency(200000).is_err());

        // Test channel configuration with different pins
        pwm.configure_channel(0, 10).unwrap();
        pwm.configure_channel(1, 11).unwrap();
        pwm.configure_channel(2, 12).unwrap();

        // Test that pin assignments are stored correctly
        assert_eq!(pwm.get_channel_pin(0).unwrap(), 10);
        assert_eq!(pwm.get_channel_pin(1).unwrap(), 11);
        assert_eq!(pwm.get_channel_pin(2).unwrap(), 12);

        // Test invalid channel
        assert!(pwm.configure_channel(6, 10).is_err());
        assert!(pwm.get_channel_pin(6).is_err());

        // Test pin conflict detection
        assert!(pwm.has_pin_conflict(10, 11));
        assert!(!pwm.has_pin_conflict(10, 10));
    }

    #[test]
    fn test_pwm_manager_duty_cycle_control() {
        let mut pwm = MockPWMManager::new(6);

        // Configure channel
        pwm.configure_channel(0, 10).unwrap();

        // Test duty cycle setting
        let duty_cycles = [0.0, 25.0, 50.0, 75.0, 100.0];
        for &duty in &duty_cycles {
            pwm.set_duty_cycle(0, duty).unwrap();
            let info = pwm.get_channel_info(0).unwrap();
            assert_eq!(info.duty_cycle, duty);
        }

        // Test invalid duty cycles
        assert!(pwm.set_duty_cycle(0, -1.0).is_err());
        assert!(pwm.set_duty_cycle(0, 101.0).is_err());

        // Test unconfigured channel
        assert!(pwm.set_duty_cycle(1, 50.0).is_err());
    }

    #[test]
    fn test_pwm_manager_enable_disable() {
        let mut pwm = MockPWMManager::new(6);

        // Configure channel
        pwm.configure_channel(0, 10).unwrap();
        pwm.set_duty_cycle(0, 50.0).unwrap();

        // Initially disabled
        let info = pwm.get_channel_info(0).unwrap();
        assert!(!info.enabled);

        // Enable channel
        pwm.enable_channel(0, true).unwrap();
        let info = pwm.get_channel_info(0).unwrap();
        assert!(info.enabled);

        // Disable channel
        pwm.enable_channel(0, false).unwrap();
        let info = pwm.get_channel_info(0).unwrap();
        assert!(!info.enabled);
    }

    /// Mock encoder manager for testing encoder operations
    struct MockEncoderManager {
        encoders: HashMap<u8, EncoderState>,
        max_encoders: u8,
    }

    #[derive(Clone)]
    struct EncoderState {
        configured: bool,
        pin_a: u8,
        pin_b: u8,
        value: i32,
        options: EncoderOptions,
    }

    impl MockEncoderManager {
        fn new(max_encoders: u8) -> Self {
            Self {
                encoders: HashMap::new(),
                max_encoders,
            }
        }

        fn configure_encoder(
            &mut self,
            encoder: u8,
            pin_a: u8,
            pin_b: u8,
            options: EncoderOptions,
        ) -> Result<()> {
            if encoder >= self.max_encoders {
                return Err(PoKeysError::InvalidParameter);
            }

            let state = EncoderState {
                configured: true,
                pin_a,
                pin_b,
                value: 0,
                options,
            };

            self.encoders.insert(encoder, state);
            Ok(())
        }

        fn get_encoder_value(&self, encoder: u8) -> Result<i32> {
            if encoder >= self.max_encoders {
                return Err(PoKeysError::InvalidParameter);
            }

            if let Some(state) = self.encoders.get(&encoder) {
                if state.configured {
                    Ok(state.value)
                } else {
                    Err(PoKeysError::UnsupportedOperation)
                }
            } else {
                Err(PoKeysError::UnsupportedOperation)
            }
        }

        fn set_encoder_value(&mut self, encoder: u8, value: i32) -> Result<()> {
            if encoder >= self.max_encoders {
                return Err(PoKeysError::InvalidParameter);
            }

            if let Some(state) = self.encoders.get_mut(&encoder) {
                if state.configured {
                    state.value = value;
                    Ok(())
                } else {
                    Err(PoKeysError::UnsupportedOperation)
                }
            } else {
                Err(PoKeysError::UnsupportedOperation)
            }
        }

        fn simulate_encoder_movement(&mut self, encoder: u8, delta: i32) -> Result<()> {
            if let Some(state) = self.encoders.get_mut(&encoder) {
                if state.configured {
                    state.value = state.value.wrapping_add(delta);
                    Ok(())
                } else {
                    Err(PoKeysError::UnsupportedOperation)
                }
            } else {
                Err(PoKeysError::UnsupportedOperation)
            }
        }

        fn get_encoder_pins(&self, encoder: u8) -> Result<(u8, u8)> {
            if encoder >= self.max_encoders {
                return Err(PoKeysError::InvalidParameter);
            }

            if let Some(state) = self.encoders.get(&encoder) {
                if state.configured {
                    Ok((state.pin_a, state.pin_b))
                } else {
                    Err(PoKeysError::UnsupportedOperation)
                }
            } else {
                Err(PoKeysError::UnsupportedOperation)
            }
        }

        fn get_encoder_options(&self, encoder: u8) -> Result<EncoderOptions> {
            if encoder >= self.max_encoders {
                return Err(PoKeysError::InvalidParameter);
            }

            if let Some(state) = self.encoders.get(&encoder) {
                if state.configured {
                    Ok(state.options)
                } else {
                    Err(PoKeysError::UnsupportedOperation)
                }
            } else {
                Err(PoKeysError::UnsupportedOperation)
            }
        }

        fn is_encoder_configured(&self, encoder: u8) -> Result<bool> {
            if encoder >= self.max_encoders {
                return Err(PoKeysError::InvalidParameter);
            }

            Ok(self
                .encoders
                .get(&encoder)
                .is_some_and(|state| state.configured))
        }
    }

    #[test]
    fn test_encoder_manager_configuration() {
        let mut encoders = MockEncoderManager::new(25);

        // Configure encoders with different pins and options
        let mut options1 = EncoderOptions::new();
        options1.enabled = true;
        options1.sampling_4x = true;

        let mut options2 = EncoderOptions::new();
        options2.enabled = true;
        options2.sampling_4x = false;

        encoders.configure_encoder(0, 1, 2, options1).unwrap();
        encoders.configure_encoder(1, 3, 4, options2).unwrap();

        // Test initial values
        assert_eq!(encoders.get_encoder_value(0).unwrap(), 0);
        assert_eq!(encoders.get_encoder_value(1).unwrap(), 0);

        // Test pin assignments (testing pin_a and pin_b fields)
        let (pin_a0, pin_b0) = encoders.get_encoder_pins(0).unwrap();
        assert_eq!(pin_a0, 1);
        assert_eq!(pin_b0, 2);

        let (pin_a1, pin_b1) = encoders.get_encoder_pins(1).unwrap();
        assert_eq!(pin_a1, 3);
        assert_eq!(pin_b1, 4);

        // Test encoder options (testing options field)
        let retrieved_options0 = encoders.get_encoder_options(0).unwrap();
        assert_eq!(retrieved_options0.enabled, options1.enabled);
        assert_eq!(retrieved_options0.sampling_4x, options1.sampling_4x);

        let retrieved_options1 = encoders.get_encoder_options(1).unwrap();
        assert_eq!(retrieved_options1.enabled, options2.enabled);
        assert_eq!(retrieved_options1.sampling_4x, options2.sampling_4x);

        // Test configuration status
        assert!(encoders.is_encoder_configured(0).unwrap());
        assert!(encoders.is_encoder_configured(1).unwrap());
        assert!(!encoders.is_encoder_configured(2).unwrap());

        // Test invalid encoder
        assert!(encoders.configure_encoder(25, 1, 2, options1).is_err());
        assert!(encoders.get_encoder_value(25).is_err());
        assert!(encoders.get_encoder_pins(25).is_err());
        assert!(encoders.get_encoder_options(25).is_err());
    }

    #[test]
    fn test_encoder_manager_value_operations() {
        let mut encoders = MockEncoderManager::new(25);

        let mut options = EncoderOptions::new();
        options.enabled = true;

        encoders.configure_encoder(0, 1, 2, options).unwrap();

        // Test setting values
        let test_values = [0, 100, -50, i32::MAX, i32::MIN];
        for &value in &test_values {
            encoders.set_encoder_value(0, value).unwrap();
            assert_eq!(encoders.get_encoder_value(0).unwrap(), value);
        }

        // Test unconfigured encoder
        assert!(encoders.set_encoder_value(1, 100).is_err());
        assert!(encoders.get_encoder_value(1).is_err());
    }

    #[test]
    fn test_encoder_manager_movement_simulation() {
        let mut encoders = MockEncoderManager::new(25);

        let mut options = EncoderOptions::new();
        options.enabled = true;

        encoders.configure_encoder(0, 1, 2, options).unwrap();

        // Test movement simulation
        encoders.simulate_encoder_movement(0, 10).unwrap();
        assert_eq!(encoders.get_encoder_value(0).unwrap(), 10);

        encoders.simulate_encoder_movement(0, -5).unwrap();
        assert_eq!(encoders.get_encoder_value(0).unwrap(), 5);

        encoders.simulate_encoder_movement(0, -10).unwrap();
        assert_eq!(encoders.get_encoder_value(0).unwrap(), -5);

        // Test overflow behavior
        encoders.set_encoder_value(0, i32::MAX).unwrap();
        encoders.simulate_encoder_movement(0, 1).unwrap();
        assert_eq!(encoders.get_encoder_value(0).unwrap(), i32::MIN);
    }

    #[test]
    fn test_encoder_pin_and_options_comprehensive() {
        let mut encoders = MockEncoderManager::new(25);

        // Test different encoder configurations with various pin assignments and options
        let mut options_basic = EncoderOptions::new();
        options_basic.enabled = true;
        options_basic.sampling_4x = false;

        let mut options_4x = EncoderOptions::new();
        options_4x.enabled = true;
        options_4x.sampling_4x = true;

        let mut options_disabled = EncoderOptions::new();
        options_disabled.enabled = false;
        options_disabled.sampling_4x = false;

        // Configure multiple encoders with different pin assignments
        encoders
            .configure_encoder(0, 10, 11, options_basic)
            .unwrap();
        encoders.configure_encoder(1, 12, 13, options_4x).unwrap();
        encoders
            .configure_encoder(2, 14, 15, options_disabled)
            .unwrap();

        // Test pin assignments for all encoders
        let (pin_a0, pin_b0) = encoders.get_encoder_pins(0).unwrap();
        assert_eq!(pin_a0, 10);
        assert_eq!(pin_b0, 11);

        let (pin_a1, pin_b1) = encoders.get_encoder_pins(1).unwrap();
        assert_eq!(pin_a1, 12);
        assert_eq!(pin_b1, 13);

        let (pin_a2, pin_b2) = encoders.get_encoder_pins(2).unwrap();
        assert_eq!(pin_a2, 14);
        assert_eq!(pin_b2, 15);

        // Test options for all encoders
        let retrieved_options0 = encoders.get_encoder_options(0).unwrap();
        assert!(retrieved_options0.enabled);
        assert!(!retrieved_options0.sampling_4x);

        let retrieved_options1 = encoders.get_encoder_options(1).unwrap();
        assert!(retrieved_options1.enabled);
        assert!(retrieved_options1.sampling_4x);

        let retrieved_options2 = encoders.get_encoder_options(2).unwrap();
        assert!(!retrieved_options2.enabled);
        assert!(!retrieved_options2.sampling_4x);

        // Test configuration status
        assert!(encoders.is_encoder_configured(0).unwrap());
        assert!(encoders.is_encoder_configured(1).unwrap());
        assert!(encoders.is_encoder_configured(2).unwrap());
        assert!(!encoders.is_encoder_configured(3).unwrap());

        // Test that unconfigured encoders return errors for pin/options access
        assert!(encoders.get_encoder_pins(3).is_err());
        assert!(encoders.get_encoder_options(3).is_err());
    }

    #[test]
    fn test_integrated_device_workflow() {
        // Test a complete workflow integrating multiple managers
        let mut device_manager = MockDeviceManager::new();
        let device_id = device_manager.add_device(DeviceTypeId::Device57U);

        device_manager.connect_device(device_id).unwrap();
        let device_info = device_manager.get_device_info(device_id).unwrap();

        let mut io_manager = MockIOManager::new(device_info.pin_count);
        let mut pwm_manager = MockPWMManager::new(device_info.pwm_count);
        let mut encoder_manager = MockEncoderManager::new(device_info.encoders_count);

        // Configure I/O
        io_manager
            .set_pin_function(1, PinFunction::DigitalOutput)
            .unwrap();
        io_manager
            .set_pin_function(2, PinFunction::DigitalInput)
            .unwrap();
        io_manager
            .set_pin_function(3, PinFunction::AnalogInput)
            .unwrap();

        // Configure PWM
        pwm_manager.set_frequency(2000).unwrap();
        pwm_manager.configure_channel(0, 10).unwrap();
        pwm_manager.set_duty_cycle(0, 75.0).unwrap();
        pwm_manager.enable_channel(0, true).unwrap();

        // Configure encoder
        let mut encoder_options = EncoderOptions::new();
        encoder_options.enabled = true;
        encoder_options.sampling_4x = true;
        encoder_manager
            .configure_encoder(0, 5, 6, encoder_options)
            .unwrap();

        // Perform operations
        io_manager.set_digital_output(1, true).unwrap();
        io_manager.simulate_digital_input(2, true);
        io_manager.simulate_analog_input(3, 2048);
        encoder_manager.simulate_encoder_movement(0, 100).unwrap();

        // Verify results
        assert!(io_manager.get_digital_input(2).unwrap());
        assert_eq!(io_manager.get_analog_input(3).unwrap(), 2048);
        assert_eq!(encoder_manager.get_encoder_value(0).unwrap(), 100);

        let pwm_info = pwm_manager.get_channel_info(0).unwrap();
        assert!(pwm_info.enabled);
        assert_eq!(pwm_info.duty_cycle, 75.0);

        // Cleanup
        device_manager.disconnect_device(device_id).unwrap();
        assert!(!device_manager.is_device_connected(device_id));
    }

    #[test]
    fn test_error_propagation_integration() {
        // Test that errors propagate correctly through integrated systems
        let mut device_manager = MockDeviceManager::new();
        let device_id = device_manager.add_device(DeviceTypeId::Device57U);

        // Don't connect device - should cause errors
        let device_info = device_manager.get_device_info(device_id).unwrap();
        let mut io_manager = MockIOManager::new(device_info.pin_count);

        // Operations should work on disconnected mock (IO manager doesn't check connection)
        io_manager
            .set_pin_function(1, PinFunction::DigitalOutput)
            .unwrap();

        // But invalid parameters should still fail
        assert!(
            io_manager
                .set_pin_function(0, PinFunction::DigitalOutput)
                .is_err()
        );
        assert!(
            io_manager
                .set_pin_function(56, PinFunction::DigitalOutput)
                .is_err()
        );

        // Test PWM errors
        let mut pwm_manager = MockPWMManager::new(device_info.pwm_count);
        assert!(pwm_manager.set_duty_cycle(0, 50.0).is_err()); // Unconfigured channel
        assert!(pwm_manager.set_frequency(0).is_err()); // Invalid frequency

        // Test encoder errors
        let encoder_manager = MockEncoderManager::new(device_info.encoders_count);
        assert!(encoder_manager.get_encoder_value(0).is_err()); // Unconfigured encoder
    }
}
