//! Device enumeration, connection, and management

use crate::communication::{CommunicationManager, NetworkInterface, UsbHidInterface};
use crate::encoders::EncoderData;
use crate::error::{PoKeysError, Result};
use crate::io::{PinData, PinFunction};
use crate::keyboard_matrix::{MatrixKeyboard, MatrixKeyboardConfig};
use crate::lcd::LcdData;
use crate::matrix::MatrixLed;
use crate::pulse_engine::PulseEngineV2;
use crate::pwm::PwmData;
use crate::sensors::EasySensor;
use crate::types::*;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

/// Matrix keyboard ID byte (spec § Matrix keyboard, byte 4).
///
/// Hard-coded to 0: every current PoKeys firmware variant exposes exactly
/// one matrix keyboard, addressed as keyboard 0. The field exists in the
/// spec for forward compatibility (multi-keyboard or PoNET/I²C-mapped
/// keyboard variants), and may need to be promoted to a parameter on the
/// public matrix-keyboard methods if such variants ship. Until then, the
/// constant keeps the API simple and prevents callers from passing a value
/// that today would just be silently ignored.
///
/// If the firmware ever defines additional keyboard IDs, this becomes a
/// parameter on [`PoKeysDevice::configure_matrix_keyboard`] and friends —
/// a breaking change that should ride a major version bump.
const MATRIX_KEYBOARD_ID: u8 = 0;

/// Encode 16 keys (codes + modifiers, optionally a triggered-mode bitmap) into the
/// `data` payload for a `0xCA` key-mapping write (options 2-9 / 22-29).
///
/// Layout matches spec § Matrix keyboard:
/// - data[0..16]  → bytes 9-24:  16 key codes
/// - data[16..32] → bytes 25-40: 16 key modifiers
/// - data[32..34] → bytes 41-42: triggered-mode bitmap (16 bits, LSB-first; only for down-event options 2-9)
fn encode_matrix_kb_mapping_chunk(
    codes: &[u8],
    modifiers: &[u8],
    triggered: Option<&[u8]>,
    chunk_base: usize,
) -> [u8; 34] {
    let mut data = [0u8; 34];
    for i in 0..16 {
        let idx = chunk_base + i;
        if idx < codes.len() {
            data[i] = codes[idx];
        }
        if idx < modifiers.len() {
            data[16 + i] = modifiers[idx];
        }
    }
    if let Some(triggered) = triggered {
        let mut bitmap: u16 = 0;
        for i in 0..16 {
            let idx = chunk_base + i;
            if idx < triggered.len() && triggered[idx] != 0 {
                bitmap |= 1 << i;
            }
        }
        data[32] = (bitmap & 0xFF) as u8;
        data[33] = (bitmap >> 8) as u8;
    }
    data
}

/// Decode a 16-key key-mapping read response into the supplied buffers.
/// Inverse of [`encode_matrix_kb_mapping_chunk`].
fn decode_matrix_kb_mapping_chunk(
    response: &[u8],
    codes: &mut [u8],
    modifiers: &mut [u8],
    triggered: Option<&mut [u8]>,
    chunk_base: usize,
) {
    // response[8..24] = codes, response[24..40] = modifiers, response[40..42] = triggered bitmap
    for i in 0..16 {
        let idx = chunk_base + i;
        if idx < codes.len() {
            codes[idx] = response[8 + i];
        }
        if idx < modifiers.len() {
            modifiers[idx] = response[24 + i];
        }
    }
    if let Some(triggered) = triggered {
        let bitmap = u16::from_le_bytes([response[40], response[41]]);
        for i in 0..16 {
            let idx = chunk_base + i;
            if idx < triggered.len() {
                triggered[idx] = if (bitmap & (1 << i)) != 0 { 1 } else { 0 };
            }
        }
    }
}

/// Build the option-16 configuration payload for `0xCA`.
///
/// Caller is expected to validate `width` (1-8) and `height` (1-16) and that the
/// pin slices contain at least `width` / `height` entries.
///
/// Layout matches spec § Matrix keyboard:
/// - `data[0]`      = byte 9:  enable bit
/// - `data[1]`      = byte 10: (width-1) << 4 | (height-1)
/// - `data[2..10]`  = bytes 11-18: row pins 0-7
/// - `data[10..18]` = bytes 19-26: column pins 0-7
/// - `data[18..34]` = bytes 27-42: direct/macro bitmap (zero-filled = all direct)
/// - `data[34..42]` = bytes 43-50: row pins 8-15 (only when height > 8)
/// - `data[42]`     = byte 51:  alternate-function pin (0 = disabled)
fn encode_matrix_kb_config_payload(
    width: u8,
    height: u8,
    column_pins: &[u8],
    row_pins: &[u8],
) -> [u8; 55] {
    let mut data = [0u8; 55];
    data[0] = 1;
    data[1] = ((width - 1) << 4) | (height - 1);

    for (i, &pin) in row_pins.iter().enumerate().take(8) {
        data[2 + i] = pin.saturating_sub(1);
    }
    for (i, &pin) in column_pins.iter().enumerate().take(8) {
        data[10 + i] = pin.saturating_sub(1);
    }
    if height > 8 {
        for (offset, &pin) in row_pins.iter().enumerate().skip(8).take(8) {
            data[34 + (offset - 8)] = pin.saturating_sub(1);
        }
    }

    data
}

/// Parse a `0xCA` option-1 (or any option < 12) readback response into a
/// [`MatrixKeyboardConfig`].
///
/// Spec § Matrix keyboard, option < 12 readback layout:
/// - `response[8]`      = byte 9:  config (bit 0 = enable)
/// - `response[9]`      = byte 10: size — bit 0-3 = height-1, bit 4-7 = width-1
/// - `response[10..18]` = bytes 11-18: row pins 0-7 (0-based pin codes)
/// - `response[18..26]` = bytes 19-26: column pins 0-7
/// - `response[26..42]` = bytes 27-42: direct/macro bitmap (16 bytes, 128 bits)
/// - `response[42..50]` = bytes 43-50: row pins 8-15 (only when height > 8)
/// - `response[50]`     = byte 51: alternate-function pin (pinID+1, 0 = disabled)
/// - `response[51]`     = byte 52: scanning decimation (0..=50)
fn parse_matrix_kb_config_readback(response: &[u8]) -> Result<MatrixKeyboardConfig> {
    if response.len() < 2 || response[1] != 0xCA {
        return Err(PoKeysError::Protocol(format!(
            "matrix keyboard config readback: echo mismatch: 0x{:02X}",
            response.get(1).copied().unwrap_or(0)
        )));
    }
    // Need at least bytes 9-52 (response[8..52]) to populate every field.
    if response.len() < 52 {
        return Err(PoKeysError::Protocol(format!(
            "matrix keyboard config readback too short: {} bytes (need 52)",
            response.len()
        )));
    }

    let enabled = (response[8] & 0x01) != 0;
    let size_byte = response[9];
    let width = (size_byte >> 4) + 1;
    let height = (size_byte & 0x0F) + 1;

    let mut row_pins = [0u8; 16];
    for (i, slot) in row_pins.iter_mut().enumerate().take(8) {
        // Wire byte is 0-based pin code; expose as 1-based, with 0 meaning
        // "no pin assigned" (consistent with the rest of MatrixKeyboardConfig).
        *slot = response[10 + i].wrapping_add(1);
    }
    for (offset, slot) in row_pins.iter_mut().enumerate().skip(8).take(8) {
        *slot = response[42 + (offset - 8)].wrapping_add(1);
    }

    let mut column_pins = [0u8; 8];
    for (i, slot) in column_pins.iter_mut().enumerate() {
        *slot = response[18 + i].wrapping_add(1);
    }

    let mut direct_macro_bitmap = [0u8; 16];
    direct_macro_bitmap.copy_from_slice(&response[26..42]);

    let alternate_function_pin = response[50];
    let scanning_decimation = response[51];

    Ok(MatrixKeyboardConfig {
        enabled,
        width,
        height,
        row_pins,
        column_pins,
        direct_macro_bitmap,
        alternate_function_pin,
        scanning_decimation,
    })
}

/// Compare a parsed `MatrixKeyboardConfig` against an expected configuration
/// and return a `Protocol` error describing the first mismatch (if any).
///
/// Used by [`PoKeysDevice::configure_matrix_keyboard`] and
/// [`PoKeysDevice::disable_matrix_keyboard`] to catch firmware silent
/// rejections where option-16 echoes `0xCA` but the device's stored
/// configuration didn't actually change (e.g. configuration locked).
fn verify_matrix_kb_config_matches(
    actual: &MatrixKeyboardConfig,
    expected_width: u8,
    expected_height: u8,
    expected_column_pins: &[u8],
    expected_row_pins: &[u8],
    expected_enabled: bool,
) -> Result<()> {
    if actual.enabled != expected_enabled {
        return Err(PoKeysError::Protocol(format!(
            "configure_matrix_keyboard: device reports enabled={}, expected {expected_enabled} \
             (configuration may be locked; raw config byte=0x{:02X})",
            actual.enabled,
            if actual.enabled { 0x01 } else { 0x00 }
        )));
    }
    // When we just disabled, the device may report stale or zero dimensions
    // / pins; only width / height / pins are interesting on the enabled side.
    if !expected_enabled {
        return Ok(());
    }

    if actual.width != expected_width || actual.height != expected_height {
        return Err(PoKeysError::Protocol(format!(
            "configure_matrix_keyboard: device reports width={} height={}, \
             expected width={expected_width} height={expected_height} (raw size byte=0x{:02X})",
            actual.width,
            actual.height,
            actual.size_byte()
        )));
    }
    for (i, &pin) in expected_row_pins
        .iter()
        .enumerate()
        .take(expected_height as usize)
        .take(8)
    {
        if actual.row_pins[i] != pin {
            return Err(PoKeysError::Protocol(format!(
                "configure_matrix_keyboard: row pin {i} mismatch: device has pin {}, expected pin {pin}",
                actual.row_pins[i]
            )));
        }
    }
    for (i, &pin) in expected_column_pins
        .iter()
        .enumerate()
        .take(expected_width as usize)
        .take(8)
    {
        if actual.column_pins[i] != pin {
            return Err(PoKeysError::Protocol(format!(
                "configure_matrix_keyboard: column pin {i} mismatch: device has pin {}, expected pin {pin}",
                actual.column_pins[i]
            )));
        }
    }
    if expected_height > 8 {
        for (offset, &pin) in expected_row_pins
            .iter()
            .enumerate()
            .skip(8)
            .take((expected_height as usize).saturating_sub(8))
            .take(8)
        {
            if actual.row_pins[offset] != pin {
                return Err(PoKeysError::Protocol(format!(
                    "configure_matrix_keyboard: extended row pin {offset} mismatch: device has pin {}, expected pin {pin}",
                    actual.row_pins[offset]
                )));
            }
        }
    }
    Ok(())
}

fn validate_matrix_kb_mapping_response(response: &[u8], option: u8) -> Result<()> {
    if response[1] != 0xCA {
        return Err(PoKeysError::Protocol(format!(
            "Invalid response command for key mapping read (option {option}): 0x{:02X}",
            response[1]
        )));
    }
    if response.len() < 42 {
        return Err(PoKeysError::Protocol(format!(
            "Key mapping response too short for option {option}: {} bytes (need 42)",
            response.len()
        )));
    }
    Ok(())
}

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

    // I2C configuration and metrics
    pub i2c_config: I2cConfig,
    pub i2c_metrics: I2cMetrics,
    pub validation_level: ValidationLevel,
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
            i2c_config: I2cConfig::default(),
            i2c_metrics: I2cMetrics::default(),
            validation_level: ValidationLevel::None,
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

    /// Get the device's current system load as a percentage (0–100).
    ///
    /// Sends the "Get system load status" command (`0x05`) defined in the
    /// PoKeys protocol specification. The device replies with the current
    /// CPU/system load in byte 3 of the response.
    pub fn get_system_load(&mut self) -> Result<u8> {
        let response = self.send_request(0x05, 0, 0, 0, 0)?;
        Ok(parse_system_load_response(&response))
    }

    /// Set the device name (up to 20 bytes for the long device name).
    ///
    /// Sends protocol command `0x06` with "write long device name" flags set.
    /// After the write succeeds, [`save_configuration`](PoKeysDevice::save_configuration)
    /// is called to persist the value to non-volatile storage — `0x06` alone
    /// does not auto-save.
    ///
    /// # Side effect
    ///
    /// The request packet includes fields for the joystick device name
    /// (spec bytes 19–34) and the product-ID offset (byte 35) that this
    /// implementation always sends as zero. If the target device was using
    /// those fields, they will be cleared as a side effect of this call.
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
                } else {
                    return Err(PoKeysError::NotConnected);
                }
            }
            DeviceConnectionType::NetworkDevice => {
                if let Some(ref mut interface) = self.network_interface {
                    interface.send(&request)?;

                    // Use timeout for network response to avoid hanging
                    let mut response = [0u8; 64];
                    let _ = interface
                        .receive_timeout(&mut response, std::time::Duration::from_millis(2000));
                } else {
                    return Err(PoKeysError::NotConnected);
                }
            }
        }

        self.save_configuration()
    }

    /// Reset device configuration to defaults (protocol command `0x52`
    /// "Disable lock and reset configuration"). Requires the magic bytes
    /// `0xAA, 0x55` to match the spec and the upstream PoKeysLib behaviour.
    ///
    /// On observed network firmware, this command interrupts the connection
    /// mid-call (similar to [`Self::reboot_device`]). Transfer-level errors
    /// after the request was sent are therefore treated as success — the
    /// device received the command and is resetting / re-establishing its
    /// state, just couldn't reply.
    ///
    /// After this call the active connection may be invalid; callers
    /// observing further command failures should reconnect before issuing
    /// additional commands.
    pub fn clear_configuration(&mut self) -> Result<()> {
        match self.send_request(0x52, 0xAA, 0x55, 0, 0) {
            Ok(_) => Ok(()),
            // The device may reset / drop the connection before replying.
            Err(PoKeysError::Transfer(_)) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Returns `true` if the device's configuration is currently locked.
    ///
    /// Reads byte 59 of the `0x00` "Read device data" response (spec
    /// "configuration lock status"). When locked, the firmware echoes
    /// configuration writes (option 16, pin-function writes, etc.) but
    /// does not apply them — the silent-failure mode that prompted the
    /// readback verification on [`Self::configure_matrix_keyboard`].
    ///
    /// Use this to preflight a configuration session and either fail fast
    /// with a clear message, or call [`Self::clear_configuration`] to
    /// disable the lock first.
    pub fn is_configuration_locked(&mut self) -> Result<bool> {
        self.read_device_data()?;
        Ok(self.device_data.device_lock_status != 0)
    }

    /// Reboot the device (command `0xF3`).
    ///
    /// Sends the "Reboot system" command defined in the PoKeys protocol
    /// specification. The device may reboot before a response reaches the
    /// host; transfer-level failures caused by the interrupted response are
    /// treated as success, as the request was already delivered.
    ///
    /// After a successful reboot the active connection is effectively
    /// invalidated — callers should re-enumerate and reconnect before issuing
    /// further commands.
    pub fn reboot_device(&mut self) -> Result<()> {
        match self.send_request(0xF3, 0, 0, 0, 0) {
            Ok(_) => Ok(()),
            // The device typically reboots before responding, so a transfer
            // error after the request was sent is expected and not fatal.
            Err(PoKeysError::Transfer(_)) => Ok(()),
            Err(e) => Err(e),
        }
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

    /// Set ethernet retry count and timeout.
    #[deprecated(
        since = "1.0.5",
        note = "Use `set_network_timeout` and `set_network_retries` separately. \
                The `read_retries` parameter has never been honoured by the \
                retry loops in this crate; passing any value here is a no-op."
    )]
    pub fn set_ethernet_retry_count_and_timeout(
        &mut self,
        send_retries: u32,
        _read_retries: u32,
        timeout_ms: u32,
    ) {
        self.communication.set_retries_and_timeout(
            send_retries,
            0,
            Duration::from_millis(timeout_ms as u64),
        );
    }

    /// Set the timeout applied to UDP/TCP receive calls during
    /// `send_request`.
    ///
    /// Each network `send_request` blocks up to this duration waiting for a
    /// reply before retrying. Default: 1000 ms.
    ///
    /// **Tuning guide:**
    /// - On a healthy LAN, PoKeys replies in 1–5 ms. A timeout of
    ///   50–200 ms is reasonable for a latency-sensitive UI.
    /// - Setting this too low (below ~20 ms) can cause spurious retries
    ///   on a healthy device under OS scheduler jitter.
    /// - **Network-only.** USB uses a fixed internal 50 × 20 ms polling
    ///   loop and is not affected by this setter.
    ///
    /// Use [`Self::tune_for_realtime_polling`] for a sensible preset.
    pub fn set_network_timeout(&mut self, timeout: Duration) {
        let send_retries = self.communication.send_retries();
        self.communication
            .set_retries_and_timeout(send_retries, 0, timeout);
    }

    /// Set how many times `send_request` retries on a network timeout
    /// before returning [`crate::PoKeysError::Transfer`].
    ///
    /// Each retry costs up to one full [`Self::network_timeout`] of
    /// blocking plus one WARN log line (subject to rate-limiting).
    /// Default: 3.
    ///
    /// For latency-sensitive pollers, consider setting this to 1: a
    /// dropped UDP reply then costs one timeout period instead of three,
    /// and the caller can drive higher-level recovery policy itself.
    pub fn set_network_retries(&mut self, retries: u32) {
        let timeout = self.communication.socket_timeout();
        self.communication
            .set_retries_and_timeout(retries, 0, timeout);
    }

    /// Current network receive timeout. See [`Self::set_network_timeout`].
    pub fn network_timeout(&self) -> Duration {
        self.communication.socket_timeout()
    }

    /// Current per-`send_request` retry count on network timeouts.
    /// See [`Self::set_network_retries`].
    pub fn send_retries(&self) -> u32 {
        self.communication.send_retries()
    }

    /// Tune the network retry loop for a latency-sensitive polling use
    /// case (e.g. a live UI or dashboard reading device state at tens
    /// of Hz).
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// device.set_network_timeout(Duration::from_millis(100));
    /// device.set_network_retries(1);
    /// ```
    ///
    /// A dropped reply then costs one 100 ms blocking call instead of
    /// three 1-second calls. Your application's tick budget is preserved
    /// and application-level recovery policy (skip / backoff / reconnect)
    /// takes precedence over library-internal retries.
    pub fn tune_for_realtime_polling(&mut self) {
        self.communication
            .set_retries_and_timeout(1, 0, Duration::from_millis(100));
    }

    /// Get connection type
    pub fn get_connection_type(&self) -> DeviceConnectionType {
        self.connection_type
    }

    /// Set I2C configuration
    pub fn set_i2c_config(&mut self, config: I2cConfig) {
        self.i2c_config = config;
    }

    /// Get I2C configuration
    pub fn get_i2c_config(&self) -> &I2cConfig {
        &self.i2c_config
    }

    /// Set validation level
    pub fn set_validation_level(&mut self, level: ValidationLevel) {
        self.validation_level = level;
    }

    /// Get I2C metrics
    pub fn get_i2c_metrics(&self) -> &I2cMetrics {
        &self.i2c_metrics
    }

    /// Reset I2C metrics
    pub fn reset_i2c_metrics(&mut self) {
        self.i2c_metrics = I2cMetrics::default();
    }

    /// Perform device health check
    pub fn health_check(&mut self) -> HealthStatus {
        HealthStatus {
            connectivity: self.test_connectivity(),
            i2c_health: self.test_i2c_health(),
            error_rate: self.calculate_error_rate(),
            performance: self.get_performance_summary(),
        }
    }

    /// Test basic connectivity
    fn test_connectivity(&mut self) -> ConnectivityStatus {
        match self.get_device_data() {
            Ok(_) => ConnectivityStatus::Healthy,
            Err(e) => ConnectivityStatus::Degraded(e.to_string()),
        }
    }

    /// Test I2C bus health
    fn test_i2c_health(&mut self) -> I2cHealthStatus {
        match self.i2c_get_status() {
            Ok(I2cStatus::Ok) => I2cHealthStatus::Healthy,
            Ok(status) => I2cHealthStatus::Degraded(format!("I2C status: {:?}", status)),
            Err(e) => I2cHealthStatus::Failed(e.to_string()),
        }
    }

    /// Calculate current error rate
    fn calculate_error_rate(&self) -> f64 {
        if self.i2c_metrics.total_commands == 0 {
            0.0
        } else {
            self.i2c_metrics.failed_commands as f64 / self.i2c_metrics.total_commands as f64
        }
    }

    /// Get performance summary
    fn get_performance_summary(&self) -> PerformanceSummary {
        let success_rate = if self.i2c_metrics.total_commands == 0 {
            1.0
        } else {
            self.i2c_metrics.successful_commands as f64 / self.i2c_metrics.total_commands as f64
        };

        PerformanceSummary {
            avg_response_time_ms: self.i2c_metrics.average_response_time.as_millis() as f64,
            success_rate,
            throughput_commands_per_sec: 0.0, // Would need timing data to calculate
        }
    }

    /// Validate packet according to current validation level
    #[allow(dead_code)]
    fn validate_packet(&self, data: &[u8]) -> Result<()> {
        match &self.validation_level {
            ValidationLevel::None => Ok(()),
            ValidationLevel::Basic => self.validate_basic_structure(data),
            ValidationLevel::Strict => self.validate_strict_protocol(data),
            ValidationLevel::Custom(config) => self.validate_custom(data, config),
        }
    }

    /// Validate basic packet structure
    #[allow(dead_code)]
    fn validate_basic_structure(&self, data: &[u8]) -> Result<()> {
        if data.len() < 3 {
            return Err(PoKeysError::InvalidPacketStructure(
                "Minimum packet size is 3 bytes".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate strict protocol compliance
    #[allow(dead_code)]
    fn validate_strict_protocol(&self, data: &[u8]) -> Result<()> {
        if data.len() < 3 {
            return Err(PoKeysError::InvalidPacketStructure(
                "Minimum packet size is 3 bytes".to_string(),
            ));
        }

        let command = data[0];
        let device_id = data[1];
        let checksum = data[data.len() - 1];

        // Validate command ID (uSPIBridge command range)
        if !matches!(command, 0x11..=0x42) {
            return Err(PoKeysError::InvalidCommand(command));
        }

        // Validate device ID
        if device_id >= 16 {
            // Reasonable max device count
            return Err(PoKeysError::InvalidDeviceId(device_id));
        }

        // Validate checksum
        let calculated_checksum = self.calculate_checksum(&data[..data.len() - 1]);
        if checksum != calculated_checksum {
            return Err(PoKeysError::InvalidChecksumDetailed {
                expected: calculated_checksum,
                received: checksum,
            });
        }

        Ok(())
    }

    /// Validate with custom configuration
    #[allow(dead_code)]
    fn validate_custom(&self, data: &[u8], config: &crate::types::ValidationConfig) -> Result<()> {
        if config.validate_packet_structure && data.len() < 3 {
            return Err(PoKeysError::InvalidPacketStructure(
                "Minimum packet size is 3 bytes".to_string(),
            ));
        }

        if data.len() >= 3 {
            let command = data[0];
            let device_id = data[1];
            let checksum = data[data.len() - 1];

            if config.validate_command_ids
                && !config.valid_commands.is_empty()
                && !config.valid_commands.contains(&command)
            {
                return Err(PoKeysError::InvalidCommand(command));
            }

            if config.validate_device_ids && device_id > config.max_device_id {
                return Err(PoKeysError::InvalidDeviceId(device_id));
            }

            if config.validate_checksums {
                let calculated_checksum = self.calculate_checksum(&data[..data.len() - 1]);
                if checksum != calculated_checksum {
                    return Err(PoKeysError::InvalidChecksumDetailed {
                        expected: calculated_checksum,
                        received: checksum,
                    });
                }
            }
        }

        Ok(())
    }

    /// Calculate XOR checksum
    #[allow(dead_code)]
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0, |acc, &byte| acc ^ byte)
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

        Ok((discovery_info, config))
    }

    /// Set network configuration on the device.
    ///
    /// Sends command `0xE0` with option `10`, which writes **and saves** the
    /// configuration to non-volatile storage in one operation — no separate
    /// [`save_configuration`](PoKeysDevice::save_configuration) call is needed.
    ///
    /// # Field mapping
    ///
    /// | `NetworkDeviceInfo` field      | Protocol byte (doc) | Notes |
    /// |-------------------------------|---------------------|-------|
    /// | `dhcp`                        | 9                   | 0 = fixed IP, 1 = DHCP |
    /// | `ip_address_setup`            | 10–13               | Applied when `dhcp == 0` |
    /// | `tcp_timeout` (ms)            | 18–19               | Stored in units of 100 ms |
    /// | `gateway_ip`                  | 20–23               | Applied when non-zero |
    /// | `subnet_mask`                 | 24–27               | Applied when non-zero |
    /// | `additional_network_options`  | 29                  | Upper nibble forced to `0xA` |
    ///
    /// `ip_address_current` is read-only (assigned by DHCP) and is ignored here.
    pub fn set_network_configuration(&mut self, config: &NetworkDeviceInfo) -> Result<()> {
        // TCP timeout is stored in units of 100 ms; NetworkDeviceInfo holds ms.
        let timeout_units = (config.tcp_timeout / 100).max(1);
        let timeout_bytes = timeout_units.to_le_bytes();

        // Set gateway/subnet flag: tell the device to apply those fields too.
        let gateway_subnet_set: u8 =
            if config.gateway_ip != [0, 0, 0, 0] || config.subnet_mask != [0, 0, 0, 0] {
                1
            } else {
                0
            };

        // Upper nibble of the options byte must always be 0xA (protocol requirement).
        let options = (config.additional_network_options & 0x0F) | 0xA0;

        // Build the data payload (doc bytes 9–29, 0-based bytes 8–28).
        let mut data = [0u8; 21];
        data[0] = config.dhcp; // doc byte  9: IP setup
        data[1..5].copy_from_slice(&config.ip_address_setup); // doc bytes 10-13: fixed IP
        // data[5..9] = reserved (zeros)               // doc bytes 14-17
        data[9] = timeout_bytes[0]; // doc bytes 18-19: TCP timeout (LE)
        data[10] = timeout_bytes[1];
        data[11..15].copy_from_slice(&config.gateway_ip); // doc bytes 20-23
        data[15..19].copy_from_slice(&config.subnet_mask); // doc bytes 24-27
        data[19] = gateway_subnet_set; // doc byte  28: apply gateway/subnet
        data[20] = options; // doc byte  29: additional options

        // option = 10 triggers write + save (per spec footnote 17 and 18).
        self.send_request_with_data(0xE0, 10, 0, 0, 0, &data)?;
        Ok(())
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

            // Configuration lock status (byte 59 in doc = byte 58 in 0-based).
            // Non-zero = configuration is locked; option-16 / pin-function
            // writes will be silently rejected until cleared via 0x52.
            let configuration_lock_status = response[58];

            // Update device data structure
            self.device_data.serial_number = serial_32bit;
            self.device_data.device_lock_status = configuration_lock_status;

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
        // PWM data is already initialized in new()
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

    /// Command 0xCA: Configure Matrix Keyboard
    ///
    /// Configures a matrix keyboard using the official PoKeys protocol.
    /// Uses command 0xCA with option 16 for configuration.
    ///
    /// # Pin function prerequisite (user manual § 8.5)
    /// The PoKeys user manual states: "Make sure the selected pin is configured
    /// as digital input [for columns] / digital output [for rows], then check
    /// the 'Matrix keyboard' option for the pin..." Without this, the firmware
    /// acknowledges option-16 but does not scan — it cannot drive a row that
    /// isn't configured as `DigitalOutput`, nor read a column that isn't
    /// configured as `DigitalInput` with its internal pull-up active.
    ///
    /// This method enforces the prerequisite by issuing a single bulk pin-
    /// function write (`0xC0`) before the option-16 configure: it reads the
    /// current 55-byte pin-settings array (preserving the invert bit on
    /// untouched pins), forces row pins to `DigitalOutput` and column pins to
    /// `DigitalInput`, and writes the array back. Total cost: 2 extra round
    /// trips on top of the configure, regardless of matrix size.
    ///
    /// # Arguments
    /// * `width` - Number of columns (1-8)
    /// * `height` - Number of rows (1-16)
    /// * `column_pins` - Pin numbers for columns (1-based, 1..=55); must be at least `width` long
    /// * `row_pins` - Pin numbers for rows (1-based, 1..=55); must be at least `height` long
    ///
    /// # Errors
    /// Returns [`PoKeysError::Parameter`] if `width`/`height` are out of
    /// range, slices are too short, a pin number is outside 1..=55, or the
    /// same pin appears in both rows and columns.
    ///
    /// # Protocol Details (spec § Matrix keyboard, option 16)
    /// - `data[0]`      = byte 9: enable bit
    /// - `data[1]`      = byte 10: (width-1) << 4 | (height-1)
    /// - `data[2..10]`  = bytes 11-18: row pins 0-7 (0-based pin codes)
    /// - `data[10..18]` = bytes 19-26: column pins 0-7
    /// - `data[18..34]` = bytes 27-42: direct/macro bitmap (zero = direct mapping)
    /// - `data[34..42]` = bytes 43-50: row pins 8-15 (only sent when height > 8)
    /// - `data[42]`     = byte 51: alternate-function pin (0 = disabled)
    pub fn configure_matrix_keyboard(
        &mut self,
        width: u8,
        height: u8,
        column_pins: &[u8],
        row_pins: &[u8],
    ) -> Result<()> {
        if width == 0 || width > 8 {
            return Err(PoKeysError::Parameter(format!(
                "Matrix width must be 1-8 (got {width})"
            )));
        }
        if height == 0 || height > 16 {
            return Err(PoKeysError::Parameter(format!(
                "Matrix height must be 1-16 (got {height})"
            )));
        }
        if column_pins.len() < width as usize {
            return Err(PoKeysError::Parameter(format!(
                "column_pins has {} entries, need at least {width}",
                column_pins.len()
            )));
        }
        if row_pins.len() < height as usize {
            return Err(PoKeysError::Parameter(format!(
                "row_pins has {} entries, need at least {height}",
                row_pins.len()
            )));
        }

        let used_rows = &row_pins[..height as usize];
        let used_cols = &column_pins[..width as usize];
        for &pin in used_rows.iter().chain(used_cols.iter()) {
            if pin == 0 || pin > 55 {
                return Err(PoKeysError::Parameter(format!(
                    "Matrix keyboard pin {pin} is out of range (must be 1-55)"
                )));
            }
        }
        for &row in used_rows {
            if used_cols.contains(&row) {
                return Err(PoKeysError::Parameter(format!(
                    "Pin {row} cannot be used as both a row and a column"
                )));
            }
        }

        // Spec / user manual prerequisite: the firmware drives rows as digital
        // outputs and reads columns as digital inputs. If a pin's function
        // isn't already set accordingly, option-16 is acknowledged but the
        // matrix doesn't scan. Apply the requirement here so callers don't
        // have to know.
        //
        // Use raw read/write (not the PinFunction view) so we preserve the
        // invert flag (bit 7 / 0x80) on every pin we don't touch.
        let mut raw = self.read_all_pin_settings_raw()?;
        for &pin in used_rows {
            let idx = (pin - 1) as usize;
            let invert = raw[idx] & 0x80;
            raw[idx] = (PinFunction::DigitalOutput as u8) | invert;
        }
        for &pin in used_cols {
            let idx = (pin - 1) as usize;
            let invert = raw[idx] & 0x80;
            raw[idx] = (PinFunction::DigitalInput as u8) | invert;
        }
        self.set_all_pin_settings_raw(&raw)?;

        let data = encode_matrix_kb_config_payload(width, height, column_pins, row_pins);

        let response = self.send_request_with_data(0xCA, 16, MATRIX_KEYBOARD_ID, 0, 0, &data)?;
        if response[1] != 0xCA {
            return Err(PoKeysError::Protocol(format!(
                "Invalid response command for matrix keyboard configure: 0x{:02X}",
                response[1]
            )));
        }

        // Read back via option 1 to confirm the configuration actually took
        // effect. Without this, a "configuration locked" or otherwise rejected
        // option-16 write would still echo 0xCA and we'd report Ok — the
        // silent-failure mode that prompted this guard.
        self.verify_matrix_kb_config_applied(
            width,
            height,
            column_pins,
            row_pins,
            /*enabled=*/ true,
        )?;

        // Update local state
        self.matrix_keyboard.configuration = 1;
        self.matrix_keyboard.width = width;
        self.matrix_keyboard.height = height;

        // Copy pin assignments verbatim (1-based, as the user supplied them)
        self.matrix_keyboard.column_pins.fill(0);
        self.matrix_keyboard.row_pins.fill(0);
        for (i, &pin) in column_pins.iter().enumerate().take(8) {
            self.matrix_keyboard.column_pins[i] = pin;
        }
        for (i, &pin) in row_pins.iter().enumerate().take(16) {
            self.matrix_keyboard.row_pins[i] = pin;
        }

        Ok(())
    }

    /// Read the current matrix-keyboard configuration via `0xCA` option 1
    /// and fail if it does not match the expected `(width, height,
    /// column_pins, row_pins, enabled)` tuple.
    ///
    /// Catches firmware silent rejections (e.g. configuration locked, an
    /// out-of-range pin code, or a pin the firmware refuses to use as
    /// row/col) where the option-16 write echoes `0xCA` but the device's
    /// stored configuration doesn't change.
    fn verify_matrix_kb_config_applied(
        &mut self,
        width: u8,
        height: u8,
        column_pins: &[u8],
        row_pins: &[u8],
        enabled: bool,
    ) -> Result<()> {
        let actual = self.get_matrix_keyboard_configuration()?;
        verify_matrix_kb_config_matches(&actual, width, height, column_pins, row_pins, enabled)
    }

    /// Command 0xCA, option 1: read the device's current matrix-keyboard
    /// configuration.
    ///
    /// Returns a [`MatrixKeyboardConfig`] reflecting exactly what the
    /// firmware has stored. Useful for:
    /// - Verifying a configure / disable call took effect (used internally
    ///   by [`Self::configure_matrix_keyboard`]).
    /// - Diff-against-device flows where the caller wants to know whether
    ///   the on-device state matches a desired configuration before writing.
    /// - Surfacing the scanning decimation, alt-function pin, and direct/
    ///   macro bitmap, which aren't otherwise readable.
    pub fn get_matrix_keyboard_configuration(&mut self) -> Result<MatrixKeyboardConfig> {
        let response = self.send_request(0xCA, 1, MATRIX_KEYBOARD_ID, 0, 0)?;
        parse_matrix_kb_config_readback(&response)
    }

    /// Command 0xCA, option 16: disable the matrix keyboard.
    ///
    /// The protocol has no dedicated disable command; the spec describes
    /// enablement as bit 0 of byte 9 in the option-16 payload, so disabling
    /// is just an option-16 write with the enable bit cleared. Pin codes are
    /// left at zero in the payload — the firmware will not scan a disabled
    /// keyboard, so they are irrelevant.
    pub fn disable_matrix_keyboard(&mut self) -> Result<()> {
        let data = [0u8; 55]; // data[0] = 0 → enable bit cleared
        let response = self.send_request_with_data(0xCA, 16, MATRIX_KEYBOARD_ID, 0, 0, &data)?;
        if response[1] != 0xCA {
            return Err(PoKeysError::Protocol(format!(
                "Invalid response command for matrix keyboard disable: 0x{:02X}",
                response[1]
            )));
        }
        // Confirm via option-1 readback that the enable bit cleared.
        // A locked configuration would echo 0xCA but leave bit 0 set.
        self.verify_matrix_kb_config_applied(0, 0, &[], &[], /*enabled=*/ false)?;
        self.matrix_keyboard.configuration = 0;
        Ok(())
    }

    /// Command 0xCA: Read Matrix Keyboard State
    ///
    /// Reads the current state of all keys in the matrix keyboard.
    /// Uses command 0xCA with option 20 to read the 16x8 matrix status.
    ///
    /// # Protocol Details (spec § Matrix keyboard, option 20)
    /// - Response bytes 9-24 (response[8..24]): 16-byte bitmap, one byte per row
    /// - Bit `c` of row `r` = 1 if the key at (row=r, col=c) is pressed
    /// - Key indexing: `row * 8 + col` (8-column internal layout regardless of configured width)
    pub fn read_matrix_keyboard(&mut self) -> Result<()> {
        let response = self.send_request(0xCA, 20, MATRIX_KEYBOARD_ID, 0, 0)?;

        if response[1] != 0xCA {
            return Err(PoKeysError::Protocol(format!(
                "Invalid response command for matrix keyboard read: 0x{:02X}",
                response[1]
            )));
        }
        if response.len() < 24 {
            return Err(PoKeysError::Protocol(format!(
                "Matrix keyboard status response too short: {} bytes (need 24)",
                response.len()
            )));
        }

        self.matrix_keyboard.key_values.fill(0);
        for (row, &byte_val) in response[8..24].iter().enumerate() {
            for col in 0..8 {
                let key_index = row * 8 + col;
                if key_index < self.matrix_keyboard.key_values.len() && (byte_val & (1 << col)) != 0
                {
                    self.matrix_keyboard.key_values[key_index] = 1;
                }
            }
        }

        Ok(())
    }

    /// Command 0xCA, option 50: set the matrix-keyboard scanning decimation (0-50).
    ///
    /// Higher values reduce the device's scan rate, useful for noisy keypads.
    pub fn set_matrix_keyboard_scanning_decimation(&mut self, decimation: u8) -> Result<()> {
        if decimation > 50 {
            return Err(PoKeysError::Parameter(format!(
                "scanning decimation must be 0-50 (got {decimation})"
            )));
        }

        let mut data = [0u8; 1];
        data[0] = decimation;
        let response = self.send_request_with_data(0xCA, 50, MATRIX_KEYBOARD_ID, 0, 0, &data)?;
        if response[1] != 0xCA {
            return Err(PoKeysError::Protocol(format!(
                "Invalid response command for scanning decimation: 0x{:02X}",
                response[1]
            )));
        }

        self.matrix_keyboard.scanning_decimation = decimation;
        Ok(())
    }

    /// Command 0xCA, options 2-9: write the down-event key mapping for all 128 keys.
    ///
    /// Sends the key codes, modifiers, and triggered-mode flags currently held in
    /// `self.matrix_keyboard.key_mapping_key_code`,
    /// `self.matrix_keyboard.key_mapping_key_modifier`, and
    /// `self.matrix_keyboard.key_mapping_triggered_key` (one byte per key, non-zero = triggered).
    ///
    /// # Protocol Details
    /// Each option writes 16 keys: option 2 → keys 0-15, option 3 → 16-31, ..., option 9 → 112-127.
    /// Per the spec footnote, even if `width < 8`, the unused columns of each row still consume
    /// indices 0-7 in the mapping (rows are always 8 keys wide internally).
    pub fn set_matrix_keyboard_key_mapping(&mut self) -> Result<()> {
        for option in 2u8..=9u8 {
            let chunk_base = (option as usize - 2) * 16;
            let data = encode_matrix_kb_mapping_chunk(
                &self.matrix_keyboard.key_mapping_key_code,
                &self.matrix_keyboard.key_mapping_key_modifier,
                Some(&self.matrix_keyboard.key_mapping_triggered_key),
                chunk_base,
            );
            let response =
                self.send_request_with_data(0xCA, option, MATRIX_KEYBOARD_ID, 0, 0, &data)?;
            if response[1] != 0xCA {
                return Err(PoKeysError::Protocol(format!(
                    "Invalid response command for key mapping write (option {option}): 0x{:02X}",
                    response[1]
                )));
            }
        }
        Ok(())
    }

    /// Command 0xCA, options 22-29: write the up-event key mapping for all 128 keys.
    ///
    /// Sends `self.matrix_keyboard.key_mapping_key_code_up` and
    /// `self.matrix_keyboard.key_mapping_key_modifier_up`. Up-event mapping only takes effect
    /// for keys whose triggered flag is set in the down mapping (see
    /// [`Self::set_matrix_keyboard_key_mapping`]).
    pub fn set_matrix_keyboard_key_mapping_up(&mut self) -> Result<()> {
        for option in 22u8..=29u8 {
            let chunk_base = (option as usize - 22) * 16;
            let data = encode_matrix_kb_mapping_chunk(
                &self.matrix_keyboard.key_mapping_key_code_up,
                &self.matrix_keyboard.key_mapping_key_modifier_up,
                None,
                chunk_base,
            );
            let response =
                self.send_request_with_data(0xCA, option, MATRIX_KEYBOARD_ID, 0, 0, &data)?;
            if response[1] != 0xCA {
                return Err(PoKeysError::Protocol(format!(
                    "Invalid response command for up-key mapping write (option {option}): 0x{:02X}",
                    response[1]
                )));
            }
        }
        Ok(())
    }

    /// Command 0xCA, options 12-19: read the down-event key mapping for all 128 keys.
    ///
    /// Populates `self.matrix_keyboard.key_mapping_key_code`,
    /// `self.matrix_keyboard.key_mapping_key_modifier`, and
    /// `self.matrix_keyboard.key_mapping_triggered_key` from the device.
    pub fn get_matrix_keyboard_key_mapping(&mut self) -> Result<()> {
        for option in 12u8..=19u8 {
            let chunk_base = (option as usize - 12) * 16;
            let response = self.send_request(0xCA, option, MATRIX_KEYBOARD_ID, 0, 0)?;
            validate_matrix_kb_mapping_response(&response, option)?;
            decode_matrix_kb_mapping_chunk(
                &response,
                &mut self.matrix_keyboard.key_mapping_key_code,
                &mut self.matrix_keyboard.key_mapping_key_modifier,
                Some(&mut self.matrix_keyboard.key_mapping_triggered_key),
                chunk_base,
            );
        }
        Ok(())
    }

    /// Command 0xCA, options 32-39: read the up-event key mapping for all 128 keys.
    ///
    /// Populates `self.matrix_keyboard.key_mapping_key_code_up` and
    /// `self.matrix_keyboard.key_mapping_key_modifier_up` from the device.
    pub fn get_matrix_keyboard_key_mapping_up(&mut self) -> Result<()> {
        for option in 32u8..=39u8 {
            let chunk_base = (option as usize - 32) * 16;
            let response = self.send_request(0xCA, option, MATRIX_KEYBOARD_ID, 0, 0)?;
            validate_matrix_kb_mapping_response(&response, option)?;
            decode_matrix_kb_mapping_chunk(
                &response,
                &mut self.matrix_keyboard.key_mapping_key_code_up,
                &mut self.matrix_keyboard.key_mapping_key_modifier_up,
                None,
                chunk_base,
            );
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

    /// Configure and control a servo motor
    pub fn configure_servo(&mut self, config: crate::pwm::ServoConfig) -> Result<()> {
        // Ensure PWM period is set for servo control (20ms = 500,000 cycles)
        if self.pwm.pwm_period == 0 {
            self.set_pwm_period(500000)?; // 20ms period for servo control
        }

        // Enable PWM for the servo pin
        self.enable_pwm_for_pin(config.pin, true)?;

        Ok(())
    }

    /// Set servo angle (for position servos)
    pub fn set_servo_angle(&mut self, config: &crate::pwm::ServoConfig, angle: f32) -> Result<()> {
        config.set_angle(self, angle)
    }

    /// Set servo speed (for speed servos)
    pub fn set_servo_speed(&mut self, config: &crate::pwm::ServoConfig, speed: f32) -> Result<()> {
        config.set_speed(self, speed)
    }

    /// Stop servo (for speed servos)
    pub fn stop_servo(&mut self, config: &crate::pwm::ServoConfig) -> Result<()> {
        config.stop(self)
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

/// Parse the "Get system load status" (0x05) response payload.
///
/// Protocol byte 3 (0-based index 2) holds the system load percentage.
fn parse_system_load_response(response: &[u8]) -> u8 {
    response[2]
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
    fn test_parse_system_load_response() {
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[1] = 0x05; // echo of command ID in byte 2
        response[2] = 42; // system load % in byte 3
        assert_eq!(parse_system_load_response(&response), 42);

        response[2] = 0;
        assert_eq!(parse_system_load_response(&response), 0);

        response[2] = 100;
        assert_eq!(parse_system_load_response(&response), 100);
    }

    #[test]
    fn test_encode_matrix_kb_config_payload_layout() {
        // 16-row keyboard: rows 1..=16, columns 21..=24.
        let row_pins: Vec<u8> = (1..=16).collect();
        let col_pins = vec![21u8, 22, 23, 24];
        let data = encode_matrix_kb_config_payload(4, 16, &col_pins, &row_pins);

        // byte 9 (data[0]): enable bit
        assert_eq!(data[0], 1);
        // byte 10 (data[1]): (width-1) << 4 | (height-1) = (3 << 4) | 15 = 0x3F
        assert_eq!(data[1], 0x3F);

        // bytes 11-18 (data[2..10]): rows 1..8 → pin codes 0..7
        for i in 0..8 {
            assert_eq!(data[2 + i], i as u8, "row {} (low half)", i);
        }
        // bytes 19-26 (data[10..18]): columns 21..24 → pin codes 20..23
        for i in 0..4 {
            assert_eq!(data[10 + i], 20 + i as u8, "column {}", i);
        }

        // bytes 27-42 (data[18..34]): direct/macro bitmap, all zero (direct)
        assert!(
            data[18..34].iter().all(|&b| b == 0),
            "direct/macro bitmap must be zero by default"
        );

        // bytes 43-50 (data[34..42]): rows 9..16 → pin codes 8..15
        // This is the regression site: previously this loop wrote to data[42..50],
        // shifting the extended rows entirely outside the spec window.
        for i in 0..8 {
            assert_eq!(
                data[34 + i],
                (8 + i) as u8,
                "row {} (extended; high half)",
                8 + i
            );
        }

        // byte 51 (data[42]): alt-function pin must be zero (disabled)
        assert_eq!(data[42], 0);
    }

    #[test]
    fn test_encode_matrix_kb_config_payload_no_extended_rows_when_height_le_8() {
        let row_pins = vec![1u8, 2, 3, 4];
        let col_pins = vec![21u8, 22, 23, 24];
        let data = encode_matrix_kb_config_payload(4, 4, &col_pins, &row_pins);

        // bytes 43-50 / data[34..42] must remain zero — the spec says the extended
        // row pins block is only used when height > 8.
        assert!(
            data[34..42].iter().all(|&b| b == 0),
            "extended row pins must not be written when height <= 8"
        );
    }

    #[test]
    fn test_encode_matrix_kb_mapping_chunk_writes_codes_modifiers_and_triggered_bitmap() {
        let codes: Vec<u8> = (0..128).map(|i| i as u8).collect();
        let modifiers: Vec<u8> = (0..128).map(|i| (i as u8).wrapping_mul(2)).collect();
        let mut triggered = vec![0u8; 128];
        // Mark keys 33 and 47 as triggered (chunk_base = 32 → bits 1 and 15 of bitmap).
        triggered[33] = 1;
        triggered[47] = 99;

        let chunk = encode_matrix_kb_mapping_chunk(&codes, &modifiers, Some(&triggered), 32);

        // codes 32..48 → data[0..16]
        for i in 0..16 {
            assert_eq!(chunk[i], (32 + i) as u8, "code {i}");
            assert_eq!(chunk[16 + i], (32 + i) as u8 * 2, "modifier {i}");
        }
        // bitmap: bit 1 (key 33) and bit 15 (key 47)
        let bitmap = u16::from_le_bytes([chunk[32], chunk[33]]);
        assert_eq!(bitmap, (1 << 1) | (1 << 15));
    }

    #[test]
    fn test_encode_matrix_kb_mapping_chunk_omits_bitmap_when_no_triggered() {
        let codes = vec![0u8; 128];
        let modifiers = vec![0u8; 128];
        let chunk = encode_matrix_kb_mapping_chunk(&codes, &modifiers, None, 0);
        assert_eq!(chunk[32], 0);
        assert_eq!(chunk[33], 0);
    }

    #[test]
    fn test_decode_matrix_kb_mapping_chunk_roundtrip() {
        let codes_in: Vec<u8> = (0..128).map(|i| (i as u8).wrapping_add(7)).collect();
        let modifiers_in: Vec<u8> = (0..128).map(|i| (i as u8).wrapping_mul(3)).collect();
        let mut triggered_in = vec![0u8; 128];
        triggered_in[5] = 1;
        triggered_in[20] = 1;

        // Build response payload from the encoded chunk for chunk_base=16
        let chunk =
            encode_matrix_kb_mapping_chunk(&codes_in, &modifiers_in, Some(&triggered_in), 16);
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[1] = 0xCA;
        response[8..8 + 16].copy_from_slice(&chunk[0..16]); // codes
        response[24..24 + 16].copy_from_slice(&chunk[16..32]); // modifiers
        response[40] = chunk[32];
        response[41] = chunk[33];

        let mut codes_out = vec![0u8; 128];
        let mut modifiers_out = vec![0u8; 128];
        let mut triggered_out = vec![0u8; 128];
        decode_matrix_kb_mapping_chunk(
            &response,
            &mut codes_out,
            &mut modifiers_out,
            Some(&mut triggered_out),
            16,
        );

        // Only keys 16..32 should be populated
        for i in 16..32 {
            assert_eq!(codes_out[i], codes_in[i], "code {i}");
            assert_eq!(modifiers_out[i], modifiers_in[i], "modifier {i}");
        }
        assert_eq!(triggered_out[20], 1);
        assert_eq!(triggered_out[16], 0);
        // Untouched buckets remain zero
        assert_eq!(codes_out[0], 0);
        assert_eq!(codes_out[40], 0);
    }

    #[test]
    fn test_validate_matrix_kb_mapping_response_rejects_wrong_command() {
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[1] = 0xC9; // not 0xCA
        assert!(validate_matrix_kb_mapping_response(&response, 12).is_err());
    }

    #[test]
    fn test_validate_matrix_kb_mapping_response_rejects_short_response() {
        let mut response = [0u8; 30]; // < 42 required
        response[1] = 0xCA;
        assert!(validate_matrix_kb_mapping_response(&response, 12).is_err());
    }

    #[test]
    fn test_validate_matrix_kb_mapping_response_accepts_valid() {
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[1] = 0xCA;
        assert!(validate_matrix_kb_mapping_response(&response, 12).is_ok());
    }

    #[test]
    fn test_configure_matrix_keyboard_rejects_pin_in_both_row_and_col() {
        let mut device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);
        let cols = [5u8, 6, 7, 8];
        let rows = [1u8, 2, 3, 5]; // pin 5 is also in cols
        let err = device
            .configure_matrix_keyboard(4, 4, &cols, &rows)
            .expect_err("must reject overlapping pin assignment");
        match err {
            PoKeysError::Parameter(msg) => {
                assert!(
                    msg.contains("Pin 5"),
                    "error must name the offending pin, got: {msg}"
                );
            }
            other => panic!("expected Parameter error, got {other:?}"),
        }
    }

    #[test]
    fn test_configure_matrix_keyboard_rejects_pin_zero() {
        let mut device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);
        let cols = [5u8, 6, 7, 8];
        let rows = [0u8, 2, 3, 4]; // pin 0 is invalid
        let err = device
            .configure_matrix_keyboard(4, 4, &cols, &rows)
            .expect_err("must reject pin 0");
        assert!(matches!(err, PoKeysError::Parameter(_)));
    }

    #[test]
    fn test_configure_matrix_keyboard_rejects_pin_above_55() {
        let mut device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);
        let cols = [5u8, 6, 7, 8];
        let rows = [1u8, 2, 3, 56]; // pin 56 is out of range
        let err = device
            .configure_matrix_keyboard(4, 4, &cols, &rows)
            .expect_err("must reject pin 56");
        assert!(matches!(err, PoKeysError::Parameter(_)));
    }

    #[test]
    fn test_configure_matrix_keyboard_only_validates_used_pins() {
        // Pins beyond `width`/`height` should be ignored entirely so callers
        // can pass the full row_pins/column_pins arrays without padding.
        let mut device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);
        let cols = [5u8, 6, 7, 8, 0, 99, 99, 99]; // unused entries are bogus
        let rows = [1u8, 2, 3, 4, 0, 99, 99, 99];
        // Validation must pass; the call will then fail on read_all_pin_settings_raw
        // because there's no connection — that's not what we're testing here, just
        // that we got past the parameter checks.
        let err = device
            .configure_matrix_keyboard(4, 4, &cols, &rows)
            .expect_err("expected NotConnected, not Parameter");
        assert!(
            !matches!(err, PoKeysError::Parameter(_)),
            "validation should have accepted these pins; got Parameter error: {err:?}"
        );
    }

    /// Build a synthetic `0xCA` option-1 readback response. Uses 1-based
    /// row/col pin arguments, encoding them as 0-based on the wire (matching
    /// the option-16 write side).
    fn make_matrix_kb_readback(
        enabled: bool,
        width: u8,
        height: u8,
        rows: &[u8],
        cols: &[u8],
    ) -> [u8; RESPONSE_BUFFER_SIZE] {
        let mut r = [0u8; RESPONSE_BUFFER_SIZE];
        r[1] = 0xCA;
        r[8] = if enabled { 1 } else { 0 };
        r[9] = ((width.saturating_sub(1)) << 4) | (height.saturating_sub(1));
        for (i, &p) in rows.iter().enumerate().take(8) {
            r[10 + i] = p.saturating_sub(1);
        }
        for (i, &p) in cols.iter().enumerate().take(8) {
            r[18 + i] = p.saturating_sub(1);
        }
        for (offset, &p) in rows.iter().enumerate().skip(8).take(8) {
            r[42 + (offset - 8)] = p.saturating_sub(1);
        }
        r
    }

    fn parse_and_verify(
        r: &[u8],
        width: u8,
        height: u8,
        cols: &[u8],
        rows: &[u8],
        enabled: bool,
    ) -> Result<()> {
        let cfg = parse_matrix_kb_config_readback(r)?;
        verify_matrix_kb_config_matches(&cfg, width, height, cols, rows, enabled)
    }

    #[test]
    fn test_parse_readback_returns_full_config() {
        let rows: Vec<u8> = (1..=16).collect();
        let cols = [17u8, 18, 19, 20, 21, 22, 23, 24];
        let mut r = make_matrix_kb_readback(true, 8, 16, &rows, &cols);
        // Fill in fields the helper doesn't populate (alt-fn pin, decimation,
        // direct/macro bitmap) to exercise the full parser.
        r[26] = 0b0000_0101; // keys 0 and 2 set to macro
        r[50] = 7; // alt function pin = 7
        r[51] = 25; // scanning decimation
        let cfg = parse_matrix_kb_config_readback(&r).expect("parse");
        assert!(cfg.enabled);
        assert_eq!(cfg.width, 8);
        assert_eq!(cfg.height, 16);
        for i in 0..16 {
            assert_eq!(cfg.row_pins[i], (i + 1) as u8);
        }
        for i in 0..8 {
            assert_eq!(cfg.column_pins[i], (17 + i) as u8);
        }
        assert_eq!(cfg.direct_macro_bitmap[0], 0b0000_0101);
        assert_eq!(cfg.alternate_function_pin, 7);
        assert_eq!(cfg.scanning_decimation, 25);
        assert_eq!(cfg.size_byte(), (8u8 - 1) << 4 | (16u8 - 1));
    }

    #[test]
    fn test_verify_readback_accepts_matching_4x4() {
        let rows = [1u8, 2, 3, 4];
        let cols = [5u8, 6, 7, 8];
        let r = make_matrix_kb_readback(true, 4, 4, &rows, &cols);
        parse_and_verify(&r, 4, 4, &cols, &rows, true).expect("must accept");
    }

    #[test]
    fn test_verify_readback_accepts_matching_16x8() {
        let rows: Vec<u8> = (1..=16).collect();
        let cols = [17u8, 18, 19, 20, 21, 22, 23, 24];
        let r = make_matrix_kb_readback(true, 8, 16, &rows, &cols);
        parse_and_verify(&r, 8, 16, &cols, &rows, true).expect("must accept");
    }

    #[test]
    fn test_verify_readback_rejects_disabled_when_expecting_enabled() {
        // Simulates "configuration locked" — option-16 write echoed but
        // device kept the keyboard disabled.
        let rows = [1u8, 2, 3, 4];
        let cols = [5u8, 6, 7, 8];
        let r = make_matrix_kb_readback(false, 4, 4, &rows, &cols);
        let err = parse_and_verify(&r, 4, 4, &cols, &rows, true)
            .expect_err("must reject silent rejection");
        match err {
            PoKeysError::Protocol(msg) => assert!(
                msg.contains("enabled=false") && msg.contains("locked"),
                "error must hint at locked state, got: {msg}"
            ),
            other => panic!("expected Protocol error, got {other:?}"),
        }
    }

    #[test]
    fn test_verify_readback_rejects_wrong_size() {
        // Device clamped width 8 → 4 (firmware doesn't support 8 cols here).
        let rows = [1u8, 2, 3, 4];
        let cols = [5u8, 6, 7, 8];
        let r = make_matrix_kb_readback(true, 4, 4, &rows, &cols);
        let err =
            parse_and_verify(&r, 8, 4, &cols, &rows, true).expect_err("must reject size mismatch");
        match err {
            PoKeysError::Protocol(msg) => assert!(
                msg.contains("width=4") && msg.contains("expected width=8"),
                "error must report both actual and expected, got: {msg}"
            ),
            other => panic!("expected Protocol error, got {other:?}"),
        }
    }

    #[test]
    fn test_verify_readback_rejects_wrong_row_pin() {
        let rows = [1u8, 2, 3, 4];
        let cols = [5u8, 6, 7, 8];
        let mut r = make_matrix_kb_readback(true, 4, 4, &rows, &cols);
        r[12] = 99; // corrupt row pin 2 (0-based wire = pin 100 1-based)
        let err =
            parse_and_verify(&r, 4, 4, &cols, &rows, true).expect_err("must reject pin mismatch");
        assert!(matches!(err, PoKeysError::Protocol(msg) if msg.contains("row pin 2")));
    }

    #[test]
    fn test_verify_readback_rejects_wrong_extended_row_pin() {
        // Tests the height > 8 path — verifies extended rows at response[42..50].
        let rows: Vec<u8> = (1..=12).collect();
        let cols = [13u8, 14, 15, 16];
        let mut r = make_matrix_kb_readback(true, 4, 12, &rows, &cols);
        r[44] = 99; // corrupt extended row pin (offset 10, response[42+(10-8)] = response[44])
        let err = parse_and_verify(&r, 4, 12, &cols, &rows, true)
            .expect_err("must reject extended row pin mismatch");
        assert!(matches!(err, PoKeysError::Protocol(msg) if msg.contains("extended row pin 10")));
    }

    #[test]
    fn test_parse_readback_rejects_short_response() {
        let mut r = [0u8; 30]; // < 52 minimum (parser requires bytes 9-52)
        r[1] = 0xCA;
        let err = parse_matrix_kb_config_readback(&r).expect_err("must reject short response");
        assert!(matches!(err, PoKeysError::Protocol(msg) if msg.contains("readback too short")));
    }

    #[test]
    fn test_verify_readback_disabled_path_skips_size_and_pin_checks() {
        // When verifying after disable, we only check the enable bit; the
        // device may report stale dimensions and we shouldn't care.
        let mut r = [0u8; RESPONSE_BUFFER_SIZE];
        r[1] = 0xCA;
        r[8] = 0; // disabled
        r[9] = 0xFF; // garbage size
        // Expect disable: should pass even with no row/col pin info.
        parse_and_verify(&r, 0, 0, &[], &[], false).expect("must accept");
    }

    #[test]
    fn test_parse_readback_rejects_wrong_command_echo() {
        let mut r = [0u8; RESPONSE_BUFFER_SIZE];
        r[1] = 0xC9; // not 0xCA
        let err = parse_matrix_kb_config_readback(&r).expect_err("must reject wrong echo");
        assert!(matches!(err, PoKeysError::Protocol(msg) if msg.contains("echo mismatch")));
    }

    #[test]
    fn test_parse_device_data_populates_lock_status() {
        let mut device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        // Extended-device signature at bytes 9-12 (0-based 8-11) → take the
        // extended-device branch.
        response[8..12].copy_from_slice(b"PK58");
        response[4] = 0x12; // software version (just non-zero)
        response[5] = 3; // revision
        // Spec byte 59 / 0-based 58 = configuration lock status.
        response[58] = 1;
        device
            .parse_device_data_response(&response)
            .expect("parse must succeed");
        assert_eq!(device.device_data.device_lock_status, 1);
        assert!(device.device_data.device_locked());
    }

    #[test]
    fn test_parse_device_data_lock_status_zero_means_unlocked() {
        let mut device = PoKeysDevice::new(DeviceConnectionType::UsbDevice);
        let mut response = [0u8; RESPONSE_BUFFER_SIZE];
        response[8..12].copy_from_slice(b"PKEx");
        response[4] = 0x10;
        response[5] = 1;
        response[58] = 0;
        device
            .parse_device_data_response(&response)
            .expect("parse must succeed");
        assert!(!device.device_data.device_locked());
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
            match device.set_pwm_period(25000) {
                Ok(_) => {
                    println!("Set PWM period to 25000 cycles");

                    // Test different duty cycles
                    for duty in [25.0, 50.0, 75.0, 0.0] {
                        match device.set_pwm_duty_cycle_percent_for_pin(17, duty) {
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
