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
//! - Pulse Engine v2 for stepper motor control
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
//! - **MAX7219**: Comprehensive LED display driver support
//!   - Individual and daisy-chained displays
//!   - 7-segment, dot matrix, and raw segment modes
//!   - Text display with justification and scrolling
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
pub mod devices;
pub mod encoders;
pub mod error;
pub mod io;
pub mod lcd;
pub mod matrix;
pub mod model_manager;
pub mod models;
pub mod network;
pub mod protocols;
pub mod pulse_engine;
pub mod pwm;
pub mod sensors;
pub mod types;

pub use device::*;
pub use error::*;
pub use types::*;

// Re-export main functionality
pub use device::{connect_to_device, connect_to_device_with_serial, enumerate_usb_devices};
pub use io::{PinCapability, PinFunction};
pub use model_manager::ModelManager;
pub use models::{DeviceModel, PinModel};

// Re-export devices functionality for external device support
pub use devices::spi::{DisplayMode, Max7219, TextJustification};

// Re-export LED matrix functionality
pub use matrix::{
    get_seven_segment_pattern, LedMatrixConfig, MatrixAction, MatrixLedProtocolConfig,
    SevenSegmentDisplay, SEVEN_SEGMENT_DIGITS, SEVEN_SEGMENT_LETTERS,
};

// Re-export protocol convenience functions
pub use protocols::{
    can_send_standard, i2c_read_simple, i2c_write_simple, spi_configure_simple, spi_read_simple,
    spi_write_simple,
};

/// Library version information
pub const VERSION_MAJOR: u8 = 0;
pub const VERSION_MINOR: u8 = 3;
pub const VERSION_PATCH: u8 = 0;

/// Get library version as string
pub fn version() -> String {
    format!("{VERSION_MAJOR}.{VERSION_MINOR}.{VERSION_PATCH}")
}
