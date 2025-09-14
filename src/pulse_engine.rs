//! Pulse Engine v2 support for motion control

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use crate::types::{PulseEngineAxisState, PulseEngineState};
use serde::{Deserialize, Serialize};

/// Pulse Engine v2 information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseEngineV2Info {
    pub nr_of_axes: u8,
    pub max_pulse_frequency: u8,
    pub buffer_depth: u8,
    pub slot_timing: u8,
}

/// Pulse Engine v2 main structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseEngineV2 {
    pub info: PulseEngineV2Info,
    pub axes_state: [u8; 8],
    pub axes_config: [u8; 8],
    pub axes_switch_config: [u8; 8],
    pub current_position: [i32; 8],
    pub position_setup: [i32; 8],
    pub reference_position_speed: [i32; 8],
    pub invert_axis_enable: [i8; 8],
    pub soft_limit_maximum: [i32; 8],
    pub soft_limit_minimum: [i32; 8],
    pub homing_speed: [u8; 8],
    pub homing_return_speed: [u8; 8],
    pub home_offsets: [i32; 8],
    pub homing_algorithm: [u8; 8],
    pub filter_limit_m_switch: [u8; 8],
    pub filter_limit_p_switch: [u8; 8],
    pub filter_home_switch: [u8; 8],
    pub probe_position: [i32; 8],
    pub probe_max_position: [i32; 8],
    pub max_speed: [f32; 8],
    pub max_acceleration: [f32; 8],
    pub max_deceleration: [f32; 8],
    pub mpg_jog_multiplier: [i32; 8],
    pub mpg_jog_encoder: [u8; 8],
    pub pin_home_switch: [u8; 8],
    pub pin_limit_m_switch: [u8; 8],
    pub pin_limit_p_switch: [u8; 8],
    pub axis_enable_output_pins: [u8; 8],
    pub home_back_off_distance: [u32; 8],
    pub mpg_jog_divider: [u16; 8],
    pub filter_probe_input: u8,
    pub axis_signal_options: [u8; 8],
    pub reference_velocity_pv: [f32; 8],
    pub pulse_engine_enabled: u8,
    pub pulse_generator_type: u8,
    pub charge_pump_enabled: u8,
    pub emergency_switch_polarity: u8,
    pub pulse_engine_activated: u8,
    pub limit_status_p: u8,
    pub limit_status_n: u8,
    pub home_status: u8,
    pub error_input_status: u8,
    pub misc_input_status: u8,
    pub limit_override: u8,
    pub limit_override_setup: u8,
    pub pulse_engine_state: u8,
    pub axis_enabled_mask: u8,
    pub emergency_input_pin: u8,
    pub sync_fast_outputs_axis_id: u8,
    pub sync_fast_outputs_mapping: [u8; 8],
    pub param1: u8,
    pub param2: u8,
    pub param3: u8,
    pub axis_enabled_states_mask: u8,
    pub pulse_engine_state_setup: u8,
    pub soft_limit_status: u8,
    pub external_relay_outputs: u8,
    pub external_oc_outputs: u8,
    pub pulse_engine_buffer_size: u8,
    pub motion_buffer_entries_accepted: u8,
    pub new_motion_buffer_entries: u8,
    pub homing_start_mask_setup: u8,
    pub probe_start_mask_setup: u8,
    pub probe_input: u8,
    pub probe_input_polarity: u8,
    pub probe_status: u8,
    pub motion_buffer: Vec<u8>,
    pub probe_speed: f32,
    pub debug_values: [u32; 16],
    pub backlash_width: [u16; 8],
    pub backlash_register: [i16; 8],
    pub backlash_acceleration: [u8; 8],
    pub backlash_compensation_enabled: u8,
    pub backlash_compensation_max_speed: u8,
    pub trigger_preparing: u8,
    pub trigger_prepared: u8,
    pub trigger_pending: u8,
    pub trigger_active: u8,
    pub spindle_speed_estimate: i32,
    pub spindle_position_error: i32,
    pub spindle_rpm: u32,
    pub spindle_index_counter: u32,
    pub dedicated_limit_n_inputs: u8,
    pub dedicated_limit_p_inputs: u8,
    pub dedicated_home_inputs: u8,
    pub trigger_ignored_axis_mask: u8,
    pub encoder_index_count: u32,
    pub encoder_ticks_per_rotation: u32,
    pub encoder_velocity: u32,
    pub internal_driver_step_config: [u8; 4],
    pub internal_driver_current_config: [u8; 4],
}

impl PulseEngineV2 {
    pub fn new() -> Self {
        Self {
            info: PulseEngineV2Info {
                nr_of_axes: 0,
                max_pulse_frequency: 0,
                buffer_depth: 0,
                slot_timing: 0,
            },
            axes_state: [0; 8],
            axes_config: [0; 8],
            axes_switch_config: [0; 8],
            current_position: [0; 8],
            position_setup: [0; 8],
            reference_position_speed: [0; 8],
            invert_axis_enable: [0; 8],
            soft_limit_maximum: [0; 8],
            soft_limit_minimum: [0; 8],
            homing_speed: [0; 8],
            homing_return_speed: [0; 8],
            home_offsets: [0; 8],
            homing_algorithm: [0; 8],
            filter_limit_m_switch: [0; 8],
            filter_limit_p_switch: [0; 8],
            filter_home_switch: [0; 8],
            probe_position: [0; 8],
            probe_max_position: [0; 8],
            max_speed: [0.0; 8],
            max_acceleration: [0.0; 8],
            max_deceleration: [0.0; 8],
            mpg_jog_multiplier: [0; 8],
            mpg_jog_encoder: [0; 8],
            pin_home_switch: [0; 8],
            pin_limit_m_switch: [0; 8],
            pin_limit_p_switch: [0; 8],
            axis_enable_output_pins: [0; 8],
            home_back_off_distance: [0; 8],
            mpg_jog_divider: [0; 8],
            filter_probe_input: 0,
            axis_signal_options: [0; 8],
            reference_velocity_pv: [0.0; 8],
            pulse_engine_enabled: 0,
            pulse_generator_type: 0,
            charge_pump_enabled: 0,
            emergency_switch_polarity: 0,
            pulse_engine_activated: 0,
            limit_status_p: 0,
            limit_status_n: 0,
            home_status: 0,
            error_input_status: 0,
            misc_input_status: 0,
            limit_override: 0,
            limit_override_setup: 0,
            pulse_engine_state: 0,
            axis_enabled_mask: 0,
            emergency_input_pin: 0,
            sync_fast_outputs_axis_id: 0,
            sync_fast_outputs_mapping: [0; 8],
            param1: 0,
            param2: 0,
            param3: 0,
            axis_enabled_states_mask: 0,
            pulse_engine_state_setup: 0,
            soft_limit_status: 0,
            external_relay_outputs: 0,
            external_oc_outputs: 0,
            pulse_engine_buffer_size: 0,
            motion_buffer_entries_accepted: 0,
            new_motion_buffer_entries: 0,
            homing_start_mask_setup: 0,
            probe_start_mask_setup: 0,
            probe_input: 0,
            probe_input_polarity: 0,
            probe_status: 0,
            motion_buffer: vec![0; 448],
            probe_speed: 0.0,
            debug_values: [0; 16],
            backlash_width: [0; 8],
            backlash_register: [0; 8],
            backlash_acceleration: [0; 8],
            backlash_compensation_enabled: 0,
            backlash_compensation_max_speed: 0,
            trigger_preparing: 0,
            trigger_prepared: 0,
            trigger_pending: 0,
            trigger_active: 0,
            spindle_speed_estimate: 0,
            spindle_position_error: 0,
            spindle_rpm: 0,
            spindle_index_counter: 0,
            dedicated_limit_n_inputs: 0,
            dedicated_limit_p_inputs: 0,
            dedicated_home_inputs: 0,
            trigger_ignored_axis_mask: 0,
            encoder_index_count: 0,
            encoder_ticks_per_rotation: 0,
            encoder_velocity: 0,
            internal_driver_step_config: [0; 4],
            internal_driver_current_config: [0; 4],
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.pulse_engine_enabled != 0
    }

    pub fn is_activated(&self) -> bool {
        self.pulse_engine_activated != 0
    }

    pub fn get_state(&self) -> PulseEngineState {
        match self.pulse_engine_state {
            0 => PulseEngineState::Stopped,
            1 => PulseEngineState::Internal,
            2 => PulseEngineState::Buffer,
            3 => PulseEngineState::Running,
            10 => PulseEngineState::Jogging,
            11 => PulseEngineState::Stopping,
            20 => PulseEngineState::Home,
            21 => PulseEngineState::Homing,
            30 => PulseEngineState::ProbeComplete,
            31 => PulseEngineState::Probe,
            32 => PulseEngineState::ProbeError,
            40 => PulseEngineState::HybridProbeStopping,
            41 => PulseEngineState::HybridProbeComplete,
            100 => PulseEngineState::StopLimit,
            101 => PulseEngineState::StopEmergency,
            _ => PulseEngineState::Stopped,
        }
    }

    /// Get pulse generator type (bits 0-3)
    pub fn get_generator_type(&self) -> u8 {
        self.pulse_generator_type & 0x0F
    }

    /// Get pulse generator type description
    pub fn get_generator_type_description(&self) -> &'static str {
        match self.get_generator_type() {
            0 => "8ch external",
            1 => "3ch internal",
            2 => "8ch smart external",
            _ => "unknown",
        }
    }

    /// Check if step/dir signals are swapped (bit 6)
    pub fn is_step_dir_swapped(&self) -> bool {
        self.pulse_generator_type & 0x40 != 0
    }

    /// Check if external extended IO features are enabled (bit 7)
    pub fn is_extended_io_enabled(&self) -> bool {
        self.pulse_generator_type & 0x80 != 0
    }

    /// Get axis state for a specific axis
    pub fn get_axis_state(&self, axis: usize) -> PulseEngineAxisState {
        if axis >= 8 {
            return PulseEngineAxisState::Stopped;
        }

        match self.axes_state[axis] {
            0 => PulseEngineAxisState::Stopped,
            1 => PulseEngineAxisState::Ready,
            2 => PulseEngineAxisState::Running,
            8 => PulseEngineAxisState::HomingResetting,
            9 => PulseEngineAxisState::HomingBackingOff,
            10 => PulseEngineAxisState::Home,
            11 => PulseEngineAxisState::HomingStart,
            12 => PulseEngineAxisState::HomingSearch,
            13 => PulseEngineAxisState::HomingBack,
            14 => PulseEngineAxisState::Probed,
            15 => PulseEngineAxisState::ProbeStart,
            16 => PulseEngineAxisState::ProbeSearch,
            20 => PulseEngineAxisState::Error,
            30 => PulseEngineAxisState::Limit,
            _ => PulseEngineAxisState::Stopped,
        }
    }

    pub fn is_axis_enabled(&self, axis: usize) -> bool {
        if axis >= 8 {
            return false;
        }
        (self.axis_enabled_mask & (1 << axis)) != 0
    }

    pub fn is_axis_homed(&self, axis: usize) -> bool {
        if axis >= 8 {
            return false;
        }
        (self.home_status & (1 << axis)) != 0
    }

    pub fn is_limit_triggered(&self, axis: usize, positive: bool) -> bool {
        if axis >= 8 {
            return false;
        }

        if positive {
            (self.limit_status_p & (1 << axis)) != 0
        } else {
            (self.limit_status_n & (1 << axis)) != 0
        }
    }
}

impl Default for PulseEngineV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl PoKeysDevice {
    /// Enable pulse engine
    pub fn enable_pulse_engine(&mut self, enable: bool) -> Result<()> {
        self.pulse_engine_v2.pulse_engine_enabled = if enable { 1 } else { 0 };
        self.send_request(0x80, if enable { 1 } else { 0 }, 0, 0, 0)?;
        Ok(())
    }

    /// Activate pulse engine
    pub fn activate_pulse_engine(&mut self, activate: bool) -> Result<()> {
        self.pulse_engine_v2.pulse_engine_activated = if activate { 1 } else { 0 };
        self.send_request(0x81, if activate { 1 } else { 0 }, 0, 0, 0)?;
        Ok(())
    }

    /// Configure axis
    pub fn configure_axis(
        &mut self,
        axis: usize,
        enabled: bool,
        inverted: bool,
        max_speed: f32,
        max_acceleration: f32,
    ) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        // Set axis configuration
        let mut config = 0u8;
        if enabled {
            config |= 1 << 0;
        }
        if inverted {
            config |= 1 << 1;
        }

        self.pulse_engine_v2.axes_config[axis] = config;
        self.pulse_engine_v2.max_speed[axis] = max_speed;
        self.pulse_engine_v2.max_acceleration[axis] = max_acceleration;
        self.pulse_engine_v2.max_deceleration[axis] = max_acceleration; // Use same value for deceleration

        // Send axis configuration to device
        self.send_request(0x82, axis as u8, config, 0, 0)?;
        Ok(())
    }

    /// Get pulse engine status (0x85/0x00)
    pub fn get_pulse_engine_status(&mut self) -> Result<()> {
        let response = self.send_request(0x85, 0x00, 0, 0, 0)?;

        if response.len() >= 64 {
            // Parse response according to specification
            self.pulse_engine_v2.soft_limit_status = response[3];
            self.pulse_engine_v2.axis_enabled_states_mask = response[4];
            self.pulse_engine_v2.limit_override = response[5];
            // Skip request ID (6) and checksum (7)
            self.pulse_engine_v2.info.nr_of_axes = response[8];
            self.pulse_engine_v2.pulse_engine_activated = response[9];
            self.pulse_engine_v2.pulse_engine_state = response[10];
            self.pulse_engine_v2.charge_pump_enabled = response[11];
            self.pulse_engine_v2.limit_status_p = response[12];
            self.pulse_engine_v2.limit_status_n = response[13];
            self.pulse_engine_v2.home_status = response[14];
            self.pulse_engine_v2.pulse_generator_type = response[15];

            // Parse axes status (bytes 16-23)
            for i in 0..8 {
                if 16 + i < response.len() {
                    self.pulse_engine_v2.axes_state[i] = response[16 + i];
                }
            }

            // Parse axes positions (bytes 24-55, 8x 32-bit LSB first)
            for i in 0..8 {
                let base = 24 + i * 4;
                if base + 3 < response.len() {
                    self.pulse_engine_v2.current_position[i] = i32::from_le_bytes([
                        response[base],
                        response[base + 1],
                        response[base + 2],
                        response[base + 3],
                    ]);
                }
            }

            // Parse remaining fields
            if response.len() >= 63 {
                self.pulse_engine_v2.info.max_pulse_frequency = response[57];
                self.pulse_engine_v2.info.buffer_depth = response[58];
                self.pulse_engine_v2.info.slot_timing = response[59];
                self.pulse_engine_v2.emergency_switch_polarity = response[60];
                self.pulse_engine_v2.error_input_status = response[61];
                self.pulse_engine_v2.misc_input_status = response[62];
            }
        }

        Ok(())
    }

    /// Set axis position
    pub fn set_axis_position(&mut self, axis: usize, position: i32) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        self.pulse_engine_v2.position_setup[axis] = position;

        self.send_request(
            0x83,
            axis as u8,
            (position & 0xFF) as u8,
            ((position >> 8) & 0xFF) as u8,
            ((position >> 16) & 0xFF) as u8,
        )?;

        Ok(())
    }

    /// Get axis position
    pub fn get_axis_position(&mut self, axis: usize) -> Result<i32> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        // Read pulse engine status
        self.read_pulse_engine_status()?;

        Ok(self.pulse_engine_v2.current_position[axis])
    }

    /// Move axis to position
    pub fn move_axis_to_position(&mut self, axis: usize, position: i32, speed: f32) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        self.pulse_engine_v2.reference_position_speed[axis] = position;
        self.pulse_engine_v2.reference_velocity_pv[axis] = speed;

        self.send_request(
            0x84,
            axis as u8,
            (position & 0xFF) as u8,
            ((position >> 8) & 0xFF) as u8,
            ((position >> 16) & 0xFF) as u8,
        )?;

        Ok(())
    }

    /// Start axis homing
    pub fn start_axis_homing(&mut self, axis_mask: u8) -> Result<()> {
        self.pulse_engine_v2.homing_start_mask_setup = axis_mask;
        self.send_request(0x85, axis_mask, 0, 0, 0)?;
        Ok(())
    }

    /// Configure axis homing
    pub fn configure_axis_homing(
        &mut self,
        axis: usize,
        home_pin: u8,
        homing_speed: u8,
        home_offset: i32,
    ) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        self.pulse_engine_v2.pin_home_switch[axis] = home_pin;
        self.pulse_engine_v2.homing_speed[axis] = homing_speed;
        self.pulse_engine_v2.home_offsets[axis] = home_offset;

        self.send_request(0x86, axis as u8, home_pin, homing_speed, 0)?;
        Ok(())
    }

    /// Configure axis limits
    pub fn configure_axis_limits(
        &mut self,
        axis: usize,
        limit_n_pin: u8,
        limit_p_pin: u8,
        soft_limit_min: i32,
        soft_limit_max: i32,
    ) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        self.pulse_engine_v2.pin_limit_m_switch[axis] = limit_n_pin;
        self.pulse_engine_v2.pin_limit_p_switch[axis] = limit_p_pin;
        self.pulse_engine_v2.soft_limit_minimum[axis] = soft_limit_min;
        self.pulse_engine_v2.soft_limit_maximum[axis] = soft_limit_max;

        self.send_request(0x87, axis as u8, limit_n_pin, limit_p_pin, 0)?;
        Ok(())
    }

    /// Emergency stop
    pub fn emergency_stop(&mut self) -> Result<()> {
        self.send_request(0x88, 0, 0, 0, 0)?;
        Ok(())
    }

    /// Read pulse engine status
    pub fn read_pulse_engine_status(&mut self) -> Result<()> {
        let response = self.send_request(0x89, 0, 0, 0, 0)?;

        // Parse pulse engine status from response
        if response.len() >= 64 {
            self.pulse_engine_v2.pulse_engine_state = response[8];
            self.pulse_engine_v2.axis_enabled_mask = response[9];
            self.pulse_engine_v2.limit_status_p = response[10];
            self.pulse_engine_v2.limit_status_n = response[11];
            self.pulse_engine_v2.home_status = response[12];

            // Parse axis positions
            for i in 0..8 {
                let pos_index = 16 + (i * 4);
                if pos_index + 3 < response.len() {
                    self.pulse_engine_v2.current_position[i] = i32::from_le_bytes([
                        response[pos_index],
                        response[pos_index + 1],
                        response[pos_index + 2],
                        response[pos_index + 3],
                    ]);
                }
            }
        }

        Ok(())
    }

    /// Get pulse engine state
    pub fn get_pulse_engine_state(&mut self) -> Result<PulseEngineState> {
        self.read_pulse_engine_status()?;
        Ok(self.pulse_engine_v2.get_state())
    }

    /// Get axis state
    pub fn get_axis_state(&mut self, axis: usize) -> Result<PulseEngineAxisState> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        self.read_pulse_engine_status()?;
        Ok(self.pulse_engine_v2.get_axis_state(axis))
    }

    /// Check if axis is homed
    pub fn is_axis_homed(&mut self, axis: usize) -> Result<bool> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        self.read_pulse_engine_status()?;
        Ok(self.pulse_engine_v2.is_axis_homed(axis))
    }

    /// Wait for axis to complete movement
    pub fn wait_for_axis(&mut self, axis: usize, timeout_ms: u32) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Invalid axis number".to_string()));
        }

        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms as u64);

        loop {
            let state = self.get_axis_state(axis)?;

            match state {
                PulseEngineAxisState::Ready | PulseEngineAxisState::Stopped => break,
                PulseEngineAxisState::Error | PulseEngineAxisState::Limit => {
                    return Err(PoKeysError::Protocol(
                        "Axis error or limit reached".to_string(),
                    ));
                }
                _ => {
                    if start_time.elapsed() > timeout {
                        return Err(PoKeysError::Timeout);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_engine_creation() {
        let pe = PulseEngineV2::new();
        assert!(!pe.is_enabled());
        assert!(!pe.is_activated());
        assert_eq!(pe.get_state(), PulseEngineState::Stopped);
    }

    #[test]
    fn test_axis_state_conversion() {
        let mut pe = PulseEngineV2::new();

        pe.axes_state[0] = 1;
        assert_eq!(pe.get_axis_state(0), PulseEngineAxisState::Ready);

        pe.axes_state[0] = 2;
        assert_eq!(pe.get_axis_state(0), PulseEngineAxisState::Running);

        pe.axes_state[0] = 20;
        assert_eq!(pe.get_axis_state(0), PulseEngineAxisState::Error);
    }

    #[test]
    fn test_axis_enable_mask() {
        let mut pe = PulseEngineV2::new();

        assert!(!pe.is_axis_enabled(0));

        pe.axis_enabled_mask = 0b00000001;
        assert!(pe.is_axis_enabled(0));
        assert!(!pe.is_axis_enabled(1));

        pe.axis_enabled_mask = 0b00000011;
        assert!(pe.is_axis_enabled(0));
        assert!(pe.is_axis_enabled(1));
        assert!(!pe.is_axis_enabled(2));
    }
}
