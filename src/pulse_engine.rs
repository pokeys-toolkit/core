//! Pulse Engine v2 support for motion control

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use crate::types::{PulseEngineAxisState, PulseEngineState};
use serde::{Deserialize, Serialize};

/// Motor driver step setting constants
pub mod step_setting {
    pub const FULL_STEP: u8 = 0; // 1/1
    pub const HALF_NON_CIRCULAR: u8 = 1; // 1/2 non-circular
    pub const HALF_STEP: u8 = 2; // 1/2
    pub const QUARTER_STEP: u8 = 3; // 1/4
    pub const EIGHTH_STEP: u8 = 4; // 1/8
    pub const SIXTEENTH_STEP: u8 = 5; // 1/16
    pub const THIRTY_SECOND_STEP: u8 = 6; // 1/32
    pub const STEP_128: u8 = 7; // 1/128
    pub const STEP_256: u8 = 8; // 1/256
}

/// Pulse engine power states (bit-mapped)
pub struct PulseEnginePowerState;

impl PulseEnginePowerState {
    // States with enabled power (bit-mapped)
    pub const PE_STOPPED: u8 = 1 << 0;
    pub const PE_STOP_LIMIT: u8 = 1 << 1;
    pub const PE_STOP_EMERGENCY: u8 = 1 << 2;

    // States with enabled charge pump (bit-mapped)
    pub const PE_STOPPED_CHARGE_PUMP: u8 = 1 << 4;
    pub const PE_STOP_LIMIT_CHARGE_PUMP: u8 = 1 << 5;
    pub const PE_STOP_EMERGENCY_CHARGE_PUMP: u8 = 1 << 6;

    // Common combinations
    pub const ALL_POWER_ENABLED: u8 =
        Self::PE_STOPPED | Self::PE_STOP_LIMIT | Self::PE_STOP_EMERGENCY;
    pub const ALL_CHARGE_PUMP_ENABLED: u8 = Self::PE_STOPPED_CHARGE_PUMP
        | Self::PE_STOP_LIMIT_CHARGE_PUMP
        | Self::PE_STOP_EMERGENCY_CHARGE_PUMP;
}

/// Pulse engine setup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseEngineConfig {
    pub enabled_axes: u8,
    pub charge_pump_enabled: u8,
    pub generator_type: u8,
    pub buffer_size: u8,
    pub emergency_switch_polarity: u8,
    pub power_states: u8,
}

impl PulseEngineConfig {
    /// Create builder for 3-channel internal generator
    pub fn three_channel_internal(axes: u8, swap_step_dir: bool) -> PulseEngineConfigBuilder {
        PulseEngineConfigBuilder {
            enabled_axes: axes,
            charge_pump_enabled: 0,
            generator_type: if swap_step_dir { 0x41 } else { 0x01 },
            buffer_size: 0,
            emergency_switch_polarity: 1,
            power_states: PulseEnginePowerState::ALL_POWER_ENABLED,
        }
    }

    /// Create configuration for 8-channel external generator
    pub fn eight_channel_external(axes: u8) -> Self {
        Self {
            enabled_axes: axes,
            charge_pump_enabled: 0,
            generator_type: 0, // 8ch external
            buffer_size: 0,    // default
            emergency_switch_polarity: 1,
            power_states: PulseEnginePowerState::ALL_POWER_ENABLED,
        }
    }
}

pub struct PulseEngineConfigBuilder {
    enabled_axes: u8,
    charge_pump_enabled: u8,
    generator_type: u8,
    buffer_size: u8,
    emergency_switch_polarity: u8,
    power_states: u8,
}

impl PulseEngineConfigBuilder {
    pub fn charge_pump_enabled(mut self, enabled: u8) -> Self {
        self.charge_pump_enabled = enabled;
        self
    }

    pub fn buffer_size(mut self, size: u8) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn emergency_switch_polarity(mut self, polarity: u8) -> Self {
        self.emergency_switch_polarity = polarity;
        self
    }

    pub fn power_states(mut self, states: u8) -> Self {
        self.power_states = states;
        self
    }

    pub fn build(self) -> PulseEngineConfig {
        PulseEngineConfig {
            enabled_axes: self.enabled_axes,
            charge_pump_enabled: self.charge_pump_enabled,
            generator_type: self.generator_type,
            buffer_size: self.buffer_size,
            emergency_switch_polarity: self.emergency_switch_polarity,
            power_states: self.power_states,
        }
    }
}

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
    pub motor_step_setting: [u8; 4],
    pub motor_current_setting: [u8; 4],
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
            motor_step_setting: [0; 4],
            motor_current_setting: [0; 4],
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

    /// Check if axis has positive limit switch triggered
    pub fn is_axis_limit_positive(&self, axis: usize) -> bool {
        if axis >= 8 {
            return false;
        }
        self.limit_status_p & (1 << axis) != 0
    }

    /// Check if axis has negative limit switch triggered
    pub fn is_axis_limit_negative(&self, axis: usize) -> bool {
        if axis >= 8 {
            return false;
        }
        self.limit_status_n & (1 << axis) != 0
    }

    /// Check if axis is at home position
    pub fn is_axis_home(&self, axis: usize) -> bool {
        if axis >= 8 {
            return false;
        }
        self.home_status & (1 << axis) != 0
    }

    /// Check if axis has soft limit triggered
    pub fn is_axis_soft_limit(&self, axis: usize) -> bool {
        if axis >= 8 {
            return false;
        }
        self.soft_limit_status & (1 << axis) != 0
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

    /// Setup pulse engine (0x85/0x01)
    pub fn setup_pulse_engine(&mut self, config: &PulseEngineConfig) -> Result<()> {
        let mut request = vec![0u8; 56]; // Only data payload (protocol bytes 9-64)

        // Build request according to specification
        request[0] = config.enabled_axes; // Protocol byte 9: Number of enabled axes
        request[1] = config.charge_pump_enabled; // Protocol byte 10: Safety charge pump
        request[2] = config.generator_type & 0x7F; // Protocol byte 11: Generator configuration (ensure bit 7 = 0)
        request[3] = config.buffer_size; // Protocol byte 12: Motion buffer size
        request[4] = config.emergency_switch_polarity; // Protocol byte 13: Emergency switch polarity

        // Protocol byte 14: States with enabled power and charge pump
        let mut power_states = config.power_states & 0x07; // Power states (bits 0-2)

        // Charge pump states (bits 4-6)
        if config.charge_pump_enabled != 0 {
            power_states |= 0x10; // peSTOPPED charge pump
            power_states |= 0x20; // peSTOP_LIMIT charge pump
            power_states |= 0x40; // peSTOP_EMERGENCY charge pump
        }

        request[5] = power_states; // Protocol byte 14: Power and charge pump states

        // Protocol bytes 15-63 are reserved (already initialized to 0)

        self.send_request_with_data(0x85, 0x01, 0, 0, 0, &request)?;
        Ok(())
    }

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

    /// Set axis position (0x85/0x03)
    pub fn set_axis_position(&mut self, axis: usize, position: i32) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Axis index must be 0-7".to_string()));
        }

        let mut request = vec![0u8; 56]; // Only data payload (protocol bytes 9-64)

        // Protocol bytes 9-40: Axis positions (32-bit LSB first)
        // Each axis position is 4 bytes, so axis N starts at byte N*4
        let pos_bytes = position.to_le_bytes();
        let start_idx = axis * 4;
        request[start_idx..start_idx + 4].copy_from_slice(&pos_bytes);

        // Protocol byte 4: Axis position setup selection (bit-mapped)
        let axis_mask = 1u8 << axis;

        // Protocol bytes 41-63 are reserved (already initialized to 0)

        self.send_request_with_data(0x85, 0x03, axis_mask, 0, 0, &request)?;
        Ok(())
    }

    /// Set pulse engine state (0x85/0x02)
    pub fn set_pulse_engine_state(
        &mut self,
        state: u8,
        limit_override: u8,
        output_enable_mask: u8,
    ) -> Result<()> {
        let request = vec![0u8; 55]; // Reserved bytes 9-63
        self.send_request_with_data(
            0x85,
            0x02,
            state,
            limit_override,
            output_enable_mask,
            &request,
        )?;
        Ok(())
    }

    /// Reboot pulse engine (0x85/0x05)
    pub fn reboot_pulse_engine(&mut self) -> Result<()> {
        let request = vec![0u8; 55]; // Reserved bytes 9-63
        self.send_request_with_data(0x85, 0x05, 0, 0, 0, &request)?;
        Ok(())
    }

    /// Set axis configuration (0x85/0x11)
    pub fn set_axis_configuration(&mut self, axis: usize) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Axis index must be 0-7".to_string()));
        }

        let mut request = vec![0u8; 56]; // Data payload (protocol bytes 9-64)

        // Byte 9: Axis options - Enable axis with internal planner, soft limits, and masked enable
        let mut axis_options = 0x05; // aoENABLED | aoINTERNAL_PLANNER
        axis_options |= 1 << 6; // aoENABLED_MASKED - required for output enable mask control
        if self.pulse_engine_v2.soft_limit_minimum[axis] != 0
            || self.pulse_engine_v2.soft_limit_maximum[axis] != 0
        {
            axis_options |= 1 << 5; // aoSOFT_LIMIT_ENABLED
        }
        request[0] = axis_options;

        // Byte 10: Axis switch options
        request[1] = 0x00; // No switches

        // Bytes 11-13: Switch pins (0 for external)
        request[2] = 0; // Home switch pin
        request[3] = 0; // Limit- switch pin  
        request[4] = 0; // Limit+ switch pin

        // Bytes 14-15: Homing speeds
        request[5] = 50; // Homing speed (50% of max)
        request[6] = 10; // Homing return speed (10% of homing)

        // Byte 16: MPG jog encoder ID
        request[7] = 0;

        // Bytes 17-20: Maximum speed (32-bit float)
        let speed_bytes = self.pulse_engine_v2.max_speed[axis].to_le_bytes();
        request[8..12].copy_from_slice(&speed_bytes);

        // Bytes 21-24: Maximum acceleration (32-bit float)
        let accel_bytes = self.pulse_engine_v2.max_acceleration[axis].to_le_bytes();
        request[12..16].copy_from_slice(&accel_bytes);

        // Bytes 25-28: Maximum deceleration (32-bit float)
        let decel_bytes = self.pulse_engine_v2.max_deceleration[axis].to_le_bytes();
        request[16..20].copy_from_slice(&decel_bytes);

        // Bytes 29-32: Soft-limit minimum position
        let min_bytes = self.pulse_engine_v2.soft_limit_minimum[axis].to_le_bytes();
        request[20..24].copy_from_slice(&min_bytes);

        // Bytes 33-36: Soft-limit maximum position
        let max_bytes = self.pulse_engine_v2.soft_limit_maximum[axis].to_le_bytes();
        request[24..28].copy_from_slice(&max_bytes);

        // Bytes 37-38: MPG jog multiplier
        request[28] = 1;
        request[29] = 0;

        // Byte 39: Axis enable output pin (0 for external)
        request[30] = 0;

        // Byte 40: Invert axis enable signal
        request[31] = 0;

        // Bytes 41-43: Filter settings
        request[32] = 0; // Limit- filter
        request[33] = 0; // Limit+ filter
        request[34] = 0; // Home filter

        // Byte 44: Home algorithm
        request[35] = 0x83; // Default algorithm

        // Bytes 46-49: Home back-off distance
        let backoff = 0i32;
        let backoff_bytes = backoff.to_le_bytes();
        request[37..41].copy_from_slice(&backoff_bytes);

        // Bytes 50-51: MPG encoder divider
        request[41] = 1;
        request[42] = 0;

        // Byte 52: Additional misc options
        request[43] = 0x00; // No inversion

        // Byte 53: Probe filter
        request[44] = 0;

        // Bytes 54-63: Reserved (already 0)

        // Send the complete configuration using send_request_with_data
        self.send_request_with_data(0x85, 0x11, axis as u8, 0, 0, &request)?;
        Ok(())
    }

    /// Get axis configuration (0x85/0x10)
    pub fn get_axis_configuration(&mut self, axis: usize) -> Result<()> {
        if axis >= 8 {
            return Err(PoKeysError::Parameter("Axis index must be 0-7".to_string()));
        }

        let response = self.send_request(0x85, 0x10, axis as u8, 0, 0)?;

        // Parse response according to specification
        if response.len() >= 64 {
            // Byte 9: Axis options
            self.pulse_engine_v2.axes_config[axis] = response[8];

            // Byte 10: Axis switch options
            self.pulse_engine_v2.axes_switch_config[axis] = response[9];

            // Byte 14: Homing speed
            self.pulse_engine_v2.homing_speed[axis] = response[13];

            // Byte 15: Homing return speed
            self.pulse_engine_v2.homing_return_speed[axis] = response[14];

            // Byte 16: MPG jog encoder ID
            self.pulse_engine_v2.mpg_jog_encoder[axis] = response[15];

            // Bytes 17-20: Maximum speed (32-bit float)
            let speed_bytes = [response[16], response[17], response[18], response[19]];
            self.pulse_engine_v2.max_speed[axis] = f32::from_le_bytes(speed_bytes);

            // Bytes 21-24: Maximum acceleration (32-bit float)
            let accel_bytes = [response[20], response[21], response[22], response[23]];
            self.pulse_engine_v2.max_acceleration[axis] = f32::from_le_bytes(accel_bytes);

            // Bytes 25-28: Maximum deceleration (32-bit float)
            let decel_bytes = [response[24], response[25], response[26], response[27]];
            self.pulse_engine_v2.max_deceleration[axis] = f32::from_le_bytes(decel_bytes);

            // Bytes 29-32: Soft-limit minimum position (32-bit int)
            let min_pos_bytes = [response[28], response[29], response[30], response[31]];
            self.pulse_engine_v2.soft_limit_minimum[axis] = i32::from_le_bytes(min_pos_bytes);

            // Bytes 33-36: Soft-limit maximum position (32-bit int)
            let max_pos_bytes = [response[32], response[33], response[34], response[35]];
            self.pulse_engine_v2.soft_limit_maximum[axis] = i32::from_le_bytes(max_pos_bytes);
        }

        Ok(())
    }
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

        // Use set_axis_position which properly implements 0x85/0x03
        self.set_axis_position(axis, position)?;

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

    /// Set internal motor drivers configuration (0x85/0x19)
    pub fn set_motor_drivers_configuration(&mut self) -> Result<()> {
        let mut request = vec![0u8; 56]; // Data payload (protocol bytes 9-64)

        // Set motor driver settings for each axis
        for axis in 0..4 {
            let byte_offset = axis * 2; // Bytes 0-7 for axes 1-4
            request[byte_offset] = self.pulse_engine_v2.motor_step_setting[axis];
            request[byte_offset + 1] = self.pulse_engine_v2.motor_current_setting[axis];
        }

        let _response = self.send_request_with_data(0x85, 0x19, 0, 0, 0, &request)?;

        // Don't parse response - the device echoes back what we sent
        // The values are already set in the device struct

        Ok(())
    }

    /// Create axis configuration builder
    pub fn configure_axis(&mut self, axis: usize) -> AxisConfigBuilder {
        AxisConfigBuilder::new(axis)
    }

    /// Set axis positions (0x85/0x03)
    pub fn set_axis_positions(&mut self, axis_mask: u8, positions: &[i32; 8]) -> Result<()> {
        let mut request = vec![0u8; 56]; // Data payload (protocol bytes 9-64)

        // Bytes 0-31: Axis positions (8x 32-bit integers, LSB first)
        for (i, &position) in positions.iter().enumerate() {
            let pos_bytes = position.to_le_bytes();
            let offset = i * 4;
            request[offset..offset + 4].copy_from_slice(&pos_bytes);
        }

        self.send_request_with_data(0x85, 0x03, axis_mask, 0, 0, &request)?;
        Ok(())
    }

    /// Move PV (Set reference position and speed) (0x85/0x25)
    pub fn move_pv(&mut self, axis_mask: u8, positions: &[i32; 8], velocity: u16) -> Result<()> {
        let mut request = vec![0u8; 56]; // Data payload (protocol bytes 9-64)

        // Bytes 0-31: Reference positions (8x 32-bit integers, LSB first)
        for (i, &position) in positions.iter().enumerate() {
            let pos_bytes = position.to_le_bytes();
            let offset = i * 4;
            request[offset..offset + 4].copy_from_slice(&pos_bytes);
        }

        // Bytes 32-47: Move velocity (16 bytes, but only first 2 used)
        let vel_bytes = velocity.to_le_bytes();
        request[32..34].copy_from_slice(&vel_bytes);

        self.send_request_with_data(0x85, 0x25, axis_mask, 0, 0, &request)?;
        Ok(())
    }

    /// Create motor driver configuration builder
    pub fn configure_motor_drivers(&mut self) -> MotorDriverConfigBuilder {
        MotorDriverConfigBuilder::new()
    }

    /// Get internal motor drivers configuration (0x85/0x18)
    pub fn get_motor_drivers_configuration(&mut self) -> Result<()> {
        let response = self.send_request(0x85, 0x18, 0, 0, 0)?;

        // Parse motor driver settings for each axis
        for axis in 0..4 {
            let byte_offset = 8 + (axis * 2); // Bytes 9-16 for axes 1-4
            if byte_offset + 1 < response.len() {
                let step_setting = response[byte_offset];
                let current_setting = response[byte_offset + 1];

                self.pulse_engine_v2.motor_step_setting[axis] = step_setting;
                self.pulse_engine_v2.motor_current_setting[axis] = current_setting;
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

/// Axis configuration builder
pub struct AxisConfigBuilder {
    axis: usize,
    max_speed: u32,
    max_acceleration: u32,
    max_deceleration: u32,
    soft_limit_min: i32,
    soft_limit_max: i32,
}

impl AxisConfigBuilder {
    pub fn new(axis: usize) -> Self {
        Self {
            axis,
            max_speed: 1000,
            max_acceleration: 100,
            max_deceleration: 100,
            soft_limit_min: 0,
            soft_limit_max: 0,
        }
    }

    pub fn max_speed(mut self, speed: u32) -> Self {
        self.max_speed = speed;
        self
    }

    pub fn max_acceleration(mut self, acceleration: u32) -> Self {
        self.max_acceleration = acceleration;
        self
    }

    pub fn max_deceleration(mut self, deceleration: u32) -> Self {
        self.max_deceleration = deceleration;
        self
    }

    pub fn soft_limit_min(mut self, min: i32) -> Self {
        self.soft_limit_min = min;
        self
    }

    pub fn soft_limit_max(mut self, max: i32) -> Self {
        self.soft_limit_max = max;
        self
    }

    pub fn build(self, device: &mut PoKeysDevice) -> Result<()> {
        if self.axis >= 8 {
            return Err(PoKeysError::Parameter("Axis index must be 0-7".to_string()));
        }

        // Convert from pulses/second to timeslot units (divide by 1000)
        device.pulse_engine_v2.max_speed[self.axis] = (self.max_speed as f32) / 1000.0;
        // Convert from pulses/s^2 to timeslot units (divide by 1000000 = 1000^2)
        device.pulse_engine_v2.max_acceleration[self.axis] =
            (self.max_acceleration as f32) / 1000000.0;
        device.pulse_engine_v2.max_deceleration[self.axis] =
            (self.max_deceleration as f32) / 1000000.0;
        device.pulse_engine_v2.soft_limit_minimum[self.axis] = self.soft_limit_min;
        device.pulse_engine_v2.soft_limit_maximum[self.axis] = self.soft_limit_max;

        device.set_axis_configuration(self.axis)
    }
}

/// Motor driver configuration builder
pub struct MotorDriverConfigBuilder {
    step_settings: [Option<u8>; 4],
    current_settings: [Option<u8>; 4],
}

impl MotorDriverConfigBuilder {
    pub fn new() -> Self {
        Self {
            step_settings: [None; 4],
            current_settings: [None; 4],
        }
    }

    pub fn axis_step_setting(mut self, axis: usize, step_setting: u8) -> Self {
        if axis < 4 {
            self.step_settings[axis] = Some(step_setting);
        }
        self
    }

    pub fn axis_current_setting(mut self, axis: usize, current_setting: u8) -> Self {
        if axis < 4 {
            self.current_settings[axis] = Some(current_setting);
        }
        self
    }

    pub fn build(self, device: &mut PoKeysDevice) -> Result<()> {
        // Only update the values that were explicitly set
        for axis in 0..4 {
            if let Some(step_setting) = self.step_settings[axis] {
                device.pulse_engine_v2.motor_step_setting[axis] = step_setting;
            }
            if let Some(current_setting) = self.current_settings[axis] {
                device.pulse_engine_v2.motor_current_setting[axis] = current_setting;
            }
        }
        device.set_motor_drivers_configuration()
    }
}
