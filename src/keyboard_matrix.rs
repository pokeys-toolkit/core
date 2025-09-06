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
//! ## Example Usage
//! ```rust,no_run
//! use pokeys_lib::*;
//!
//! let mut device = connect_to_device(0)?;
//!
//! // Configure 4x4 matrix keyboard
//! let column_pins = [21, 22, 23, 24];
//! let row_pins = [13, 14, 15, 16];
//! device.configure_matrix_keyboard(4, 4, &column_pins, &row_pins)?;
//!
//! // Read keyboard state
//! device.read_matrix_keyboard()?;
//! let key_pressed = device.matrix_keyboard.get_key_state(0, 0);
//! ```

use serde::{Deserialize, Serialize};

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
