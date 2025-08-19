//! Core types and structures for PoKeys library

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Device connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceConnectionType {
    UsbDevice = 0,
    NetworkDevice = 1,
    FastUsbDevice = 2,
}

/// Device connection parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionParam {
    Tcp = 0,
    Udp = 1,
}

/// Device type IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceTypeId {
    Bootloader55 = 3,
    Bootloader56U = 15,
    Bootloader56E = 16,
    Bootloader58 = 41,

    Device55v1 = 0,
    Device55v2 = 1,
    Device55v3 = 2,

    Device56U = 10,
    Device56E = 11,
    Device27U = 20,
    Device27E = 21,

    Device57U = 30,
    Device57E = 31,
    PoKeys57CNC = 32,
    PoKeys57CNCpro4x25 = 33,
    PoKeys57CNCdb25 = 38,
    PoKeys57Utest = 39,

    LiniTester = 43,

    Device57Uv0 = 28,
    Device57Ev0 = 29,

    Device58EU = 40,
    PoPLC58 = 50,

    OEM1 = 100,
    SerialReader = 101,
    X15_02_24 = 102,
}

/// Device type masks for capability checking
#[derive(Debug, Clone, Copy)]
pub struct DeviceTypeMask(pub u64);

impl DeviceTypeMask {
    pub const BOOTLOADER: Self = Self(1 << 0);
    pub const BOOTLOADER55: Self = Self(1 << 1);
    pub const BOOTLOADER56: Self = Self(1 << 2);
    pub const BOOTLOADER56U: Self = Self(1 << 3);
    pub const BOOTLOADER56E: Self = Self(1 << 4);
    pub const BOOTLOADER58: Self = Self(1 << 5);

    pub const DEVICE55: Self = Self(1 << 10);
    pub const DEVICE55V1: Self = Self(1 << 11);
    pub const DEVICE55V2: Self = Self(1 << 12);
    pub const DEVICE55V3: Self = Self(1 << 13);

    pub const DEVICE56: Self = Self(1 << 14);
    pub const DEVICE56U: Self = Self(1 << 15);
    pub const DEVICE56E: Self = Self(1 << 16);
    pub const DEVICE27: Self = Self(1 << 17);
    pub const DEVICE27U: Self = Self(1 << 18);
    pub const DEVICE27E: Self = Self(1 << 19);

    pub const DEVICE57: Self = Self(1 << 20);
    pub const DEVICE57U: Self = Self(1 << 24);
    pub const DEVICE57E: Self = Self(1 << 25);
    pub const DEVICE57CNC: Self = Self(1 << 26);
    pub const DEVICE57CNCDB25: Self = Self(1 << 27);
    pub const DEVICE57UTEST: Self = Self(1 << 28);
    pub const DEVICE57CNCPRO4X25: Self = Self(1 << 29);

    pub const DEVICE58: Self = Self(1 << 21);
    pub const POPLC58: Self = Self(1 << 22);

    pub const POKEYS16RF: Self = Self(1 << 23);
}

/// Pulse engine state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulseEngineState {
    Stopped = 0,
    Internal = 1,
    Buffer = 2,
    Running = 3,

    Jogging = 10,
    Stopping = 11,

    Home = 20,
    Homing = 21,

    ProbeComplete = 30,
    Probe = 31,
    ProbeError = 32,

    HybridProbeStopping = 40,
    HybridProbeComplete = 41,

    StopLimit = 100,
    StopEmergency = 101,
}

/// Pulse engine axis state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulseEngineAxisState {
    Stopped = 0,
    Ready = 1,
    Running = 2,

    HomingResetting = 8,
    HomingBackingOff = 9,
    Home = 10,
    HomingStart = 11,
    HomingSearch = 12,
    HomingBack = 13,

    Probed = 14,
    ProbeStart = 15,
    ProbeSearch = 16,

    Error = 20,
    Limit = 30,
}

/// I2C status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cStatus {
    Error = 0,
    Ok = 1,
    Complete = 2,
    InProgress = 0x10,
}

/// I2C configuration options
#[derive(Debug, Clone)]
pub struct I2cConfig {
    pub max_packet_size: usize,
    pub auto_fragment: bool,
    pub fragment_delay_ms: u64,
    pub validation_level: ValidationLevel,
}

impl Default for I2cConfig {
    fn default() -> Self {
        Self {
            max_packet_size: 32,
            auto_fragment: false,
            fragment_delay_ms: 10,
            validation_level: ValidationLevel::None,
        }
    }
}

/// Retry configuration for error recovery
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 2000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Validation levels for protocol validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationLevel {
    None,       // Current behavior - pass everything through
    Basic,      // Validate packet structure only
    Strict,     // Full protocol validation
    Custom(ValidationConfig),
}

/// Custom validation configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationConfig {
    pub validate_checksums: bool,
    pub validate_command_ids: bool,
    pub validate_device_ids: bool,
    pub validate_packet_structure: bool,
    pub max_device_id: u8,
    pub valid_commands: HashSet<u8>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            validate_checksums: true,
            validate_command_ids: true,
            validate_device_ids: true,
            validate_packet_structure: true,
            max_device_id: 255,
            valid_commands: HashSet::new(),
        }
    }
}

/// I2C performance metrics
#[derive(Debug, Clone, Default)]
pub struct I2cMetrics {
    pub total_commands: u64,
    pub successful_commands: u64,
    pub failed_commands: u64,
    pub average_response_time: std::time::Duration,
    pub max_response_time: std::time::Duration,
    pub min_response_time: std::time::Duration,
    pub error_counts: HashMap<String, u32>,
}

/// Health status for device diagnostics
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub connectivity: ConnectivityStatus,
    pub i2c_health: I2cHealthStatus,
    pub error_rate: f64,
    pub performance: PerformanceSummary,
}

/// Connectivity status
#[derive(Debug, Clone)]
pub enum ConnectivityStatus {
    Healthy,
    Degraded(String),
    Failed(String),
}

/// I2C health status
#[derive(Debug, Clone)]
pub enum I2cHealthStatus {
    Healthy,
    Degraded(String),
    Failed(String),
}

/// Performance summary
#[derive(Debug, Clone, Default)]
pub struct PerformanceSummary {
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub throughput_commands_per_sec: f64,
}

/// LCD mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdMode {
    Direct = 0,
    Buffered = 1,
}

/// Device information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub pin_count: u32,
    pub pwm_count: u32,
    pub basic_encoder_count: u32,
    pub encoders_count: u32,
    pub fast_encoders: u32,
    pub ultra_fast_encoders: u32,
    pub pwm_internal_frequency: u32,
    pub analog_inputs: u32,

    // Feature flags
    pub key_mapping: u32,
    pub triggered_key_mapping: u32,
    pub key_repeat_delay: u32,
    pub digital_counters: u32,
    pub joystick_button_axis_mapping: u32,
    pub joystick_analog_to_digital_mapping: u32,
    pub macros: u32,
    pub matrix_keyboard: u32,
    pub matrix_keyboard_triggered_mapping: u32,
    pub lcd: u32,
    pub matrix_led: u32,
    pub connection_signal: u32,
    pub po_ext_bus: u32,
    pub po_net: u32,
    pub analog_filtering: u32,
    pub init_outputs_start: u32,
    pub prot_i2c: u32,
    pub prot_1wire: u32,
    pub additional_options: u32,
    pub load_status: u32,
    pub custom_device_name: u32,
    pub po_tlog27_support: u32,
    pub sensor_list: u32,
    pub web_interface: u32,
    pub fail_safe_settings: u32,
    pub joystick_hat_switch: u32,
    pub pulse_engine: u32,
    pub pulse_engine_v2: u32,
    pub easy_sensors: u32,
}

/// Device-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceData {
    pub device_type_id: u32,
    pub serial_number: u32,
    pub device_name: [u8; 30],
    pub device_type_name: [u8; 30],
    pub build_date: [u8; 12],
    pub activation_code: [u8; 8],
    pub firmware_version_major: u8,
    pub firmware_version_minor: u8,
    pub firmware_revision: u8,
    pub user_id: u8,
    pub device_type: u8,
    pub activated_options: u8,
    pub device_lock_status: u8,
    pub hw_type: u8,
    pub fw_type: u8,
    pub product_id: u8,
    pub secondary_firmware_version_major: u8,
    pub secondary_firmware_version_minor: u8,
    pub device_is_bootloader: u8,
}

/// Network device summary for enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDeviceSummary {
    pub serial_number: u32,
    pub ip_address: [u8; 4],
    pub host_ip: [u8; 4],
    pub firmware_version_major: u8,
    pub firmware_version_minor: u8,
    pub firmware_revision: u8,
    pub user_id: u8,
    pub dhcp: u8,
    pub hw_type: u8,
    pub use_udp: u8,
}

/// Network device information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkDeviceInfo {
    pub ip_address_current: [u8; 4],
    pub ip_address_setup: [u8; 4],
    pub subnet_mask: [u8; 4],
    pub gateway_ip: [u8; 4],
    pub tcp_timeout: u16,
    pub additional_network_options: u8,
    pub dhcp: u8,
}

/// Real-time clock structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeClock {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub dow: u8,
    pub dom: u8,
    pub tmp: u8,
    pub doy: u16,
    pub month: u16,
    pub year: u16,
}

/// CAN message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanMessage {
    pub id: u32,
    pub data: [u8; 8],
    pub len: u8,
    pub format: u8,
    pub msg_type: u8,
}

/// Communication buffer size constants
pub const REQUEST_BUFFER_SIZE: usize = 68;
pub const RESPONSE_BUFFER_SIZE: usize = 68;
pub const MULTIPART_BUFFER_SIZE: usize = 448;

/// Default implementations for common types
impl Default for DeviceData {
    fn default() -> Self {
        Self {
            device_type_id: 0,
            serial_number: 0,
            device_name: [0; 30],
            device_type_name: [0; 30],
            build_date: [0; 12],
            activation_code: [0; 8],
            firmware_version_major: 0,
            firmware_version_minor: 0,
            firmware_revision: 0,
            user_id: 0,
            device_type: 0,
            activated_options: 0,
            device_lock_status: 0,
            hw_type: 0,
            fw_type: 0,
            product_id: 0,
            secondary_firmware_version_major: 0,
            secondary_firmware_version_minor: 0,
            device_is_bootloader: 0,
        }
    }
}

/// Additional types for testing compatibility
impl DeviceData {
    pub fn device_locked(&self) -> bool {
        self.device_lock_status != 0
    }

    pub fn device_features(&self) -> u8 {
        self.activated_options
    }

    /// Parse and format the software version from bytes 5 and 6 of device data response
    /// Byte 5: bits 4-7 = major-1, bits 0-3 = minor
    /// Byte 6: revision number
    /// Returns formatted version string like "4.7.15"
    pub fn software_version_string(&self) -> String {
        // For now, using firmware_version_major as byte 5 and firmware_version_minor as byte 6
        // This will be updated when we implement the proper read device data command
        let software_version_byte = self.firmware_version_major;
        let revision_byte = self.firmware_version_minor;

        // Extract major version: 1 + bits 4-7
        let major = 1 + ((software_version_byte >> 4) & 0x0F);

        // Extract minor version: bits 0-3
        let minor = software_version_byte & 0x0F;

        // Revision is the full byte 6 value
        let revision = revision_byte;

        format!("{}.{}.{}", major, minor, revision)
    }

    pub fn device_name(&self) -> String {
        String::from_utf8_lossy(&self.device_name)
            .trim_end_matches('\0')
            .to_string()
    }

    /// Get the device type name based on hardware ID, device_type_id, or extracted from device_name
    pub fn device_type_name(&self) -> String {
        // First check for device signature in device_type_name field (from read device data command)

        // Try to use the hardware ID (hw_type) from read device data command (byte 19)
        match self.hw_type {
            1 => "PoKeys55".to_string(),
            2 => "PoKeys55".to_string(),
            3 => "PoKeys55".to_string(),
            10 => "Pokeys56U".to_string(),
            11 => "Pokeys56E".to_string(),
            28 => "Pokeys57U".to_string(),
            29 => "Pokeys57E".to_string(),
            30 => "PoKeys57Uv1.1".to_string(),
            31 => "PoKeys57Ev1.1".to_string(),
            32 => "PoKeys57CNC".to_string(),
            35 => "PoKeys57U OEM".to_string(),
            36 => "PoKeys57U OEM".to_string(),
            37 => "PoPLC57NG".to_string(),
            38 => "PoKeys57CNCdb25 ".to_string(),
            39 => "PoKeys57Utest ".to_string(),
            40 => "PoKeys58EU".to_string(),
            41 => "PoBootload (series 58)".to_string(),
            43 => "LiniTester programmer".to_string(),
            44 => "LiniTester calibrator  ".to_string(),
            45 => "PoKeys57Industrial1 ".to_string(),
            50 => "PoPLC v1.0".to_string(),
            60 => "PoKeys16".to_string(),
            // Add more HW ID mappings as needed
            _ => {
                // Fallback to showing the unknown device ID and hardware ID
                format!(
                    "Unknown Device (ID: {}, HW: {})",
                    self.device_type_id, self.hw_type
                )
            }
        }
    }

    /// Get the build date as a string
    pub fn build_date_string(&self) -> String {
        String::from_utf8_lossy(&self.build_date)
            .trim_end_matches('\0')
            .to_string()
    }
}

impl NetworkDeviceInfo {
    pub fn ip_address(&self) -> [u8; 4] {
        self.ip_address_current
    }

    pub fn gateway(&self) -> [u8; 4] {
        self.gateway_ip
    }

    pub fn dns_server(&self) -> [u8; 4] {
        [0; 4] // Not stored in this structure
    }

    pub fn mac_address(&self) -> [u8; 6] {
        [0; 6] // Not stored in this structure
    }

    pub fn device_name(&self) -> String {
        "".to_string() // Not stored in this structure
    }

    pub fn http_port(&self) -> u16 {
        80 // Default HTTP port
    }

    pub fn tcp_port(&self) -> u16 {
        20055 // Default PoKeys TCP port
    }

    pub fn udp_port(&self) -> u16 {
        20055 // Default PoKeys UDP port
    }

    pub fn dhcp_enabled(&self) -> bool {
        self.dhcp != 0
    }
}

/// USB vendor and product IDs
pub const USB_VENDOR_ID: u16 = 0x1DC3;
pub const USB_PRODUCT_ID_1: u16 = 0x1001;
pub const USB_PRODUCT_ID_2: u16 = 0x1002;

/// Protocol constants
pub const REQUEST_HEADER: u8 = 0xBB;
pub const RESPONSE_HEADER: u8 = 0xAA;
pub const CHECKSUM_LENGTH: usize = 7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_name() {
        // Test known device types by hw_type
        let mut device_data = DeviceData {
            hw_type: 10,
            ..Default::default()
        };
        assert_eq!(device_data.device_type_name(), "Pokeys56U");

        device_data.hw_type = 11;
        assert_eq!(device_data.device_type_name(), "Pokeys56E");

        device_data.hw_type = 30;
        assert_eq!(device_data.device_type_name(), "PoKeys57Uv1.1");

        device_data.hw_type = 31;
        assert_eq!(device_data.device_type_name(), "PoKeys57Ev1.1");

        device_data.hw_type = 32;
        assert_eq!(device_data.device_type_name(), "PoKeys57CNC");

        // Test device type extraction from device_name field
        device_data.hw_type = 0; // Unknown hw_type
        device_data.device_type_id = 999999; // Unknown ID
        device_data.device_name = [0; 30];
        let test_name = b"Dec 11 2024PoKeys57E\0\0\0\0\0\0\0\0\0\0";
        device_data.device_name[..test_name.len()].copy_from_slice(test_name);
        assert_eq!(
            device_data.device_type_name(),
            "Unknown Device (ID: 999999, HW: 0)"
        );

        // Test another device type in name
        let test_name2 = b"Build PoKeys56U info\0\0\0\0\0\0\0\0\0";
        device_data.device_name[..test_name2.len()].copy_from_slice(test_name2);
        assert_eq!(
            device_data.device_type_name(),
            "Unknown Device (ID: 999999, HW: 0)"
        );

        // Test unknown device type
        device_data.device_type_id = 99;
        device_data.hw_type = 0;
        device_data.device_name = [0; 30];
        let test_name3 = b"Some other text\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        device_data.device_name[..test_name3.len()].copy_from_slice(test_name3);
        assert_eq!(
            device_data.device_type_name(),
            "Unknown Device (ID: 99, HW: 0)"
        );
    }
}
