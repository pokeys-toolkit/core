//! Encoder support for PoKeys devices
//!
//! This module implements the complete PoKeys encoder protocol specification,
//! supporting up to 25 normal encoders, 3 fast encoders, and 1 ultra-fast encoder.
//!
//! Features:
//! - 4x and 2x sampling modes for precise position tracking
//! - Key mapping for encoder directions
//! - Bulk operations for efficient multi-encoder management
//! - Fast and ultra-fast encoder support for high-speed applications

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use serde::{Deserialize, Serialize};

/// Maximum number of normal encoders supported
pub const MAX_ENCODERS: usize = 25;

/// Maximum number of fast encoders supported  
pub const MAX_FAST_ENCODERS: usize = 3;

/// Ultra-fast encoder index (encoder 25)
pub const ULTRA_FAST_ENCODER_INDEX: u8 = 25;

/// Encoder configuration options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncoderOptions {
    /// Enable encoder
    pub enabled: bool,
    /// 4x sampling mode (both A and B edges counted)
    pub sampling_4x: bool,
    /// 2x sampling mode (only A edges counted)
    pub sampling_2x: bool,
    /// Direct key mapping for direction A
    pub direct_key_mapping_a: bool,
    /// Macro mapping for direction A
    pub macro_mapping_a: bool,
    /// Direct key mapping for direction B
    pub direct_key_mapping_b: bool,
    /// Macro mapping for direction B
    pub macro_mapping_b: bool,
}

impl Default for EncoderOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl EncoderOptions {
    /// Create new encoder options with all features disabled
    pub fn new() -> Self {
        Self {
            enabled: false,
            sampling_4x: false,
            sampling_2x: false,
            direct_key_mapping_a: false,
            macro_mapping_a: false,
            direct_key_mapping_b: false,
            macro_mapping_b: false,
        }
    }

    /// Create encoder options with 4x sampling enabled
    pub fn with_4x_sampling() -> Self {
        Self {
            enabled: true,
            sampling_4x: true,
            sampling_2x: false,
            direct_key_mapping_a: false,
            macro_mapping_a: false,
            direct_key_mapping_b: false,
            macro_mapping_b: false,
        }
    }

    /// Create encoder options with 2x sampling enabled
    pub fn with_2x_sampling() -> Self {
        Self {
            enabled: true,
            sampling_4x: false,
            sampling_2x: true,
            direct_key_mapping_a: false,
            macro_mapping_a: false,
            direct_key_mapping_b: false,
            macro_mapping_b: false,
        }
    }

    /// Convert options to protocol byte format
    /// Bit layout: [macro_b][key_b][macro_a][key_a][reserved][2x][4x][enable]
    pub fn to_byte(&self) -> u8 {
        let mut options = 0u8;
        if self.enabled {
            options |= 1 << 0;
        }
        if self.sampling_4x {
            options |= 1 << 1;
        }
        if self.sampling_2x {
            options |= 1 << 2;
        }
        // bit 3 is reserved
        if self.direct_key_mapping_a {
            options |= 1 << 4;
        }
        if self.macro_mapping_a {
            options |= 1 << 5;
        }
        if self.direct_key_mapping_b {
            options |= 1 << 6;
        }
        if self.macro_mapping_b {
            options |= 1 << 7;
        }
        options
    }

    /// Create options from protocol byte format
    pub fn from_byte(byte: u8) -> Self {
        Self {
            enabled: (byte & (1 << 0)) != 0,
            sampling_4x: (byte & (1 << 1)) != 0,
            sampling_2x: (byte & (1 << 2)) != 0,
            direct_key_mapping_a: (byte & (1 << 4)) != 0,
            macro_mapping_a: (byte & (1 << 5)) != 0,
            direct_key_mapping_b: (byte & (1 << 6)) != 0,
            macro_mapping_b: (byte & (1 << 7)) != 0,
        }
    }
}

/// Fast-encoder hardware configuration selector (protocol command `0xCE`
/// byte 3 = "FastEncodersConfiguration").
///
/// Only one configuration may be active at a time. Newer PoKeys56/57 devices
/// support only [`Config2`](Self::Config2); older PoKeys55 hardware supports
/// both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastEncoderConfiguration {
    /// Fast encoders disabled.
    Disabled,
    /// Config 1 — encoder 1 on pins 1-2, encoder 2 on pins 3-4, encoder 3 on pins 15-16.
    /// (PoKeys55-era devices only.)
    Config1,
    /// Config 2 — encoder 1 on pins 1-2, encoder 2 on pins 5-6, encoder 3 on pins 15-16.
    /// Supported by all devices with fast encoder hardware.
    Config2,
}

impl FastEncoderConfiguration {
    /// Byte value for the "FastEncodersConfiguration" field of command `0xCE`.
    /// Matches the PoKeysLib C reference constants:
    /// `PK_FASTENCODER_CONF_CFG1 = 0x01`, `PK_FASTENCODER_CONF_CFG2 = 0x10`.
    pub fn to_byte(self) -> u8 {
        match self {
            FastEncoderConfiguration::Disabled => 0x00,
            FastEncoderConfiguration::Config1 => 0x01,
            FastEncoderConfiguration::Config2 => 0x10,
        }
    }

    /// Decode the byte field back to a configuration. Unknown values map to
    /// [`Disabled`](Self::Disabled).
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x01 => FastEncoderConfiguration::Config1,
            0x10 => FastEncoderConfiguration::Config2,
            _ => FastEncoderConfiguration::Disabled,
        }
    }
}

/// Per-encoder options for the fast encoders (protocol command `0xCE`
/// byte 4 = "FastEncodersOptions").
///
/// Bit layout matches the PoKeysLib C reference (`ePK_FastEncoderOptions`):
///
/// | Bit | Mask | Meaning                           |
/// |-----|------|-----------------------------------|
/// | 4   | 0x10 | Disable 4x sampling (default is 4x on). |
/// | 5   | 0x20 | Invert direction of fast encoder 1. |
/// | 6   | 0x40 | Invert direction of fast encoder 2. |
/// | 7   | 0x80 | Invert direction of fast encoder 3. |
///
/// Bits 0-3 are reserved (the low nibble carries nothing — the configuration
/// selector lives in a separate byte, see [`FastEncoderConfiguration`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FastEncoderOptions {
    /// Disable 4x sampling for the fast encoders (default is 4x enabled).
    pub disable_4x_sampling: bool,
    /// Invert direction of fast encoder 1.
    pub invert_direction_1: bool,
    /// Invert direction of fast encoder 2.
    pub invert_direction_2: bool,
    /// Invert direction of fast encoder 3.
    pub invert_direction_3: bool,
}

/// Bit 4 (`0x10`) of the fast-encoders options byte — disable 4x sampling.
pub(crate) const FAST_ENCODER_DISABLE_4X_SAMPLING: u8 = 0x10;
/// Bit 5 (`0x20`) — invert direction of fast encoder 1.
pub(crate) const FAST_ENCODER_INVERT_E1: u8 = 0x20;
/// Bit 6 (`0x40`) — invert direction of fast encoder 2.
pub(crate) const FAST_ENCODER_INVERT_E2: u8 = 0x40;
/// Bit 7 (`0x80`) — invert direction of fast encoder 3.
pub(crate) const FAST_ENCODER_INVERT_E3: u8 = 0x80;

impl FastEncoderOptions {
    /// Pack the options into the protocol byte (byte 4 of command `0xCE`).
    pub fn to_byte(self) -> u8 {
        let mut b = 0u8;
        if self.disable_4x_sampling {
            b |= FAST_ENCODER_DISABLE_4X_SAMPLING;
        }
        if self.invert_direction_1 {
            b |= FAST_ENCODER_INVERT_E1;
        }
        if self.invert_direction_2 {
            b |= FAST_ENCODER_INVERT_E2;
        }
        if self.invert_direction_3 {
            b |= FAST_ENCODER_INVERT_E3;
        }
        b
    }

    /// Decode an options byte back into the struct (reserved bits ignored).
    pub fn from_byte(byte: u8) -> Self {
        Self {
            disable_4x_sampling: (byte & FAST_ENCODER_DISABLE_4X_SAMPLING) != 0,
            invert_direction_1: (byte & FAST_ENCODER_INVERT_E1) != 0,
            invert_direction_2: (byte & FAST_ENCODER_INVERT_E2) != 0,
            invert_direction_3: (byte & FAST_ENCODER_INVERT_E3) != 0,
        }
    }
}

/// Ultra-fast encoder options (protocol command `0x1C` byte 4 = "additional options").
///
/// Bit layout matches PoKeysLib C reference (`ePK_UltraFastEncoderOptions`):
///
/// | Bit | Mask | Meaning                        |
/// |-----|------|--------------------------------|
/// | 0   | 0x01 | Invert direction.              |
/// | 1   | 0x02 | Signal mode: `false` = quadrature (A/B), `true` = direction + clock. |
/// | 2   | 0x04 | Enable 4x sampling.            |
///
/// Bits 3-7 are reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UltraFastEncoderOptions {
    /// Invert direction (bit 0).
    pub invert_direction: bool,
    /// Signal mode. `false` = A/B are quadrature inputs; `true` = A is the
    /// direction signal and B is the clock signal.
    pub signal_mode_direction_clock: bool,
    /// Enable 4x sampling (both edges of A and B counted).
    pub enable_4x_sampling: bool,
}

/// Bit 0 — invert direction (ultra-fast).
pub(crate) const UFENC_INVERT_DIRECTION: u8 = 0x01;
/// Bit 1 — signal mode (ultra-fast).
pub(crate) const UFENC_SIGNAL_MODE: u8 = 0x02;
/// Bit 2 — enable 4x sampling (ultra-fast).
pub(crate) const UFENC_ENABLE_4X_SAMPLING: u8 = 0x04;

impl UltraFastEncoderOptions {
    /// Pack into the options byte sent to the device.
    pub fn to_byte(self) -> u8 {
        let mut b = 0u8;
        if self.invert_direction {
            b |= UFENC_INVERT_DIRECTION;
        }
        if self.signal_mode_direction_clock {
            b |= UFENC_SIGNAL_MODE;
        }
        if self.enable_4x_sampling {
            b |= UFENC_ENABLE_4X_SAMPLING;
        }
        b
    }

    /// Decode from a device-reported options byte.
    pub fn from_byte(byte: u8) -> Self {
        Self {
            invert_direction: (byte & UFENC_INVERT_DIRECTION) != 0,
            signal_mode_direction_clock: (byte & UFENC_SIGNAL_MODE) != 0,
            enable_4x_sampling: (byte & UFENC_ENABLE_4X_SAMPLING) != 0,
        }
    }
}

/// Encoder data structure containing all encoder state and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderData {
    /// Current encoder value (32-bit signed)
    pub encoder_value: i32,
    /// Encoder configuration options
    pub encoder_options: u8,
    /// Channel A input pin (0-54)
    pub channel_a_pin: u8,
    /// Channel B input pin (0-54)
    pub channel_b_pin: u8,
    /// Direction A key code for keyboard mapping
    pub dir_a_key_code: u8,
    /// Direction A key modifier for keyboard mapping
    pub dir_a_key_modifier: u8,
    /// Direction B key code for keyboard mapping
    pub dir_b_key_code: u8,
    /// Direction B key modifier for keyboard mapping
    pub dir_b_key_modifier: u8,
}

impl EncoderData {
    /// Create new encoder data with default values
    pub fn new() -> Self {
        Self {
            encoder_value: 0,
            encoder_options: 0,
            channel_a_pin: 0,
            channel_b_pin: 0,
            dir_a_key_code: 0,
            dir_a_key_modifier: 0,
            dir_b_key_code: 0,
            dir_b_key_modifier: 0,
        }
    }

    /// Get encoder options as structured data
    pub fn get_options(&self) -> EncoderOptions {
        EncoderOptions::from_byte(self.encoder_options)
    }

    /// Set encoder options from structured data
    pub fn set_options(&mut self, options: EncoderOptions) {
        self.encoder_options = options.to_byte();
    }

    /// Check if encoder is enabled
    pub fn is_enabled(&self) -> bool {
        (self.encoder_options & 1) != 0
    }

    /// Check if 4x sampling is enabled
    pub fn is_4x_sampling(&self) -> bool {
        (self.encoder_options & (1 << 1)) != 0
    }

    /// Check if 2x sampling is enabled
    pub fn is_2x_sampling(&self) -> bool {
        (self.encoder_options & (1 << 2)) != 0
    }

    /// Get sampling mode as string for debugging
    pub fn sampling_mode_str(&self) -> &'static str {
        if self.is_4x_sampling() {
            "4x (both edges)"
        } else if self.is_2x_sampling() {
            "2x (A edges only)"
        } else {
            "1x (disabled)"
        }
    }
}

impl Default for EncoderData {
    fn default() -> Self {
        Self::new()
    }
}

impl PoKeysDevice {
    /// Configure encoder with pins and options
    /// Protocol: 0x11 - Individual encoder settings (per protocol spec)
    pub fn configure_encoder(
        &mut self,
        encoder_id: u8,
        channel_a_pin: u8,
        channel_b_pin: u8,
        options: EncoderOptions,
    ) -> Result<()> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        // Convert 1-based pin numbers to 0-based for protocol
        // The calling code uses 1-based pin numbers, but the protocol expects 0-based
        let protocol_pin_a = if channel_a_pin > 0 {
            channel_a_pin - 1
        } else {
            0
        };
        let protocol_pin_b = if channel_b_pin > 0 {
            channel_b_pin - 1
        } else {
            0
        };

        let encoder = &mut self.encoders[encoder_id as usize];
        encoder.channel_a_pin = protocol_pin_a; // Store 0-based internally
        encoder.channel_b_pin = protocol_pin_b; // Store 0-based internally
        encoder.set_options(options);

        log::debug!(
            "Configuring encoder {} with pins A={}, B={} (1-based: A={}, B={}), options={:08b} using protocol 0x11",
            encoder_id,
            protocol_pin_a,
            protocol_pin_b,
            channel_a_pin,
            channel_b_pin,
            options.to_byte()
        );

        // Use protocol command 0x11 for individual encoder configuration
        // Per spec: byte 2: 0x11, byte 3: encoder ID (0-25), byte 4: option, byte 5: channel A, byte 6: channel B
        let response = self.send_request(
            0x11,              // Command: Encoder settings (per protocol spec)
            encoder_id,        // Encoder ID (0-25)
            options.to_byte(), // Options byte
            protocol_pin_a,    // Channel A input pin (0-based)
            protocol_pin_b,    // Channel B input pin (0-based)
        )?;

        log::debug!("Encoder configuration response: {:?}", &response[0..8]);

        // Check response status per spec: byte 3 (index 2) = 0 = OK, 1 = encoder ID out of range or configuration locked
        if response.len() > 2 {
            match response[2] {
                // Status is at index 2 (spec byte 3)
                0 => {
                    log::debug!("Encoder {} configuration successful", encoder_id);
                }
                1 => {
                    return Err(PoKeysError::Protocol(format!(
                        "Encoder {} configuration failed: encoder ID out of range or configuration locked",
                        encoder_id
                    )));
                }
                other => {
                    return Err(PoKeysError::Protocol(format!(
                        "Encoder {} configuration failed with status: {} (0x{:02X})",
                        encoder_id, other, other
                    )));
                }
            }
        }

        Ok(())
    }

    /// Read encoder settings using protocol 0x16
    pub fn read_encoder_settings(&mut self, encoder_id: u8) -> Result<EncoderData> {
        if encoder_id >= MAX_ENCODERS as u8 {
            return Err(PoKeysError::Parameter(format!(
                "Encoder ID {} exceeds maximum {}",
                encoder_id, MAX_ENCODERS
            )));
        }

        log::debug!(
            "Reading encoder {} settings using protocol 0x16",
            encoder_id
        );

        // Use protocol command 0x16 for reading individual encoder settings
        // Per spec: byte 2: 0x16, byte 3: encoder (0-25), byte 4-6: 0
        let response = self.send_request(
            0x16,       // Command: Read encoder settings
            encoder_id, // Encoder ID (0-25)
            0,          // Reserved
            0,          // Reserved
            0,          // Reserved
        )?;

        log::debug!("Read encoder settings response: {:?}", &response[0..8]);

        // Parse response per spec, accounting for header byte:
        // Index 0: header (0xAA), Index 1: command (0x16), Index 2: encoder, Index 3: options, Index 4: channel A, Index 5: channel B
        if response.len() >= 6 {
            let returned_encoder_id = response[2]; // Encoder ID at index 2
            let options_byte = response[3]; // Options at index 3
            let channel_a_pin = response[4]; // Channel A pin at index 4
            let channel_b_pin = response[5]; // Channel B pin at index 5

            if returned_encoder_id != encoder_id {
                log::warn!(
                    "Response encoder ID {} doesn't match requested {}",
                    returned_encoder_id,
                    encoder_id
                );
            }

            // Convert 0-based protocol pins to 1-based for display/API consistency
            let display_pin_a = channel_a_pin + 1;
            let display_pin_b = channel_b_pin + 1;

            let settings = EncoderData {
                channel_a_pin: display_pin_a, // Return 1-based pin numbers
                channel_b_pin: display_pin_b, // Return 1-based pin numbers
                encoder_options: options_byte,
                ..Default::default()
            };

            log::debug!(
                "Encoder {} settings: A={}, B={} (protocol: A={}, B={}), options={:08b}",
                encoder_id,
                display_pin_a,
                display_pin_b,
                channel_a_pin,
                channel_b_pin,
                options_byte
            );

            Ok(settings)
        } else {
            Err(PoKeysError::Protocol(
                "Invalid encoder settings response length".to_string(),
            ))
        }
    }

    /// Enable or disable encoder
    pub fn enable_encoder(&mut self, encoder_id: u8, enable: bool) -> Result<()> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        // Get current pin assignments
        let (channel_a_pin, channel_b_pin) = {
            let encoder = &self.encoders[encoder_id as usize];
            (encoder.channel_a_pin, encoder.channel_b_pin)
        };

        // Update options
        let mut options = {
            let encoder = &self.encoders[encoder_id as usize];
            encoder.get_options()
        };
        options.enabled = enable;

        self.configure_encoder(encoder_id, channel_a_pin, channel_b_pin, options)
    }

    /// Set encoder sampling mode (4x or 2x)
    pub fn set_encoder_sampling(
        &mut self,
        encoder_id: u8,
        sampling_4x: bool,
        sampling_2x: bool,
    ) -> Result<()> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        // Validate sampling mode - only one can be active
        if sampling_4x && sampling_2x {
            return Err(PoKeysError::Parameter(
                "Cannot enable both 4x and 2x sampling simultaneously".to_string(),
            ));
        }

        // Get current pin assignments
        let (channel_a_pin, channel_b_pin) = {
            let encoder = &self.encoders[encoder_id as usize];
            (encoder.channel_a_pin, encoder.channel_b_pin)
        };

        // Update options
        let mut options = {
            let encoder = &self.encoders[encoder_id as usize];
            encoder.get_options()
        };
        options.sampling_4x = sampling_4x;
        options.sampling_2x = sampling_2x;

        self.configure_encoder(encoder_id, channel_a_pin, channel_b_pin, options)
    }

    /// Configure encoder key mapping for direction A
    /// Protocol: 0x12 - Encoder key mapping for direction A
    pub fn configure_encoder_key_mapping_a(
        &mut self,
        encoder_id: u8,
        key_code: u8,
        key_modifier: u8,
    ) -> Result<()> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        let encoder = &mut self.encoders[encoder_id as usize];
        encoder.dir_a_key_code = key_code;
        encoder.dir_a_key_modifier = key_modifier;

        // Send key mapping configuration using protocol command 0x12
        let response = self.send_request(
            0x12,         // Command: Encoder key mapping for direction A
            encoder_id,   // Encoder ID (0-25)
            0,            // Reserved
            key_code,     // Key code or macro ID
            key_modifier, // Key modifier
        )?;

        // Check response status
        if response.len() > 3 && response[3] != 0 {
            return Err(PoKeysError::Protocol(format!(
                "Encoder key mapping A failed for encoder {}: status {}",
                encoder_id, response[3]
            )));
        }

        Ok(())
    }

    /// Configure encoder key mapping for direction B
    /// Protocol: 0x13 - Encoder key mapping for direction B
    pub fn configure_encoder_key_mapping_b(
        &mut self,
        encoder_id: u8,
        key_code: u8,
        key_modifier: u8,
    ) -> Result<()> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        let encoder = &mut self.encoders[encoder_id as usize];
        encoder.dir_b_key_code = key_code;
        encoder.dir_b_key_modifier = key_modifier;

        // Send key mapping configuration using protocol command 0x13
        let response = self.send_request(
            0x13,         // Command: Encoder key mapping for direction B
            encoder_id,   // Encoder ID (0-25)
            0,            // Reserved
            key_code,     // Key code or macro ID
            key_modifier, // Key modifier
        )?;

        // Check response status
        if response.len() > 3 && response[3] != 0 {
            return Err(PoKeysError::Protocol(format!(
                "Encoder key mapping B failed for encoder {}: status {}",
                encoder_id, response[3]
            )));
        }

        Ok(())
    }

    /// Read encoder key mapping for direction A
    /// Protocol: 0x17 - Read encoder key mapping for direction A
    pub fn read_encoder_key_mapping_a(&mut self, encoder_id: u8) -> Result<(u8, u8)> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        let response = self.send_request(
            0x17,       // Command: Read encoder key mapping for direction A
            encoder_id, // Encoder ID (0-25)
            0,          // Reserved
            0,          // Reserved
            0,          // Reserved
        )?;

        if response.len() < 8 {
            return Err(PoKeysError::Protocol("Invalid response length".to_string()));
        }

        // Parse response: byte 5 = key code, byte 6 = key modifier
        let key_code = response[5];
        let key_modifier = response[6];

        // Update local cache
        self.encoders[encoder_id as usize].dir_a_key_code = key_code;
        self.encoders[encoder_id as usize].dir_a_key_modifier = key_modifier;

        Ok((key_code, key_modifier))
    }

    /// Read encoder key mapping for direction B
    /// Protocol: 0x18 - Read encoder key mapping for direction B
    pub fn read_encoder_key_mapping_b(&mut self, encoder_id: u8) -> Result<(u8, u8)> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        let response = self.send_request(
            0x18,       // Command: Read encoder key mapping for direction B
            encoder_id, // Encoder ID (0-25)
            0,          // Reserved
            0,          // Reserved
            0,          // Reserved
        )?;

        if response.len() < 8 {
            return Err(PoKeysError::Protocol("Invalid response length".to_string()));
        }

        // Parse response: byte 5 = key code, byte 6 = key modifier
        let key_code = response[5];
        let key_modifier = response[6];

        // Update local cache
        self.encoders[encoder_id as usize].dir_b_key_code = key_code;
        self.encoders[encoder_id as usize].dir_b_key_modifier = key_modifier;

        Ok((key_code, key_modifier))
    }

    /// Read encoder RAW value
    /// Protocol: 0x19 - Read encoder RAW value
    pub fn read_encoder_raw_value(&mut self, encoder_id: u8) -> Result<i32> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        let response = self.send_request(
            0x19,       // Command: Read encoder RAW value
            encoder_id, // Encoder ID (0-25)
            0,          // Reserved
            0,          // Reserved
            0,          // Reserved
        )?;

        if response.len() < 8 {
            return Err(PoKeysError::Protocol("Invalid response length".to_string()));
        }

        // Parse response: byte 4 = RAW value (8-bit for individual read)
        // Note: For full 32-bit values, use bulk read operations
        let raw_value = response[4] as i8 as i32; // Sign-extend 8-bit to 32-bit

        // Update local cache
        self.encoders[encoder_id as usize].encoder_value = raw_value;

        Ok(raw_value)
    }

    /// Reset encoder RAW value to zero
    /// Protocol: 0x1A - Reset encoder RAW value
    pub fn reset_encoder_raw_value(&mut self, encoder_id: u8) -> Result<()> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        let response = self.send_request(
            0x1A,       // Command: Reset encoder RAW value
            encoder_id, // Encoder ID (0-25)
            0,          // Reserved
            0,          // Reserved
            0,          // Reserved
        )?;

        // Check if command was successful (no specific error checking in protocol)
        if response.len() < 8 {
            return Err(PoKeysError::Protocol("Invalid response length".to_string()));
        }

        // Update local cache
        self.encoders[encoder_id as usize].encoder_value = 0;

        Ok(())
    }

    /// Get encoder value (convenience method)
    pub fn get_encoder_value(&mut self, encoder_id: u8) -> Result<i32> {
        self.read_encoder_raw_value(encoder_id)
    }

    /// Reset encoder value (convenience method)
    pub fn reset_encoder(&mut self, encoder_id: u8) -> Result<()> {
        self.reset_encoder_raw_value(encoder_id)
    }

    /// Get encoder long RAW values (bulk operation)
    /// Protocol: 0xCD - Get encoder long RAW values
    /// option: 0 = encoders 1-13, 1 = encoders 14-26
    pub fn read_encoder_long_values(&mut self, group: u8) -> Result<Vec<i32>> {
        if group > 1 {
            return Err(PoKeysError::Parameter(
                "Group must be 0 (encoders 1-13) or 1 (encoders 14-26)".to_string(),
            ));
        }

        let response = self.send_request(
            0xCD,  // Command: Get encoder long RAW values
            group, // Option: 0 or 1
            0,     // Reserved
            0,     // Reserved
            0,     // Reserved
        )?;

        if response.len() < 64 {
            return Err(PoKeysError::Protocol(
                "Invalid response length for bulk encoder read".to_string(),
            ));
        }

        let mut values = Vec::new();

        // Parse 32-bit values starting from byte 8 (protocol spec says byte 9, but that's 1-based)
        // Group 0: encoders 1-13, Group 1: encoders 14-26
        for i in 0..13 {
            let byte_offset = 8 + (i * 4); // Start at byte 8 in 0-based array, 4 bytes per encoder
            if byte_offset + 3 < response.len() {
                let value = i32::from_le_bytes([
                    response[byte_offset],
                    response[byte_offset + 1],
                    response[byte_offset + 2],
                    response[byte_offset + 3],
                ]);
                values.push(value);

                // Update local cache - map to 0-based encoder IDs
                let encoder_index = if group == 0 { i } else { 13 + i };
                if encoder_index < self.encoders.len() {
                    self.encoders[encoder_index].encoder_value = value;
                }
            }
        }

        // Handle ultra-fast encoder for group 1 (bytes 56-59 in 0-based array)
        if group == 1 && response.len() >= 60 {
            let ultra_fast_value = i32::from_le_bytes([
                response[56], // Protocol spec byte 57 = array index 56
                response[57], // Protocol spec byte 58 = array index 57
                response[58], // Protocol spec byte 59 = array index 58
                response[59], // Protocol spec byte 60 = array index 59
            ]);
            // Ultra-fast encoder is at index 25 (encoder 26 in 1-based numbering)
            if self.encoders.len() > 25 {
                self.encoders[25].encoder_value = ultra_fast_value;
            }
        }

        log::debug!("Bulk read group {} returned {} values", group, values.len());
        Ok(values)
    }

    /// Set encoder long RAW values (bulk operation).
    ///
    /// Protocol `0xCD` with `option = 10` for encoders 1-13 or `11` for
    /// encoders 14-26. Payload at protocol bytes 9-60 (0-based 8-59) carries
    /// the 32-bit LE RAW values.
    pub fn set_encoder_long_values(&mut self, group: u8, values: &[i32]) -> Result<()> {
        if group > 1 {
            return Err(PoKeysError::Parameter(
                "Group must be 0 (encoders 1-13) or 1 (encoders 14-26)".to_string(),
            ));
        }

        let expected_count = if group == 0 { 13 } else { 12 }; // Group 1 has 12 regular + ultra-fast
        if values.len() < expected_count {
            return Err(PoKeysError::Parameter(format!(
                "Need {} values for group {}",
                expected_count, group
            )));
        }

        // Pack the 32-bit LE values into the data payload. `send_request_with_data`
        // copies this to request[8..], which the firmware reads as protocol
        // bytes 9..
        let count = values.len().min(13);
        let mut payload = [0u8; 52];
        for (i, &value) in values.iter().take(count).enumerate() {
            let offset = i * 4;
            payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }

        let _response = self.send_request_with_data(
            0xCD,
            group + 10, // option: 10 = set group 0, 11 = set group 1
            0,
            0,
            0,
            &payload[..count * 4],
        )?;

        // Update local cache
        let start_encoder = if group == 0 { 1 } else { 14 };
        for (i, &value) in values.iter().enumerate() {
            let encoder_index = start_encoder + i;
            if encoder_index < self.encoders.len() {
                self.encoders[encoder_index].encoder_value = value;
            }
        }

        Ok(())
    }

    /// Read all encoder values using bulk operations (more efficient)
    pub fn read_all_encoder_values(&mut self) -> Result<Vec<i32>> {
        let mut all_values = Vec::new();

        // Read encoders 1-13
        let group1_values = self.read_encoder_long_values(0)?;
        all_values.extend(group1_values);

        // Read encoders 14-25 + ultra-fast
        let group2_values = self.read_encoder_long_values(1)?;
        all_values.extend(group2_values);

        Ok(all_values)
    }

    /// Configure encoder options (bulk operation).
    ///
    /// Protocol `0xC4` with `option = 1` (set). The 25 option bytes are
    /// placed at protocol bytes 9-33 (0-based 8-32).
    pub fn configure_encoder_options_bulk(&mut self, options: &[u8]) -> Result<Vec<u8>> {
        if options.len() != 25 {
            return Err(PoKeysError::Parameter(
                "Need exactly 25 encoder options".to_string(),
            ));
        }

        let response = self.send_request_with_data(0xC4, 1, 0, 0, 0, options)?;

        // Parse returned options from response bytes 9-33 (0-based 8-32).
        let mut returned_options = Vec::with_capacity(25);
        if response.len() >= 33 {
            for i in 0..25 {
                returned_options.push(response[8 + i]);
                if i < self.encoders.len() {
                    self.encoders[i].encoder_options = response[8 + i];
                }
            }
        }

        Ok(returned_options)
    }

    /// Read encoder options (bulk operation).
    ///
    /// Protocol `0xC4` with `option = 0` (get). Response carries 25 option
    /// bytes at protocol bytes 9-33 (0-based 8-32).
    pub fn read_encoder_options_bulk(&mut self) -> Result<Vec<u8>> {
        let response = self.send_request(0xC4, 0, 0, 0, 0)?;

        let mut options = Vec::with_capacity(25);
        if response.len() >= 33 {
            for i in 0..25 {
                options.push(response[8 + i]);
                if i < self.encoders.len() {
                    self.encoders[i].encoder_options = response[8 + i];
                }
            }
        }

        Ok(options)
    }

    /// Configure the fast encoders via protocol command `0xCE`.
    ///
    /// `0xCE` uses two bytes: the **configuration selector** (which pin
    /// configuration / enable state to use) and the **options byte**
    /// (per-encoder direction invert + 4x-sampling control). They are passed
    /// as typed values so callers don't have to know the bit layout.
    ///
    /// Per PoKeysLib C reference (`PoKeysLibEncoders.c:154`):
    /// `req[2] = FastEncodersConfiguration`, `req[3] = FastEncodersOptions`.
    pub fn configure_fast_encoders(
        &mut self,
        config: FastEncoderConfiguration,
        options: FastEncoderOptions,
    ) -> Result<()> {
        let config_byte = config.to_byte();
        let options_byte = options.to_byte();

        self.fast_encoders_configuration = config_byte;
        self.fast_encoders_options = options_byte;

        let response = self.send_request(
            0xCE,         // Command: Enable/disable fast encoders
            config_byte,  // param1: FastEncodersConfiguration
            options_byte, // param2: FastEncodersOptions
            0,            // param3: reserved
            0,            // param4: reserved
        )?;

        // Response byte 3 (0-based index 2) is status.
        if response.len() > 2 {
            let status = response[2];
            if status != 0 {
                return Err(PoKeysError::Protocol(format!(
                    "Fast encoder configuration failed: status {}",
                    status
                )));
            }
        }

        Ok(())
    }

    /// Read fast encoder values
    pub fn read_fast_encoder_values(&mut self) -> Result<[i32; 3]> {
        // Fast encoders use the bulk read operation for encoders 0, 1, 2
        let values = self.read_encoder_long_values(0)?;

        let mut fast_values = [0i32; 3];
        let copy_len = 3.min(values.len());
        fast_values[..copy_len].copy_from_slice(&values[..copy_len]);

        Ok(fast_values)
    }

    /// Configure the ultra-fast encoder via protocol command `0x1C`
    /// (PoKeys56E and later).
    ///
    /// The ultra-fast encoder uses fixed pins: Pin 8 (Phase A), Pin 12 (Phase B),
    /// Pin 13 (Index).
    ///
    /// Bit layout of the options byte matches PoKeysLib C reference
    /// (`ePK_UltraFastEncoderOptions`): bit 0 = invert direction, bit 1 = signal
    /// mode, bit 2 = enable 4x sampling. See [`UltraFastEncoderOptions`].
    pub fn configure_ultra_fast_encoder(
        &mut self,
        enable: bool,
        options: UltraFastEncoderOptions,
        reset_on_index: bool,
        filter_delay: u32,
    ) -> Result<()> {
        let options_byte = options.to_byte();

        self.ultra_fast_encoder_configuration = if enable { 1 } else { 0 };
        self.ultra_fast_encoder_options = options_byte;
        self.ultra_fast_encoder_filter = filter_delay;

        // Packet layout per spec / PoKeysLib:
        //   byte 2 (param1) = enable flag (1 = enable, 0 = disable, 0xFF = read)
        //   byte 3 (param2) = options byte
        //   byte 4 (param3) = reset-on-index flag
        //   byte 5 (param4) = reserved (0)
        //   bytes 8..12     = filter delay (u32 LE) — placed via `with_data`
        let filter_bytes = filter_delay.to_le_bytes();
        let response = self.send_request_with_data(
            0x1C,
            if enable { 1 } else { 0 },
            options_byte,
            if reset_on_index { 1 } else { 0 },
            0,
            &filter_bytes,
        )?;

        // Response byte 3 (0-based index 2) is status.
        if response.len() > 2 {
            let status = response[2];
            if status != 0 {
                return Err(PoKeysError::Protocol(format!(
                    "Ultra-fast encoder configuration failed: status {}",
                    status
                )));
            }
        }

        Ok(())
    }

    /// Read the ultra-fast encoder's current configuration.
    ///
    /// Protocol: `0x1C` with `param1 = 0xFF` to request a read. Returns the
    /// enable flag, decoded [`UltraFastEncoderOptions`], and the filter delay
    /// (u32 LE at response bytes 8..12).
    pub fn read_ultra_fast_encoder_config(
        &mut self,
    ) -> Result<(bool, UltraFastEncoderOptions, u32)> {
        let response = self.send_request(0x1C, 0xFF, 0, 0, 0)?;

        if response.len() < 12 {
            return Err(PoKeysError::Protocol("Invalid response length".to_string()));
        }

        // Spec: byte 3 (index 2) = enable flag echo, byte 4 (index 3) = options.
        let enabled = response[2] != 0;
        let options = UltraFastEncoderOptions::from_byte(response[3]);
        let filter_delay =
            u32::from_le_bytes([response[8], response[9], response[10], response[11]]);

        Ok((enabled, options, filter_delay))
    }

    /// Read ultra-fast encoder value
    pub fn read_ultra_fast_encoder_value(&mut self) -> Result<i32> {
        // Ultra-fast encoder is included in bulk read group 1
        let values = self.read_encoder_long_values(1)?;

        // Ultra-fast encoder is the last value in group 1
        if let Some(&value) = values.last() {
            Ok(value)
        } else {
            Err(PoKeysError::Protocol(
                "No ultra-fast encoder value in response".to_string(),
            ))
        }
    }

    /// Set ultra-fast encoder value
    pub fn set_ultra_fast_encoder_value(&mut self, value: i32) -> Result<()> {
        // Use bulk set operation for group 1 with only the ultra-fast encoder value
        // We need to read current values first, then set only the ultra-fast one
        let mut values = self.read_encoder_long_values(1)?;

        // Set the ultra-fast encoder value (last in the array)
        if let Some(last) = values.last_mut() {
            *last = value;
        } else {
            return Err(PoKeysError::Protocol(
                "Cannot set ultra-fast encoder value".to_string(),
            ));
        }

        self.set_encoder_long_values(1, &values)
    }
    /// Configure encoder with keyboard mapping (convenience method)
    #[allow(clippy::too_many_arguments)]
    pub fn configure_encoder_with_keys(
        &mut self,
        encoder_id: u8,
        channel_a_pin: u8,
        channel_b_pin: u8,
        sampling_4x: bool,
        sampling_2x: bool,
        dir_a_key_code: u8,
        dir_a_key_modifier: u8,
        dir_b_key_code: u8,
        dir_b_key_modifier: u8,
    ) -> Result<()> {
        // Configure basic encoder settings
        let mut options = EncoderOptions::new();
        options.enabled = true;
        options.sampling_4x = sampling_4x;
        options.sampling_2x = sampling_2x;
        options.direct_key_mapping_a = true;
        options.direct_key_mapping_b = true;

        self.configure_encoder(encoder_id, channel_a_pin, channel_b_pin, options)?;

        // Configure key mappings
        self.configure_encoder_key_mapping_a(encoder_id, dir_a_key_code, dir_a_key_modifier)?;
        self.configure_encoder_key_mapping_b(encoder_id, dir_b_key_code, dir_b_key_modifier)?;

        Ok(())
    }

    /// Get encoder sampling mode as string (for debugging/display)
    pub fn get_encoder_sampling_mode(&self, encoder_id: u8) -> Result<String> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        let encoder = &self.encoders[encoder_id as usize];
        Ok(encoder.sampling_mode_str().to_string())
    }

    /// Check if encoder is configured for 4x sampling
    pub fn is_encoder_4x_sampling(&self, encoder_id: u8) -> Result<bool> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        Ok(self.encoders[encoder_id as usize].is_4x_sampling())
    }

    /// Check if encoder is configured for 2x sampling
    pub fn is_encoder_2x_sampling(&self, encoder_id: u8) -> Result<bool> {
        if encoder_id as usize >= self.encoders.len() {
            return Err(PoKeysError::Parameter(format!(
                "Invalid encoder ID: {}",
                encoder_id
            )));
        }

        Ok(self.encoders[encoder_id as usize].is_2x_sampling())
    }

    /// Get all enabled encoders
    pub fn get_enabled_encoders(&self) -> Vec<u8> {
        self.encoders
            .iter()
            .enumerate()
            .filter_map(|(i, encoder)| {
                if encoder.is_enabled() {
                    Some(i as u8)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_options_4x_sampling() {
        let options = EncoderOptions::with_4x_sampling();
        assert!(options.enabled);
        assert!(options.sampling_4x);
        assert!(!options.sampling_2x);

        let byte = options.to_byte();
        assert_eq!(byte & 0b00000011, 0b00000011); // enabled + 4x sampling

        let options_from_byte = EncoderOptions::from_byte(byte);
        assert!(options_from_byte.enabled);
        assert!(options_from_byte.sampling_4x);
        assert!(!options_from_byte.sampling_2x);
    }

    #[test]
    fn test_encoder_options_2x_sampling() {
        let options = EncoderOptions::with_2x_sampling();
        assert!(options.enabled);
        assert!(!options.sampling_4x);
        assert!(options.sampling_2x);

        let byte = options.to_byte();
        assert_eq!(byte & 0b00000111, 0b00000101); // enabled + 2x sampling

        let options_from_byte = EncoderOptions::from_byte(byte);
        assert!(options_from_byte.enabled);
        assert!(!options_from_byte.sampling_4x);
        assert!(options_from_byte.sampling_2x);
    }

    #[test]
    fn test_encoder_options_key_mapping() {
        let mut options = EncoderOptions::new();
        options.enabled = true;
        options.direct_key_mapping_a = true;
        options.macro_mapping_b = true;

        let byte = options.to_byte();
        assert_eq!(byte & 0b11110001, 0b10010001); // enabled + key_a + macro_b

        let options_from_byte = EncoderOptions::from_byte(byte);
        assert!(options_from_byte.enabled);
        assert!(options_from_byte.direct_key_mapping_a);
        assert!(options_from_byte.macro_mapping_b);
        assert!(!options_from_byte.direct_key_mapping_b);
        assert!(!options_from_byte.macro_mapping_a);
    }

    #[test]
    fn test_encoder_data_sampling_modes() {
        let mut encoder = EncoderData::new();

        // Test 4x sampling
        let options_4x = EncoderOptions::with_4x_sampling();
        encoder.set_options(options_4x);
        assert!(encoder.is_4x_sampling());
        assert!(!encoder.is_2x_sampling());
        assert_eq!(encoder.sampling_mode_str(), "4x (both edges)");

        // Test 2x sampling
        let options_2x = EncoderOptions::with_2x_sampling();
        encoder.set_options(options_2x);
        assert!(!encoder.is_4x_sampling());
        assert!(encoder.is_2x_sampling());
        assert_eq!(encoder.sampling_mode_str(), "2x (A edges only)");

        // Test disabled
        let options_disabled = EncoderOptions::new();
        encoder.set_options(options_disabled);
        assert!(!encoder.is_4x_sampling());
        assert!(!encoder.is_2x_sampling());
        assert_eq!(encoder.sampling_mode_str(), "1x (disabled)");
    }

    #[test]
    fn test_encoder_constants() {
        assert_eq!(MAX_ENCODERS, 25);
        assert_eq!(MAX_FAST_ENCODERS, 3);
        assert_eq!(ULTRA_FAST_ENCODER_INDEX, 25);
    }

    // ---- Fast encoder (0xCE) bit-layout tests ----------------------------
    //
    // These bit positions are authoritative per the PoLabs PoKeysLib C
    // reference (`ePK_FastEncoderOptions` in LinuxCnc_PokeysLibComp/
    // pokeys_uspace/PoKeysComp.h, also matching PoLabsEE/PoKeysLib usage).

    #[test]
    fn test_fast_encoder_configuration_to_byte() {
        assert_eq!(FastEncoderConfiguration::Disabled.to_byte(), 0x00);
        assert_eq!(FastEncoderConfiguration::Config1.to_byte(), 0x01);
        assert_eq!(FastEncoderConfiguration::Config2.to_byte(), 0x10);
    }

    #[test]
    fn test_fast_encoder_configuration_round_trip() {
        for cfg in [
            FastEncoderConfiguration::Disabled,
            FastEncoderConfiguration::Config1,
            FastEncoderConfiguration::Config2,
        ] {
            assert_eq!(FastEncoderConfiguration::from_byte(cfg.to_byte()), cfg);
        }
    }

    #[test]
    fn test_fast_encoder_options_bit_masks() {
        // Each flag alone must land on the exact bit the firmware expects.
        assert_eq!(
            FastEncoderOptions {
                disable_4x_sampling: true,
                ..Default::default()
            }
            .to_byte(),
            0x10
        );
        assert_eq!(
            FastEncoderOptions {
                invert_direction_1: true,
                ..Default::default()
            }
            .to_byte(),
            0x20
        );
        assert_eq!(
            FastEncoderOptions {
                invert_direction_2: true,
                ..Default::default()
            }
            .to_byte(),
            0x40
        );
        assert_eq!(
            FastEncoderOptions {
                invert_direction_3: true,
                ..Default::default()
            }
            .to_byte(),
            0x80
        );
    }

    #[test]
    fn test_fast_encoder_options_combined() {
        // All four flags set → 0x10 | 0x20 | 0x40 | 0x80 = 0xF0.
        let all = FastEncoderOptions {
            disable_4x_sampling: true,
            invert_direction_1: true,
            invert_direction_2: true,
            invert_direction_3: true,
        };
        assert_eq!(all.to_byte(), 0xF0);
    }

    #[test]
    fn test_fast_encoder_options_round_trip() {
        let o = FastEncoderOptions {
            disable_4x_sampling: false,
            invert_direction_1: true,
            invert_direction_2: false,
            invert_direction_3: true,
        };
        assert_eq!(FastEncoderOptions::from_byte(o.to_byte()), o);
    }

    #[test]
    fn test_fast_encoder_options_ignores_reserved_low_nibble() {
        // Low nibble must not leak into any field.
        let decoded = FastEncoderOptions::from_byte(0x0F);
        assert_eq!(decoded, FastEncoderOptions::default());
    }

    // ---- Ultra-fast encoder (0x1C) bit-layout tests ---------------------
    //
    // Per `ePK_UltraFastEncoderOptions`: bit 0 = invert direction,
    // bit 1 = signal mode, bit 2 = enable 4x sampling.

    #[test]
    fn test_ultra_fast_encoder_options_bit_masks() {
        assert_eq!(
            UltraFastEncoderOptions {
                invert_direction: true,
                ..Default::default()
            }
            .to_byte(),
            0x01
        );
        assert_eq!(
            UltraFastEncoderOptions {
                signal_mode_direction_clock: true,
                ..Default::default()
            }
            .to_byte(),
            0x02
        );
        assert_eq!(
            UltraFastEncoderOptions {
                enable_4x_sampling: true,
                ..Default::default()
            }
            .to_byte(),
            0x04
        );
    }

    #[test]
    fn test_ultra_fast_encoder_options_combined() {
        // invert + signal mode + 4x sampling → 0x07.
        let all = UltraFastEncoderOptions {
            invert_direction: true,
            signal_mode_direction_clock: true,
            enable_4x_sampling: true,
        };
        assert_eq!(all.to_byte(), 0x07);
    }

    #[test]
    fn test_ultra_fast_encoder_options_round_trip() {
        let o = UltraFastEncoderOptions {
            invert_direction: true,
            signal_mode_direction_clock: false,
            enable_4x_sampling: true,
        };
        assert_eq!(UltraFastEncoderOptions::from_byte(o.to_byte()), o);
    }

    #[test]
    fn test_ultra_fast_encoder_options_ignores_reserved_high_bits() {
        // Bits 3-7 are reserved; must not leak into any field.
        let decoded = UltraFastEncoderOptions::from_byte(0xF8);
        assert_eq!(decoded, UltraFastEncoderOptions::default());
    }
}
