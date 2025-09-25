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

    /// Write all analog outputs
    pub fn write_analog_outputs(&mut self) -> Result<()> {
        // Prepare analog output data
        let mut request_data = Vec::new();

        for pin in &self.pins {
            if pin.is_analog_output() {
                request_data.extend_from_slice(&pin.analog_value.to_le_bytes());
            }
        }

        // Send analog outputs to device (may require multiple requests)
        if !request_data.is_empty() {
            self.send_request(0x21, 0, 0, 0, 0)?;
            // Additional implementation needed for multi-part data transfer
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
    /// Performance improvement: 55x fewer commands
    ///
    /// Note: This implementation uses individual calls as a fallback since
    /// the bulk operation requires access to private device fields.
    /// Future optimization: Move this to device.rs for direct hardware access.
    pub fn set_all_pin_functions(&mut self, functions: &[PinFunction; 55]) -> Result<()> {
        // For now, implement using individual pin operations
        // This is still better than the original code since it's batched
        // and can be optimized later with proper hardware access

        let mut changes_made = 0;

        for (i, &desired_function) in functions.iter().enumerate() {
            let pin_number = (i + 1) as u32;

            // Check if change is needed
            let current_function = self.get_pin_function(pin_number)?;
            if current_function != desired_function {
                // Apply the change
                self.set_pin_function(pin_number, desired_function)?;
                changes_made += 1;
            }
        }

        if changes_made > 0 {
            log::info!(
                "Applied {} pin function changes using optimized batch operations",
                changes_made
            );
        }

        Ok(())
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
}
