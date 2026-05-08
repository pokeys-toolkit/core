//! # PoKeys Core Library - Pure Rust Implementation
//!
//! This is the **core library** of the PoKeys ecosystem, providing a pure Rust implementation
//! of the PoKeysLib functionality for controlling PoKeys devices without external dependencies.
//!
//! ## Core Features
//!
//! ### Device Connectivity
//! - USB and Network device enumeration and connection
//! - Auto-detection of connection types
//! - Multi-device concurrent management
//!
//! ### Digital & Analog I/O
//! - Digital I/O operations with bulk configuration
//! - Multi-channel analog input with configurable reference voltage
//! - Pin function validation and safety checks
//!
//! ### Advanced Control Systems
//! - PWM control with configurable frequency and duty cycle
//! - Quadrature encoder support (4x/2x sampling modes)

//! - Matrix keyboard scanning and LED matrix control
//!
//! ### Communication Protocols
//! - **SPI**: Full master support with multiple chip select pins
//! - **I2C**: Master operations with device scanning
//! - **1-Wire**: Temperature sensor support
//! - **CAN Bus**: Message transmission and reception
//! - **UART**: Serial communication
//!
//! ### Display & Interface Support
//! - LCD display control and management
//! - Seven-segment character mapping utilities
//!
//! ### Sensor Integration
//! - EasySensors support and data acquisition
//! - Real-time clock operations and synchronization
//! - Temperature sensor integration
//!
//! ### Safety & Reliability
//! - Device model validation with pin capability checks
//! - Comprehensive error handling with context
//! - Thread-safe concurrent device access
//! - Configurable failsafe behavior
//! - SPI pin reservation and conflict prevention
//!
//! ## Performance Optimizations
//!
//! - **Bulk Operations**: 28x faster pin configuration (96.4% time reduction)
//! - **Single Enumeration**: 3x faster multi-device sync (65% improvement)
//! - **Encoder Fix**: Correct pin numbering conversion
//!
//! ## Usage
//!
//! ```rust,no_run
//! use pokeys_lib::{enumerate_usb_devices, connect_to_device, PinFunction, Result};
//!
//! fn main() -> Result<()> {
//!     // Enumerate available devices
//!     let device_count = enumerate_usb_devices()?;
//!
//!     // Connect to first device
//!     if device_count > 0 {
//!         let mut device = connect_to_device(0)?;
//!
//!         // Read device information
//!         device.get_device_data()?;
//!
//!         // Set pin as digital output
//!         device.set_pin_function(1, PinFunction::DigitalOutput)?;
//!
//!         // Set pin high
//!         device.set_digital_output(1, true)?;
//!     }
//!
//!     Ok(())
//! }
//! ```
// Allow clippy warnings for cleanup PR - these will be addressed in a separate PR
#![allow(clippy::derivable_impls)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::uninlined_format_args)]

pub mod communication;
pub mod device;
pub mod encoders;
pub mod error;
pub mod io;
pub mod keyboard_matrix;
pub mod lcd;
pub mod matrix;
pub mod model_manager;
pub mod models;
pub mod network;
pub mod oem_parameters;
pub mod protocols;
pub mod pulse_engine;
pub mod pwm;
pub mod sensors;
pub mod types;

pub use device::*;
pub use error::*;
pub use pulse_engine::PulseEngineConfig;
pub use types::*;

// Re-export main functionality
pub use device::{connect_to_device, connect_to_device_with_serial, enumerate_usb_devices};
pub use io::{PinCapability, PinFunction};
pub use keyboard_matrix::MatrixKeyboard;
pub use model_manager::ModelManager;
pub use models::{DeviceModel, PinModel};

// Re-export OEM parameter constants
pub use oem_parameters::{LOCATION_PARAMETER_INDEX, OEM_PARAMETER_MAX_INDEX};

// Re-export LED matrix functionality
pub use matrix::{
    LedMatrixConfig, MatrixAction, MatrixLedProtocolConfig, SEVEN_SEGMENT_DIGITS,
    SEVEN_SEGMENT_LETTERS, SevenSegmentDisplay, get_seven_segment_pattern,
};

// Re-export network configuration helpers
pub use network::NetworkDeviceConfig;

// Re-export protocol convenience functions
pub use protocols::{
    can_send_standard, i2c_read_simple, i2c_write_simple, spi_configure_simple, spi_read_simple,
    spi_write_simple,
};

// Re-export uSPIBridge functionality
pub use protocols::{SegmentMapping, SegmentMappingType, USPIBridgeCommand, USPIBridgeConfig};

// Re-export servo control functionality
pub use pwm::{ServoConfig, ServoType};

/// Library version string, sourced from `Cargo.toml` at compile time.
///
/// Prefer this constant over the [`version()`] helper when a `&'static str`
/// is acceptable — it avoids the allocation.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the library version as an owned `String`.
///
/// Kept for backward compatibility; returns the same value as [`VERSION`].
pub fn version() -> String {
    VERSION.to_string()
}
