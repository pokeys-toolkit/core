//! uSPIBridge protocol implementation
//!
//! This module provides uSPIBridge-specific functionality for PoKeys devices,
//! including custom segment mapping and display configuration for MAX7219 devices.

use crate::device::PoKeysDevice;
use crate::error::Result;
use crate::types::I2cStatus;

/// uSPIBridge-specific I2C commands for segment mapping and display control
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum USPIBridgeCommand {
    // Device Commands (0x10-0x2F)
    SetBrightness = 0x11,           // Set device brightness
    DisplayText = 0x20,             // Display text on device
    DisplayNumber = 0x21,           // Display number on device
    SetCharacter = 0x22,            // Set character at position
    SetPattern = 0x23,              // Set raw segment pattern
    SetDecimal = 0x24,              // Set decimal point
    ClearDevice = 0x25,             // Clear device
    
    // Segment Mapping Commands (0x26-0x2F) - Custom Pinout Feature
    SetSegmentMapping = 0x26,       // Set custom segment mapping array
    SetSegmentMappingType = 0x27,   // Set predefined segment mapping type
    GetSegmentMapping = 0x28,       // Get current segment mapping
    TestSegmentMapping = 0x29,      // Test segment mapping with pattern
    
    // Virtual Display Commands (0x40-0x4F)
    CreateVirtualDevice = 0x40,     // Create virtual device
    DeleteVirtualDevice = 0x41,     // Delete virtual device
    ListVirtualDevices = 0x42,      // List virtual devices
    VirtualText = 0x43,             // Virtual display text
    VirtualBrightness = 0x44,       // Virtual device brightness
    VirtualClear = 0x45,            // Clear virtual display
    VirtualScrollLeft = 0x46,       // Virtual scroll left
    VirtualScrollRight = 0x47,      // Virtual scroll right
    VirtualFlash = 0x48,            // Virtual flash
    VirtualStop = 0x49,             // Stop virtual effect
    
    // System Commands (0x50-0x5F)
    SystemReset = 0x50,             // Reset system/devices
    SystemStatus = 0x51,            // Get system status
    SystemConfig = 0x52,            // System configuration
}

/// Predefined segment mapping types for different display manufacturers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SegmentMappingType {
    /// Standard MAX7219 mapping (A=6, B=5, C=4, D=3, E=2, F=1, G=0, DP=7)
    Standard = 0,
    /// Completely reversed bit order mapping
    Reversed = 1,
    /// Common cathode display variant mapping
    CommonCathode = 2,
    /// SparkFun Serial 7-Segment Display mapping
    SparkfunSerial = 3,
    /// Adafruit LED Backpack mapping
    AdafruitBackpack = 4,
    /// User-defined custom mapping
    Custom = 255,
}

impl Default for SegmentMappingType {
    fn default() -> Self {
        Self::Standard
    }
}

/// Segment mapping configuration for custom 7-segment display wiring
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMapping {
    /// The type of segment mapping being used
    pub mapping_type: SegmentMappingType,
    /// Custom bit mapping array (only used when mapping_type is Custom)
    pub custom_mapping: Option<[u8; 8]>,
}

impl Default for SegmentMapping {
    fn default() -> Self {
        Self {
            mapping_type: SegmentMappingType::Standard,
            custom_mapping: None,
        }
    }
}

impl SegmentMapping {
    /// Create a new segment mapping with a predefined type
    pub fn new(mapping_type: SegmentMappingType) -> Self {
        Self {
            mapping_type,
            custom_mapping: None,
        }
    }
    
    /// Create a new segment mapping with a custom mapping array
    pub fn custom(mapping: [u8; 8]) -> Self {
        Self {
            mapping_type: SegmentMappingType::Custom,
            custom_mapping: Some(mapping),
        }
    }
    
    /// Check if this mapping uses a custom array
    pub fn is_custom(&self) -> bool {
        self.mapping_type == SegmentMappingType::Custom
    }
    
    /// Get the mapping array if it's custom
    pub fn get_custom_mapping(&self) -> Option<&[u8; 8]> {
        self.custom_mapping.as_ref()
    }
}

/// uSPIBridge configuration for multiple MAX7219 devices
#[derive(Debug, Clone)]
pub struct USPIBridgeConfig {
    /// Number of MAX7219 devices in the daisy chain
    pub device_count: u8,
    /// Segment mappings for each device
    pub segment_mappings: Vec<SegmentMapping>,
    /// Default brightness level (0-15)
    pub default_brightness: u8,
    /// Maximum number of virtual devices supported
    pub max_virtual_devices: u8,
}

impl Default for USPIBridgeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl USPIBridgeConfig {
    /// Create a new uSPIBridge configuration with default settings
    pub fn new() -> Self {
        Self {
            device_count: 5,
            segment_mappings: vec![SegmentMapping::default(); 8],
            default_brightness: 8,
            max_virtual_devices: 16,
        }
    }
    
    /// Set the number of devices in the daisy chain
    pub fn with_device_count(mut self, count: u8) -> Self {
        self.device_count = count;
        // Ensure we have enough segment mappings
        self.segment_mappings.resize(count as usize, SegmentMapping::default());
        self
    }
    
    /// Set the segment mapping for a specific device
    pub fn with_segment_mapping(mut self, device_id: usize, mapping: SegmentMapping) -> Self {
        if device_id < self.segment_mappings.len() {
            self.segment_mappings[device_id] = mapping;
        }
        self
    }
    
    /// Set the same segment mapping for all devices
    pub fn with_all_devices_segment_mapping(mut self, mapping: SegmentMapping) -> Self {
        for device_mapping in &mut self.segment_mappings {
            *device_mapping = mapping.clone();
        }
        self
    }
    
    /// Set the default brightness level
    pub fn with_default_brightness(mut self, brightness: u8) -> Self {
        self.default_brightness = brightness.min(15);
        self
    }
    
    /// Set the maximum number of virtual devices
    pub fn with_max_virtual_devices(mut self, max_devices: u8) -> Self {
        self.max_virtual_devices = max_devices;
        self
    }
}

/// uSPIBridge protocol implementation
impl PoKeysDevice {
    /// Write I2C command with proper uSPIBridge packet structure
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `command` - uSPIBridge command type
    /// * `device_id` - Target device ID (for device-specific commands)
    /// * `data` - Command payload data
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_write_command(
        &mut self,
        slave_address: u8,
        command: USPIBridgeCommand,
        device_id: u8,
        data: &[u8],
    ) -> Result<I2cStatus> {
        let mut packet = Vec::new();
        packet.push(command as u8);     // Command type
        packet.push(device_id);         // Device ID
        packet.push(data.len() as u8);  // Data length
        packet.extend_from_slice(data); // Data payload
        
        // Calculate XOR checksum
        let mut checksum = 0u8;
        for &byte in &packet {
            checksum ^= byte;
        }
        packet.push(checksum);
        
        self.i2c_write(slave_address, &packet)
    }

    /// Set custom segment mapping for a specific device
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `device_id` - Target MAX7219 device ID (0-based)
    /// * `mapping` - Array of 8 values mapping standard bits to custom bits
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_set_segment_mapping(
        &mut self,
        slave_address: u8,
        device_id: u8,
        mapping: &[u8; 8],
    ) -> Result<I2cStatus> {
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::SetSegmentMapping,
            device_id,
            mapping,
        )
    }

    /// Set predefined segment mapping type for a specific device
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `device_id` - Target MAX7219 device ID (0-based)
    /// * `mapping_type` - Predefined mapping type to use
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_set_segment_mapping_type(
        &mut self,
        slave_address: u8,
        device_id: u8,
        mapping_type: SegmentMappingType,
    ) -> Result<I2cStatus> {
        let data = vec![mapping_type as u8];
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::SetSegmentMappingType,
            device_id,
            &data,
        )
    }

    /// Get current segment mapping for a specific device
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `device_id` - Target MAX7219 device ID (0-based)
    ///
    /// # Returns
    /// Tuple of (I2C status, optional segment mapping array)
    pub fn uspibridge_get_segment_mapping(
        &mut self,
        slave_address: u8,
        device_id: u8,
    ) -> Result<(I2cStatus, Option<[u8; 8]>)> {
        // Send get mapping command
        let status = self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::GetSegmentMapping,
            device_id,
            &[],
        )?;
        
        if status != I2cStatus::Ok {
            return Ok((status, None));
        }
        
        // Wait for response processing
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Read response (expecting 8 bytes of mapping data)
        let (read_status, response_data) = self.i2c_read(slave_address, 10)?;
        
        if read_status == I2cStatus::Ok && response_data.len() >= 8 {
            let mut mapping = [0u8; 8];
            mapping.copy_from_slice(&response_data[0..8]);
            Ok((read_status, Some(mapping)))
        } else {
            Ok((read_status, None))
        }
    }

    /// Test segment mapping with a specific pattern
    ///
    /// This command displays a test pattern on the specified device to verify
    /// that the segment mapping is working correctly.
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `device_id` - Target MAX7219 device ID (0-based)
    /// * `test_pattern` - 8-bit pattern to display for testing
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_test_segment_mapping(
        &mut self,
        slave_address: u8,
        device_id: u8,
        test_pattern: u8,
    ) -> Result<I2cStatus> {
        let data = vec![test_pattern];
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::TestSegmentMapping,
            device_id,
            &data,
        )
    }

    /// Display text on a specific MAX7219 device
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `device_id` - Target MAX7219 device ID (0-based)
    /// * `text` - Text to display on the device
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_display_text(
        &mut self,
        slave_address: u8,
        device_id: u8,
        text: &str,
    ) -> Result<I2cStatus> {
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::DisplayText,
            device_id,
            text.as_bytes(),
        )
    }

    /// Set brightness for a specific MAX7219 device
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `device_id` - Target MAX7219 device ID (0-based)
    /// * `brightness` - Brightness level (0-15)
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_set_brightness(
        &mut self,
        slave_address: u8,
        device_id: u8,
        brightness: u8,
    ) -> Result<I2cStatus> {
        let data = vec![brightness.min(15)];
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::SetBrightness,
            device_id,
            &data,
        )
    }

    /// Clear a specific MAX7219 device
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `device_id` - Target MAX7219 device ID (0-based)
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_clear_device(
        &mut self,
        slave_address: u8,
        device_id: u8,
    ) -> Result<I2cStatus> {
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::ClearDevice,
            device_id,
            &[],
        )
    }

    /// Set text on a virtual display
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `virtual_id` - Virtual display ID (0-based)
    /// * `text` - Text to display
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_virtual_text(
        &mut self,
        slave_address: u8,
        virtual_id: u8,
        text: &str,
    ) -> Result<I2cStatus> {
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::VirtualText,
            virtual_id,
            text.as_bytes(),
        )
    }

    /// Start scrolling effect on a virtual display
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `virtual_id` - Virtual display ID (0-based)
    /// * `text` - Text to scroll
    /// * `speed_ms` - Scroll speed in milliseconds
    /// * `direction_left` - True for left scroll, false for right scroll
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_virtual_scroll(
        &mut self,
        slave_address: u8,
        virtual_id: u8,
        text: &str,
        speed_ms: u16,
        direction_left: bool,
    ) -> Result<I2cStatus> {
        let mut data = text.as_bytes().to_vec();
        data.extend_from_slice(&speed_ms.to_le_bytes());
        
        let command = if direction_left {
            USPIBridgeCommand::VirtualScrollLeft
        } else {
            USPIBridgeCommand::VirtualScrollRight
        };
        
        self.uspibridge_write_command(slave_address, command, virtual_id, &data)
    }

    /// Start flashing effect on a virtual display
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `virtual_id` - Virtual display ID (0-based)
    /// * `text` - Text to flash
    /// * `interval_ms` - Flash interval in milliseconds
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_virtual_flash(
        &mut self,
        slave_address: u8,
        virtual_id: u8,
        text: &str,
        interval_ms: u16,
    ) -> Result<I2cStatus> {
        let mut data = text.as_bytes().to_vec();
        data.extend_from_slice(&interval_ms.to_le_bytes());
        
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::VirtualFlash,
            virtual_id,
            &data,
        )
    }

    /// Stop effects on a virtual display
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `virtual_id` - Virtual display ID (0-based)
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_virtual_stop(
        &mut self,
        slave_address: u8,
        virtual_id: u8,
    ) -> Result<I2cStatus> {
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::VirtualStop,
            virtual_id,
            &[],
        )
    }

    /// Clear a virtual display
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    /// * `virtual_id` - Virtual display ID (0-based)
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_virtual_clear(
        &mut self,
        slave_address: u8,
        virtual_id: u8,
    ) -> Result<I2cStatus> {
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::VirtualClear,
            virtual_id,
            &[],
        )
    }

    /// Reset the uSPIBridge system
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    ///
    /// # Returns
    /// I2C operation status
    pub fn uspibridge_system_reset(&mut self, slave_address: u8) -> Result<I2cStatus> {
        self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::SystemReset,
            0,
            &[],
        )
    }

    /// Get uSPIBridge system status
    ///
    /// # Arguments
    /// * `slave_address` - I2C slave address of the uSPIBridge device
    ///
    /// # Returns
    /// Tuple of (I2C status, optional status data)
    pub fn uspibridge_system_status(
        &mut self,
        slave_address: u8,
    ) -> Result<(I2cStatus, Option<Vec<u8>>)> {
        let status = self.uspibridge_write_command(
            slave_address,
            USPIBridgeCommand::SystemStatus,
            0,
            &[],
        )?;
        
        if status != I2cStatus::Ok {
            return Ok((status, None));
        }
        
        // Wait for response processing
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Read response
        let (read_status, response_data) = self.i2c_read(slave_address, 16)?;
        Ok((read_status, Some(response_data)))
    }
}
