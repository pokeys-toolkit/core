//! Digital and analog I/O operations

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use log::info;
use serde::{Deserialize, Serialize};

mod private;

use private::INVERT_PIN_BIT;

// PoKeys pin-function identifier (defined in a tiny private module so the
// `#[allow(deprecated)]` scope covers the derive-macro expansions, which
// otherwise flag every variant reference inside the generated Debug /
// PartialEq / Serialize / Deserialize code).
#[allow(deprecated)]
mod pin_function {
    use serde::{Deserialize, Serialize};

    /// PoKeys pin-function identifier.
    ///
    /// The wire-level "pin settings" byte (byte 4 of protocol command
    /// `0x10`) is a bitfield: the low 7 bits carry the base function and
    /// bit 7 (`0x80`, [`PinFunction::InvertPin`]) composes with any
    /// digital function to request firmware-level polarity inversion.
    /// This enum models the base function only. To apply the invert flag,
    /// use [`crate::PoKeysDevice::set_pin_function_with_invert`] — passing
    /// `PinFunction::InvertPin` directly to
    /// [`crate::PoKeysDevice::set_pin_function`] is not meaningful and is
    /// kept only for backward compatibility.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PinFunction {
        PinRestricted = 0,
        Reserved = 1,
        DigitalInput = 2,
        DigitalOutput = 4,
        AnalogInput = 8,
        AnalogOutput = 16,
        TriggeredInput = 32,
        DigitalCounter = 64,
        #[deprecated(
            since = "0.23.0",
            note = "Use PoKeysDevice::set_pin_function_with_invert to apply the invert flag; the variant is a protocol bit, not a standalone pin function."
        )]
        InvertPin = 128,
    }
}

pub use pin_function::PinFunction;

/// Decode a cached pin-settings byte into a `PinFunction`, ignoring bit 7
/// (the invert flag). Unknown bit patterns decode to `PinRestricted`.
///
/// Split out from [`PoKeysDevice::get_pin_function`] so the mapping can be
/// unit-tested and reused by [`PinData::base_function`].
pub(crate) fn decode_pin_function_from_cache(byte: u8) -> PinFunction {
    match byte & 0x7F {
        0 => PinFunction::PinRestricted,
        1 => PinFunction::Reserved,
        2 => PinFunction::DigitalInput,
        4 => PinFunction::DigitalOutput,
        8 => PinFunction::AnalogInput,
        16 => PinFunction::AnalogOutput,
        32 => PinFunction::TriggeredInput,
        64 => PinFunction::DigitalCounter,
        _ => PinFunction::PinRestricted,
    }
}

impl PinFunction {
    /// Convert u8 value to PinFunction enum
    /// Note: PoKeys uses bit flags for pin functions
    pub fn from_u8(value: u8) -> Result<Self> {
        #[allow(deprecated)]
        match value {
            0 => Ok(PinFunction::PinRestricted),
            1 => Ok(PinFunction::Reserved),
            2 => Ok(PinFunction::DigitalInput),
            4 => Ok(PinFunction::DigitalOutput),
            8 => Ok(PinFunction::AnalogInput),
            16 => Ok(PinFunction::AnalogOutput),
            32 => Ok(PinFunction::TriggeredInput),
            64 => Ok(PinFunction::DigitalCounter),
            128 => Ok(PinFunction::InvertPin),
            // Handle combined flags by returning the primary function
            v if (v & PinFunction::DigitalOutput as u8) != 0 => Ok(PinFunction::DigitalOutput),
            v if (v & PinFunction::DigitalInput as u8) != 0 => Ok(PinFunction::DigitalInput),
            v if (v & PinFunction::AnalogInput as u8) != 0 => Ok(PinFunction::AnalogInput),
            v if (v & PinFunction::AnalogOutput as u8) != 0 => Ok(PinFunction::AnalogOutput),
            v if (v & PinFunction::DigitalCounter as u8) != 0 => Ok(PinFunction::DigitalCounter),
            v if (v & PinFunction::TriggeredInput as u8) != 0 => Ok(PinFunction::TriggeredInput),
            _ => Ok(PinFunction::PinRestricted), // Default to restricted for unknown values
        }
    }
}

/// Pin capabilities for checking device support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinCapability {
    DigitalInput = 1,
    DigitalOutput,
    AnalogInput,
    MfAnalogInput,
    AnalogOutput,
    KeyboardMapping,
    TriggeredInput,
    DigitalCounter,
    PwmOutput,
    FastEncoder1A,
    FastEncoder1B,
    FastEncoder1I,
    FastEncoder2A,
    FastEncoder2B,
    FastEncoder2I,
    FastEncoder3A,
    FastEncoder3B,
    FastEncoder3I,
    UltraFastEncoderA,
    UltraFastEncoderB,
    UltraFastEncoderI,
    LcdE,
    LcdRw,
    LcdRs,
    LcdD4,
    LcdD5,
    LcdD6,
    LcdD7,
}

/// Pin-specific data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinData {
    pub digital_counter_value: u32,
    pub analog_value: u32,
    pub pin_function: u8,
    pub counter_options: u8,
    pub digital_value_get: u8,
    pub digital_value_set: u8,
    pub digital_counter_available: u8,
    pub mapping_type: u8,
    pub key_code_macro_id: u8,
    pub key_modifier: u8,
    pub down_key_code_macro_id: u8,
    pub down_key_modifier: u8,
    pub up_key_code_macro_id: u8,
    pub up_key_modifier: u8,
    pub prevent_update: u8,
}

impl PinData {
    pub fn new() -> Self {
        Self {
            digital_counter_value: 0,
            analog_value: 0,
            pin_function: PinFunction::PinRestricted as u8,
            counter_options: 0,
            digital_value_get: 0,
            digital_value_set: 0,
            digital_counter_available: 0,
            mapping_type: 0,
            key_code_macro_id: 0,
            key_modifier: 0,
            down_key_code_macro_id: 0,
            down_key_modifier: 0,
            up_key_code_macro_id: 0,
            up_key_modifier: 0,
            prevent_update: 0,
        }
    }

    pub fn is_digital_input(&self) -> bool {
        (self.pin_function & PinFunction::DigitalInput as u8) != 0
    }

    pub fn is_digital_output(&self) -> bool {
        (self.pin_function & PinFunction::DigitalOutput as u8) != 0
    }

    pub fn is_analog_input(&self) -> bool {
        (self.pin_function & PinFunction::AnalogInput as u8) != 0
    }

    pub fn is_analog_output(&self) -> bool {
        (self.pin_function & PinFunction::AnalogOutput as u8) != 0
    }

    pub fn is_digital_counter(&self) -> bool {
        (self.pin_function & PinFunction::DigitalCounter as u8) != 0
    }

    /// True if bit 7 (the hardware invert flag) is set on this pin's
    /// cached function byte.
    pub fn is_inverted(&self) -> bool {
        (self.pin_function & INVERT_PIN_BIT) != 0
    }

    /// Decode the base pin function from the cached byte, ignoring the
    /// invert bit. Pair with [`Self::is_inverted`] for the full picture.
    pub fn base_function(&self) -> PinFunction {
        decode_pin_function_from_cache(self.pin_function)
    }
}

impl Default for PinData {
    fn default() -> Self {
        Self::new()
    }
}

impl PoKeysDevice {
    /// Set a pin's function (non-inverted polarity).
    ///
    /// Thin wrapper over [`Self::set_pin_function_with_invert`] with
    /// `inverted = false`; byte-identical on the wire to the pre-invert
    /// behaviour for every existing caller.
    pub fn set_pin_function(
        &mut self,
        pin: u32,
        pin_function: PinFunction,
    ) -> Result<(u32, PinFunction)> {
        self.set_pin_function_with_invert(pin, pin_function, false)
    }

    /// Set a pin's function with an optional hardware invert flag.
    ///
    /// When `inverted == true`, bit 7 (`0x80`) of protocol byte 4 is set,
    /// producing a combined wire byte such as `0x82` for inverted
    /// `DigitalInput`. The firmware then reports/drives the logical
    /// complement of the electrical state at no CPU cost to the caller.
    ///
    /// Pin functions that honor the invert bit:
    /// - [`PinFunction::DigitalInput`]
    /// - [`PinFunction::DigitalOutput`]
    /// - [`PinFunction::TriggeredInput`]
    ///
    /// Pin functions that ignore the invert bit (firmware silently drops it):
    /// - [`PinFunction::AnalogInput`], [`PinFunction::AnalogOutput`]
    /// - [`PinFunction::DigitalCounter`]
    /// - [`PinFunction::PinRestricted`], [`PinFunction::Reserved`]
    ///
    /// For functions that ignore the flag, this method logs a warning via the
    /// `log` crate and still sends the byte as requested. Use
    /// [`Self::get_pin_invert`] to read the invert state back from the cache.
    pub fn set_pin_function_with_invert(
        &mut self,
        pin: u32,
        pin_function: PinFunction,
        inverted: bool,
    ) -> Result<(u32, PinFunction)> {
        self.write_pin_function(pin, pin_function, inverted)
    }

    /// Get pin function (base function only; the invert bit is ignored).
    ///
    /// To read the invert flag, use [`Self::get_pin_invert`].
    pub fn get_pin_function(&self, pin: u32) -> Result<PinFunction> {
        let pin_index = self.check_pin_range(pin)?;
        Ok(decode_pin_function_from_cache(
            self.pins[pin_index].pin_function,
        ))
    }

    /// Return whether bit 7 (the hardware invert flag) is set on the pin's
    /// cached configuration byte.
    ///
    /// Reflects the most recently written or read-back wire byte; call
    /// [`Self::read_all_pin_functions`] first if you want the device's
    /// current state rather than the local cache.
    pub fn get_pin_invert(&self, pin: u32) -> Result<bool> {
        let pin_index = self.check_pin_range(pin)?;
        Ok((self.pins[pin_index].pin_function & INVERT_PIN_BIT) != 0)
    }

    /// Read digital input
    pub fn get_digital_input(&mut self, pin: u32) -> Result<bool> {
        if pin == 0 || pin as usize > self.pins.len() {
            return Err(PoKeysError::Parameter("Invalid pin number".to_string()));
        }

        // Read all digital inputs from device
        let res = self.read_digital_input(pin)?;

        let pin_index = (pin - 1) as usize;
        self.pins[pin_index].digital_value_get = res;
        Ok(res != 0)
    }

    /// Set digital output
    pub fn set_digital_output(&mut self, pin: u32, value: bool) -> Result<bool> {
        if pin == 0 || pin as usize > self.pins.len() {
            return Err(PoKeysError::Parameter("Invalid pin number".to_string()));
        }

        // Send digital output to device
        match self.write_digital_output(pin, !value) {
            Ok(_) => {
                let pin_index = (pin - 1) as usize;
                self.pins[pin_index].digital_value_set = if value { 1 } else { 0 };
                info!(
                    "Pin {} set to {:?}",
                    pin,
                    if value { "High" } else { "Low" }
                );
                Ok(true)
            }
            Err(e) => Err(e),
        }
    }

    /// Read analog input
    pub fn get_analog_input(&mut self, pin: u32) -> Result<u32> {
        if pin == 0 || pin as usize > self.pins.len() {
            return Err(PoKeysError::Parameter("Invalid pin number".to_string()));
        }

        // Read all analog inputs from device
        self.read_analog_inputs()?;

        let pin_index = (pin - 1) as usize;
        Ok(self.pins[pin_index].analog_value)
    }

    /// Set analog output
    pub fn set_analog_output(&mut self, pin: u32, value: u32) -> Result<()> {
        if pin == 0 || pin as usize > self.pins.len() {
            return Err(PoKeysError::Parameter("Invalid pin number".to_string()));
        }

        let pin_index = (pin - 1) as usize;
        self.pins[pin_index].analog_value = value;

        // Send analog output to device
        self.write_analog_outputs()?;
        Ok(())
    }

    /// Read digital counter value
    pub fn get_digital_counter(&mut self, pin: u32) -> Result<u32> {
        if pin == 0 || pin as usize > self.pins.len() {
            return Err(PoKeysError::Parameter("Invalid pin number".to_string()));
        }

        let pin_index = (pin - 1) as usize;
        if self.pins[pin_index].digital_counter_available == 0 {
            return Err(PoKeysError::NotSupported);
        }

        // Read digital counters from device
        self.read_digital_counters()?;

        Ok(self.pins[pin_index].digital_counter_value)
    }

    /// Reset all digital counter values (protocol command `0x1D`).
    ///
    /// The PoKeys protocol does not support per-pin counter resets — `0x1D`
    /// clears all digital counters at once.
    pub fn reset_all_digital_counters(&mut self) -> Result<()> {
        self.send_request(0x1D, 0, 0, 0, 0)?;
        Ok(())
    }

    /// Read all digital inputs.
    ///
    /// Uses two protocol commands:
    /// - `0x31` "Block inputs reading" — pins 1–32 packed into response bytes 3–6 (0-based 2–5)
    /// - `0x32` "Block inputs reading – part 2" — pins 33–55 packed into response bytes 3–5 (0-based 2–4)
    pub fn get_digital_inputs(&mut self) -> Result<()> {
        let resp1 = self.send_request(0x31, 0, 0, 0, 0)?;
        for i in 0..self.pins.len().min(32) {
            let byte_index = 2 + (i / 8);
            let bit_index = i % 8;
            self.pins[i].digital_value_get = if (resp1[byte_index] & (1 << bit_index)) != 0 {
                1
            } else {
                0
            };
        }

        if self.pins.len() > 32 {
            let resp2 = self.send_request(0x32, 0, 0, 0, 0)?;
            for i in 32..self.pins.len().min(55) {
                let rel = i - 32;
                let byte_index = 2 + (rel / 8);
                let bit_index = rel % 8;
                self.pins[i].digital_value_get = if (resp2[byte_index] & (1 << bit_index)) != 0 {
                    1
                } else {
                    0
                };
            }
        }

        Ok(())
    }

    /// Write all digital outputs
    pub fn write_digital_outputs(&mut self) -> Result<()> {
        // Prepare output data
        let mut output_data = [0u8; 8];

        for i in 0..self.pins.len().min(55) {
            if self.pins[i].is_digital_output() && self.pins[i].digital_value_set != 0 {
                let byte_index = i / 8;
                let bit_index = i % 8;
                output_data[byte_index] |= 1 << bit_index;
            }
        }

        // Send digital outputs to device
        self.send_request(
            0x11,
            output_data[0],
            output_data[1],
            output_data[2],
            output_data[3],
        )?;

        // Send remaining bytes if needed
        if self.pins.len() > 32 {
            self.send_request(
                0x12,
                output_data[4],
                output_data[5],
                output_data[6],
                output_data[7],
            )?;
        }

        Ok(())
    }

    /// Read all analog inputs
    pub fn read_analog_inputs(&mut self) -> Result<()> {
        let response = self.send_request(0x20, 0, 0, 0, 0)?;

        // Parse analog input data from response
        let mut data_index = 8;
        for i in 0..self.pins.len() {
            if self.pins[i].is_analog_input() && data_index + 3 < response.len() {
                self.pins[i].analog_value = u32::from_le_bytes([
                    response[data_index],
                    response[data_index + 1],
                    response[data_index + 2],
                    response[data_index + 3],
                ]);
                data_index += 4;
            }
        }

        Ok(())
    }

    /// Write analog outputs for every pin currently configured as an analog output.
    ///
    /// Sends one "Analog outputs settings" request (`0x41`) per pin, carrying the pin's
    /// `analog_value` as a 10-bit DAC value (0–1023). Values larger than 10 bits are
    /// clamped by masking. The `pin ID` sent to the device is the 0-based pin code
    /// per the protocol spec (pin 43 → pin code 42).
    pub fn write_analog_outputs(&mut self) -> Result<()> {
        let targets: Vec<(u8, u32)> = self
            .pins
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.is_analog_output() {
                    Some((i as u8, p.analog_value))
                } else {
                    None
                }
            })
            .collect();

        for (pin_code, value) in targets {
            let (msb, lsb) = encode_analog_output_10bit(value);
            let response = self.send_request(0x41, pin_code, msb, lsb, 0)?;
            // Response byte 3 (0-based index 2): 0 = OK, non-zero = error ID
            if response[2] != 0 {
                return Err(PoKeysError::Protocol(format!(
                    "Analog output write failed for pin code {}: error code {}",
                    pin_code, response[2]
                )));
            }
        }

        Ok(())
    }

    /// Read digital counter values via protocol command `0xD8`.
    ///
    /// The request carries up to 13 pin IDs at spec bytes 9–21 (0-based 8–20)
    /// identifying which counters to return. The response packs thirteen 32-bit
    /// LE counter values at spec bytes 9–60 (0-based 8–59).
    pub fn read_digital_counters(&mut self) -> Result<()> {
        // Collect up to 13 counter-capable pin IDs (0-based pin codes per spec).
        let mut pin_ids = [0u8; 13];
        let mut selected: Vec<usize> = Vec::with_capacity(13);
        for (i, pin) in self.pins.iter().enumerate() {
            if selected.len() == 13 {
                break;
            }
            if pin.digital_counter_available != 0 {
                pin_ids[selected.len()] = i as u8;
                selected.push(i);
            }
        }

        let response = self.send_request_with_data(0xD8, 0, 0, 0, 0, &pin_ids)?;

        // Response: 13 × 4-byte LE counter values starting at 0-based byte 8.
        for (slot, pin_index) in selected.iter().enumerate() {
            let start = 8 + slot * 4;
            if start + 4 <= response.len() {
                self.pins[*pin_index].digital_counter_value = u32::from_le_bytes([
                    response[start],
                    response[start + 1],
                    response[start + 2],
                    response[start + 3],
                ]);
            }
        }

        Ok(())
    }

    /// Read all pin functions at once using extended mode (0xC0)
    /// This is much more efficient than reading pins individually
    /// Performance improvement: 55x fewer commands
    pub fn read_all_pin_functions(&mut self) -> Result<[PinFunction; 55]> {
        use crate::io::private::Command;

        // Send extended I/O command: Read all pin functions
        // Command 0xC0, option1=0 (read), option2=0 (pin functions)
        let response = self.send_request(
            Command::InputOutputExtended as u8,
            0, // option1: 0 = read all pin functions
            0, // option2: 0 = pin functions (not additional settings)
            0, // reserved
            0, // request ID will be set by send_request
        )?;

        let raw = parse_bulk_pin_settings_response(&response)?;

        // Decode to the public `[PinFunction; 55]` view (invert bit dropped).
        let mut functions = [PinFunction::PinRestricted; 55];
        for i in 0..55 {
            functions[i] = PinFunction::from_u8(raw[i])?;
        }

        // Update local pin cache with the full wire byte so bit 7 (invert)
        // is preserved; the returned `[PinFunction; 55]` still carries only
        // the base function, matching the existing public contract.
        for i in 0..55 {
            if i < self.pins.len() {
                self.pins[i].pin_function = raw[i];
            }
        }

        Ok(functions)
    }

    /// Read all 55 pin-setting bytes verbatim, including the invert bit
    /// (`0x80`) when set. Unlike [`Self::read_all_pin_functions`], this
    /// preserves the full protocol byte so callers can reason about
    /// hardware-level polarity inversion.
    ///
    /// Also refreshes the local pin cache with the returned bytes.
    pub fn read_all_pin_settings_raw(&mut self) -> Result<[u8; 55]> {
        use crate::io::private::Command;

        let response = self.send_request(Command::InputOutputExtended as u8, 0, 0, 0, 0)?;
        let raw = parse_bulk_pin_settings_response(&response)?;

        for i in 0..55 {
            if i < self.pins.len() {
                self.pins[i].pin_function = raw[i];
            }
        }

        Ok(raw)
    }

    /// Send all 55 pin-setting bytes verbatim using the bulk `0xC0` command.
    /// Callers compose each byte themselves — e.g.
    /// `(PinFunction::DigitalInput as u8) | 0x80` for an inverted digital
    /// input.
    ///
    /// The local pin cache is updated to match the bytes sent.
    pub fn set_all_pin_settings_raw(&mut self, raw: &[u8; 55]) -> Result<()> {
        use crate::io::private::Command;

        let response = self.send_request_with_data(
            Command::InputOutputExtended as u8,
            1, // option1: 1 = set all pin functions
            0, // option2: 0 = pin functions (not additional settings)
            0,
            0,
            raw,
        )?;

        if response.len() < 64 {
            return Err(PoKeysError::Protocol(
                "Response too short for bulk pin write".to_string(),
            ));
        }

        if response[1] != Command::InputOutputExtended as u8 {
            return Err(PoKeysError::Protocol(
                "Invalid response command".to_string(),
            ));
        }

        for i in 0..55 {
            if i < self.pins.len() {
                self.pins[i].pin_function = raw[i];
            }
        }

        Ok(())
    }

    /// Set all pin functions at once using extended mode (0xC0)
    /// This is much more efficient than setting pins individually
    /// Performance improvement: 55x fewer commands — the 55-byte function
    /// array is sent in a single request instead of one request per pin.
    pub fn set_all_pin_functions(&mut self, functions: &[PinFunction; 55]) -> Result<()> {
        use crate::io::private::Command;

        // Payload: 55 pin-function bytes that land at protocol bytes 8..63
        // (prepare_request_with_data copies payload starting at request[8]).
        let payload: [u8; 55] = std::array::from_fn(|i| functions[i] as u8);

        let response = self.send_request_with_data(
            Command::InputOutputExtended as u8,
            1, // option1: 1 = set all pin functions
            0, // option2: 0 = pin functions (not additional settings)
            0, // reserved
            0, // request ID will be set by send_request_with_data
            &payload,
        )?;

        if response.len() < 64 {
            return Err(PoKeysError::Protocol(
                "Response too short for bulk pin write".to_string(),
            ));
        }

        if response[1] != Command::InputOutputExtended as u8 {
            return Err(PoKeysError::Protocol(
                "Invalid response command".to_string(),
            ));
        }

        // Update local pin cache to reflect the values we just wrote.
        for (i, &function) in functions.iter().enumerate() {
            if i < self.pins.len() {
                self.pins[i].pin_function = function as u8;
            }
        }

        Ok(())
    }

    /// Read combined device status (digital inputs, analog inputs, and encoder values)
    /// in a single round-trip using protocol command `0xCC`
    /// ("Get device status (IO, analog, encoders)").
    ///
    /// Updates `self.pins[*].digital_value_get`, `self.pins[*].analog_value` (for up to 5
    /// analog-input pins), and `self.encoders[*].encoder_value` (up to 25 channels).
    ///
    /// Matrix keyboard rows and ultra-fast encoder data from the response are not
    /// applied here — use the dedicated methods for those when needed.
    pub fn get_device_status(&mut self) -> Result<()> {
        let response = self.send_request(0xCC, 0, 0, 0, 0)?;
        apply_device_status_response(&response, &mut self.pins, &mut self.encoders);
        Ok(())
    }
}

/// Pack a 10-bit analog output value into the `(MSB, LSB)` pair expected by
/// protocol command `0x41`. Byte 4 holds the top 8 bits; the upper 2 bits of
/// byte 5 hold the low 2 bits of the value. Values wider than 10 bits are
/// truncated.
fn encode_analog_output_10bit(value: u32) -> (u8, u8) {
    let v = value & 0x3FF;
    let msb = ((v >> 2) & 0xFF) as u8;
    let lsb = ((v & 0x03) << 6) as u8;
    (msb, lsb)
}

/// Validate a bulk `0xC0` pin-settings response and extract the 55 setting
/// bytes starting at offset 8. Shared by [`PoKeysDevice::read_all_pin_functions`]
/// and [`PoKeysDevice::read_all_pin_settings_raw`] so the parsing can be
/// unit-tested without a live device.
pub(crate) fn parse_bulk_pin_settings_response(response: &[u8]) -> Result<[u8; 55]> {
    use crate::io::private::Command;

    if response.len() < 64 {
        return Err(PoKeysError::Protocol(
            "Response too short for bulk pin read".to_string(),
        ));
    }

    if response[1] != Command::InputOutputExtended as u8 {
        return Err(PoKeysError::Protocol(
            "Invalid response command".to_string(),
        ));
    }

    let mut raw = [0u8; 55];
    raw.copy_from_slice(&response[8..8 + 55]);
    Ok(raw)
}

/// Apply a `0xCC` ("Get device status") response to the caller-owned pin and
/// encoder arrays. Split out from [`PoKeysDevice::get_device_status`] so the
/// parsing can be unit-tested without a live device.
fn apply_device_status_response(
    response: &[u8],
    pins: &mut [PinData],
    encoders: &mut [crate::encoders::EncoderData],
) {
    // Digital inputs: doc bytes 9-15 (0-based 8-14), bit-mapped pin 1..=55.
    for i in 0..pins.len().min(55) {
        let byte_index = 8 + (i / 8);
        let bit_index = i % 8;
        if byte_index < response.len() {
            pins[i].digital_value_get = if (response[byte_index] & (1 << bit_index)) != 0 {
                1
            } else {
                0
            };
        }
    }

    // Analog inputs: doc bytes 16-25 (0-based 15-24), 5 channels × (MSB, LSB).
    let mut data_index = 15;
    let mut channels_consumed = 0;
    for pin in pins.iter_mut() {
        if channels_consumed >= 5 {
            break;
        }
        if pin.is_analog_input() && data_index + 1 < response.len() {
            let msb = response[data_index] as u32;
            let lsb = response[data_index + 1] as u32;
            pin.analog_value = (msb << 8) | lsb;
            data_index += 2;
            channels_consumed += 1;
        }
    }

    // Encoders: doc bytes 26-50 (0-based 25-49), 25 × 8-bit signed RAW values.
    for i in 0..encoders.len().min(25) {
        let idx = 25 + i;
        if idx < response.len() {
            encoders[i].encoder_value = response[idx] as i8 as i32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_data_creation() {
        let pin_data = PinData::new();
        assert_eq!(pin_data.pin_function, PinFunction::PinRestricted as u8);
        assert_eq!(pin_data.digital_value_get, 0);
        assert_eq!(pin_data.digital_value_set, 0);
    }

    #[test]
    fn test_pin_function_checks() {
        let mut pin_data = PinData::new();

        pin_data.pin_function = PinFunction::DigitalInput as u8;
        assert!(pin_data.is_digital_input());
        assert!(!pin_data.is_digital_output());

        pin_data.pin_function = PinFunction::DigitalOutput as u8;
        assert!(!pin_data.is_digital_input());
        assert!(pin_data.is_digital_output());

        pin_data.pin_function =
            (PinFunction::DigitalInput as u8) | (PinFunction::DigitalOutput as u8);
        assert!(pin_data.is_digital_input());
        assert!(pin_data.is_digital_output());
    }

    #[test]
    fn test_encode_analog_output_10bit() {
        assert_eq!(encode_analog_output_10bit(0), (0x00, 0x00));
        assert_eq!(encode_analog_output_10bit(1023), (0xFF, 0xC0));
        assert_eq!(encode_analog_output_10bit(512), (0x80, 0x00));
        // low two bits land in LSB byte's upper two bits
        assert_eq!(encode_analog_output_10bit(3), (0x00, 0xC0));
        assert_eq!(encode_analog_output_10bit(1), (0x00, 0x40));
        // values wider than 10 bits are truncated
        assert_eq!(encode_analog_output_10bit(0xFFFF), (0xFF, 0xC0));
    }

    #[test]
    fn test_apply_device_status_response() {
        use crate::encoders::EncoderData;

        let mut response = [0u8; 64];

        // Digital inputs: set pins 1, 9, and 55 high.
        response[8] = 0b0000_0001; // pin 1
        response[9] = 0b0000_0001; // pin 9
        response[14] = 0b0100_0000; // pin 55 = bit (55-1) % 8 = 6 of byte 14

        // Analog channel 1 → 0x1234 (MSB=0x12, LSB=0x34) at doc bytes 16-17 (0-based 15-16)
        response[15] = 0x12;
        response[16] = 0x34;
        // Analog channel 2 → 0xABCD (MSB=0xAB, LSB=0xCD) at doc bytes 18-19 (0-based 17-18)
        response[17] = 0xAB;
        response[18] = 0xCD;

        // Encoder 1 raw = -1 (0xFF signed), encoder 2 raw = 5
        response[25] = 0xFF;
        response[26] = 0x05;

        let mut pins = vec![PinData::new(); 55];
        pins[0].pin_function = PinFunction::DigitalInput as u8; // pin 1 digital
        pins[8].pin_function = PinFunction::DigitalInput as u8; // pin 9 digital
        pins[40].pin_function = PinFunction::AnalogInput as u8; // first analog-capable pin
        pins[41].pin_function = PinFunction::AnalogInput as u8; // second analog-capable pin

        let mut encoders = vec![EncoderData::new(); 25];

        apply_device_status_response(&response, &mut pins, &mut encoders);

        assert_eq!(pins[0].digital_value_get, 1);
        assert_eq!(pins[1].digital_value_get, 0);
        assert_eq!(pins[8].digital_value_get, 1);
        assert_eq!(pins[54].digital_value_get, 1);

        assert_eq!(pins[40].analog_value, 0x1234);
        assert_eq!(pins[41].analog_value, 0xABCD);

        assert_eq!(encoders[0].encoder_value, -1);
        assert_eq!(encoders[1].encoder_value, 5);
        assert_eq!(encoders[2].encoder_value, 0);
    }

    #[test]
    fn test_compose_pin_function_byte() {
        use crate::io::private::compose_pin_function_byte;

        // Non-inverted: exactly the enum discriminant.
        assert_eq!(
            compose_pin_function_byte(PinFunction::DigitalInput, false),
            0x02
        );
        assert_eq!(
            compose_pin_function_byte(PinFunction::DigitalOutput, false),
            0x04
        );
        assert_eq!(
            compose_pin_function_byte(PinFunction::TriggeredInput, false),
            0x20
        );

        // Inverted: bit 7 set, base function preserved.
        assert_eq!(
            compose_pin_function_byte(PinFunction::DigitalInput, true),
            0x82
        );
        assert_eq!(
            compose_pin_function_byte(PinFunction::DigitalOutput, true),
            0x84
        );
        assert_eq!(
            compose_pin_function_byte(PinFunction::TriggeredInput, true),
            0xA0
        );
    }

    #[test]
    fn test_decode_pin_function_from_cache() {
        // Combined bytes decode to their base function; the invert bit is
        // ignored by the decoder (caller uses `is_inverted` / `get_pin_invert`
        // to recover it).
        assert_eq!(
            decode_pin_function_from_cache(0x82),
            PinFunction::DigitalInput
        );
        assert_eq!(
            decode_pin_function_from_cache(0x84),
            PinFunction::DigitalOutput
        );
        assert_eq!(
            decode_pin_function_from_cache(0xA0),
            PinFunction::TriggeredInput
        );

        // Base-only values round-trip.
        assert_eq!(
            decode_pin_function_from_cache(0x02),
            PinFunction::DigitalInput
        );
        assert_eq!(
            decode_pin_function_from_cache(0x00),
            PinFunction::PinRestricted
        );

        // `0x80` alone (invert flag with no base function) is not a valid
        // pin configuration — decode as PinRestricted.
        assert_eq!(
            decode_pin_function_from_cache(0x80),
            PinFunction::PinRestricted
        );
    }

    #[test]
    fn test_pin_data_invert_accessors() {
        let mut pin_data = PinData::new();

        pin_data.pin_function = 0x82; // DigitalInput | Invert
        assert!(pin_data.is_inverted());
        assert_eq!(pin_data.base_function(), PinFunction::DigitalInput);
        // Existing masked helpers still work on the OR'd byte.
        assert!(pin_data.is_digital_input());
        assert!(!pin_data.is_digital_output());

        pin_data.pin_function = 0x02; // DigitalInput, no invert
        assert!(!pin_data.is_inverted());
        assert_eq!(pin_data.base_function(), PinFunction::DigitalInput);
    }

    #[test]
    fn test_parse_bulk_pin_settings_response_preserves_invert_bit() {
        use crate::io::private::Command;

        let mut response = [0u8; 64];
        response[1] = Command::InputOutputExtended as u8;
        response[8] = 0x82; // pin 1 = DigitalInput | Invert
        response[9] = 0x04; // pin 2 = DigitalOutput (no invert)
        response[10] = 0xA0; // pin 3 = TriggeredInput | Invert

        let raw = parse_bulk_pin_settings_response(&response).unwrap();
        assert_eq!(raw[0], 0x82);
        assert_eq!(raw[1], 0x04);
        assert_eq!(raw[2], 0xA0);

        // The typed-view decode in `read_all_pin_functions` drops the invert
        // bit; verify `from_u8` returns the base function for a combined byte.
        assert_eq!(
            PinFunction::from_u8(raw[0]).unwrap(),
            PinFunction::DigitalInput
        );
        assert_eq!(
            PinFunction::from_u8(raw[2]).unwrap(),
            PinFunction::TriggeredInput
        );
    }

    #[test]
    fn test_parse_bulk_pin_settings_response_rejects_short() {
        let short = [0u8; 32];
        assert!(parse_bulk_pin_settings_response(&short).is_err());
    }

    #[test]
    fn test_parse_bulk_pin_settings_response_rejects_wrong_command() {
        let mut response = [0u8; 64];
        response[1] = 0x12; // not InputOutputExtended
        assert!(parse_bulk_pin_settings_response(&response).is_err());
    }
}
