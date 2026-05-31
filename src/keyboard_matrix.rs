//! Matrix keyboard support
//!
//! This module provides matrix keyboard functionality for PoKeys devices using the official
//! protocol specification (command 0xCA). The implementation supports up to 16x8 matrix
//! keyboards with proper key indexing according to the PoKeys protocol.
//!
//! ## Key Features
//! - Supports matrix keyboards up to 16 rows x 8 columns
//! - Protocol-compliant implementation using command 0xCA
//! - Proper key indexing with 8-column internal layout
//! - Real-time key state monitoring
//! - Configurable pin assignments for rows and columns
//!
//! ## Key Indexing
//! The PoKeys protocol uses a fixed 8-column internal layout regardless of configured width:
//! - Row 0: keys 0-7 (only 0-width used)
//! - Row 1: keys 8-15 (only 8-(8+width) used)
//! - Row 2: keys 16-23, etc.
//!
//! ## Byte numbering convention
//! The protocol specification uses **1-based** byte indices ("byte 2: 0xCA",
//! "byte 9: new configuration", etc.); the Rust code uses **0-based** indices into
//! the request/response buffers. Mapping: spec byte `N` → `request[N-1]` / `response[N-1]`.
//! The data payload of a request lands at `request[8]` (= spec byte 9), so the `data` slice
//! passed to `send_request_with_data` has `data[k]` ≡ spec byte `9+k`.
//!
//! ## Spec vs vendor reference library
//! The matrix keyboard section of the protocol specification is internally inconsistent:
//! the request layout shows "If option == 16" but footnote 6 says "Use option 1 to setup".
//! The vendor reference library (PoLabsEE/PoKeysLib `PK_MatrixKBConfigurationSet`) uses
//! **option 1** for the configuration write — that is the load-bearing answer. Sending
//! option 16 is silently echoed but does not bind pins to row/column slots.
//!
//! The configuration write is also a **two-phase sequence** in the vendor library:
//! deactivate (option 1, enable=0) → key mappings → activate (option 1, enable=1). The
//! single-shot variant doesn't reliably enable scanning. [`crate::PoKeysDevice::configure_matrix_keyboard`]
//! follows the vendor sequence.
//!
//! ## Required setup sequence
//! [`crate::PoKeysDevice::configure_matrix_keyboard`] handles the underlying pin-function
//! prerequisites internally — row pins are forced to `DigitalOutput`, column pins to
//! `DigitalInput` — so callers do not need to set pin functions first.
//!
//! ## Example Usage
//! ```rust,no_run
//! use pokeys_lib::*;
//!
//! fn main() -> Result<()> {
//!     let mut device = connect_to_device(0)?;
//!
//!     // Configure 4x4 matrix keyboard. Pin functions are set automatically;
//!     // a successful return guarantees the device confirmed the configuration.
//!     let column_pins = [21, 22, 23, 24];
//!     let row_pins = [13, 14, 15, 16];
//!     device.configure_matrix_keyboard(4, 4, &column_pins, &row_pins)?;
//!
//!     // Read keyboard state
//!     device.read_matrix_keyboard()?;
//!     let key_pressed = device.matrix_keyboard.get_key_state(0, 0);
//!
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};

/// A snapshot of the device-side matrix-keyboard configuration, returned by
/// [`crate::PoKeysDevice::get_matrix_keyboard_configuration`].
///
/// Read-only view of what the firmware currently has stored — reflects the
/// result of the most recent successful configuration write or pre-existing
/// state from non-volatile storage. Useful for diff-against-device flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixKeyboardConfig {
    /// `true` if the matrix keyboard is currently enabled (bit 0 of the
    /// configuration byte at spec byte 9).
    pub enabled: bool,
    /// Number of columns (1..=8). Decoded from `(size_byte >> 4) + 1`.
    pub width: u8,
    /// Number of rows (1..=16). Decoded from `(size_byte & 0x0F) + 1`.
    pub height: u8,
    /// 1-based pin numbers for rows 0..16. Trailing entries beyond `height`
    /// are 0. Stored 1-based for human readability; the protocol uses
    /// 0-based pin codes on the wire.
    pub row_pins: [u8; 16],
    /// 1-based pin numbers for columns 0..8. Trailing entries beyond `width`
    /// are 0.
    pub column_pins: [u8; 8],
    /// Direct/macro mapping bitmap, 128 bits. Bit `k` set means key `k`
    /// uses macro mapping; clear means direct key mapping.
    pub direct_macro_bitmap: [u8; 16],
    /// Alternate-function pin (1-based, or 0 if disabled). When set, the
    /// state of this digital input pin selects between the primary and
    /// alternate keyboard mapping for each key.
    pub alternate_function_pin: u8,
    /// Scanning decimation factor (0..=50). Higher values reduce the
    /// device-side scan rate.
    pub scanning_decimation: u8,
}

impl MatrixKeyboardConfig {
    /// Wire size byte: `(width-1) << 4 | (height-1)`.
    pub fn size_byte(&self) -> u8 {
        ((self.width.saturating_sub(1)) << 4) | (self.height.saturating_sub(1) & 0x0F)
    }
}

/// Matrix keyboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixKeyboard {
    pub configuration: u8,
    pub width: u8,
    pub height: u8,
    pub scanning_decimation: u8,
    pub column_pins: [u8; 8],
    pub row_pins: [u8; 16],
    pub macro_mapping_options: Vec<u8>,
    pub key_mapping_key_code: Vec<u8>,
    pub key_mapping_key_modifier: Vec<u8>,
    pub key_mapping_triggered_key: Vec<u8>,
    pub key_mapping_key_code_up: Vec<u8>,
    pub key_mapping_key_modifier_up: Vec<u8>,
    pub key_values: Vec<u8>,
}

impl MatrixKeyboard {
    pub fn new() -> Self {
        Self {
            configuration: 0,
            width: 0,
            height: 0,
            scanning_decimation: 0,
            column_pins: [0; 8],
            row_pins: [0; 16],
            macro_mapping_options: vec![0; 128],
            key_mapping_key_code: vec![0; 128],
            key_mapping_key_modifier: vec![0; 128],
            key_mapping_triggered_key: vec![0; 128],
            key_mapping_key_code_up: vec![0; 128],
            key_mapping_key_modifier_up: vec![0; 128],
            key_values: vec![0; 128],
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.configuration != 0
    }

    pub fn get_key_state(&self, row: usize, col: usize) -> bool {
        if row >= self.height as usize || col >= self.width as usize {
            return false;
        }
        // Protocol uses 8-column layout internally: key_index = row * 8 + col
        let key_index = row * 8 + col;
        if key_index < self.key_values.len() {
            self.key_values[key_index] != 0
        } else {
            false
        }
    }
}

impl Default for MatrixKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_keyboard_config_size_byte() {
        let cfg = MatrixKeyboardConfig {
            enabled: true,
            width: 4,
            height: 12,
            row_pins: [0u8; 16],
            column_pins: [0u8; 8],
            direct_macro_bitmap: [0u8; 16],
            alternate_function_pin: 0,
            scanning_decimation: 0,
        };
        // (4-1) << 4 | (12-1) = 0x3B
        assert_eq!(cfg.size_byte(), 0x3B);
    }

    #[test]
    fn test_matrix_keyboard_config_size_byte_min() {
        let cfg = MatrixKeyboardConfig {
            enabled: false,
            width: 1,
            height: 1,
            row_pins: [0u8; 16],
            column_pins: [0u8; 8],
            direct_macro_bitmap: [0u8; 16],
            alternate_function_pin: 0,
            scanning_decimation: 0,
        };
        assert_eq!(cfg.size_byte(), 0);
    }

    #[test]
    fn test_matrix_keyboard_creation() {
        let kb = MatrixKeyboard::new();
        assert!(!kb.is_enabled());
        assert_eq!(kb.width, 0);
        assert_eq!(kb.height, 0);
    }

    #[test]
    fn test_matrix_keyboard_configuration() {
        let mut kb = MatrixKeyboard::new();

        // Test initial state
        assert_eq!(kb.configuration, 0);
        assert_eq!(kb.width, 0);
        assert_eq!(kb.height, 0);
        assert_eq!(kb.scanning_decimation, 0);
        assert_eq!(kb.column_pins.len(), 8);
        assert_eq!(kb.row_pins.len(), 16);

        // Test enabling
        kb.configuration = 1;
        kb.width = 4;
        kb.height = 4;

        assert!(kb.is_enabled());
        assert_eq!(kb.width, 4);
        assert_eq!(kb.height, 4);
    }

    #[test]
    fn test_get_key_state() {
        let mut kb = MatrixKeyboard::new();
        kb.width = 4;
        kb.height = 4;

        // Test bounds checking
        assert!(!kb.get_key_state(0, 0)); // Should be false initially
        assert!(!kb.get_key_state(4, 0)); // Out of bounds
        assert!(!kb.get_key_state(0, 4)); // Out of bounds

        // Test setting key state
        kb.key_values[0] = 1; // Row 0, Col 0
        assert!(kb.get_key_state(0, 0));

        kb.key_values[8] = 1; // Row 1, Col 0 (1*8 + 0 = 8)
        assert!(kb.get_key_state(1, 0));
    }

    #[test]
    fn test_key_index_calculation() {
        let mut kb = MatrixKeyboard::new();
        kb.width = 3;
        kb.height = 3;

        // Test key index calculation: row * 8 + col (protocol uses 8-column layout)
        kb.key_values[0] = 1; // (0,0) = 0*8 + 0 = 0
        kb.key_values[1] = 1; // (0,1) = 0*8 + 1 = 1
        kb.key_values[8] = 1; // (1,0) = 1*8 + 0 = 8
        kb.key_values[9] = 1; // (1,1) = 1*8 + 1 = 9

        assert!(kb.get_key_state(0, 0));
        assert!(kb.get_key_state(0, 1));
        assert!(!kb.get_key_state(0, 2));
        assert!(kb.get_key_state(1, 0));
        assert!(kb.get_key_state(1, 1));
        assert!(!kb.get_key_state(1, 2));
    }
}
