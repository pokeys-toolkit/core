//! Matrix keyboard support

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
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
        let key_index = row * self.width as usize + col;
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

impl PoKeysDevice {
    /// Configure matrix keyboard
    pub fn configure_matrix_keyboard(
        &mut self,
        width: u8,
        height: u8,
        column_pins: &[u8],
        row_pins: &[u8],
    ) -> Result<()> {
        if width > 8 || height > 16 {
            return Err(PoKeysError::Parameter("Matrix size too large".to_string()));
        }

        self.matrix_keyboard.configuration = 1;
        self.matrix_keyboard.width = width;
        self.matrix_keyboard.height = height;

        // Copy pin assignments
        for (i, &pin) in column_pins.iter().enumerate().take(8) {
            self.matrix_keyboard.column_pins[i] = pin;
        }

        for (i, &pin) in row_pins.iter().enumerate().take(16) {
            self.matrix_keyboard.row_pins[i] = pin;
        }

        // Send configuration to device
        self.send_request(0x60, width, height, 0, 0)?;
        Ok(())
    }

    /// Read matrix keyboard state
    pub fn read_matrix_keyboard(&mut self) -> Result<()> {
        let response = self.send_request(0x61, 0, 0, 0, 0)?;

        // Parse keyboard state from response
        let data_start = 8;
        let data_len =
            (self.matrix_keyboard.width as usize * self.matrix_keyboard.height as usize).min(128);

        if response.len() >= data_start + data_len {
            self.matrix_keyboard.key_values[..data_len]
                .copy_from_slice(&response[data_start..data_start + data_len]);
        }

        Ok(())
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
        
        kb.key_values[5] = 1; // Row 1, Col 1 (1*4 + 1 = 5)
        assert!(kb.get_key_state(1, 1));
    }

    #[test]
    fn test_key_index_calculation() {
        let mut kb = MatrixKeyboard::new();
        kb.width = 3;
        kb.height = 3;
        
        // Test key index calculation: row * width + col
        kb.key_values[0] = 1;  // (0,0)
        kb.key_values[1] = 1;  // (0,1)
        kb.key_values[3] = 1;  // (1,0)
        kb.key_values[4] = 1;  // (1,1)
        
        assert!(kb.get_key_state(0, 0));
        assert!(kb.get_key_state(0, 1));
        assert!(!kb.get_key_state(0, 2));
        assert!(kb.get_key_state(1, 0));
        assert!(kb.get_key_state(1, 1));
        assert!(!kb.get_key_state(1, 2));
    }
}
