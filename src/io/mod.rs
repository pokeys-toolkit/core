//! Digital and analog I/O operations

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use log::info;
use serde::{Deserialize, Serialize};

mod private;

/// Pin function configuration
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
    InvertPin = 128,
}

impl PinFunction {
    /// Convert u8 value to PinFunction enum
    /// Note: PoKeys uses bit flags for pin functions
    pub fn from_u8(value: u8) -> Result<Self> {
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
}

impl Default for PinData {
    fn default() -> Self {
        Self::new()
    }
}

impl PoKeysDevice {
    /// Set pin function
    pub fn set_pin_function(
        &mut self,
        pin: u32,
        pin_function: PinFunction,
    ) -> Result<(u32, PinFunction)> {
        match self.write_pin_function(pin, pin_function) {
            Ok((pin, pin_function)) => Ok((pin, pin_function)),
            Err(e) => Err(e),
        }
    }

    /// Get pin function
    pub fn get_pin_function(&self, pin: u32) -> Result<PinFunction> {
        match self.check_pin_range(pin) {
            Ok(pin_index) => {
                let function_value = self.pins[pin_index].pin_function;

                // Convert back to enum (simplified)
                match function_value {
                    0 => Ok(PinFunction::PinRestricted),
                    1 => Ok(PinFunction::Reserved),
                    2 => Ok(PinFunction::DigitalInput),
                    4 => Ok(PinFunction::DigitalOutput),
                    8 => Ok(PinFunction::AnalogInput),
                    16 => Ok(PinFunction::AnalogOutput),
                    32 => Ok(PinFunction::TriggeredInput),
                    64 => Ok(PinFunction::DigitalCounter),
                    128 => Ok(PinFunction::InvertPin),
                    _ => Ok(PinFunction::PinRestricted), // Default for combined flags
                }
            }

            Err(e) => Err(e),
        }
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
        Ok(true)
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

    /// Reset digital counter
    pub fn reset_digital_counter(&mut self, pin: u32) -> Result<()> {
        if pin == 0 || pin as usize > self.pins.len() {
            return Err(PoKeysError::Parameter("Invalid pin number".to_string()));
        }

        let pin_index = (pin - 1) as usize;
        if self.pins[pin_index].digital_counter_available == 0 {
            return Err(PoKeysError::NotSupported);
        }
        // Send counter reset command
        self.send_request(0x30, pin as u8, 0, 0, 0)?;
        Ok(())
    }

    /// Read all digital inputs
    pub fn get_digital_inputs(&mut self) -> Result<()> {
        let response = self.send_request(0x10, 0, 0, 0, 0)?;

        // Parse digital input data from response
        for i in 0..self.pins.len().min(55) {
            let byte_index = 8 + (i / 8);
            let bit_index = i % 8;

            if byte_index < response.len() {
                self.pins[i].digital_value_get = if (response[byte_index] & (1 << bit_index)) != 0 {
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
    /// clamped by masking.
    pub fn write_analog_outputs(&mut self) -> Result<()> {
        let targets: Vec<(u8, u32)> = self
            .pins
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.is_analog_output() {
                    Some(((i + 1) as u8, p.analog_value))
                } else {
                    None
                }
            })
            .collect();

        for (pin_id, value) in targets {
            let (msb, lsb) = encode_analog_output_10bit(value);
            let response = self.send_request(0x41, pin_id, msb, lsb, 0)?;
            // Response byte 3 (0-based index 2): 0 = OK, non-zero = error ID
            if response[2] != 0 {
                return Err(PoKeysError::Protocol(format!(
                    "Analog output write failed for pin {}: error code {}",
                    pin_id, response[2]
                )));
            }
        }

        Ok(())
    }

    /// Read all digital counters
    pub fn read_digital_counters(&mut self) -> Result<()> {
        let response = self.send_request(0x31, 0, 0, 0, 0)?;

        // Parse digital counter data from response
        let mut data_index = 8;
        for i in 0..self.pins.len() {
            if self.pins[i].digital_counter_available != 0 && data_index + 3 < response.len() {
                self.pins[i].digital_counter_value = u32::from_le_bytes([
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

        // Parse pin functions from bytes 8-62 (55 bytes)
        // Note: PoKeys response format has data starting at byte 8
        let mut functions = [PinFunction::PinRestricted; 55];
        for i in 0..55 {
            let function_value = response[8 + i];
            functions[i] = PinFunction::from_u8(function_value)?;
        }

        // Update local pin cache
        for (i, &function) in functions.iter().enumerate() {
            if i < self.pins.len() {
                self.pins[i].pin_function = function as u8;
            }
        }

        Ok(functions)
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
}
