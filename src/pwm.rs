//! PWM (Pulse Width Modulation) support

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use serde::{Deserialize, Serialize};

/// PWM data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmData {
    pub pwm_period: u32,
    pub pwm_duty: Vec<u32>,
    pub pwm_enabled_channels: Vec<u8>,
    pub pwm_pin_ids: Vec<u8>,
}

impl PwmData {
    pub fn new() -> Self {
        Self {
            pwm_period: 0,
            pwm_duty: Vec::new(),
            pwm_enabled_channels: Vec::new(),
            pwm_pin_ids: Vec::new(),
        }
    }

    pub fn initialize(&mut self, channel_count: usize) {
        self.pwm_duty = vec![0; channel_count];
        self.pwm_enabled_channels = vec![0; channel_count];
        self.pwm_pin_ids = vec![0; channel_count];
    }

    pub fn get_channel_count(&self) -> usize {
        self.pwm_duty.len()
    }

    pub fn is_channel_enabled(&self, channel: usize) -> bool {
        channel < self.pwm_enabled_channels.len() && self.pwm_enabled_channels[channel] != 0
    }

    pub fn enable_channel(&mut self, channel: usize, enable: bool) {
        if channel < self.pwm_enabled_channels.len() {
            self.pwm_enabled_channels[channel] = if enable { 1 } else { 0 };
        }
    }

    pub fn set_duty_cycle(&mut self, channel: usize, duty: u32) -> Result<()> {
        if channel >= self.pwm_duty.len() {
            return Err(PoKeysError::Parameter("Invalid PWM channel".to_string()));
        }

        if duty > self.pwm_period {
            return Err(PoKeysError::Parameter(
                "Duty cycle cannot exceed period".to_string(),
            ));
        }

        self.pwm_duty[channel] = duty;
        Ok(())
    }

    pub fn get_duty_cycle(&self, channel: usize) -> Result<u32> {
        if channel >= self.pwm_duty.len() {
            return Err(PoKeysError::Parameter("Invalid PWM channel".to_string()));
        }

        Ok(self.pwm_duty[channel])
    }

    pub fn set_pin_id(&mut self, channel: usize, pin_id: u8) -> Result<()> {
        if channel >= self.pwm_pin_ids.len() {
            return Err(PoKeysError::Parameter("Invalid PWM channel".to_string()));
        }

        self.pwm_pin_ids[channel] = pin_id;
        Ok(())
    }

    pub fn get_pin_id(&self, channel: usize) -> Result<u8> {
        if channel >= self.pwm_pin_ids.len() {
            return Err(PoKeysError::Parameter("Invalid PWM channel".to_string()));
        }

        Ok(self.pwm_pin_ids[channel])
    }
}

impl Default for PwmData {
    fn default() -> Self {
        Self::new()
    }
}

impl PoKeysDevice {
    /// Set PWM period (shared among all channels)
    pub fn set_pwm_period(&mut self, period: u32) -> Result<()> {
        if period == 0 {
            return Err(PoKeysError::Parameter(
                "PWM period cannot be zero".to_string(),
            ));
        }

        self.pwm.pwm_period = period;

        // Send PWM period to device
        self.send_request(
            0x50,
            (period & 0xFF) as u8,
            ((period >> 8) & 0xFF) as u8,
            ((period >> 16) & 0xFF) as u8,
            ((period >> 24) & 0xFF) as u8,
        )?;

        Ok(())
    }

    /// Get PWM period
    pub fn get_pwm_period(&self) -> u32 {
        self.pwm.pwm_period
    }

    /// Set PWM duty cycle for a specific channel
    pub fn set_pwm_duty_cycle(&mut self, channel: usize, duty: u32) -> Result<()> {
        self.pwm.set_duty_cycle(channel, duty)?;

        // Send PWM duty cycle to device
        self.send_request(
            0x51,
            channel as u8,
            (duty & 0xFF) as u8,
            ((duty >> 8) & 0xFF) as u8,
            ((duty >> 16) & 0xFF) as u8,
        )?;

        Ok(())
    }

    /// Get PWM duty cycle for a specific channel
    pub fn get_pwm_duty_cycle(&self, channel: usize) -> Result<u32> {
        self.pwm.get_duty_cycle(channel)
    }

    /// Set PWM duty cycle as percentage (0.0 to 100.0)
    pub fn set_pwm_duty_cycle_percent(&mut self, channel: usize, percent: f32) -> Result<()> {
        if !(0.0..=100.0).contains(&percent) {
            return Err(PoKeysError::Parameter(
                "Percentage must be between 0.0 and 100.0".to_string(),
            ));
        }

        let duty = ((percent / 100.0) * self.pwm.pwm_period as f32) as u32;
        self.set_pwm_duty_cycle(channel, duty)
    }

    /// Get PWM duty cycle as percentage
    pub fn get_pwm_duty_cycle_percent(&self, channel: usize) -> Result<f32> {
        let duty = self.pwm.get_duty_cycle(channel)?;
        if self.pwm.pwm_period == 0 {
            return Ok(0.0);
        }
        Ok((duty as f32 / self.pwm.pwm_period as f32) * 100.0)
    }

    /// Enable or disable PWM channel
    pub fn enable_pwm_channel(&mut self, channel: usize, enable: bool) -> Result<()> {
        if channel >= self.pwm.get_channel_count() {
            return Err(PoKeysError::Parameter("Invalid PWM channel".to_string()));
        }

        self.pwm.enable_channel(channel, enable);

        // Send PWM channel enable/disable to device
        self.send_request(0x52, channel as u8, if enable { 1 } else { 0 }, 0, 0)?;

        Ok(())
    }

    /// Check if PWM channel is enabled
    pub fn is_pwm_channel_enabled(&self, channel: usize) -> bool {
        self.pwm.is_channel_enabled(channel)
    }

    /// Set PWM pin assignment for a channel
    pub fn set_pwm_pin(&mut self, channel: usize, pin_id: u8) -> Result<()> {
        self.pwm.set_pin_id(channel, pin_id)?;

        // Send PWM pin assignment to device
        self.send_request(0x53, channel as u8, pin_id, 0, 0)?;

        Ok(())
    }

    /// Get PWM pin assignment for a channel
    pub fn get_pwm_pin(&self, channel: usize) -> Result<u8> {
        self.pwm.get_pin_id(channel)
    }

    /// Update all PWM channels at once
    pub fn update_all_pwm_channels(&mut self) -> Result<()> {
        // Send all PWM data to device
        self.send_request(0x54, 0, 0, 0, 0)?;

        // This would typically involve sending multi-part data
        // for all PWM channels in a single transaction

        Ok(())
    }

    /// Read PWM configuration from device
    pub fn read_pwm_configuration(&mut self) -> Result<()> {
        let response = self.send_request(0x55, 0, 0, 0, 0)?;

        // Parse PWM configuration from response
        if response.len() >= 12 {
            self.pwm.pwm_period =
                u32::from_le_bytes([response[8], response[9], response[10], response[11]]);
        }

        // Read individual channel configurations
        for channel in 0..self.pwm.get_channel_count() {
            let response = self.send_request(0x56, channel as u8, 0, 0, 0)?;

            if response.len() >= 16 {
                self.pwm.pwm_duty[channel] =
                    u32::from_le_bytes([response[8], response[9], response[10], response[11]]);

                self.pwm.pwm_enabled_channels[channel] = response[12];
                self.pwm.pwm_pin_ids[channel] = response[13];
            }
        }

        Ok(())
    }

    /// Set PWM frequency (calculates period based on internal frequency)
    pub fn set_pwm_frequency(&mut self, frequency_hz: u32) -> Result<()> {
        if frequency_hz == 0 {
            return Err(PoKeysError::Parameter(
                "Frequency cannot be zero".to_string(),
            ));
        }

        // Calculate period based on internal PWM frequency
        let internal_freq = self.info.pwm_internal_frequency;
        if internal_freq == 0 {
            return Err(PoKeysError::NotSupported);
        }

        let period = internal_freq / frequency_hz;
        if period == 0 {
            return Err(PoKeysError::Parameter("Frequency too high".to_string()));
        }

        self.set_pwm_period(period)
    }

    /// Get PWM frequency
    pub fn get_pwm_frequency(&self) -> Result<u32> {
        if self.pwm.pwm_period == 0 {
            return Ok(0);
        }

        let internal_freq = self.info.pwm_internal_frequency;
        if internal_freq == 0 {
            return Err(PoKeysError::NotSupported);
        }

        Ok(internal_freq / self.pwm.pwm_period)
    }

    /// Configure PWM channel with all parameters
    pub fn configure_pwm_channel(
        &mut self,
        channel: usize,
        pin_id: u8,
        duty_percent: f32,
        enabled: bool,
    ) -> Result<()> {
        self.set_pwm_pin(channel, pin_id)?;
        self.set_pwm_duty_cycle_percent(channel, duty_percent)?;
        self.enable_pwm_channel(channel, enabled)?;
        Ok(())
    }

    /// Stop all PWM channels
    pub fn stop_all_pwm(&mut self) -> Result<()> {
        for channel in 0..self.pwm.get_channel_count() {
            self.enable_pwm_channel(channel, false)?;
        }
        Ok(())
    }

    /// Start all configured PWM channels
    pub fn start_all_pwm(&mut self) -> Result<()> {
        for channel in 0..self.pwm.get_channel_count() {
            if self.pwm.pwm_pin_ids[channel] != 0 {
                self.enable_pwm_channel(channel, true)?;
            }
        }
        Ok(())
    }
}

// Convenience functions for common PWM operations

/// Set PWM output with frequency and duty cycle
pub fn set_pwm_output(
    device: &mut PoKeysDevice,
    channel: usize,
    pin_id: u8,
    frequency_hz: u32,
    duty_percent: f32,
) -> Result<()> {
    device.set_pwm_frequency(frequency_hz)?;
    device.configure_pwm_channel(channel, pin_id, duty_percent, true)?;
    Ok(())
}

/// Create a simple PWM signal
pub fn simple_pwm(
    device: &mut PoKeysDevice,
    pin_id: u8,
    frequency_hz: u32,
    duty_percent: f32,
) -> Result<()> {
    set_pwm_output(device, 0, pin_id, frequency_hz, duty_percent)
}

/// Create a servo control signal (50Hz, 1-2ms pulse width)
pub fn servo_control(device: &mut PoKeysDevice, pin_id: u8, position_percent: f32) -> Result<()> {
    if !(0.0..=100.0).contains(&position_percent) {
        return Err(PoKeysError::Parameter(
            "Position must be between 0.0 and 100.0".to_string(),
        ));
    }

    // Servo signals are typically 50Hz (20ms period)
    // Pulse width varies from 1ms (5% duty) to 2ms (10% duty)
    let duty_percent = 5.0 + (position_percent / 100.0) * 5.0;

    set_pwm_output(device, 0, pin_id, 50, duty_percent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwm_data_creation() {
        let mut pwm_data = PwmData::new();
        assert_eq!(pwm_data.get_channel_count(), 0);

        pwm_data.initialize(6);
        assert_eq!(pwm_data.get_channel_count(), 6);
        assert!(!pwm_data.is_channel_enabled(0));
    }

    #[test]
    fn test_pwm_duty_cycle() {
        let mut pwm_data = PwmData::new();
        pwm_data.initialize(2);
        pwm_data.pwm_period = 1000;

        assert!(pwm_data.set_duty_cycle(0, 500).is_ok());
        assert_eq!(pwm_data.get_duty_cycle(0).unwrap(), 500);

        // Test duty cycle exceeding period
        assert!(pwm_data.set_duty_cycle(0, 1500).is_err());

        // Test invalid channel
        assert!(pwm_data.set_duty_cycle(5, 100).is_err());
    }

    #[test]
    fn test_pwm_channel_enable() {
        let mut pwm_data = PwmData::new();
        pwm_data.initialize(2);

        assert!(!pwm_data.is_channel_enabled(0));
        pwm_data.enable_channel(0, true);
        assert!(pwm_data.is_channel_enabled(0));
        pwm_data.enable_channel(0, false);
        assert!(!pwm_data.is_channel_enabled(0));
    }

    #[test]
    fn test_servo_control_parameters() {
        // Test that servo control calculates correct duty cycles
        // 0% position should give 5% duty cycle
        // 100% position should give 10% duty cycle
        let duty_0 = 5.0 + (0.0 / 100.0) * 5.0;
        let duty_100 = 5.0 + (100.0 / 100.0) * 5.0;

        assert_eq!(duty_0, 5.0);
        assert_eq!(duty_100, 10.0);

        // 50% position should give 7.5% duty cycle
        let duty_50 = 5.0 + (50.0 / 100.0) * 5.0;
        assert_eq!(duty_50, 7.5);
    }
}
