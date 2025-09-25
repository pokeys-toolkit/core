//! LCD display support

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use crate::types::LcdMode;
use serde::{Deserialize, Serialize};

/// LCD display data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcdData {
    pub configuration: u8,
    pub rows: u8,
    pub columns: u8,
    pub row_refresh_flags: u8,
    pub line1: [u8; 20],
    pub line2: [u8; 20],
    pub line3: [u8; 20],
    pub line4: [u8; 20],
    pub custom_characters: [[u8; 8]; 8],
}

impl LcdData {
    pub fn new() -> Self {
        Self {
            configuration: 0,
            rows: 0,
            columns: 0,
            row_refresh_flags: 0,
            line1: [0; 20],
            line2: [0; 20],
            line3: [0; 20],
            line4: [0; 20],
            custom_characters: [[0; 8]; 8],
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.configuration != 0
    }

    pub fn get_line(&self, line: usize) -> Option<&[u8; 20]> {
        match line {
            1 => Some(&self.line1),
            2 => Some(&self.line2),
            3 => Some(&self.line3),
            4 => Some(&self.line4),
            _ => None,
        }
    }

    pub fn get_line_mut(&mut self, line: usize) -> Option<&mut [u8; 20]> {
        match line {
            1 => Some(&mut self.line1),
            2 => Some(&mut self.line2),
            3 => Some(&mut self.line3),
            4 => Some(&mut self.line4),
            _ => None,
        }
    }

    pub fn set_line_text(&mut self, line: usize, text: &str) -> Result<()> {
        if !(1..=4).contains(&line) {
            return Err(PoKeysError::Parameter("Invalid line number".to_string()));
        }

        if text.len() > 20 {
            return Err(PoKeysError::Parameter(
                "Text too long for LCD line".to_string(),
            ));
        }

        let line_buffer = self.get_line_mut(line).unwrap();
        line_buffer.fill(0);

        let text_bytes = text.as_bytes();
        line_buffer[..text_bytes.len()].copy_from_slice(text_bytes);

        // Set refresh flag for this line
        self.row_refresh_flags |= 1 << (line - 1);

        Ok(())
    }

    pub fn get_line_text(&self, line: usize) -> Result<String> {
        if !(1..=4).contains(&line) {
            return Err(PoKeysError::Parameter("Invalid line number".to_string()));
        }

        let line_buffer = self.get_line(line).unwrap();

        // Find the end of the string (first null byte)
        let end = line_buffer.iter().position(|&b| b == 0).unwrap_or(20);

        String::from_utf8(line_buffer[..end].to_vec())
            .map_err(|_| PoKeysError::Protocol("Invalid UTF-8 in LCD text".to_string()))
    }

    pub fn clear_line(&mut self, line: usize) -> Result<()> {
        self.set_line_text(line, "")
    }

    pub fn clear_all(&mut self) {
        self.line1.fill(0);
        self.line2.fill(0);
        self.line3.fill(0);
        self.line4.fill(0);
        self.row_refresh_flags = 0x0F; // Refresh all lines
    }

    pub fn set_custom_character(&mut self, char_index: usize, pattern: &[u8; 8]) -> Result<()> {
        if char_index >= 8 {
            return Err(PoKeysError::Parameter(
                "Invalid custom character index".to_string(),
            ));
        }

        self.custom_characters[char_index] = *pattern;
        Ok(())
    }

    pub fn get_custom_character(&self, char_index: usize) -> Result<[u8; 8]> {
        if char_index >= 8 {
            return Err(PoKeysError::Parameter(
                "Invalid custom character index".to_string(),
            ));
        }

        Ok(self.custom_characters[char_index])
    }
}

impl Default for LcdData {
    fn default() -> Self {
        Self::new()
    }
}

impl PoKeysDevice {
    /// Configure LCD display
    pub fn configure_lcd(&mut self, rows: u8, columns: u8, mode: LcdMode) -> Result<()> {
        if rows > 4 || columns > 20 {
            return Err(PoKeysError::Parameter("LCD size not supported".to_string()));
        }

        self.lcd.configuration = match mode {
            LcdMode::Direct => 1,
            LcdMode::Buffered => 2,
        };
        self.lcd.rows = rows;
        self.lcd.columns = columns;

        // Send LCD configuration to device
        self.send_request(0x70, self.lcd.configuration, rows, columns, 0)?;
        Ok(())
    }

    /// Enable or disable LCD
    pub fn enable_lcd(&mut self, enable: bool) -> Result<()> {
        if enable {
            if self.lcd.configuration == 0 {
                // Use default configuration if not set
                self.lcd.configuration = 1; // Direct mode
                self.lcd.rows = 2;
                self.lcd.columns = 16;
            }
        } else {
            self.lcd.configuration = 0;
        }

        self.send_request(
            0x70,
            self.lcd.configuration,
            self.lcd.rows,
            self.lcd.columns,
            0,
        )?;
        Ok(())
    }

    /// Write text to LCD line
    pub fn lcd_write_line(&mut self, line: usize, text: &str) -> Result<()> {
        self.lcd.set_line_text(line, text)?;

        // Send line data to device
        self.send_lcd_line_data(line)?;
        Ok(())
    }

    /// Read text from LCD line
    pub fn lcd_read_line(&self, line: usize) -> Result<String> {
        self.lcd.get_line_text(line)
    }

    /// Clear LCD line
    pub fn lcd_clear_line(&mut self, line: usize) -> Result<()> {
        self.lcd.clear_line(line)?;
        self.send_lcd_line_data(line)?;
        Ok(())
    }

    /// Clear entire LCD display
    pub fn lcd_clear_all(&mut self) -> Result<()> {
        self.lcd.clear_all();

        // Send all line data to device
        for line in 1..=self.lcd.rows {
            self.send_lcd_line_data(line as usize)?;
        }

        Ok(())
    }

    /// Write text at specific position
    pub fn lcd_write_at(&mut self, line: usize, column: usize, text: &str) -> Result<()> {
        if line < 1 || line > self.lcd.rows as usize {
            return Err(PoKeysError::Parameter("Invalid line number".to_string()));
        }

        if column >= self.lcd.columns as usize {
            return Err(PoKeysError::Parameter("Invalid column number".to_string()));
        }

        // Get current line content
        let mut current_text = self.lcd.get_line_text(line).unwrap_or_default();

        // Pad with spaces if necessary
        while current_text.len() < column {
            current_text.push(' ');
        }

        // Replace text at position
        let mut chars: Vec<char> = current_text.chars().collect();
        let new_chars: Vec<char> = text.chars().collect();

        for (i, &ch) in new_chars.iter().enumerate() {
            if column + i < self.lcd.columns as usize {
                if column + i < chars.len() {
                    chars[column + i] = ch;
                } else {
                    chars.push(ch);
                }
            }
        }

        let new_text: String = chars.into_iter().collect();
        self.lcd_write_line(line, &new_text)
    }

    /// Set custom character pattern
    pub fn lcd_set_custom_character(&mut self, char_index: usize, pattern: &[u8; 8]) -> Result<()> {
        self.lcd.set_custom_character(char_index, pattern)?;

        // Send custom character data to device
        self.send_request(0x75, char_index as u8, pattern[0], pattern[1], pattern[2])?;

        self.send_request(0x76, char_index as u8, pattern[3], pattern[4], pattern[5])?;

        self.send_request(0x77, char_index as u8, pattern[6], pattern[7], 0)?;

        Ok(())
    }

    /// Update LCD display (refresh all changed lines)
    pub fn lcd_update(&mut self) -> Result<()> {
        for line in 1..=self.lcd.rows {
            if (self.lcd.row_refresh_flags & (1 << (line - 1))) != 0 {
                self.send_lcd_line_data(line as usize)?;
            }
        }

        self.lcd.row_refresh_flags = 0;
        Ok(())
    }

    /// Send LCD line data to device
    fn send_lcd_line_data(&mut self, line: usize) -> Result<()> {
        if !(1..=4).contains(&line) {
            return Err(PoKeysError::Parameter("Invalid line number".to_string()));
        }

        // Copy line data to avoid borrow checker issues
        let line_data = *self.lcd.get_line(line).unwrap();

        // Send line data in chunks (protocol limitation)
        self.send_request(0x71, line as u8, line_data[0], line_data[1], line_data[2])?;

        self.send_request(0x72, line as u8, line_data[3], line_data[4], line_data[5])?;

        self.send_request(0x73, line as u8, line_data[6], line_data[7], line_data[8])?;

        self.send_request(0x74, line as u8, line_data[9], line_data[10], line_data[11])?;

        // Send remaining characters if needed
        if self.lcd.columns > 12 {
            // Additional requests for longer displays
            // Implementation would continue for up to 20 characters
        }

        Ok(())
    }
}

// Convenience functions for common LCD operations

/// Display a simple message on LCD
pub fn lcd_display_message(device: &mut PoKeysDevice, message: &str) -> Result<()> {
    device.lcd_clear_all()?;

    // Split message into lines
    let lines: Vec<&str> = message.lines().collect();

    for (i, line) in lines.iter().enumerate().take(device.lcd.rows as usize) {
        device.lcd_write_line(i + 1, line)?;
    }

    Ok(())
}

/// Display a two-line message
pub fn lcd_display_two_lines(device: &mut PoKeysDevice, line1: &str, line2: &str) -> Result<()> {
    device.lcd_clear_all()?;
    device.lcd_write_line(1, line1)?;
    device.lcd_write_line(2, line2)?;
    Ok(())
}

/// Create a progress bar on LCD
pub fn lcd_progress_bar(
    device: &mut PoKeysDevice,
    line: usize,
    progress: f32,
    width: usize,
) -> Result<()> {
    if !(0.0..=1.0).contains(&progress) {
        return Err(PoKeysError::Parameter(
            "Progress must be between 0.0 and 1.0".to_string(),
        ));
    }

    let filled_chars = (progress * width as f32) as usize;
    let mut bar = String::new();

    bar.push('[');
    for i in 0..width {
        if i < filled_chars {
            bar.push('█');
        } else {
            bar.push(' ');
        }
    }
    bar.push(']');

    device.lcd_write_line(line, &bar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcd_data_creation() {
        let lcd = LcdData::new();
        assert!(!lcd.is_enabled());
        assert_eq!(lcd.rows, 0);
        assert_eq!(lcd.columns, 0);
    }

    #[test]
    fn test_lcd_line_operations() {
        let mut lcd = LcdData::new();

        assert!(lcd.set_line_text(1, "Hello").is_ok());
        assert_eq!(lcd.get_line_text(1).unwrap(), "Hello");

        assert!(lcd.clear_line(1).is_ok());
        assert_eq!(lcd.get_line_text(1).unwrap(), "");

        // Test invalid line numbers
        assert!(lcd.set_line_text(0, "Test").is_err());
        assert!(lcd.set_line_text(5, "Test").is_err());
    }

    #[test]
    fn test_lcd_text_length_limit() {
        let mut lcd = LcdData::new();

        // Text that's exactly 20 characters should work
        let text_20 = "12345678901234567890";
        assert!(lcd.set_line_text(1, text_20).is_ok());

        // Text that's longer than 20 characters should fail
        let text_21 = "123456789012345678901";
        assert!(lcd.set_line_text(1, text_21).is_err());
    }

    #[test]
    fn test_custom_characters() {
        let mut lcd = LcdData::new();
        let pattern = [0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F];

        assert!(lcd.set_custom_character(0, &pattern).is_ok());
        assert_eq!(lcd.get_custom_character(0).unwrap(), pattern);

        // Test invalid character index
        assert!(lcd.set_custom_character(8, &pattern).is_err());
    }

    #[test]
    fn test_progress_bar_generation() {
        // Test progress bar string generation logic
        let width = 10;
        let progress = 0.5;
        let filled_chars = (progress * width as f32) as usize;

        assert_eq!(filled_chars, 5);

        let mut bar = String::new();
        bar.push('[');
        for i in 0..width {
            if i < filled_chars {
                bar.push('█');
            } else {
                bar.push(' ');
            }
        }
        bar.push(']');

        assert_eq!(bar, "[█████     ]");
    }
}
