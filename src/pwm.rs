//! PWM (Pulse Width Modulation) support

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use serde::{Deserialize, Serialize};

/// PWM channel mapping according to specification
/// PWM1 = pin 22, PWM2 = pin 21, PWM3 = pin 20, PWM4 = pin 19, PWM5 = pin 18, PWM6 = pin 17
const PWM_PIN_MAP: [u8; 6] = [22, 21, 20, 19, 18, 17];

/// Servo type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServoType {
    /// 180-degree servo with calibrated 0° and 180° positions
    OneEighty { pos_0: u32, pos_180: u32 },
    /// 360-degree position servo (multi-turn with position feedback)
    ThreeSixtyPosition { pos_0: u32, pos_360: u32 },
    /// 360-degree speed servo (continuous rotation)
    ThreeSixtySpeed {
        stop: u32,
        clockwise: u32,
        anti_clockwise: u32,
    },
}

/// Servo configuration for a specific pin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoConfig {
    pub pin: u8,
    pub servo_type: ServoType,
}

impl ServoConfig {
    /// Create new 180-degree servo configuration
    pub fn one_eighty(pin: u8, pos_0: u32, pos_180: u32) -> Self {
        Self {
            pin,
            servo_type: ServoType::OneEighty { pos_0, pos_180 },
        }
    }

    /// Create new 360-degree position servo configuration
    pub fn three_sixty_position(pin: u8, pos_0: u32, pos_360: u32) -> Self {
        Self {
            pin,
            servo_type: ServoType::ThreeSixtyPosition { pos_0, pos_360 },
        }
    }

    /// Create new 360-degree speed servo configuration
    pub fn three_sixty_speed(pin: u8, stop: u32, clockwise: u32, anti_clockwise: u32) -> Self {
        Self {
            pin,
            servo_type: ServoType::ThreeSixtySpeed {
                stop,
                clockwise,
                anti_clockwise,
            },
        }
    }

    /// Set servo to specific angle (0-180 degrees for OneEighty, 0-360 for ThreeSixtyPosition)
    pub fn set_angle(&self, device: &mut PoKeysDevice, angle: f32) -> Result<()> {
        let duty = match &self.servo_type {
            ServoType::OneEighty { pos_0, pos_180 } => {
                if !(0.0..=180.0).contains(&angle) {
                    return Err(PoKeysError::Parameter(
                        "Angle must be between 0.0 and 180.0 degrees".to_string(),
                    ));
                }
                let range = *pos_180 as f32 - *pos_0 as f32;
                (*pos_0 as f32 + (angle / 180.0) * range) as u32
            }
            ServoType::ThreeSixtyPosition { pos_0, pos_360 } => {
                if !(0.0..=360.0).contains(&angle) {
                    return Err(PoKeysError::Parameter(
                        "Angle must be between 0.0 and 360.0 degrees".to_string(),
                    ));
                }
                let range = *pos_360 as f32 - *pos_0 as f32;
                (*pos_0 as f32 + (angle / 360.0) * range) as u32
            }
            ServoType::ThreeSixtySpeed { .. } => {
                return Err(PoKeysError::Parameter(
                    "Cannot set angle on speed servo. Use set_speed() instead".to_string(),
                ));
            }
        };

        device.set_pwm_duty_cycle_for_pin(self.pin, duty)
    }

    /// Set servo speed (-100.0 to 100.0, where 0 is stop, positive is clockwise)
    pub fn set_speed(&self, device: &mut PoKeysDevice, speed: f32) -> Result<()> {
        let duty = match &self.servo_type {
            ServoType::ThreeSixtySpeed {
                stop,
                clockwise,
                anti_clockwise,
            } => {
                if !(-100.0..=100.0).contains(&speed) {
                    return Err(PoKeysError::Parameter(
                        "Speed must be between -100.0 and 100.0".to_string(),
                    ));
                }

                if speed == 0.0 {
                    *stop
                } else if speed > 0.0 {
                    // Clockwise: interpolate between stop and clockwise
                    let range = *clockwise as f32 - *stop as f32;
                    (*stop as f32 + (speed / 100.0) * range) as u32
                } else {
                    // Anti-clockwise: interpolate between stop and anti_clockwise
                    let range = *anti_clockwise as f32 - *stop as f32;
                    (*stop as f32 + (speed.abs() / 100.0) * range) as u32
                }
            }
            _ => {
                return Err(PoKeysError::Parameter(
                    "Cannot set speed on position servo. Use set_angle() instead".to_string(),
                ));
            }
        };

        device.set_pwm_duty_cycle_for_pin(self.pin, duty)
    }

    /// Stop the servo (for speed servos)
    pub fn stop(&self, device: &mut PoKeysDevice) -> Result<()> {
        match &self.servo_type {
            ServoType::ThreeSixtySpeed { stop, .. } => {
                device.set_pwm_duty_cycle_for_pin(self.pin, *stop)
            }
            _ => Err(PoKeysError::Parameter(
                "Stop command only applies to speed servos".to_string(),
            )),
        }
    }
}

/// PWM data structure matching the protocol specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmData {
    /// PWM period (shared among all channels)
    pub pwm_period: u32,
    /// PWM duty cycle values for channels 1-6
    pub pwm_values: [u32; 6],
    /// Enabled channels bitmask (bit 0 = PWM1, bit 1 = PWM2, etc.)
    pub enabled_channels: u8,
}

impl Default for PwmData {
    fn default() -> Self {
        Self::new()
    }
}

impl PwmData {
    pub fn new() -> Self {
        Self {
            pwm_period: 0,
            pwm_values: [0; 6],
            enabled_channels: 0,
        }
    }

    /// Get PWM channel index from pin number (17-22)
    pub fn pin_to_channel(pin: u8) -> Result<usize> {
        match pin {
            22 => Ok(0), // PWM1
            21 => Ok(1), // PWM2
            20 => Ok(2), // PWM3
            19 => Ok(3), // PWM4
            18 => Ok(4), // PWM5
            17 => Ok(5), // PWM6
            _ => Err(PoKeysError::Parameter(format!(
                "Pin {} does not support PWM. PWM is only supported on pins 17-22",
                pin
            ))),
        }
    }

    /// Get pin number from PWM channel index
    pub fn channel_to_pin(channel: usize) -> Result<u8> {
        if channel >= 6 {
            return Err(PoKeysError::Parameter(format!(
                "Invalid PWM channel {}. Valid channels are 0-5",
                channel
            )));
        }
        Ok(PWM_PIN_MAP[channel])
    }

    /// Enable or disable a PWM channel
    pub fn set_channel_enabled(&mut self, channel: usize, enabled: bool) -> Result<()> {
        if channel >= 6 {
            return Err(PoKeysError::Parameter(format!(
                "Invalid PWM channel {}. Valid channels are 0-5",
                channel
            )));
        }

        if enabled {
            self.enabled_channels |= 1 << channel;
        } else {
            self.enabled_channels &= !(1 << channel);
        }
        Ok(())
    }

    /// Check if a PWM channel is enabled
    pub fn is_channel_enabled(&self, channel: usize) -> bool {
        if channel >= 6 {
            return false;
        }
        (self.enabled_channels & (1 << channel)) != 0
    }

    /// Set PWM duty cycle for a channel
    pub fn set_duty_cycle(&mut self, channel: usize, duty: u32) -> Result<()> {
        if channel >= 6 {
            return Err(PoKeysError::Parameter(format!(
                "Invalid PWM channel {}. Valid channels are 0-5",
                channel
            )));
        }
        self.pwm_values[channel] = duty;
        Ok(())
    }

    /// Get PWM duty cycle for a channel
    pub fn get_duty_cycle(&self, channel: usize) -> Result<u32> {
        if channel >= 6 {
            return Err(PoKeysError::Parameter(format!(
                "Invalid PWM channel {}. Valid channels are 0-5",
                channel
            )));
        }
        Ok(self.pwm_values[channel])
    }
}

impl PoKeysDevice {
    /// Set PWM configuration using command 0xCB
    pub fn set_pwm_configuration(&mut self) -> Result<()> {
        let mut data = [0u8; 32];

        // PWM enabled channels bitmask
        data[0] = self.pwm.enabled_channels;

        // PWM values (LSB first)
        for i in 0..6 {
            let value = self.pwm.pwm_values[i];
            let base_idx = 1 + (i * 4);
            data[base_idx] = (value & 0xFF) as u8;
            data[base_idx + 1] = ((value >> 8) & 0xFF) as u8;
            data[base_idx + 2] = ((value >> 16) & 0xFF) as u8;
            data[base_idx + 3] = ((value >> 24) & 0xFF) as u8;
        }

        // PWM period (LSB first)
        let period = self.pwm.pwm_period;
        data[25] = (period & 0xFF) as u8;
        data[26] = ((period >> 8) & 0xFF) as u8;
        data[27] = ((period >> 16) & 0xFF) as u8;
        data[28] = ((period >> 24) & 0xFF) as u8;

        self.send_request_with_data(0xCB, 1, 0, 0, 0, &data)?;
        Ok(())
    }

    /// Update only PWM duty values using command 0xCB
    pub fn update_pwm_duty_values(&mut self) -> Result<()> {
        let mut data = [0u8; 32];

        // PWM enabled channels bitmask
        data[0] = self.pwm.enabled_channels;

        // PWM values (LSB first)
        for i in 0..6 {
            let value = self.pwm.pwm_values[i];
            let base_idx = 1 + (i * 4);
            data[base_idx] = (value & 0xFF) as u8;
            data[base_idx + 1] = ((value >> 8) & 0xFF) as u8;
            data[base_idx + 2] = ((value >> 16) & 0xFF) as u8;
            data[base_idx + 3] = ((value >> 24) & 0xFF) as u8;
        }

        self.send_request_with_data(0xCB, 1, 1, 0, 0, &data)?;
        Ok(())
    }

    /// Get PWM configuration using command 0xCB
    pub fn get_pwm_configuration(&mut self) -> Result<()> {
        let response = self.send_request(0xCB, 0, 0, 0, 0)?;

        // Parse response according to specification
        if response.len() >= 38 {
            // PWM enabled channels
            self.pwm.enabled_channels = response[9];

            // PWM values (LSB first)
            for i in 0..6 {
                let base_idx = 10 + (i * 4);
                if base_idx + 3 < response.len() {
                    self.pwm.pwm_values[i] = response[base_idx] as u32
                        | ((response[base_idx + 1] as u32) << 8)
                        | ((response[base_idx + 2] as u32) << 16)
                        | ((response[base_idx + 3] as u32) << 24);
                }
            }

            // PWM period (LSB first)
            if response.len() >= 38 {
                self.pwm.pwm_period = response[34] as u32
                    | ((response[35] as u32) << 8)
                    | ((response[36] as u32) << 16)
                    | ((response[37] as u32) << 24);
            }
        }

        Ok(())
    }

    /// Set PWM period (shared among all channels)
    pub fn set_pwm_period(&mut self, period: u32) -> Result<()> {
        if period == 0 {
            return Err(PoKeysError::Parameter(
                "PWM period cannot be zero".to_string(),
            ));
        }

        self.pwm.pwm_period = period;
        self.set_pwm_configuration()
    }

    /// Get PWM period
    pub fn get_pwm_period(&self) -> u32 {
        self.pwm.pwm_period
    }

    /// Set PWM duty cycle for a pin (17-22)
    pub fn set_pwm_duty_cycle_for_pin(&mut self, pin: u8, duty: u32) -> Result<()> {
        let channel = PwmData::pin_to_channel(pin)?;
        self.pwm.set_duty_cycle(channel, duty)?;
        self.update_pwm_duty_values()
    }

    /// Get PWM duty cycle for a pin (17-22)
    pub fn get_pwm_duty_cycle_for_pin(&self, pin: u8) -> Result<u32> {
        let channel = PwmData::pin_to_channel(pin)?;
        self.pwm.get_duty_cycle(channel)
    }

    /// Enable or disable PWM for a pin (17-22)
    pub fn enable_pwm_for_pin(&mut self, pin: u8, enabled: bool) -> Result<()> {
        let channel = PwmData::pin_to_channel(pin)?;
        self.pwm.set_channel_enabled(channel, enabled)?;
        self.set_pwm_configuration()
    }

    /// Check if PWM is enabled for a pin (17-22)
    pub fn is_pwm_enabled_for_pin(&self, pin: u8) -> Result<bool> {
        let channel = PwmData::pin_to_channel(pin)?;
        Ok(self.pwm.is_channel_enabled(channel))
    }

    /// Set PWM duty cycle as percentage (0.0 to 100.0) for a pin
    pub fn set_pwm_duty_cycle_percent_for_pin(&mut self, pin: u8, percent: f32) -> Result<()> {
        if !(0.0..=100.0).contains(&percent) {
            return Err(PoKeysError::Parameter(
                "PWM duty cycle percentage must be between 0.0 and 100.0".to_string(),
            ));
        }

        let duty = ((percent / 100.0) * self.pwm.pwm_period as f32) as u32;
        self.set_pwm_duty_cycle_for_pin(pin, duty)
    }

    /// Get PWM duty cycle as percentage for a pin
    pub fn get_pwm_duty_cycle_percent_for_pin(&self, pin: u8) -> Result<f32> {
        let duty = self.get_pwm_duty_cycle_for_pin(pin)?;
        if self.pwm.pwm_period == 0 {
            return Ok(0.0);
        }
        Ok((duty as f32 / self.pwm.pwm_period as f32) * 100.0)
    }
}

/// Simple PWM function for easy servo control
pub fn simple_pwm(
    device: &mut PoKeysDevice,
    pin: u8,
    frequency_hz: u32,
    duty_percent: f32,
) -> Result<()> {
    // Validate pin
    PwmData::pin_to_channel(pin)?;

    // PoKeys PWM operates at 25MHz clock frequency
    // Calculate period in clock cycles: period_seconds × 25,000,000
    let period = if frequency_hz > 0 {
        25_000_000 / frequency_hz // 25MHz clock cycles for the given frequency
    } else {
        return Err(PoKeysError::Parameter(
            "Frequency must be greater than 0".to_string(),
        ));
    };

    // Set period
    device.set_pwm_period(period)?;

    // Enable PWM for the pin
    device.enable_pwm_for_pin(pin, true)?;

    // Set duty cycle
    device.set_pwm_duty_cycle_percent_for_pin(pin, duty_percent)
}
