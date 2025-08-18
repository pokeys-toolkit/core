//! Device Implementations
//!
//! This module contains high-level device abstractions for various external
//! devices that can be connected to PoKeys devices via different communication
//! protocols (SPI, I2C, 1-Wire, etc.).
//!
//! Each device implementation provides a convenient, type-safe interface that
//! handles the low-level protocol communication details, making it easy to
//! work with complex devices without needing to understand their register
//! maps or communication protocols.
//!
//! # Organization
//!
//! Devices are organized by communication protocol:
//!
//! - **SPI Devices** (`spi/`) - Devices connected via SPI bus
//!   - MAX7219 - 7-segment display controller
//!
//! # Future Expansion
//!
//! Additional protocol modules can be added:
//! - `i2c/` - I2C devices (sensors, EEPROMs, etc.)
//! - `onewire/` - 1-Wire devices (temperature sensors, etc.)
//! - `uart/` - UART devices (GPS modules, etc.)
//!
//! # Usage Pattern
//!
//! All device implementations follow a similar pattern:
//!
//! 1. Create device instance with PoKeys device reference and pin configuration
//! 2. Configure the device for your specific use case
//! 3. Use high-level methods to interact with the device
//!
//! ```rust,no_run
//! use pokeys_lib::*;
//! use pokeys_lib::devices::spi::Max7219;
//!
//! fn main() -> Result<()> {
//!     // Step 1: Connect to PoKeys device
//!     let mut pokeys = connect_to_device(0)?;
//!     
//!     // Step 2: Create device instance
//!     let mut display = Max7219::new(&mut pokeys, 24)?; // CS pin 24
//!     
//!     // Step 3: Configure device
//!     display.configure_numeric(8)?; // Brightness level 8
//!     
//!     // Step 4: Use device
//!     display.display_number(12345)?;
//!     display.display_text("HELLO")?;
//!     
//!     Ok(())
//! }
//! ```

pub mod spi;

// Re-export commonly used devices for convenience
pub use spi::{DisplayMode, Max7219, TextJustification};
