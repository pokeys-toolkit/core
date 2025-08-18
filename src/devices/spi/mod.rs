//! SPI Device Implementations
//!
//! This module contains high-level device abstractions for various SPI devices
//! that can be connected to PoKeys devices. Each device provides a convenient
//! interface that handles the low-level SPI communication details.
//!
//! # Available Devices
//!
//! - **MAX7219** - 7-segment display controller
//!
//! # Usage
//!
//! ```rust,no_run
//! use pokeys_lib::*;
//! use pokeys_lib::devices::spi::Max7219;
//!
//! fn main() -> Result<()> {
//!     let mut device = connect_to_device(0)?;
//!     let mut display = Max7219::new(&mut device, 24)?;
//!     
//!     display.configure_numeric(8)?;
//!     display.display_number(12345)?;
//!     
//!     Ok(())
//! }
//! ```

pub mod max7219;

// Re-export for convenience
pub use max7219::{DisplayMode, Max7219, TextJustification};
