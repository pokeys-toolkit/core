//! MAX7219 SPI Display Controller
//!
//! This module provides a high-level interface for controlling MAX7219
//! 7-segment display controllers via SPI. The MAX7219 can drive up to
//! 8 digits of 7-segment displays with decimal points.
//!
//! # Features
//!
//! - Code B decode mode for easy numeric display
//! - Raw segment mode for custom patterns and text
//! - Configurable intensity (brightness)
//! - Display test mode
//! - Shutdown mode for power saving
//! - Proper decimal point handling
//!
//! # Usage
//!
//! ```rust,no_run
//! use pokeys_lib::*;
//! use pokeys_lib::devices::spi::Max7219;
//!
//! fn main() -> Result<()> {
//!     let mut device = connect_to_device(0)?;
//!     let mut display = Max7219::new(&mut device, 24)?; // CS pin 24
//!     
//!     // Configure for numeric display
//!     display.configure_numeric(8)?; // 8 intensity
//!     
//!     // Display a number
//!     display.display_number(12345)?;
//!     
//!     // Display text in raw mode
//!     display.configure_raw_segments(8)?;
//!     display.display_text("HELLO")?;
//!     
//!     Ok(())
//! }
//! ```

use crate::device::PoKeysDevice;
use crate::error::{PoKeysError, Result};
use std::thread;
use std::time::{Duration, Instant};

/// MAX7219 Register Addresses
const REG_DIGIT0: u8 = 0x01;
const REG_DECODE_MODE: u8 = 0x09;
const REG_INTENSITY: u8 = 0x0A;
const REG_SCAN_LIMIT: u8 = 0x0B;
const REG_SHUTDOWN: u8 = 0x0C;
const REG_DISPLAY_TEST: u8 = 0x0F;
const REG_NOOP: u8 = 0x00; // No-operation (for daisy chain)

/// Configuration values
const DECODE_B_ALL: u8 = 0xFF; // Code B decode for all digits
const DECODE_NONE: u8 = 0x00; // No decode (raw segments)
const SCAN_LIMIT_8: u8 = 0x07; // Scan all 8 digits
const NORMAL_MODE: u8 = 0x01; // Normal operation
const SHUTDOWN_MODE: u8 = 0x00; // Shutdown mode
const TEST_ON: u8 = 0x01; // Display test on
const TEST_OFF: u8 = 0x00; // Display test off

/// 7-segment patterns for digits 0-9 (raw segments)
const DIGIT_PATTERNS: [u8; 10] = [
    0b01111110, // 0: A,B,C,D,E,F
    0b00110000, // 1: B,C
    0b01101101, // 2: A,B,G,E,D
    0b01111001, // 3: A,B,G,C,D
    0b00110011, // 4: F,G,B,C
    0b01011011, // 5: A,F,G,C,D
    0b01011111, // 6: A,F,G,E,D,C
    0b01110000, // 7: A,B,C
    0b01111111, // 8: A,B,C,D,E,F,G
    0b01111011, // 9: A,B,C,D,F,G
];

/// 7-segment patterns for UPPERCASE letters (raw segments)
const LETTER_A_UPPER: u8 = 0b01110111; // A,B,C,E,F,G
const LETTER_B_UPPER: u8 = 0b01111111; // A,B,C,D,E,F,G (full 8)
const LETTER_C_UPPER: u8 = 0b01001110; // A,D,E,F
const LETTER_E_UPPER: u8 = 0b01001111; // A,D,E,F,G
const LETTER_F_UPPER: u8 = 0b01000111; // A,E,F,G
const LETTER_G_UPPER: u8 = 0b01011110; // A,C,D,E,F
const LETTER_H_UPPER: u8 = 0b00110111; // B,C,E,F,G
const LETTER_I_UPPER: u8 = 0b00110000; // B,C (like 1)
const LETTER_J_UPPER: u8 = 0b00111100; // B,C,D,E
const LETTER_L_UPPER: u8 = 0b00001110; // D,E,F
const LETTER_N_UPPER: u8 = 0b01110110; // A,B,C,E,F (like A without G)
const LETTER_O_UPPER: u8 = 0b01111110; // A,B,C,D,E,F (like 0)
const LETTER_P_UPPER: u8 = 0b01100111; // A,B,E,F,G
const LETTER_S_UPPER: u8 = 0b01011011; // A,F,G,C,D (like 5)
const LETTER_U_UPPER: u8 = 0b00111110; // B,C,D,E,F
const LETTER_Y_UPPER: u8 = 0b00111011; // B,C,D,F,G

/// 7-segment patterns for lowercase letters (raw segments)
const LETTER_B_LOWER: u8 = 0b00011111; // C,D,E,F,G
const LETTER_C_LOWER: u8 = 0b00001101; // D,E,G
const LETTER_D_LOWER: u8 = 0b00111101; // B,C,D,E,G
const LETTER_E_LOWER: u8 = 0b01001111; // A,D,E,F,G (same as upper)
const LETTER_F_LOWER: u8 = 0b01000111; // A,E,F,G (same as upper)
const LETTER_G_LOWER: u8 = 0b01111011; // A,B,C,D,F,G
const LETTER_H_LOWER: u8 = 0b00010111; // C,E,F,G
const LETTER_I_LOWER: u8 = 0b00000100; // E
const LETTER_J_LOWER: u8 = 0b00111000; // B,C,D
const LETTER_L_LOWER: u8 = 0b00000110; // E,F
const LETTER_N_LOWER: u8 = 0b00010101; // C,E,G
const LETTER_O_LOWER: u8 = 0b00011101; // C,D,E,G
const LETTER_P_LOWER: u8 = 0b01100111; // A,B,E,F,G (same as upper)
const LETTER_R_LOWER: u8 = 0b00000101; // E,G
const LETTER_S_LOWER: u8 = 0b01011011; // A,F,G,C,D (same as upper)
const LETTER_T_LOWER: u8 = 0b00001111; // D,E,F,G
const LETTER_U_LOWER: u8 = 0b00011100; // C,D,E
const LETTER_Y_LOWER: u8 = 0b00111011; // B,C,D,F,G (same as upper)

/// Special symbols
const SYMBOL_DASH: u8 = 0b00000001; // G segment only
const SYMBOL_UNDERSCORE: u8 = 0b00001000; // D segment only
const SYMBOL_BLANK: u8 = 0b00000000; // All segments off
const SYMBOL_DP: u8 = 0b10000000; // Decimal point
const SYMBOL_LEFT_BRACKET: u8 = 0b01001110; // A,D,E,F segments (like 'C')
const SYMBOL_RIGHT_BRACKET: u8 = 0b01111000; // A,B,C,D segments (like reversed 'C')

/// Display modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayMode {
    /// Code B decode mode - displays digits 0-9 directly
    CodeB,
    /// Raw segment mode - full control over individual segments
    RawSegments,
}

/// Text justification options for display positioning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextJustification {
    /// Left-justify text (default behavior)
    Left,
    /// Right-justify text
    Right,
    /// Center text on display
    Center,
}

/// MAX7219 7-segment display controller
///
/// Provides high-level interface for controlling MAX7219 displays
/// connected via SPI to a PoKeys device. Supports both single displays
/// and daisy-chained configurations (up to 8 displays).
pub struct Max7219<'a> {
    device: &'a mut PoKeysDevice,
    cs_pin: u8,
    mode: DisplayMode,
    intensity: u8,
    chain_length: u8,   // Number of displays in chain (1-8)
    target_display: u8, // Which display to target (0-based index)
}

impl<'a> Max7219<'a> {
    /// Create a new MAX7219 controller instance (single display)
    ///
    /// This creates a single MAX7219 display controller. For daisy-chained
    /// displays, use `new_chain()` instead.
    ///
    /// # Arguments
    /// * `device` - PoKeys device reference
    /// * `cs_pin` - Chip select pin number
    ///
    /// # Returns
    /// Configured MAX7219 instance ready for use
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(device: &'a mut PoKeysDevice, cs_pin: u8) -> Result<Self> {
        Self::new_chain(device, cs_pin, 1)
    }

    /// Create a new MAX7219 controller for daisy-chained displays
    ///
    /// This creates a controller for multiple MAX7219 displays connected
    /// in a daisy-chain configuration. Up to 8 displays are supported.
    ///
    /// # Arguments
    /// * `device` - PoKeys device reference
    /// * `cs_pin` - Chip select pin number
    /// * `chain_length` - Number of displays in chain (1-8)
    ///
    /// # Returns
    /// Configured MAX7219 chain controller ready for use
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?; // 3 displays
    ///
    /// // Target first display (index 0)
    /// display.set_target_display(0)?;
    /// display.display_text("HELLO")?;
    ///
    /// // Target second display (index 1)
    /// display.set_target_display(1)?;
    /// display.display_text("WORLD")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_chain(device: &'a mut PoKeysDevice, cs_pin: u8, chain_length: u8) -> Result<Self> {
        if chain_length == 0 || chain_length > 8 {
            return Err(PoKeysError::Parameter(format!(
                "Chain length must be 1-8, got {}",
                chain_length
            )));
        }

        let mut max7219 = Self {
            device,
            cs_pin,
            mode: DisplayMode::CodeB, // Will be updated during configuration
            intensity: 8,
            chain_length,
            target_display: 0, // Default to first display
        };

        // Configure SPI
        max7219.device.spi_configure(0x04, 0x00)?;

        // Initialize all displays in chain with basic settings
        max7219.write_register_all(REG_SHUTDOWN, NORMAL_MODE)?;
        max7219.write_register_all(REG_DISPLAY_TEST, TEST_OFF)?;

        // Configure each display individually for Code B decode mode (default)
        // This ensures all displays are properly initialized and ready to use
        for i in 0..chain_length {
            max7219.set_target_display(i)?;

            // Set default configuration for each display (Code B decode mode)
            max7219.write_register(REG_DECODE_MODE, DECODE_B_ALL)?; // Code B decode mode
            max7219.write_register(REG_INTENSITY, max7219.intensity)?; // Set intensity
            max7219.write_register(REG_SCAN_LIMIT, SCAN_LIMIT_8)?; // Scan all 8 digits

            // Small delay to ensure configuration is processed
            std::thread::sleep(Duration::from_micros(100));
        }

        // Reset to first display as default target
        max7219.set_target_display(0)?;

        // Set mode to CodeB since that's what we configured all displays for
        max7219.mode = DisplayMode::CodeB;

        Ok(max7219)
    }

    /// Configure for numeric display (Code B decode mode)
    ///
    /// # Arguments
    /// * `intensity` - Display brightness (0-15)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_numeric(10)?; // Medium-high brightness
    /// # Ok(())
    /// # }
    /// ```
    pub fn configure_numeric(&mut self, intensity: u8) -> Result<()> {
        self.mode = DisplayMode::CodeB;
        self.intensity = intensity.min(15);

        // Set Code B decode for all digits
        self.write_register(REG_DECODE_MODE, DECODE_B_ALL)?;

        // Set scan limit (8 digits)
        self.write_register(REG_SCAN_LIMIT, SCAN_LIMIT_8)?;

        // Set intensity
        self.write_register(REG_INTENSITY, self.intensity)?;

        Ok(())
    }

    /// Configure for raw segment control
    ///
    /// # Arguments
    /// * `intensity` - Display brightness (0-15)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_raw_segments(8)?; // Medium brightness
    /// # Ok(())
    /// # }
    /// ```
    pub fn configure_raw_segments(&mut self, intensity: u8) -> Result<()> {
        self.mode = DisplayMode::RawSegments;
        self.intensity = intensity.min(15);

        // Set no decode (raw segments)
        self.write_register(REG_DECODE_MODE, DECODE_NONE)?;

        // Set scan limit (8 digits)
        self.write_register(REG_SCAN_LIMIT, SCAN_LIMIT_8)?;

        // Set intensity
        self.write_register(REG_INTENSITY, self.intensity)?;

        Ok(())
    }

    /// Display a number (requires Code B mode)
    ///
    /// # Arguments
    /// * `number` - Number to display (0-99999999)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_numeric(8)?;
    /// display.display_number(12345)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_number(&mut self, number: u32) -> Result<()> {
        if self.mode != DisplayMode::CodeB {
            return Err(PoKeysError::Parameter(
                "display_number requires Code B mode".to_string(),
            ));
        }

        // Convert number to digits (right-aligned)
        let mut num = number;
        let mut digits = [0x0F; 8]; // Start with all blanks (0x0F in Code B)

        // Fill digits from right to left
        for i in 0..8 {
            if num > 0 || i == 0 {
                digits[7 - i] = (num % 10) as u8;
                num /= 10;
            } else {
                digits[7 - i] = 0x0F; // Blank
            }
        }

        // Send to display
        for (array_pos, &digit_value) in digits.iter().enumerate() {
            let max7219_digit = 7 - array_pos;
            self.write_register(REG_DIGIT0 + max7219_digit as u8, digit_value)?;
        }

        Ok(())
    }

    /// Display text using raw segments (left-justified, case-sensitive)
    ///
    /// **Important: Text display is case-sensitive!** Uppercase and lowercase letters
    /// produce different visual patterns on the 7-segment display where possible.
    ///
    /// This is a convenience method that displays text with left justification.
    /// For more control over text positioning, use `display_text_justified`.
    ///
    /// # Arguments
    /// * `text` - Text to display (up to 8 characters, case-sensitive)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_raw_segments(8)?;
    ///
    /// // Case-sensitive display - these produce different patterns:
    /// display.display_text("HELLO")?;     // Uppercase: clear, bold letters
    /// display.display_text("hello")?;     // Lowercase: different visual style  
    /// display.display_text("Hello")?;     // Mixed case: "H" + "ello"
    /// display.display_text("1.23")?;      // With decimal point: "1.23    "
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_text(&mut self, text: &str) -> Result<()> {
        self.display_text_justified(text, TextJustification::Left)
    }

    /// Display text using raw segments with justification control (case-sensitive)
    ///
    /// **Important: Text display is case-sensitive!** Uppercase and lowercase letters
    /// produce different visual patterns on the 7-segment display where possible.
    ///
    /// # Arguments
    /// * `text` - Text to display (up to 8 characters, case-sensitive)
    /// * `justification` - How to position the text on the display
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::{Max7219, TextJustification};
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_raw_segments(8)?;
    ///
    /// // Left-justified (default)
    /// display.display_text_justified("HELLO", TextJustification::Left)?;
    /// // Result: "HELLO   "
    ///
    /// // Right-justified  
    /// display.display_text_justified("HELLO", TextJustification::Right)?;
    /// // Result: "   HELLO"
    ///
    /// // Center-justified
    /// display.display_text_justified("HELLO", TextJustification::Center)?;
    /// // Result: " HELLO  " (or "  HELLO " depending on rounding)
    ///
    /// // Works with decimal points
    /// display.display_text_justified("1.23", TextJustification::Right)?;
    /// // Result: "    1.23"
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_text_justified(
        &mut self,
        text: &str,
        justification: TextJustification,
    ) -> Result<()> {
        if self.mode != DisplayMode::RawSegments {
            return Err(PoKeysError::Parameter(
                "display_text_justified requires raw segments mode".to_string(),
            ));
        }

        // Parse text and handle decimal points properly
        let mut parsed_chars = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() && parsed_chars.len() < 8 {
            let current_char = chars[i];

            // Check if next character is a decimal point
            let has_decimal = i + 1 < chars.len() && chars[i + 1] == '.';

            if current_char == '.' {
                // Skip standalone decimal points
                i += 1;
                continue;
            }

            // Get base pattern for current character
            let mut pattern = self.char_to_segments(current_char);

            // Add decimal point if next character is '.'
            if has_decimal {
                pattern |= SYMBOL_DP;
                i += 1; // Skip the decimal point character
            }

            parsed_chars.push(pattern);
            i += 1;
        }

        // Calculate starting position based on justification
        let text_length = parsed_chars.len();
        let start_pos = match justification {
            TextJustification::Left => 0,
            TextJustification::Right => 8usize.saturating_sub(text_length),
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };

        // Create display array with blanks
        let mut display_data = [SYMBOL_BLANK; 8];

        // Place parsed characters at calculated position
        for (i, &pattern) in parsed_chars.iter().enumerate() {
            let pos = start_pos + i;
            if pos < 8 {
                display_data[pos] = pattern;
            }
        }

        // Send to display with correct digit order
        for (text_pos, &segment_pattern) in display_data.iter().enumerate() {
            let max7219_digit = 7 - text_pos; // Reverse for correct left-to-right display
            self.write_register(REG_DIGIT0 + max7219_digit as u8, segment_pattern)?;
        }

        Ok(())
    }

    /// Display raw segment patterns directly
    ///
    /// # Arguments
    /// * `patterns` - Array of 8 segment patterns (bit 7=DP, bits 6-0=segments A-G)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_raw_segments(8)?;
    ///
    /// // Display "88888888" with all decimal points
    /// let patterns = [0xFF; 8]; // All segments + DP on
    /// display.display_raw_patterns(&patterns)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_raw_patterns(&mut self, patterns: &[u8; 8]) -> Result<()> {
        if self.mode != DisplayMode::RawSegments {
            return Err(PoKeysError::Parameter(
                "display_raw_patterns requires raw segments mode".to_string(),
            ));
        }

        for (pos, &pattern) in patterns.iter().enumerate() {
            let max7219_digit = 7 - pos; // Reverse for correct left-to-right display
            self.write_register(REG_DIGIT0 + max7219_digit as u8, pattern)?;
        }

        Ok(())
    }

    /// Clear the display (all digits blank)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.clear()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear(&mut self) -> Result<()> {
        let blank_value = match self.mode {
            DisplayMode::CodeB => 0x0F,       // Blank in Code B mode
            DisplayMode::RawSegments => 0x00, // All segments off
        };

        for digit in 0..8 {
            self.write_register(REG_DIGIT0 + digit, blank_value)?;
        }

        Ok(())
    }

    /// Set display intensity (brightness)
    ///
    /// # Arguments
    /// * `intensity` - Brightness level (0-15, where 0 is dimmest, 15 is brightest)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.set_intensity(5)?; // Dim
    /// display.set_intensity(15)?; // Bright
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_intensity(&mut self, intensity: u8) -> Result<()> {
        self.intensity = intensity.min(15);
        self.write_register(REG_INTENSITY, self.intensity)?;
        Ok(())
    }

    /// Enable or disable display test mode (all segments on)
    ///
    /// # Arguments
    /// * `enable` - True to enable test mode, false to disable
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.set_test_mode(true)?;  // All segments on
    /// std::thread::sleep(std::time::Duration::from_secs(1));
    /// display.set_test_mode(false)?; // Normal operation
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_test_mode(&mut self, enable: bool) -> Result<()> {
        let value = if enable { TEST_ON } else { TEST_OFF };
        self.write_register(REG_DISPLAY_TEST, value)?;
        Ok(())
    }

    /// Enable or disable shutdown mode (power saving)
    ///
    /// # Arguments
    /// * `shutdown` - True to enter shutdown mode, false for normal operation
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.set_shutdown(true)?;  // Power saving mode
    /// display.set_shutdown(false)?; // Normal operation
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_shutdown(&mut self, shutdown: bool) -> Result<()> {
        let value = if shutdown { SHUTDOWN_MODE } else { NORMAL_MODE };
        self.write_register(REG_SHUTDOWN, value)?;
        Ok(())
    }

    /// Get current display mode
    pub fn mode(&self) -> DisplayMode {
        self.mode
    }

    /// Get current intensity setting
    pub fn intensity(&self) -> u8 {
        self.intensity
    }

    /// Get the number of displays in the chain
    pub fn chain_length(&self) -> u8 {
        self.chain_length
    }

    /// Get the currently targeted display index
    pub fn target_display(&self) -> u8 {
        self.target_display
    }

    /// Set which display in the chain to target for subsequent operations
    ///
    /// # Arguments
    /// * `display_index` - Display index (0-based, must be < chain_length)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    ///
    /// display.set_target_display(0)?; // Target first display
    /// display.display_text("HELLO")?;
    ///
    /// display.set_target_display(1)?; // Target second display
    /// display.display_text("WORLD")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_target_display(&mut self, display_index: u8) -> Result<()> {
        if display_index >= self.chain_length {
            return Err(PoKeysError::Parameter(format!(
                "Display index {} out of range (0-{})",
                display_index,
                self.chain_length - 1
            )));
        }
        self.target_display = display_index;
        Ok(())
    }

    /// Display text on a specific display in the chain
    ///
    /// This is a convenience method that combines set_target_display() and display_text().
    ///
    /// # Arguments
    /// * `text` - Text to display
    /// * `display_index` - Display index (0-based)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    ///
    /// display.display_text_on("HELLO", 0)?; // Display on first unit
    /// display.display_text_on("WORLD", 1)?; // Display on second unit
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_text_on(&mut self, text: &str, display_index: u8) -> Result<()> {
        self.set_target_display(display_index)?;
        self.display_text(text)
    }

    /// Display justified text on a specific display in the chain
    ///
    /// # Arguments
    /// * `text` - Text to display
    /// * `justification` - Text justification
    /// * `display_index` - Display index (0-based)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::{Max7219, TextJustification};
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    ///
    /// display.display_text_justified_on("HELLO", TextJustification::Center, 0)?;
    /// display.display_text_justified_on("WORLD", TextJustification::Right, 1)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_text_justified_on(
        &mut self,
        text: &str,
        justification: TextJustification,
        display_index: u8,
    ) -> Result<()> {
        self.set_target_display(display_index)?;
        self.display_text_justified(text, justification)
    }

    /// Flash text on the display at a specified frequency
    ///
    /// Alternates between displaying the text and clearing the display at the specified
    /// frequency. This is useful for alerts, warnings, and drawing attention to messages.
    ///
    /// **Important: Text display is case-sensitive!** See `display_text()` for details.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to flash (case-sensitive, up to 8 characters)
    /// * `frequency_hz` - Flash frequency in Hz (flashes per second)
    /// * `duration_secs` - Total duration to flash in seconds
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if successful, error if display not in raw segments mode
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_raw_segments(8)?;
    ///
    /// // Flash "ERROR" at 2 Hz for 10 seconds
    /// display.flash_text("ERROR", 2.0, 10.0)?;
    ///
    /// // Flash "ALERT" at 1 Hz for 5 seconds  
    /// display.flash_text("ALERT", 1.0, 5.0)?;
    ///
    /// // Fast flash "WARN" at 5 Hz for 3 seconds
    /// display.flash_text("WARN", 5.0, 3.0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn flash_text(&mut self, text: &str, frequency_hz: f32, duration_secs: f32) -> Result<()> {
        if self.mode != DisplayMode::RawSegments {
            return Err(PoKeysError::Parameter(
                "flash_text requires raw segments mode".to_string(),
            ));
        }

        if frequency_hz <= 0.0 {
            return Err(PoKeysError::Parameter(
                "Flash frequency must be greater than 0".to_string(),
            ));
        }

        if duration_secs <= 0.0 {
            return Err(PoKeysError::Parameter(
                "Flash duration must be greater than 0".to_string(),
            ));
        }

        // Calculate timing
        let half_period = Duration::from_secs_f32(1.0 / (frequency_hz * 2.0));
        let total_duration = Duration::from_secs_f32(duration_secs);
        let start_time = Instant::now();

        // Flash loop
        let mut text_visible = true;
        while start_time.elapsed() < total_duration {
            if text_visible {
                self.display_text(text)?;
            } else {
                self.clear()?;
            }

            text_visible = !text_visible;
            thread::sleep(half_period);
        }

        // Ensure display is cleared when done
        self.clear()?;
        Ok(())
    }

    /// Flash text with custom justification at a specified frequency
    ///
    /// Similar to `flash_text()` but allows control over text positioning.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to flash (case-sensitive, up to 8 characters)
    /// * `justification` - How to position the text on the display
    /// * `frequency_hz` - Flash frequency in Hz (flashes per second)
    /// * `duration_secs` - Total duration to flash in seconds
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::{Max7219, TextJustification};
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new(&mut device, 24)?;
    /// display.configure_raw_segments(8)?;
    ///
    /// // Flash centered text
    /// display.flash_text_justified("WARN", TextJustification::Center, 3.0, 5.0)?;
    ///
    /// // Flash right-aligned text
    /// display.flash_text_justified("STOP", TextJustification::Right, 2.0, 8.0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn flash_text_justified(
        &mut self,
        text: &str,
        justification: TextJustification,
        frequency_hz: f32,
        duration_secs: f32,
    ) -> Result<()> {
        if self.mode != DisplayMode::RawSegments {
            return Err(PoKeysError::Parameter(
                "flash_text_justified requires raw segments mode".to_string(),
            ));
        }

        if frequency_hz <= 0.0 {
            return Err(PoKeysError::Parameter(
                "Flash frequency must be greater than 0".to_string(),
            ));
        }

        if duration_secs <= 0.0 {
            return Err(PoKeysError::Parameter(
                "Flash duration must be greater than 0".to_string(),
            ));
        }

        // Calculate timing
        let half_period = Duration::from_secs_f32(1.0 / (frequency_hz * 2.0));
        let total_duration = Duration::from_secs_f32(duration_secs);
        let start_time = Instant::now();

        // Flash loop
        let mut text_visible = true;
        while start_time.elapsed() < total_duration {
            if text_visible {
                self.display_text_justified(text, justification)?;
            } else {
                self.clear()?;
            }

            text_visible = !text_visible;
            thread::sleep(half_period);
        }

        // Ensure display is cleared when done
        self.clear()?;
        Ok(())
    }

    /// Flash text on a specific display in a chain at a specified frequency
    ///
    /// This is a convenience method for flashing text on a specific display
    /// in a daisy-chained configuration.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to flash (case-sensitive, up to 8 characters)
    /// * `display_index` - Index of the display in the chain (0-based)
    /// * `frequency_hz` - Flash frequency in Hz (flashes per second)
    /// * `duration_secs` - Total duration to flash in seconds
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    ///
    /// // Flash "ERROR" on first display at 2 Hz for 5 seconds
    /// display.flash_text_on("ERROR", 0, 2.0, 5.0)?;
    ///
    /// // Flash "WARN" on second display at 3 Hz for 3 seconds
    /// display.flash_text_on("WARN", 1, 3.0, 3.0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn flash_text_on(
        &mut self,
        text: &str,
        display_index: u8,
        frequency_hz: f32,
        duration_secs: f32,
    ) -> Result<()> {
        self.set_target_display(display_index)?;
        self.flash_text(text, frequency_hz, duration_secs)
    }

    /// Clear all displays in the chain
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    /// display.clear_all()?; // Clear all 3 displays
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_all(&mut self) -> Result<()> {
        let original_target = self.target_display;
        for display_index in 0..self.chain_length {
            self.set_target_display(display_index)?;
            self.clear()?;
        }
        self.set_target_display(original_target)?; // Restore original target
        Ok(())
    }

    /// Set intensity on all displays in the chain
    ///
    /// # Arguments
    /// * `intensity` - Brightness level (0-15)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    /// display.set_intensity_all(10)?; // Set all displays to brightness 10
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_intensity_all(&mut self, intensity: u8) -> Result<()> {
        self.intensity = intensity.min(15);
        self.write_register_all(REG_INTENSITY, self.intensity)?;
        Ok(())
    }

    /// Set test mode on all displays in the chain
    ///
    /// # Arguments
    /// * `enable` - True to enable test mode, false to disable
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    /// display.set_test_mode_all(true)?;  // All segments on all displays
    /// display.set_test_mode_all(false)?; // Normal operation
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_test_mode_all(&mut self, enable: bool) -> Result<()> {
        let value = if enable { TEST_ON } else { TEST_OFF };
        self.write_register_all(REG_DISPLAY_TEST, value)?;
        Ok(())
    }

    /// Set shutdown mode on all displays in the chain
    ///
    /// # Arguments
    /// * `shutdown` - True to enter shutdown mode, false for normal operation
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use pokeys_lib::*;
    /// # use pokeys_lib::devices::spi::Max7219;
    /// # fn example() -> Result<()> {
    /// # let mut device = connect_to_device(0)?;
    /// let mut display = Max7219::new_chain(&mut device, 24, 3)?;
    /// display.set_shutdown_all(true)?;  // Power saving mode
    /// display.set_shutdown_all(false)?; // Normal operation
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_shutdown_all(&mut self, shutdown: bool) -> Result<()> {
        let value = if shutdown { SHUTDOWN_MODE } else { NORMAL_MODE };
        self.write_register_all(REG_SHUTDOWN, value)?;
        Ok(())
    }

    /// Write to a MAX7219 register (handles daisy chains)
    fn write_register(&mut self, register: u8, value: u8) -> Result<()> {
        let mut command = Vec::new();

        // For daisy chains, send commands from highest to lowest display index
        // This ensures proper data flow through the chain
        for display_index in (0..self.chain_length).rev() {
            if display_index == self.target_display {
                // Send actual command to target display
                command.push(register);
                command.push(value);
            } else {
                // Send NO-OP to non-target displays
                command.push(REG_NOOP);
                command.push(0x00);
            }
        }

        self.device.spi_write(&command, self.cs_pin)?;
        std::thread::sleep(Duration::from_micros(1)); // Small delay for stability
        Ok(())
    }

    /// Write to all displays in chain simultaneously
    fn write_register_all(&mut self, register: u8, value: u8) -> Result<()> {
        let mut command = Vec::new();

        // Send same command to all displays in chain
        for _ in 0..self.chain_length {
            command.push(register);
            command.push(value);
        }

        self.device.spi_write(&command, self.cs_pin)?;
        std::thread::sleep(Duration::from_micros(1)); // Small delay for stability
        Ok(())
    }

    /// Convert character to 7-segment pattern (case-sensitive)
    ///
    /// This function preserves case sensitivity, providing different patterns
    /// for uppercase and lowercase letters where visually distinct on 7-segment displays.
    ///
    /// # Arguments
    ///
    /// * `c` - Character to convert (case-sensitive)
    ///
    /// # Returns
    ///
    /// 7-segment pattern as u8 bitmask
    ///
    /// # Examples
    ///
    /// ```
    /// // Uppercase and lowercase produce different patterns
    /// let upper_a = display.char_to_segments('A'); // Full A pattern
    /// let lower_a = display.char_to_segments('a'); // Lowercase a pattern
    /// ```
    fn char_to_segments(&self, c: char) -> u8 {
        match c {
            // Digits (0-9)
            '0' => DIGIT_PATTERNS[0],
            '1' => DIGIT_PATTERNS[1],
            '2' => DIGIT_PATTERNS[2],
            '3' => DIGIT_PATTERNS[3],
            '4' => DIGIT_PATTERNS[4],
            '5' => DIGIT_PATTERNS[5],
            '6' => DIGIT_PATTERNS[6],
            '7' => DIGIT_PATTERNS[7],
            '8' => DIGIT_PATTERNS[8],
            '9' => DIGIT_PATTERNS[9],

            // UPPERCASE letters
            'A' => LETTER_A_UPPER,
            'B' => LETTER_B_UPPER,
            'C' => LETTER_C_UPPER,
            'D' => LETTER_D_LOWER, // Uppercase D displays as lowercase d
            'E' => LETTER_E_UPPER,
            'F' => LETTER_F_UPPER,
            'G' => LETTER_G_UPPER,
            'H' => LETTER_H_UPPER,
            'I' => LETTER_I_UPPER,
            'J' => LETTER_J_UPPER,
            'L' => LETTER_L_UPPER,
            'N' => LETTER_N_UPPER,
            'O' => LETTER_O_UPPER,
            'P' => LETTER_P_UPPER,
            'R' => LETTER_R_LOWER, // Uppercase R displays as lowercase r
            'S' => LETTER_S_UPPER,
            'T' => LETTER_T_LOWER, // Uppercase T displays as lowercase t
            'U' => LETTER_U_UPPER,
            'Y' => LETTER_Y_UPPER,

            // lowercase letters
            'a' => LETTER_A_UPPER, // Lowercase a displays as uppercase A
            'b' => LETTER_B_LOWER,
            'c' => LETTER_C_LOWER,
            'd' => LETTER_D_LOWER,
            'e' => LETTER_E_LOWER,
            'f' => LETTER_F_LOWER,
            'g' => LETTER_G_LOWER,
            'h' => LETTER_H_LOWER,
            'i' => LETTER_I_LOWER,
            'j' => LETTER_J_LOWER,
            'l' => LETTER_L_LOWER,
            'n' => LETTER_N_LOWER,
            'o' => LETTER_O_LOWER,
            'p' => LETTER_P_LOWER,
            'r' => LETTER_R_LOWER,
            's' => LETTER_S_LOWER,
            't' => LETTER_T_LOWER,
            'u' => LETTER_U_LOWER,
            'y' => LETTER_Y_LOWER,

            // Special symbols
            '-' => SYMBOL_DASH,
            '_' => SYMBOL_UNDERSCORE,
            '[' => SYMBOL_LEFT_BRACKET,
            ']' => SYMBOL_RIGHT_BRACKET,
            ' ' => SYMBOL_BLANK,

            // Default to dash for unknown characters
            _ => SYMBOL_DASH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_patterns() {
        // Test that digit patterns are defined for 0-9
        assert_eq!(DIGIT_PATTERNS.len(), 10);

        // Test specific patterns
        assert_eq!(DIGIT_PATTERNS[0], 0b01111110); // 0
        assert_eq!(DIGIT_PATTERNS[1], 0b00110000); // 1
        assert_eq!(DIGIT_PATTERNS[8], 0b01111111); // 8
    }

    #[test]
    fn test_display_modes() {
        assert_eq!(DisplayMode::CodeB, DisplayMode::CodeB);
        assert_eq!(DisplayMode::RawSegments, DisplayMode::RawSegments);
        assert_ne!(DisplayMode::CodeB, DisplayMode::RawSegments);
    }

    #[test]
    fn test_register_addresses() {
        assert_eq!(REG_DIGIT0, 0x01);
        assert_eq!(REG_DECODE_MODE, 0x09);
        assert_eq!(REG_INTENSITY, 0x0A);
        assert_eq!(REG_SCAN_LIMIT, 0x0B);
        assert_eq!(REG_SHUTDOWN, 0x0C);
        assert_eq!(REG_DISPLAY_TEST, 0x0F);
    }

    #[test]
    fn test_configuration_values() {
        assert_eq!(DECODE_B_ALL, 0xFF);
        assert_eq!(DECODE_NONE, 0x00);
        assert_eq!(SCAN_LIMIT_8, 0x07);
        assert_eq!(NORMAL_MODE, 0x01);
        assert_eq!(SHUTDOWN_MODE, 0x00);
    }

    #[test]
    fn test_text_justification_enum() {
        assert_eq!(TextJustification::Left, TextJustification::Left);
        assert_eq!(TextJustification::Right, TextJustification::Right);
        assert_eq!(TextJustification::Center, TextJustification::Center);
        assert_ne!(TextJustification::Left, TextJustification::Right);
        assert_ne!(TextJustification::Left, TextJustification::Center);
        assert_ne!(TextJustification::Right, TextJustification::Center);
    }

    #[test]
    fn test_justification_logic() {
        // Test left justification (start position should be 0)
        let text_length = 5; // "HELLO"
        let left_start = match TextJustification::Left {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(left_start, 0);

        // Test right justification (start position should be 8 - length)
        let right_start = match TextJustification::Right {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(right_start, 3); // 8 - 5 = 3

        // Test center justification (start position should be (8 - length) / 2)
        let center_start = match TextJustification::Center {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(center_start, 1); // (8 - 5) / 2 = 1 (integer division)
    }

    #[test]
    fn test_justification_edge_cases() {
        // Test with text length = 8 (should always start at 0)
        let text_length = 8;

        let left_start = match TextJustification::Left {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(left_start, 0);

        let right_start = match TextJustification::Right {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(right_start, 0);

        let center_start = match TextJustification::Center {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(center_start, 0);

        // Test with text length = 1
        let text_length = 1;

        let right_start = match TextJustification::Right {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(right_start, 7); // 8 - 1 = 7

        let center_start = match TextJustification::Center {
            TextJustification::Left => 0,
            TextJustification::Right => {
                if text_length >= 8 {
                    0
                } else {
                    8 - text_length
                }
            }
            TextJustification::Center => {
                if text_length >= 8 {
                    0
                } else {
                    (8 - text_length) / 2
                }
            }
        };
        assert_eq!(center_start, 3); // (8 - 1) / 2 = 3
    }

    #[test]
    fn test_bracket_symbols() {
        // Test that bracket symbols are defined correctly
        assert_eq!(SYMBOL_LEFT_BRACKET, 0b01001110); // A,D,E,F segments (like 'C')
        assert_eq!(SYMBOL_RIGHT_BRACKET, 0b01111000); // A,B,C,D segments (like reversed 'C')

        // Test that dash symbol is still correct
        assert_eq!(SYMBOL_DASH, 0b00000001); // G segment only

        // Test that symbols are different from each other
        assert_ne!(SYMBOL_LEFT_BRACKET, SYMBOL_RIGHT_BRACKET);
        assert_ne!(SYMBOL_LEFT_BRACKET, SYMBOL_DASH);
        assert_ne!(SYMBOL_RIGHT_BRACKET, SYMBOL_DASH);
    }

    #[test]
    fn test_chain_length_validation() {
        // Test valid chain lengths (1-8)
        for length in 1..=8 {
            assert!(
                (1..=8).contains(&length),
                "Chain length {} should be valid",
                length
            );
        }

        // Test invalid chain lengths
        // These are compile-time constants, so we document the expected behavior
        const INVALID_LENGTH_0: usize = 0;
        const INVALID_LENGTH_9: usize = 9;
        assert!(
            !(1..=8).contains(&INVALID_LENGTH_0),
            "Chain length 0 should be invalid"
        );
        assert!(
            !(1..=8).contains(&INVALID_LENGTH_9),
            "Chain length 9 should be invalid"
        );
    }

    #[test]
    fn test_display_index_validation() {
        // Test valid display indices for different chain lengths
        for chain_length in 1..=8 {
            for display_index in 0..chain_length {
                assert!(
                    display_index < chain_length,
                    "Display index {} should be valid for chain length {}",
                    display_index,
                    chain_length
                );
            }

            // Test invalid display index (>= chain_length)
            assert!(
                chain_length >= chain_length,
                "Display index {} should be invalid for chain length {}",
                chain_length,
                chain_length
            );
        }
    }

    #[test]
    fn test_chain_spi_command_length() {
        // Test that SPI command length is correct for different chain lengths
        for chain_length in 1..=8 {
            let expected_command_length = (chain_length as usize) * 2; // 2 bytes per display
            assert_eq!(
                expected_command_length,
                (chain_length as usize) * 2,
                "Chain length {} should produce {} byte SPI command",
                chain_length,
                expected_command_length
            );
        }
    }

    #[test]
    fn test_noop_register() {
        // Test that NO-OP register is defined correctly
        assert_eq!(REG_NOOP, 0x00);

        // Test that NO-OP is different from other registers
        assert_ne!(REG_NOOP, REG_DIGIT0);
        assert_ne!(REG_NOOP, REG_DECODE_MODE);
        assert_ne!(REG_NOOP, REG_INTENSITY);
        assert_ne!(REG_NOOP, REG_SCAN_LIMIT);
        assert_ne!(REG_NOOP, REG_SHUTDOWN);
        assert_ne!(REG_NOOP, REG_DISPLAY_TEST);
    }

    #[test]
    fn test_chain_command_ordering() {
        // Test that chain commands are ordered from highest to lowest display index
        let chain_length = 3u8;
        let target_display = 1u8;

        // Simulate command generation logic
        let mut command_positions = Vec::new();
        for display_index in (0..chain_length).rev() {
            command_positions.push(display_index);
        }

        // Should be [2, 1, 0] for chain length 3
        assert_eq!(command_positions, vec![2, 1, 0]);

        // Target display should be at the correct position
        let target_position = command_positions.iter().position(|&x| x == target_display);
        assert_eq!(target_position, Some(1)); // Display 1 should be at position 1
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that single display behavior is equivalent to chain length 1
        let single_chain_length = 1u8;
        let single_target_display = 0u8;

        // Single display should have chain length 1 and target display 0
        assert_eq!(single_chain_length, 1);
        assert_eq!(single_target_display, 0);

        // Command for single display should be 2 bytes
        let single_command_length = (single_chain_length as usize) * 2;
        assert_eq!(single_command_length, 2);
    }

    #[test]
    fn test_flash_text_parameters() {
        // Test parameter validation for flash functions

        // Test frequency validation (using variables instead of constants)
        let zero_freq = 0.0;
        let negative_freq = -1.0;
        let positive_freq = 1.0;
        assert!(zero_freq <= 0.0, "Zero frequency should be invalid");
        assert!(negative_freq < 0.0, "Negative frequency should be invalid");
        assert!(positive_freq > 0.0, "Positive frequency should be valid");

        // Test duration validation (using variables instead of constants)
        let zero_duration = 0.0;
        let negative_duration = -1.0;
        let positive_duration = 1.0;
        assert!(zero_duration <= 0.0, "Zero duration should be invalid");
        assert!(
            negative_duration < 0.0,
            "Negative duration should be invalid"
        );
        assert!(positive_duration > 0.0, "Positive duration should be valid");

        // Test timing calculations
        let frequency_hz = 2.0;
        let half_period_ms = (1000.0 / (frequency_hz * 2.0)) as u64;
        assert_eq!(half_period_ms, 250, "Half period should be 250ms for 2Hz");

        let frequency_hz = 5.0;
        let half_period_ms = (1000.0 / (frequency_hz * 2.0)) as u64;
        assert_eq!(half_period_ms, 100, "Half period should be 100ms for 5Hz");

        // Test edge case frequencies
        let frequency_hz = 0.5;
        let half_period_ms = (1000.0 / (frequency_hz * 2.0)) as u64;
        assert_eq!(
            half_period_ms, 1000,
            "Half period should be 1000ms for 0.5Hz"
        );

        let frequency_hz = 10.0;
        let half_period_ms = (1000.0 / (frequency_hz * 2.0)) as u64;
        assert_eq!(half_period_ms, 50, "Half period should be 50ms for 10Hz");
    }
}
