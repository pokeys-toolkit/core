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

/// Bit 7 of the "pin settings" byte (`0x10` command, byte 4) is the
/// firmware-level invert flag. When set, the device reports the logical
/// complement of the electrical state for inputs and drives the complement
/// of the configured level for outputs.
pub(crate) const INVERT_PIN_BIT: u8 = 0x80;

/// Compose the pin-settings wire byte by ORing the base pin function with
/// the invert flag when requested.
pub(crate) fn compose_pin_function_byte(f: PinFunction, inverted: bool) -> u8 {
    (f as u8) | if inverted { INVERT_PIN_BIT } else { 0 }
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

    /// Send pin configuration to device.
    ///
    /// When `inverted` is true, bit 7 of the pin-settings byte is set so the
    /// firmware reports/drives the logical complement of the electrical state.
    /// Only `DigitalInput`, `DigitalOutput`, and `TriggeredInput` honor the
    /// flag on real hardware; analog and counter functions ignore it.
    pub(crate) fn write_pin_function(
        &mut self,
        pin: u32,
        pin_function: crate::io::PinFunction,
        inverted: bool,
    ) -> Result<(u32, PinFunction)> {
        match self.check_pin_range(pin) {
            Ok(pin_index) => {
                let wire_byte = compose_pin_function_byte(pin_function, inverted);

                // Check if the pin is already configured with this exact
                // byte (including any invert bit) — skip the wire round-trip.
                if self.pins[pin_index].pin_function == wire_byte {
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

                if inverted {
                    match pin_function {
                        PinFunction::DigitalInput
                        | PinFunction::DigitalOutput
                        | PinFunction::TriggeredInput => {}
                        other => log::warn!(
                            "Pin {}: invert flag has no effect on {:?}; firmware will ignore bit 7",
                            pin,
                            other
                        ),
                    }
                }

                // 0x10 is unambiguous even for pin_index=0 because byte 4
                // (pin_settings) is always non-zero for a real function write.
                // The bulk-read form has all bytes zero; the firmware dispatches
                // on byte 4, not byte 3 alone.
                let res = self.send_request(
                    Command::SetInputOutput as u8,
                    pin_index as u8,
                    wire_byte,
                    self.pins[pin_index].counter_options,
                    0,
                )?;

                if res[2] != 0 {
                    Err(PoKeysError::InternalError(
                        "Invalid pin or configuration locked".to_string(),
                    ))
                } else {
                    self.pins[pin_index].pin_function = wire_byte;
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
