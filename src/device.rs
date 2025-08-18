//! Device enumeration, connection, and management

use crate::communication::{CommunicationManager, NetworkInterface, UsbHidInterface};
use crate::encoders::EncoderData;
use crate::error::{PoKeysError, Result};
use crate::io::PinData;
use crate::lcd::LcdData;
use crate::matrix::{MatrixKeyboard, MatrixLed};
use crate::pulse_engine::PulseEngineV2;
use crate::pwm::PwmData;
use crate::sensors::EasySensor;
use crate::types::*;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

/// Main PoKeys device structure
pub struct PoKeysDevice {
    // Connection information
    connection_type: DeviceConnectionType,
    connection_param: ConnectionParam,

    // Device information
    pub info: DeviceInfo,
    pub device_data: DeviceData,
    pub network_device_data: Option<NetworkDeviceInfo>,

    // Device model
    pub model: Option<crate::models::DeviceModel>,

    // Pin and I/O data
    pub pins: Vec<PinData>,
    pub encoders: Vec<EncoderData>,
    pub pwm: PwmData,

    // Peripheral data
    pub matrix_keyboard: MatrixKeyboard,
    pub matrix_led: Vec<MatrixLed>,
    pub lcd: LcdData,
    pub pulse_engine_v2: PulseEngineV2,
    pub easy_sensors: Vec<EasySensor>,
    pub rtc: RealTimeClock,

    // Communication
    pub(crate) communication: CommunicationManager,
    usb_interface: Option<Box<dyn UsbHidInterface>>,
    network_interface: Option<Box<dyn NetworkInterface>>,

    // Configuration
    pub fast_encoders_configuration: u8,
    pub fast_encoders_options: u8,
    pub ultra_fast_encoder_configuration: u8,
    pub ultra_fast_encoder_options: u8,
    pub ultra_fast_encoder_filter: u32,
    pub po_ext_bus_data: Vec<u8>,

    // Internal state
    #[allow(dead_code)]
    request_buffer: [u8; REQUEST_BUFFER_SIZE],
    #[allow(dead_code)]
    response_buffer: [u8; RESPONSE_BUFFER_SIZE],
    #[allow(dead_code)]
    multipart_buffer: Vec<u8>,
}

impl PoKeysDevice {
    /// Create a new device instance
    fn new(connection_type: DeviceConnectionType) -> Self {
        Self {
            connection_type,
            connection_param: ConnectionParam::Tcp,
            info: DeviceInfo::default(),
            device_data: DeviceData::default(),
            network_device_data: None,
            model: None,
            pins: Vec::new(),
            encoders: Vec::new(),
            pwm: PwmData::new(),
            matrix_keyboard: MatrixKeyboard::new(),
            matrix_led: Vec::new(),
            lcd: LcdData::new(),
            pulse_engine_v2: PulseEngineV2::new(),
            easy_sensors: Vec::new(),
            rtc: RealTimeClock::default(),
            communication: CommunicationManager::new(connection_type),
            usb_interface: None,
            network_interface: None,
            fast_encoders_configuration: 0,
            fast_encoders_options: 0,
            ultra_fast_encoder_configuration: 0,
            ultra_fast_encoder_options: 0,
            ultra_fast_encoder_filter: 0,
            po_ext_bus_data: Vec::new(),
            request_buffer: [0; REQUEST_BUFFER_SIZE],
            response_buffer: [0; RESPONSE_BUFFER_SIZE],
            multipart_buffer: Vec::new(),
        }
    }

    /// Get device information from the connected device
    pub fn get_device_data(&mut self) -> Result<()> {
        // Use the comprehensive read device data command instead of the old method
        self.read_device_data()?;
        self.initialize_device_structures()?;

        // Try to load the device model
        self.load_device_model()?;

        Ok(())
    }

    /// Load the device model based on the device type
    fn load_device_model(&mut self) -> Result<()> {
        use crate::models::load_model;

        // Determine the model name based on the device type
        let device_type_name = self.device_data.device_type_name();
        let model_name = match device_type_name.as_str() {
            "PoKeys56U" => "PoKeys56U",
            "PoKeys56E" => "PoKeys56E",
            "PoKeys57U" => "PoKeys57U",
            "PoKeys57E" => "PoKeys57E",
            "PoKeys57Ev1.1" => "PoKeys57E", // Map v1.1 to base model
            "PoKeys57CNC" => "PoKeys57CNC",
            _ => return Ok(()), // No model for other device types
        };

        // Try to load the model
        match load_model(model_name, None) {
            Ok(model) => {
                log::info!("Loaded device model: {}", model_name);
                self.model = Some(model);
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to load device model {}: {}", model_name, e);
                Ok(()) // Continue without a model
            }
        }
    }

    /// Check if a pin supports a specific capability
    ///
    /// # Arguments
    ///
    /// * `pin` - The pin number to check
    /// * `capability` - The capability to check for
    ///
    /// # Returns
    ///
    /// * `bool` - True if the pin supports the capability, false otherwise
    pub fn is_pin_capability_supported(&self, pin: u32, capability: &str) -> bool {
        if let Some(model) = &self.model {
            model.is_pin_capability_supported(pin as u8, capability)
        } else {
            // If no model is loaded, assume all capabilities are supported
            true
        }
    }

    /// Get all capabilities for a pin
    ///
    /// # Arguments
    ///
    /// * `pin` - The pin number to get capabilities for
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - List of capabilities supported by the pin
    pub fn get_pin_capabilities(&self, pin: u32) -> Vec<String> {
        if let Some(model) = &self.model {
            model.get_pin_capabilities(pin as u8)
        } else {
            // If no model is loaded, return an empty list
            Vec::new()
        }
    }

    /// Validate that a pin can be configured with a specific capability
    ///
    /// # Arguments
    ///
    /// * `pin` - The pin number to check
    /// * `capability` - The capability to check for
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the capability is valid, an error otherwise
    pub fn validate_pin_capability(&self, pin: u32, capability: &str) -> Result<()> {
        if let Some(model) = &self.model {
            model.validate_pin_capability(pin as u8, capability)
        } else {
            // If no model is loaded, assume all capabilities are valid
            Ok(())
        }
    }

    /// Get related capabilities for a specific capability
    ///
    /// # Arguments
    ///
    /// * `pin` - The pin number with the capability
    /// * `capability` - The capability to find related capabilities for
    ///
    /// # Returns
    ///
    /// * `Vec<(String, u8)>` - List of related capabilities and their pin numbers
    pub fn get_related_capabilities(&self, pin: u32, capability: &str) -> Vec<(String, u8)> {
        if let Some(model) = &self.model {
            model.get_related_capabilities(pin as u8, capability)
        } else {
            // If no model is loaded, return an empty list
            Vec::new()
        }
    }

    /// Save current configuration to device
    pub fn save_configuration(&mut self) -> Result<()> {
        // Based on official PoKeysLib source code:
        // CreateRequest(device->request, 0x50, 0xAA, 0x55, 0, 0);
        // Command 0x50 with parameters 0xAA, 0x55, 0, 0
        self.send_request(0x50, 0xAA, 0x55, 0, 0)?;
        Ok(())
    }

    /// Set device name (up to 20 bytes for long device name)
    pub fn set_device_name(&mut self, name: &str) -> Result<()> {
        // Based on official documentation:
        // - byte 2: 0x06
        // - byte 3: Bit 0 for writing device name (0x01)
        // - byte 4: use long device name (1)
        // - byte 5-6: 0
        // - byte 7: request ID
        // - bytes 36-55: long device name string (20 bytes)

        // Prepare long device name (20 bytes for long name)
        let mut name_bytes = [0u8; 20];
        let name_str = if name.len() > 20 { &name[..20] } else { name };
        let name_bytes_slice = name_str.as_bytes();
        name_bytes[..name_bytes_slice.len()].copy_from_slice(name_bytes_slice);

        // Create request manually according to documentation
        let mut request = [0u8; 64];
        request[0] = 0xBB; // header
        request[1] = 0x06; // command (byte 2 in doc)
        request[2] = 0x01; // bit 0 set for writing device name (byte 3 in doc)
        request[3] = 0x01; // use long device name (byte 4 in doc)
        request[4] = 0x00; // byte 5 in doc
        request[5] = 0x00; // byte 6 in doc
        request[6] = self.communication.get_next_request_id();

        // Long device name at bytes 36-55 in doc = bytes 35-54 in 0-based array
        request[35..55].copy_from_slice(&name_bytes);

        // Calculate checksum exactly like official PoKeysLib getChecksum function
        // Sum bytes 0-6 (not including byte 7 where checksum goes)
        let mut checksum: u8 = 0;
        for i in 0..7 {
            checksum = checksum.wrapping_add(request[i]);
        }
        request[7] = checksum;

        // Send the request using the appropriate interface
        match self.connection_type {
            DeviceConnectionType::UsbDevice | DeviceConnectionType::FastUsbDevice => {
                if let Some(ref mut interface) = self.usb_interface {
                    let mut hid_packet = [0u8; 65];
                    hid_packet[0] = 0; // Report ID
                    hid_packet[1..65].copy_from_slice(&request);

                    interface.write(&hid_packet)?;

                    let mut response = [0u8; 65];
                    interface.read(&mut response)?;

                    Ok(())
                } else {
                    Err(PoKeysError::NotConnected)
                }
            }
            DeviceConnectionType::NetworkDevice => {
                if let Some(ref mut interface) = self.network_interface {
                    interface.send(&request)?;

                    // Use timeout for network response to avoid hanging
                    let mut response = [0u8; 64];
                    match interface
                        .receive_timeout(&mut response, std::time::Duration::from_millis(2000))
                    {
                        Ok(_) => Ok(()),
                        Err(_) => {
                            // Don't fail the entire operation if device name setting fails
                            Ok(())
                        }
                    }
                } else {
                    Err(PoKeysError::NotConnected)
                }
            }
        }
    }

    /// Clear device configuration
    pub fn clear_configuration(&mut self) -> Result<()> {
        self.send_request(0x02, 0, 0, 0, 0)?;
        Ok(())
    }

    /// Send custom request to device
    pub fn custom_request(
        &mut self,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        self.send_request(request_type, param1, param2, param3, param4)
    }

    /// Set ethernet retry count and timeout
    pub fn set_ethernet_retry_count_and_timeout(
        &mut self,
        send_retries: u32,
        read_retries: u32,
        timeout_ms: u32,
    ) {
        self.communication.set_retries_and_timeout(
            send_retries,
            read_retries,
            Duration::from_millis(timeout_ms as u64),
        );
    }

    /// Get connection type
    pub fn get_connection_type(&self) -> DeviceConnectionType {
        self.connection_type
    }

    /// Check if device supports a specific capability
    pub fn check_pin_capability(&self, pin: u32, capability: crate::io::PinCapability) -> bool {
        if pin as usize >= self.pins.len() {
            return false;
        }

        // Implementation would check device-specific pin capabilities
        // This is a simplified version
        match self.device_data.device_type_id {
            32 => check_pokeys57cnc_pin_capability(pin, capability), // PoKeys57CNC
            _ => false, // Add other device types as needed
        }
    }

    /// Get complete network configuration including discovery info
    pub fn get_network_configuration(
        &mut self,
        timeout_ms: u32,
    ) -> Result<(Option<NetworkDeviceSummary>, NetworkDeviceInfo)> {
        // Get discovery info
        let discovery_info = enumerate_network_devices(timeout_ms)?
            .into_iter()
            .find(|d| d.serial_number == self.device_data.serial_number);

        // Get detailed config using 0xBB request
        let response = self.send_request(0xE0, 0x00, 0x00, 0, 0)?;

        // Parse network configuration from response
        let config = NetworkDeviceInfo {
            dhcp: response.get(8).copied().unwrap_or(0),
            ip_address_setup: [
                response.get(9).copied().unwrap_or(0),
                response.get(10).copied().unwrap_or(0),
                response.get(11).copied().unwrap_or(0),
                response.get(12).copied().unwrap_or(0),
            ],
            ip_address_current: [
                response.get(13).copied().unwrap_or(0),
                response.get(14).copied().unwrap_or(0),
                response.get(15).copied().unwrap_or(0),
                response.get(16).copied().unwrap_or(0),
            ],
            tcp_timeout: u16::from_le_bytes([
                response.get(17).copied().unwrap_or(0),
                response.get(18).copied().unwrap_or(0),
            ])
            .saturating_mul(100),
            gateway_ip: [
                response.get(19).copied().unwrap_or(0),
                response.get(20).copied().unwrap_or(0),
                response.get(21).copied().unwrap_or(0),
                response.get(22).copied().unwrap_or(0),
            ],
            subnet_mask: [
                response.get(23).copied().unwrap_or(0),
                response.get(24).copied().unwrap_or(0),
                response.get(25).copied().unwrap_or(0),
                response.get(26).copied().unwrap_or(0),
            ],
            additional_network_options: response.get(27).copied().unwrap_or(0),
        };

        //print the config
        Ok((discovery_info, config))
    }

    // Internal methods

    pub fn send_request(
        &mut self,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        match self.connection_type {
            DeviceConnectionType::UsbDevice | DeviceConnectionType::FastUsbDevice => {
                if let Some(ref mut interface) = self.usb_interface {
                    self.communication.send_usb_request(
                        interface,
                        request_type,
                        param1,
                        param2,
                        param3,
                        param4,
                    )
                } else {
                    Err(PoKeysError::NotConnected)
                }
            }
            DeviceConnectionType::NetworkDevice => {
                if let Some(ref mut interface) = self.network_interface {
                    self.communication.send_network_request(
                        interface,
                        request_type,
                        param1,
                        param2,
                        param3,
                        param4,
                    )
                } else {
                    Err(PoKeysError::NotConnected)
                }
            }
        }
    }

    /// Send request with data payload
    pub fn send_request_with_data(
        &mut self,
        request_type: u8,
        param1: u8,
        param2: u8,
        param3: u8,
        param4: u8,
        data: &[u8],
    ) -> Result<[u8; RESPONSE_BUFFER_SIZE]> {
        let request = self.communication.prepare_request_with_data(
            request_type,
            param1,
            param2,
            param3,
            param4,
            Some(data),
        );

        match self.connection_type {
            DeviceConnectionType::UsbDevice | DeviceConnectionType::FastUsbDevice => {
                if let Some(ref mut interface) = self.usb_interface {
                    self.communication.send_usb_request_raw(interface, &request)
                } else {
                    Err(PoKeysError::NotConnected)
                }
            }
            DeviceConnectionType::NetworkDevice => {
                if let Some(ref mut interface) = self.network_interface {
                    self.communication
                        .send_network_request_raw(interface, &request)
                } else {
                    Err(PoKeysError::NotConnected)
                }
            }
        }
    }

    /// Read comprehensive device data using command 0x00
    /// This provides accurate device information including proper firmware version and device type
    pub fn read_device_data(&mut self) -> Result<()> {
        // Send read device data command (byte 2: 0x00, bytes 3-6: 0)
        let response = self.send_request(0x00, 0, 0, 0, 0)?;

        if response.len() < 64 {
            return Err(PoKeysError::Protocol(
                "Read device data response too short".to_string(),
            ));
        }

        self.parse_device_data_response(&response)?;
        Ok(())
    }

    /// Parse the comprehensive device data response according to PoKeys protocol specification
    fn parse_device_data_response(&mut self, response: &[u8]) -> Result<()> {
        // Check for extended device signature (PK58/PKEx) at bytes 9-12 (doc says 9-12, 0-based = 8-11)
        if response.len() >= 64 && (&response[8..12] == b"PK58" || &response[8..12] == b"PKEx") {
            // Extended device parsing (based on official documentation)

            // Basic firmware info (bytes 5-6)
            let software_version_encoded = response[4]; // Byte 5 in doc = byte 4 in 0-based
            let revision_number = response[5]; // Byte 6 in doc = byte 5 in 0-based

            // Decode software version: v(1+[bits 4-7]).(bits [0-3])
            let major_bits = (software_version_encoded >> 4) & 0x0F; // Extract bits 4-7
            let minor_bits = software_version_encoded & 0x0F; // Extract bits 0-3
            let decoded_major = 1 + major_bits;
            let decoded_minor = minor_bits;

            // 32-bit serial number (bytes 13-16 in doc = bytes 12-15 in 0-based)
            let serial_32bit =
                u32::from_le_bytes([response[12], response[13], response[14], response[15]]);

            // Extended firmware info (bytes 17-18 in doc = bytes 16-17 in 0-based)
            let firmware_version = response[16];
            let firmware_revision = response[17];

            // Hardware ID (byte 19 in doc = byte 18 in 0-based)
            let hw_id = response[18];

            // User ID (byte 20 in doc = byte 19 in 0-based)
            let user_id = response[19];

            // Build date (bytes 21-31 in doc = bytes 20-30 in 0-based)
            let build_date_bytes = &response[20..31];

            // Device name (bytes 32-41 in doc = bytes 31-40 in 0-based)
            let device_name_bytes = &response[31..41];

            // Firmware type (byte 42 in doc = byte 41 in 0-based)
            let firmware_type = response[41];

            // Application firmware version (bytes 43-44 in doc = bytes 42-43 in 0-based)
            let _app_firmware_major = response[42];
            let _app_firmware_minor = response[43];

            // Product ID offset (byte 58 in doc = byte 57 in 0-based)
            let product_id = response[57];

            // Update device data structure
            self.device_data.serial_number = serial_32bit;

            // Use the decoded firmware version for display
            self.device_data.firmware_version_major = decoded_major;
            self.device_data.firmware_version_minor = decoded_minor;
            self.device_data.firmware_revision = revision_number;

            // Store the extended firmware info in secondary fields
            self.device_data.secondary_firmware_version_major = firmware_version;
            self.device_data.secondary_firmware_version_minor = firmware_revision;

            self.device_data.hw_type = hw_id;
            self.device_data.user_id = user_id;
            self.device_data.fw_type = firmware_type;
            self.device_data.product_id = product_id;

            // Store device name (copy raw bytes)
            self.device_data.device_name.fill(0);
            let name_len =
                std::cmp::min(device_name_bytes.len(), self.device_data.device_name.len());
            self.device_data.device_name[..name_len]
                .copy_from_slice(&device_name_bytes[..name_len]);

            // Store build date (copy raw bytes)
            self.device_data.build_date.fill(0);
            let date_len = std::cmp::min(build_date_bytes.len(), self.device_data.build_date.len());
            self.device_data.build_date[..date_len].copy_from_slice(&build_date_bytes[..date_len]);

            log::debug!("Extended device info parsed:");
            log::debug!("  Serial: {}", serial_32bit);
            log::debug!(
                "  Decoded firmware: {}.{}.{}",
                decoded_major,
                decoded_minor,
                revision_number
            );
            log::debug!(
                "  Raw software version byte: 0x{:02X}",
                software_version_encoded
            );
            log::debug!(
                "  Extended firmware: {}.{}",
                firmware_version,
                firmware_revision
            );
            log::debug!("  Hardware type: {}", hw_id);
        } else {
            // Legacy device parsing

            // Serial number (bytes 3-4 in doc = bytes 2-3 in 0-based)
            let serial_16bit = ((response[2] as u32) << 8) | (response[3] as u32);

            // Software version (byte 5 in doc = byte 4 in 0-based)
            let software_version_encoded = response[4];
            let revision_number = response[5];

            // Decode software version: v(1+[bits 4-7]).(bits [0-3])
            let major_bits = (software_version_encoded >> 4) & 0x0F;
            let minor_bits = software_version_encoded & 0x0F;
            let decoded_major = 1 + major_bits;
            let decoded_minor = minor_bits;

            self.device_data.serial_number = serial_16bit;
            self.device_data.firmware_version_major = decoded_major;
            self.device_data.firmware_version_minor = decoded_minor;
            self.device_data.firmware_revision = revision_number;
            self.device_data.hw_type = 0; // Unknown for legacy
            self.device_data.device_name.fill(0);
            self.device_data.build_date.fill(0);

            log::debug!("Legacy device info parsed:");
            log::debug!("  Serial: {}", serial_16bit);
            log::debug!(
                "  Decoded firmware: {}.{}.{}",
                decoded_major,
                decoded_minor,
                revision_number
            );
        }

        Ok(())
    }

    fn initialize_device_structures(&mut self) -> Result<()> {
        // Initialize device-specific structures based on device type
        match self.device_data.device_type_id {
            32 => self.initialize_pokeys57cnc(), // PoKeys57CNC
            _ => self.initialize_generic_device(),
        }
    }

    fn initialize_pokeys57cnc(&mut self) -> Result<()> {
        // PoKeys57CNC specific initialization
        self.info.pin_count = 55;
        self.info.pwm_count = 6;
        self.info.encoders_count = 25;
        self.info.fast_encoders = 3;
        self.info.ultra_fast_encoders = 1;
        self.info.analog_inputs = 8;
        self.info.pulse_engine_v2 = 1;

        // Initialize pin array
        self.pins = vec![PinData::new(); self.info.pin_count as usize];
        self.encoders = vec![EncoderData::new(); self.info.encoders_count as usize];
        self.pwm.initialize(self.info.pwm_count as usize);
        self.po_ext_bus_data = vec![0; 10]; // PoKeys57CNC has 10 PoExtBus outputs

        Ok(())
    }

    fn initialize_generic_device(&mut self) -> Result<()> {
        // Generic device initialization
        self.info.pin_count = 55; // Default
        self.pins = vec![PinData::new(); self.info.pin_count as usize];
        self.encoders = vec![EncoderData::new(); 25]; // Default encoder count
        Ok(())
    }

    // LED Matrix Protocol Implementation

    /// Command 0xD5: Get/Set Matrix LED Configuration
    pub fn configure_led_matrix(
        &mut self,
        config: &crate::matrix::MatrixLedProtocolConfig,
    ) -> Result<()> {
        let enabled_flags = self.encode_matrix_enabled(config);
        let display1_size = self.encode_display_size(
            config.display1_characters,
            crate::matrix::SEVEN_SEGMENT_COLUMNS,
        );
        let display2_size = self.encode_display_size(
            config.display2_characters,
            crate::matrix::SEVEN_SEGMENT_COLUMNS,
        );

        let _response =
            self.send_request(0xD5, 0x00, enabled_flags, display1_size, display2_size)?;
        Ok(())
    }

    /// Read Matrix LED Configuration
    pub fn read_led_matrix_config(&mut self) -> Result<crate::matrix::MatrixLedProtocolConfig> {
        // Read the configuration using command 0xD5 with parameter 0x01
        let response = self.send_request(0xD5, 0x01, 0, 0, 0)?;

        // Parse the response
        let enabled_flags = response[2];
        let display1_size = response[3];
        let display2_size = response[4];

        // Decode enabled flags
        let display1_enabled = (enabled_flags & 0x01) != 0;
        let display2_enabled = (enabled_flags & 0x02) != 0;

        // Decode character counts (lower 4 bits)
        // Note: The protocol response has display parameters swapped
        let display1_characters = display2_size & 0x0F; // Use display2_size for display1
        let display2_characters = display1_size & 0x0F; // Use display1_size for display2

        Ok(crate::matrix::MatrixLedProtocolConfig {
            display1_enabled,
            display2_enabled,
            display1_characters,
            display2_characters,
        })
    }

    /// Command 0xD6: Update Matrix Display
    pub fn update_led_matrix(
        &mut self,
        matrix_id: u8,
        action: crate::matrix::MatrixAction,
        row: u8,
        column: u8,
        data: &[u8],
    ) -> Result<()> {
        let action_code = match (matrix_id, action) {
            (1, crate::matrix::MatrixAction::UpdateWhole) => 1,
            (1, crate::matrix::MatrixAction::SetPixel) => 5,
            (1, crate::matrix::MatrixAction::ClearPixel) => 6,
            (2, crate::matrix::MatrixAction::UpdateWhole) => 11,
            (2, crate::matrix::MatrixAction::SetPixel) => 15,
            (2, crate::matrix::MatrixAction::ClearPixel) => 16,
            _ => {
                return Err(PoKeysError::Parameter(format!(
                    "Invalid matrix ID: {}",
                    matrix_id
                )));
            }
        };

        // Prepare the request with data payload
        let request = self.communication.prepare_request_with_data(
            0xD6,
            action_code,
            row,
            column,
            0,
            Some(data),
        );

        // Send the full request
        match self.connection_type {
            DeviceConnectionType::NetworkDevice => {
                if let Some(ref mut interface) = self.network_interface {
                    let request_id = request[6];

                    // Send request
                    interface.send(&request[..64])?;

                    // Receive response
                    let mut response = [0u8; RESPONSE_BUFFER_SIZE];
                    interface.receive(&mut response)?;

                    // Validate response
                    self.communication
                        .validate_response(&response, request_id)?;
                }
            }
            _ => {
                return Err(PoKeysError::NotSupported);
            }
        }

        Ok(())
    }

    /// Helper to encode display size (characters as rows, always 8 columns for 7-segment)
    fn encode_display_size(&self, characters: u8, columns: u8) -> u8 {
        // bits 0-3: number of rows (characters)
        // bits 4-7: number of columns (always 8 for 7-segment)
        (characters & 0x0F) | ((columns & 0x0F) << 4)
    }

    /// Helper to encode matrix enabled flags
    fn encode_matrix_enabled(&self, config: &crate::matrix::MatrixLedProtocolConfig) -> u8 {
        let mut enabled = 0u8;
        if config.display1_enabled {
            enabled |= 1 << 0;
        }
        if config.display2_enabled {
            enabled |= 1 << 1;
        }
        enabled
    }

    /// Configure multiple LED matrices with device model validation
    ///
    /// This method validates all configurations against the device model first,
    /// then applies the configurations and reserves the necessary pins.
    ///
    /// # Arguments
    ///
    /// * `configs` - Array of LED matrix configurations
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if all configurations were applied successfully
    pub fn configure_led_matrices(
        &mut self,
        configs: &[crate::matrix::LedMatrixConfig],
    ) -> Result<()> {
        // Basic validation (always performed)
        for config in configs {
            // Validate matrix ID
            if config.matrix_id < 1 || config.matrix_id > 2 {
                return Err(PoKeysError::InvalidConfiguration(
                    "Matrix ID must be 1 or 2".to_string(),
                ));
            }

            // Validate character count (1-8 characters for 7-segment displays)
            if config.characters < 1 || config.characters > 8 {
                return Err(PoKeysError::InvalidConfiguration(
                    "Character count must be between 1 and 8 for 7-segment displays".to_string(),
                ));
            }
        }

        // Device model validation (if available)
        for config in configs {
            if let Some(model) = &self.model {
                model.validate_led_matrix_config(config)?;
            }
        }

        // Apply configurations
        let mut protocol_config = crate::matrix::MatrixLedProtocolConfig {
            display1_enabled: false,
            display2_enabled: false,
            display1_characters: 1,
            display2_characters: 1,
        };

        for config in configs {
            match config.matrix_id {
                1 => {
                    protocol_config.display1_enabled = config.enabled;
                    protocol_config.display1_characters = config.characters;
                }
                2 => {
                    protocol_config.display2_enabled = config.enabled;
                    protocol_config.display2_characters = config.characters;
                }
                _ => {
                    return Err(PoKeysError::InvalidConfiguration(
                        "Invalid matrix ID".to_string(),
                    ));
                }
            }
        }

        self.configure_led_matrix(&protocol_config)?;

        // Reserve pins in model
        if let Some(model) = &mut self.model {
            for config in configs {
                if config.enabled {
                    model.reserve_led_matrix_pins(config.matrix_id)?;
                }
            }
        }

        Ok(())
    }
}

// Default implementations for device structures
impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            pin_count: 0,
            pwm_count: 0,
            basic_encoder_count: 0,
            encoders_count: 0,
            fast_encoders: 0,
            ultra_fast_encoders: 0,
            pwm_internal_frequency: 0,
            analog_inputs: 0,
            key_mapping: 0,
            triggered_key_mapping: 0,
            key_repeat_delay: 0,
            digital_counters: 0,
            joystick_button_axis_mapping: 0,
            joystick_analog_to_digital_mapping: 0,
            macros: 0,
            matrix_keyboard: 0,
            matrix_keyboard_triggered_mapping: 0,
            lcd: 0,
            matrix_led: 0,
            connection_signal: 0,
            po_ext_bus: 0,
            po_net: 0,
            analog_filtering: 0,
            init_outputs_start: 0,
            prot_i2c: 0,
            prot_1wire: 0,
            additional_options: 0,
            load_status: 0,
            custom_device_name: 0,
            po_tlog27_support: 0,
            sensor_list: 0,
            web_interface: 0,
            fail_safe_settings: 0,
            joystick_hat_switch: 0,
            pulse_engine: 0,
            pulse_engine_v2: 0,
            easy_sensors: 0,
        }
    }
}

impl Default for RealTimeClock {
    fn default() -> Self {
        Self {
            sec: 0,
            min: 0,
            hour: 0,
            dow: 0,
            dom: 0,
            tmp: 0,
            doy: 0,
            month: 0,
            year: 0,
        }
    }
}

// Device capability checking functions
fn check_pokeys57cnc_pin_capability(pin: u32, capability: crate::io::PinCapability) -> bool {
    use crate::io::PinCapability;

    match pin {
        1..=2 => matches!(
            capability,
            PinCapability::DigitalInput
                | PinCapability::DigitalOutput
                | PinCapability::DigitalCounter
                | PinCapability::FastEncoder1A
        ),
        3..=6 => matches!(
            capability,
            PinCapability::DigitalInput
                | PinCapability::DigitalCounter
                | PinCapability::FastEncoder1A
        ),
        8 | 12..=13 => matches!(
            capability,
            PinCapability::DigitalInput | PinCapability::DigitalOutput
        ),
        9..=11 | 15..=16 => matches!(
            capability,
            PinCapability::DigitalInput | PinCapability::DigitalCounter
        ),
        14 => matches!(capability, PinCapability::DigitalInput),
        17..=18 | 20..=21 => matches!(
            capability,
            PinCapability::DigitalOutput | PinCapability::PwmOutput
        ),
        19 => matches!(
            capability,
            PinCapability::DigitalInput | PinCapability::DigitalCounter
        ),
        _ => false,
    }
}

// Global device enumeration and connection functions

static USB_DEVICE_LIST: LazyLock<Mutex<Vec<UsbDeviceInfo>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
#[allow(dead_code)]
static NETWORK_DEVICE_LIST: LazyLock<Mutex<Vec<NetworkDeviceSummary>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UsbDeviceInfo {
    vendor_id: u16,
    product_id: u16,
    serial_number: Option<String>,
    path: String,
}

/// Enumerate USB PoKeys devices
pub fn enumerate_usb_devices() -> Result<i32> {
    let mut device_list = USB_DEVICE_LIST
        .lock()
        .map_err(|_| PoKeysError::InternalError("Failed to lock USB device list".to_string()))?;
    device_list.clear();

    // Platform-specific USB enumeration
    #[cfg(target_os = "macos")]
    {
        enumerate_usb_devices_macos(&mut device_list)
    }
    #[cfg(target_os = "linux")]
    {
        enumerate_usb_devices_linux(&mut device_list)
    }
    #[cfg(target_os = "windows")]
    {
        enumerate_usb_devices_windows(&mut device_list)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(PoKeysError::NotSupported)
    }
}

/// Enumerate network PoKeys devices
pub fn enumerate_network_devices(timeout_ms: u32) -> Result<Vec<NetworkDeviceSummary>> {
    use crate::network::discover_all_devices;

    log::info!("Enumerating network devices with timeout {timeout_ms}ms");

    // Use the network discovery implementation
    discover_all_devices(timeout_ms)
}

/// Connect to USB device by index
pub fn connect_to_device(device_index: u32) -> Result<PoKeysDevice> {
    let device_list = USB_DEVICE_LIST
        .lock()
        .map_err(|_| PoKeysError::InternalError("Failed to lock USB device list".to_string()))?;

    if device_index as usize >= device_list.len() {
        return Err(PoKeysError::Parameter("Invalid device index".to_string()));
    }

    let device_info = device_list[device_index as usize].clone();
    drop(device_list); // Release the lock early

    // Create device instance
    let mut device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);

    // Platform-specific USB connection
    #[cfg(target_os = "macos")]
    {
        device.usb_interface = Some(Box::new(connect_usb_device_macos(&device_info.path)?));
    }
    #[cfg(target_os = "linux")]
    {
        device.usb_interface = Some(Box::new(connect_usb_device_linux(&device_info.path)?));
    }
    #[cfg(target_os = "windows")]
    {
        device.usb_interface = Some(Box::new(connect_usb_device_windows(&device_info.path)?));
    }

    // Get device data
    device.get_device_data()?;

    Ok(device)
}

/// Connect to device by serial number
pub fn connect_to_device_with_serial(
    serial_number: u32,
    check_network: bool,
    timeout_ms: u32,
) -> Result<PoKeysDevice> {
    if check_network {
        let network_devices = enumerate_network_devices(timeout_ms)?;
        for network_device in network_devices {
            if network_device.serial_number == serial_number {
                return connect_to_network_device(&network_device);
            }
        }
    }

    // First try USB devices
    let usb_count = enumerate_usb_devices()?;

    for i in 0..usb_count {
        if let Ok(device) = connect_to_device(i as u32) {
            if device.device_data.serial_number == serial_number {
                return Ok(device);
            }
        }
    }

    Err(PoKeysError::CannotConnect)
}

/// Connect to network device
pub fn connect_to_network_device(device_summary: &NetworkDeviceSummary) -> Result<PoKeysDevice> {
    let mut device = PoKeysDevice::new(DeviceConnectionType::NetworkDevice);

    // Create network interface based on device settings
    if device_summary.use_udp != 0 {
        device.connection_param = ConnectionParam::Udp;
    } else {
        device.connection_param = ConnectionParam::Tcp;
    }

    // Platform-specific network connection
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    {
        device.network_interface = Some(connect_network_device(device_summary)?);
    }

    // Get device data
    device.get_device_data()?;

    Ok(device)
}

// Stub implementations for compilation
struct StubUsbInterface;

impl UsbHidInterface for StubUsbInterface {
    fn write(&mut self, _data: &[u8]) -> Result<usize> {
        Err(PoKeysError::NotSupported)
    }

    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize> {
        Err(PoKeysError::NotSupported)
    }

    fn read_timeout(&mut self, _buffer: &mut [u8], _timeout: Duration) -> Result<usize> {
        Err(PoKeysError::NotSupported)
    }
}

#[cfg(target_os = "macos")]
fn enumerate_usb_devices_macos(device_list: &mut Vec<UsbDeviceInfo>) -> Result<i32> {
    use std::process::Command;

    // Use system_profiler to get USB device information
    let output = Command::new("system_profiler")
        .args(["SPUSBDataType", "-xml"])
        .output()
        .map_err(|e| PoKeysError::InternalError(format!("Failed to run system_profiler: {}", e)))?;

    if !output.status.success() {
        return Err(PoKeysError::InternalError(
            "system_profiler command failed".to_string(),
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Look for PoKeys devices (vendor ID 0x1DC3)
    // This is a simple text-based search since we don't want to add XML parsing dependencies
    let lines: Vec<&str> = output_str.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for vendor_id entries
        if line.contains("<key>vendor_id</key>") && i + 1 < lines.len() {
            let next_line = lines[i + 1].trim();

            // Check if it's a PoKeys device (vendor ID 0x1DC3 = 7619 decimal)
            if next_line.contains("<integer>7619</integer>")
                || next_line.contains("<string>0x1dc3</string>")
            {
                // Found a PoKeys device, now look for product_id and serial
                let mut product_id = 0u16;
                let mut serial_number = None;
                let mut location_id = String::new();

                // Search forward for product_id and serial_number
                for j in (i + 2)..(i + 50).min(lines.len()) {
                    let search_line = lines[j].trim();

                    if search_line.contains("<key>product_id</key>") && j + 1 < lines.len() {
                        let product_line = lines[j + 1].trim();
                        if let Some(start) = product_line.find("<integer>") {
                            if let Some(end) = product_line.find("</integer>") {
                                if let Ok(pid) = product_line[start + 9..end].parse::<u16>() {
                                    product_id = pid;
                                }
                            }
                        }
                    }

                    if search_line.contains("<key>serial_num</key>") && j + 1 < lines.len() {
                        let serial_line = lines[j + 1].trim();
                        if let Some(start) = serial_line.find("<string>") {
                            if let Some(end) = serial_line.find("</string>") {
                                serial_number = Some(serial_line[start + 8..end].to_string());
                            }
                        }
                    }

                    if search_line.contains("<key>location_id</key>") && j + 1 < lines.len() {
                        let location_line = lines[j + 1].trim();
                        if let Some(start) = location_line.find("<string>") {
                            if let Some(end) = location_line.find("</string>") {
                                location_id = location_line[start + 8..end].to_string();
                            }
                        }
                    }

                    // Stop searching when we hit the next device or end of this device
                    if search_line.contains("<key>vendor_id</key>") && j > i + 2 {
                        break;
                    }
                }

                // Add the device to our list
                device_list.push(UsbDeviceInfo {
                    vendor_id: 0x1DC3,
                    product_id,
                    serial_number,
                    path: format!("macos_usb_{}", location_id),
                });
            }
        }
        i += 1;
    }

    Ok(device_list.len() as i32)
}

#[cfg(target_os = "linux")]
fn enumerate_usb_devices_linux(device_list: &mut Vec<UsbDeviceInfo>) -> Result<i32> {
    // Linux-specific USB enumeration using libudev
    // This is a placeholder implementation
    // When implemented, add devices to device_list
    Ok(device_list.len() as i32)
}

#[cfg(target_os = "windows")]
fn enumerate_usb_devices_windows(device_list: &mut Vec<UsbDeviceInfo>) -> Result<i32> {
    // Windows-specific USB enumeration using WinAPI
    // This is a placeholder implementation
    // When implemented, add devices to device_list
    Ok(device_list.len() as i32)
}

// Network connection function
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn connect_network_device(
    device_summary: &NetworkDeviceSummary,
) -> Result<Box<dyn NetworkInterface>> {
    use crate::network::{TcpNetworkInterface, UdpNetworkInterface};

    if device_summary.use_udp != 0 {
        // Use UDP connection
        let interface = UdpNetworkInterface::new(device_summary.ip_address, 20055)?;
        Ok(Box::new(interface))
    } else {
        // Use TCP connection
        let interface = TcpNetworkInterface::new(device_summary.ip_address, 20055)?;
        Ok(Box::new(interface))
    }
}

// Placeholder USB connection functions
#[cfg(target_os = "macos")]
fn connect_usb_device_macos(_path: &str) -> Result<StubUsbInterface> {
    Err(PoKeysError::NotSupported)
}

#[cfg(target_os = "linux")]
fn connect_usb_device_linux(_path: &str) -> Result<StubUsbInterface> {
    Err(PoKeysError::NotSupported)
}

#[cfg(target_os = "windows")]
fn connect_usb_device_windows(_path: &str) -> Result<StubUsbInterface> {
    Err(PoKeysError::NotSupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);
        assert_eq!(device.connection_type, DeviceConnectionType::UsbDevice);
        assert_eq!(device.pins.len(), 0); // Not initialized yet
    }

    #[test]
    fn test_pin_capability_checking() {
        // Test PoKeys57CNC pin capabilities
        assert!(check_pokeys57cnc_pin_capability(
            1,
            crate::io::PinCapability::DigitalInput
        ));
        assert!(check_pokeys57cnc_pin_capability(
            1,
            crate::io::PinCapability::DigitalOutput
        ));
        assert!(!check_pokeys57cnc_pin_capability(
            14,
            crate::io::PinCapability::DigitalOutput
        )); // Input only
        assert!(!check_pokeys57cnc_pin_capability(
            100,
            crate::io::PinCapability::DigitalInput
        )); // Invalid pin
    }

    // Hardware tests - only run when hardware-tests feature is enabled
    #[cfg(feature = "hardware-tests")]
    mod hardware_tests {
        use super::*;
        use std::thread;
        use std::time::Duration;

        #[test]
        fn test_hardware_device_enumeration() {
            println!("Testing hardware device enumeration...");

            match enumerate_usb_devices() {
                Ok(count) => {
                    println!("Found {} USB PoKeys devices", count);
                    if count == 0 {
                        println!(
                            "WARNING: No PoKeys devices found. Connect a device to run hardware tests."
                        );
                    }
                }
                Err(e) => {
                    panic!("Failed to enumerate USB devices: {}", e);
                }
            }
        }

        #[test]
        fn test_hardware_device_connection() {
            println!("Testing hardware device connection...");

            let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");

            if device_count == 0 {
                println!("SKIP: No PoKeys devices found for hardware test");
                return;
            }

            match connect_to_device(0) {
                Ok(mut device) => {
                    println!("Successfully connected to device");

                    // Test getting device data
                    match device.get_device_data() {
                        Ok(_) => {
                            println!("Device Serial: {}", device.device_data.serial_number);
                            println!(
                                "Firmware: {}.{}",
                                device.device_data.firmware_version_major,
                                device.device_data.firmware_version_minor
                            );
                            println!("Pin Count: {}", device.info.pin_count);
                        }
                        Err(e) => {
                            println!("WARNING: Could not get device data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    panic!("Failed to connect to device: {}", e);
                }
            }
        }

        #[test]
        fn test_hardware_digital_io() {
            println!("Testing hardware digital I/O...");

            let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");
            if device_count == 0 {
                println!("SKIP: No PoKeys devices found for hardware test");
                return;
            }

            let mut device = connect_to_device(0).expect("Failed to connect to device");
            device.get_device_data().expect("Failed to get device data");

            // Test setting pin as digital output
            match device.set_pin_function(1, crate::io::PinFunction::DigitalOutput) {
                Ok(_) => {
                    println!("Successfully configured pin 1 as digital output");

                    // Test setting output high and low
                    for state in [true, false, true, false] {
                        match device.set_digital_output(1, state) {
                            Ok(_) => {
                                println!("Set pin 1 to {}", if state { "HIGH" } else { "LOW" })
                            }
                            Err(e) => println!("WARNING: Failed to set digital output: {}", e),
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(e) => {
                    println!("WARNING: Could not configure pin function: {}", e);
                }
            }
        }

        #[test]
        fn test_hardware_analog_input() {
            println!("Testing hardware analog input...");

            let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");
            if device_count == 0 {
                println!("SKIP: No PoKeys devices found for hardware test");
                return;
            }

            let mut device = connect_to_device(0).expect("Failed to connect to device");
            device.get_device_data().expect("Failed to get device data");

            // Test reading analog input
            match device.get_analog_input(1) {
                Ok(value) => {
                    println!("Analog input 1 value: {}", value);
                    // Basic sanity check - value should be within ADC range
                    assert!(value <= 4095, "Analog value out of range for 12-bit ADC");
                }
                Err(e) => {
                    println!("WARNING: Could not read analog input: {}", e);
                }
            }
        }

        #[test]
        fn test_hardware_pwm_output() {
            println!("Testing hardware PWM output...");

            let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");
            if device_count == 0 {
                println!("SKIP: No PoKeys devices found for hardware test");
                return;
            }

            let mut device = connect_to_device(0).expect("Failed to connect to device");
            device.get_device_data().expect("Failed to get device data");

            // Test PWM configuration
            match device.set_pwm_frequency(1000) {
                Ok(_) => {
                    println!("Set PWM frequency to 1000 Hz");

                    // Test different duty cycles
                    for duty in [25.0, 50.0, 75.0, 0.0] {
                        match device.set_pwm_duty_cycle_percent(0, duty) {
                            Ok(_) => {
                                println!("Set PWM duty cycle to {}%", duty);
                                thread::sleep(Duration::from_millis(200));
                            }
                            Err(e) => {
                                println!("WARNING: Could not set PWM duty cycle: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("WARNING: Could not configure PWM: {}", e);
                }
            }
        }

        #[test]
        fn test_hardware_encoder_reading() {
            println!("Testing hardware encoder reading...");

            let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");
            if device_count == 0 {
                println!("SKIP: No PoKeys devices found for hardware test");
                return;
            }

            let mut device = connect_to_device(0).expect("Failed to connect to device");
            device.get_device_data().expect("Failed to get device data");

            // Test encoder configuration
            let mut options = crate::encoders::EncoderOptions::new();
            options.enabled = true;
            options.sampling_4x = true;

            match device.configure_encoder(0, 1, 2, options) {
                Ok(_) => {
                    println!("Configured encoder 0 on pins 1 and 2");

                    // Read encoder value multiple times
                    for i in 0..5 {
                        match device.get_encoder_value(0) {
                            Ok(value) => {
                                println!("Encoder 0 reading {}: {}", i + 1, value);
                            }
                            Err(e) => {
                                println!("WARNING: Could not read encoder: {}", e);
                            }
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(e) => {
                    println!("WARNING: Could not configure encoder: {}", e);
                }
            }
        }

        #[test]
        fn test_hardware_network_discovery() {
            println!("Testing hardware network device discovery...");

            match enumerate_network_devices(2000) {
                Ok(devices) => {
                    println!("Found {} network PoKeys devices", devices.len());

                    for (i, device) in devices.iter().enumerate() {
                        println!(
                            "Network device {}: Serial {}, IP {}.{}.{}.{}",
                            i + 1,
                            device.serial_number,
                            device.ip_address[0],
                            device.ip_address[1],
                            device.ip_address[2],
                            device.ip_address[3]
                        );
                    }

                    if devices.is_empty() {
                        println!(
                            "No network devices found - this is normal if no network PoKeys devices are present"
                        );
                    }
                }
                Err(e) => {
                    println!("WARNING: Network discovery failed: {}", e);
                }
            }
        }

        #[test]
        fn test_hardware_device_info_validation() {
            println!("Testing hardware device info validation...");

            let device_count = enumerate_usb_devices().expect("Failed to enumerate devices");
            if device_count == 0 {
                println!("SKIP: No PoKeys devices found for hardware test");
                return;
            }

            let mut device = connect_to_device(0).expect("Failed to connect to device");
            device.get_device_data().expect("Failed to get device data");

            // Validate device information makes sense
            assert!(
                device.device_data.serial_number > 0,
                "Serial number should be non-zero"
            );
            assert!(device.info.pin_count > 0, "Pin count should be non-zero");
            assert!(
                device.info.pin_count <= 100,
                "Pin count should be reasonable"
            );

            println!("Device validation passed:");
            println!("  Serial: {}", device.device_data.serial_number);
            println!("  Pins: {}", device.info.pin_count);
            println!("  PWM Channels: {}", device.info.pwm_count);
            println!("  Encoders: {}", device.info.encoders_count);
            println!("  Analog Inputs: {}", device.info.analog_inputs);
        }
    }
}
