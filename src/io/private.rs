//! Private implementation details for I/O operations
use crate::PinFunction;
use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};

/// Command codes for I/O operations
pub(crate) enum Command {
    SetInputOutput = 16,       // 0x10
    ReadDigitalInput = 48,     // 0x30
    SetPinOutput = 64,         // 0x40
    InputOutputExtended = 192, // 0xC0 - Bulk pin operations
}

impl PoKeysDevice {
    pub(crate) fn get_pin_index(&self, pin: u32) -> usize {
        let pin_index: usize = (pin - 1) as usize;
        pin_index
    }

    /// Helper to check if pins is in a valid range
    pub(crate) fn check_pin_range(&self, pin: u32) -> Result<usize> {
        if pin == 0 || pin as usize > self.pins.len() {
            Err(PoKeysError::Parameter("Invalid pin number".to_string()))
        } else {
            Ok(self.get_pin_index(pin))
        }
    }

    /// Send pin configuration to device
    pub(crate) fn write_pin_function(
        &mut self,
        pin: u32,
        pin_function: crate::io::PinFunction,
    ) -> Result<(u32, PinFunction)> {
        match self.check_pin_range(pin) {
            Ok(pin_index) => {
                // Check if the pin is the correct function already
                if self.pins[pin_index].pin_function == pin_function as u8 {
                    return Ok((pin, pin_function));
                }

                // Convert PinFunction to capability string
                let capability = match pin_function {
                    PinFunction::DigitalInput => "DigitalInput",
                    PinFunction::DigitalOutput => "DigitalOutput",
                    PinFunction::AnalogInput => "AnalogInput",
                    PinFunction::AnalogOutput => "AnalogOutput",
                    _ => "", // Other functions don't have direct mappings
                };

                // Check if the pin supports this capability
                if !capability.is_empty() && !self.is_pin_capability_supported(pin, capability) {
                    log::warn!("Pin {} does not support capability: {}", pin, capability);

                    // Mark the pin as inactive in the model
                    if let Some(model) = &mut self.model {
                        if let Some(pin_model) = model.pins.get_mut(&(pin as u8)) {
                            pin_model.active = false;
                            log::warn!("Pin {} marked as inactive", pin);
                        }
                    }

                    return Err(PoKeysError::UnsupportedPinCapability(
                        pin as u8,
                        capability.to_string(),
                    ));
                }

                // pin_index == 0 (pin 1) is ambiguous on 0x10: the device
                // cannot distinguish "set pin 1" from the bulk-read form
                // (all params zero). Use a read-modify-write via 0xC0 instead,
                // which sets all 55 pins atomically and has no such ambiguity.
                if pin_index == 0 {
                    let mut functions = self.read_all_pin_functions()?;
                    functions[0] = pin_function;
                    self.set_all_pin_functions(&functions)?;
                    return Ok((pin, pin_function));
                }

                // All other pins: use the single-pin 0x10 command.
                let res = self.send_request(
                    Command::SetInputOutput as u8,
                    pin as u8,
                    pin_function as u8,
                    self.pins[pin_index].counter_options,
                    0,
                )?;

                if res[2] != 0 {
                    Err(PoKeysError::InternalError(
                        "Invalid pin or configuration locked".to_string(),
                    ))
                } else {
                    self.pins[pin_index].pin_function = pin_function as u8;
                    Ok((pin, pin_function))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Read digital input from device
    pub(crate) fn read_digital_input(&mut self, pin: u32) -> Result<u8> {
        match self.check_pin_range(pin) {
            Ok(pin_index) => {
                let res =
                    self.send_request(Command::ReadDigitalInput as u8, pin_index as u8, 0, 0, 0)?;

                if res[2] != 0 {
                    Err(PoKeysError::InternalError(
                        "Invalid pin or configuration locked".to_string(),
                    ))
                } else {
                    Ok(res[3])
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Write digital output to device
    pub(crate) fn write_digital_output(&mut self, pin: u32, value: bool) -> Result<bool> {
        match self.check_pin_range(pin) {
            Ok(pin_index) => {
                let res = self.send_request(
                    Command::SetPinOutput as u8,
                    pin_index as u8,
                    if value { 1 } else { 0 },
                    0,
                    0,
                )?;

                if res[2] != 0 || res[1] != Command::SetPinOutput as u8 {
                    Err(PoKeysError::InternalError(
                        "Invalid pin or configuration locked".to_string(),
                    ))
                } else {
                    Ok(true)
                }
            }
            Err(e) => Err(e),
        }
    }
}
