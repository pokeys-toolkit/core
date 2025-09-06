//! Matrix LED support

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use serde::{Deserialize, Serialize};

/// Matrix LED configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixLed {
    pub display_enabled: u8,
    pub rows: u8,
    pub columns: u8,
    pub refresh_flag: u8,
    pub data: [u8; 8],
}

impl MatrixLed {
    pub fn new() -> Self {
        Self {
            display_enabled: 0,
            rows: 0,
            columns: 0,
            refresh_flag: 0,
            data: [0; 8],
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.display_enabled != 0
    }

    pub fn set_led(&mut self, row: usize, col: usize, state: bool) -> Result<()> {
        if row >= self.rows as usize || col >= 8 {
            return Err(PoKeysError::Parameter("Invalid LED position".to_string()));
        }

        if state {
            self.data[row] |= 1 << col;
        } else {
            self.data[row] &= !(1 << col);
        }

        self.refresh_flag = 1;
        Ok(())
    }

    pub fn get_led(&self, row: usize, col: usize) -> bool {
        if row >= self.rows as usize || col >= 8 {
            return false;
        }
        (self.data[row] & (1 << col)) != 0
    }

    pub fn clear_all(&mut self) {
        self.data.fill(0);
        self.refresh_flag = 1;
    }

    pub fn set_all(&mut self) {
        self.data.fill(0xFF);
        self.refresh_flag = 1;
    }
}

impl Default for MatrixLed {
    fn default() -> Self {
        Self::new()
    }
}

/// 7-Segment Display Configuration for LED Matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedMatrixConfig {
    pub name: String,
    pub description: Option<String>,
    pub matrix_id: u8, // 1 or 2
    pub enabled: bool,
    pub characters: u8, // 1-8 (number of 7-segment characters/digits)
}

/// 7-Segment Display Helper
pub struct SevenSegmentDisplay {
    pub matrix_id: u8,
    pub character_count: u8, // Number of 7-segment characters
    pub decimal_points: Vec<bool>,
}

/// Protocol structures for LED Matrix
pub struct MatrixLedProtocolConfig {
    pub display1_enabled: bool,
    pub display2_enabled: bool,
    pub display1_characters: u8, // Number of characters (becomes rows in protocol)
    pub display2_characters: u8, // Number of characters (becomes rows in protocol)
}

/// Constants for 7-segment displays
pub const SEVEN_SEGMENT_COLUMNS: u8 = 8; // Always 8: 7 segments + decimal point

/// Pin assignments for LED matrices
pub const LED_MATRIX_1_PINS: [u8; 3] = [9, 10, 11]; // Data, Latch, Clock
pub const LED_MATRIX_2_PINS: [u8; 3] = [23, 24, 25]; // Data, Latch, Clock

/// 7-Segment digit patterns with correct bit mapping
/// Bit mapping: 0=DP, 1=G, 2=F, 3=E, 4=D, 5=C, 6=B, 7=A
pub const SEVEN_SEGMENT_DIGITS: [u8; 10] = [
    0b11111100, // 0: A,B,C,D,E,F (bits 7,6,5,4,3,2)
    0b01100000, // 1: B,C (bits 6,5)
    0b11011010, // 2: A,B,D,E,G (bits 7,6,4,3,1)
    0b11110010, // 3: A,B,C,D,G (bits 7,6,5,4,1)
    0b01100110, // 4: B,C,F,G (bits 6,5,2,1)
    0b10110110, // 5: A,C,D,F,G (bits 7,5,4,2,1)
    0b10111110, // 6: A,C,D,E,F,G (bits 7,5,4,3,2,1)
    0b11100000, // 7: A,B,C (bits 7,6,5)
    0b11111110, // 8: A,B,C,D,E,F,G (bits 7,6,5,4,3,2,1)
    0b11110110, // 9: A,B,C,D,F,G (bits 7,6,5,4,2,1)
];

/// 7-Segment letter patterns with correct bit mapping
/// Bit mapping: 0=DP, 1=G, 2=F, 3=E, 4=D, 5=C, 6=B, 7=A
pub const SEVEN_SEGMENT_LETTERS: [(char, u8); 19] = [
    ('a', 0b11101110), // a: A,B,C,E,F,G (bits 7,6,5,3,2,1)
    ('b', 0b00111110), // b: C,D,E,F,G (bits 5,4,3,2,1)
    ('c', 0b10011100), // c: A,D,E,F (bits 7,4,3,2)
    ('d', 0b01111010), // d: B,C,D,E,G (bits 6,5,4,3,1)
    ('e', 0b10011110), // e: A,D,E,F,G (bits 7,4,3,2,1)
    ('f', 0b10001110), // f: A,E,F,G (bits 7,3,2,1)
    ('h', 0b01101110), // h: B,C,E,F,G (bits 6,5,3,2,1)
    ('i', 0b01100000), // i: B,C (bits 6,5)
    ('j', 0b01110000), // j: B,C,D (bits 6,5,4)
    ('l', 0b00011100), // L: D,E,F (bits 4,3,2)
    ('n', 0b00101010), // n: C,E,G (bits 5,3,1)
    ('o', 0b11111100), // o: A,B,C,D,E,F (bits 7,6,5,4,3,2) - same as 0
    ('p', 0b11001110), // p: A,B,E,F,G (bits 7,6,3,2,1)
    ('q', 0b11100110), // q: A,B,C,F,G (bits 7,6,5,2,1)
    ('r', 0b00001010), // r: E,G (bits 3,1)
    ('s', 0b10110110), // S: A,C,D,F,G (bits 7,5,4,2,1) - same as 5
    ('t', 0b00011110), // t: D,E,F,G (bits 4,3,2,1)
    ('u', 0b01111000), // u: B,C,D,E,F (bits 6,5,4,3,2)
    ('y', 0b01110110), // y: B,C,D,F,G (bits 6,5,4,2,1)
];

/// Matrix action types for LED matrix updates
#[derive(Debug, Clone, Copy)]
pub enum MatrixAction {
    UpdateWhole,
    SetPixel,
    ClearPixel,
}

/// Helper function to get pattern for a character
pub fn get_seven_segment_pattern(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => {
            let digit = (ch as u8) - b'0';
            Some(SEVEN_SEGMENT_DIGITS[digit as usize])
        }
        // Special case for capital N
        'N' => Some(0b11101100), // N: E,F,A,B,C segments (bits 3,2,7,6,5)
        'a'..='z' | 'A'..='Z' => {
            let lower_ch = ch.to_ascii_lowercase();
            SEVEN_SEGMENT_LETTERS
                .iter()
                .find(|(c, _)| *c == lower_ch)
                .map(|(_, pattern)| *pattern)
        }
        // Special characters
        '-' => Some(0b00000010), // Minus: G segment only (bit 1)
        '_' => Some(0b00010000), // Underscore: D segment only (bit 4)
        ']' => Some(0b11110000), // Right bracket: A,B,C,D segments (bits 7,6,5,4)
        '[' => Some(0b10011100), // Left bracket: A,D,E,F segments (bits 7,4,3,2)
        ' ' => Some(0b00000000), // Space: all segments off
        _ => None,
    }
}

impl SevenSegmentDisplay {
    pub fn new(matrix_id: u8, character_count: u8) -> Self {
        Self {
            matrix_id,
            character_count,
            decimal_points: vec![false; character_count as usize],
        }
    }

    pub fn display_number(&self, device: &mut PoKeysDevice, number: u32) -> Result<()> {
        let digits = self.number_to_digits(number);
        let mut row_data = [0u8; 8];

        for (row, digit) in digits.iter().enumerate() {
            if row < self.character_count as usize {
                row_data[row] = SEVEN_SEGMENT_DIGITS[*digit as usize];
                if self.decimal_points[row] {
                    row_data[row] |= 0b00000001; // Set decimal point (bit 0)
                }
            }
        }

        device.update_led_matrix(self.matrix_id, MatrixAction::UpdateWhole, 0, 0, &row_data)
    }

    pub fn display_text(&self, device: &mut PoKeysDevice, text: &str) -> Result<()> {
        let mut row_data = [0u8; 8];
        let chars: Vec<char> = text.chars().collect();

        for (row, &ch) in chars.iter().enumerate() {
            if row < self.character_count as usize {
                if let Some(pattern) = get_seven_segment_pattern(ch) {
                    row_data[row] = pattern;
                    if self.decimal_points[row] {
                        row_data[row] |= 0b00000001; // Set decimal point (bit 0)
                    }
                } else {
                    // Unknown character - display nothing (all segments off)
                    row_data[row] = 0;
                }
            }
        }

        device.update_led_matrix(self.matrix_id, MatrixAction::UpdateWhole, 0, 0, &row_data)
    }

    pub fn display_mixed(&self, device: &mut PoKeysDevice, text: &str) -> Result<()> {
        // Alias for display_text - handles both numbers and letters
        self.display_text(device, text)
    }

    pub fn set_decimal_point(&mut self, character: u8, enabled: bool) {
        if character < self.character_count {
            self.decimal_points[character as usize] = enabled;
        }
    }

    fn number_to_digits(&self, number: u32) -> Vec<u8> {
        let mut digits = Vec::new();
        let mut n = number;

        if n == 0 {
            digits.push(0);
        } else {
            while n > 0 {
                digits.push((n % 10) as u8);
                n /= 10;
            }
            digits.reverse();
        }

        // Pad with leading zeros if needed
        while digits.len() < self.character_count as usize {
            digits.insert(0, 0);
        }

        digits.truncate(self.character_count as usize);
        digits
    }
}

impl PoKeysDevice {
    /// Configure matrix LED display
    pub fn configure_matrix_led(&mut self, led_index: usize, rows: u8, columns: u8) -> Result<()> {
        if led_index >= self.matrix_led.len() {
            return Err(PoKeysError::Parameter(
                "Invalid LED matrix index".to_string(),
            ));
        }

        if rows > 8 || columns > 8 {
            return Err(PoKeysError::Parameter(
                "Matrix LED size too large".to_string(),
            ));
        }

        self.matrix_led[led_index].display_enabled = 1;
        self.matrix_led[led_index].rows = rows;
        self.matrix_led[led_index].columns = columns;

        // Send configuration to device
        self.send_request(0x62, led_index as u8, rows, columns, 1)?;
        Ok(())
    }

    /// Update matrix LED display
    pub fn update_matrix_led(&mut self, led_index: usize) -> Result<()> {
        if led_index >= self.matrix_led.len() {
            return Err(PoKeysError::Parameter(
                "Invalid LED matrix index".to_string(),
            ));
        }

        if !self.matrix_led[led_index].is_enabled() {
            return Err(PoKeysError::NotSupported);
        }

        // Copy data to avoid borrow checker issues
        let data = self.matrix_led[led_index].data;
        let rows = self.matrix_led[led_index].rows;

        // Send LED data to device
        self.send_request(0x63, led_index as u8, data[0], data[1], data[2])?;

        // Send remaining data if needed
        if rows > 3 {
            self.send_request(0x64, led_index as u8, data[3], data[4], data[5])?;
        }

        if rows > 6 {
            self.send_request(0x65, led_index as u8, data[6], data[7], 0)?;
        }

        self.matrix_led[led_index].refresh_flag = 0;
        Ok(())
    }

    /// Set individual LED in matrix
    pub fn set_matrix_led(
        &mut self,
        led_index: usize,
        row: usize,
        col: usize,
        state: bool,
    ) -> Result<()> {
        if led_index >= self.matrix_led.len() {
            return Err(PoKeysError::Parameter(
                "Invalid LED matrix index".to_string(),
            ));
        }

        self.matrix_led[led_index].set_led(row, col, state)?;
        self.update_matrix_led(led_index)?;
        Ok(())
    }

    /// Get individual LED state in matrix
    pub fn get_matrix_led(&self, led_index: usize, row: usize, col: usize) -> Result<bool> {
        if led_index >= self.matrix_led.len() {
            return Err(PoKeysError::Parameter(
                "Invalid LED matrix index".to_string(),
            ));
        }

        Ok(self.matrix_led[led_index].get_led(row, col))
    }

    /// Clear all LEDs in matrix
    pub fn clear_matrix_led(&mut self, led_index: usize) -> Result<()> {
        if led_index >= self.matrix_led.len() {
            return Err(PoKeysError::Parameter(
                "Invalid LED matrix index".to_string(),
            ));
        }

        self.matrix_led[led_index].clear_all();
        self.update_matrix_led(led_index)?;
        Ok(())
    }

    /// Set all LEDs in matrix
    pub fn set_all_matrix_led(&mut self, led_index: usize) -> Result<()> {
        if led_index >= self.matrix_led.len() {
            return Err(PoKeysError::Parameter(
                "Invalid LED matrix index".to_string(),
            ));
        }

        self.matrix_led[led_index].set_all();
        self.update_matrix_led(led_index)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_led_creation() {
        let mut led = MatrixLed::new();
        assert!(!led.is_enabled());

        led.display_enabled = 1;
        led.rows = 8;
        led.columns = 8;

        assert!(led.is_enabled());
        assert!(!led.get_led(0, 0));

        assert!(led.set_led(0, 0, true).is_ok());
        assert!(led.get_led(0, 0));

        assert!(led.set_led(0, 0, false).is_ok());
        assert!(!led.get_led(0, 0));
    }

    #[test]
    fn test_matrix_led_bounds_checking() {
        let mut led = MatrixLed::new();
        led.rows = 4;
        led.columns = 8;

        // Valid positions
        assert!(led.set_led(0, 0, true).is_ok());
        assert!(led.set_led(3, 7, true).is_ok());

        // Invalid positions
        assert!(led.set_led(4, 0, true).is_err()); // Row out of bounds
        assert!(led.set_led(0, 8, true).is_err()); // Column out of bounds
    }
}
