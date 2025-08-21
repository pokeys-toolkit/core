//! Integration tests for uSPIBridge functionality

use pokeys_lib::{
    SegmentMapping, SegmentMappingType, USPIBridgeCommand, USPIBridgeConfig,
};

#[test]
fn test_segment_mapping_creation() {
    // Test default segment mapping
    let default_mapping = SegmentMapping::default();
    assert_eq!(default_mapping.mapping_type, SegmentMappingType::Standard);
    assert!(!default_mapping.is_custom());
    assert!(default_mapping.get_custom_mapping().is_none());

    // Test predefined mapping
    let reversed_mapping = SegmentMapping::new(SegmentMappingType::Reversed);
    assert_eq!(reversed_mapping.mapping_type, SegmentMappingType::Reversed);
    assert!(!reversed_mapping.is_custom());

    // Test custom mapping
    let custom_array = [7, 6, 5, 4, 3, 2, 1, 0];
    let custom_mapping = SegmentMapping::custom(custom_array);
    assert_eq!(custom_mapping.mapping_type, SegmentMappingType::Custom);
    assert!(custom_mapping.is_custom());
    assert_eq!(custom_mapping.get_custom_mapping(), Some(&custom_array));
}

#[test]
fn test_uspibridge_config() {
    // Test default configuration
    let default_config = USPIBridgeConfig::default();
    assert_eq!(default_config.device_count, 5);
    assert_eq!(default_config.default_brightness, 8);
    assert_eq!(default_config.max_virtual_devices, 16);
    assert_eq!(default_config.segment_mappings.len(), 8);

    // Test builder pattern
    let custom_config = USPIBridgeConfig::new()
        .with_device_count(3)
        .with_default_brightness(10)
        .with_max_virtual_devices(8)
        .with_segment_mapping(0, SegmentMapping::new(SegmentMappingType::Reversed))
        .with_all_devices_segment_mapping(SegmentMapping::new(SegmentMappingType::CommonCathode));

    assert_eq!(custom_config.device_count, 3);
    assert_eq!(custom_config.default_brightness, 10);
    assert_eq!(custom_config.max_virtual_devices, 8);
    assert_eq!(custom_config.segment_mappings.len(), 3);
    
    // All devices should have CommonCathode mapping (set last)
    for mapping in &custom_config.segment_mappings {
        assert_eq!(mapping.mapping_type, SegmentMappingType::CommonCathode);
    }
}

#[test]
fn test_segment_mapping_types() {
    // Test all predefined mapping types
    let standard = SegmentMappingType::Standard;
    let reversed = SegmentMappingType::Reversed;
    let common_cathode = SegmentMappingType::CommonCathode;
    let sparkfun = SegmentMappingType::SparkfunSerial;
    let adafruit = SegmentMappingType::AdafruitBackpack;
    let custom = SegmentMappingType::Custom;

    // Test that they have different values
    assert_ne!(standard as u8, reversed as u8);
    assert_ne!(standard as u8, common_cathode as u8);
    assert_ne!(standard as u8, sparkfun as u8);
    assert_ne!(standard as u8, adafruit as u8);
    assert_ne!(standard as u8, custom as u8);

    // Test default
    assert_eq!(SegmentMappingType::default(), SegmentMappingType::Standard);
}

#[test]
fn test_uspibridge_commands() {
    // Test that command values match the expected protocol
    assert_eq!(USPIBridgeCommand::SetBrightness as u8, 0x11);
    assert_eq!(USPIBridgeCommand::DisplayText as u8, 0x20);
    assert_eq!(USPIBridgeCommand::SetSegmentMapping as u8, 0x26);
    assert_eq!(USPIBridgeCommand::SetSegmentMappingType as u8, 0x27);
    assert_eq!(USPIBridgeCommand::GetSegmentMapping as u8, 0x28);
    assert_eq!(USPIBridgeCommand::TestSegmentMapping as u8, 0x29);
    assert_eq!(USPIBridgeCommand::VirtualText as u8, 0x43);
    assert_eq!(USPIBridgeCommand::VirtualScrollLeft as u8, 0x46);
    assert_eq!(USPIBridgeCommand::VirtualFlash as u8, 0x48);
    assert_eq!(USPIBridgeCommand::SystemReset as u8, 0x50);
}

#[test]
fn test_segment_mapping_equality() {
    let mapping1 = SegmentMapping::new(SegmentMappingType::Standard);
    let mapping2 = SegmentMapping::new(SegmentMappingType::Standard);
    let mapping3 = SegmentMapping::new(SegmentMappingType::Reversed);

    assert_eq!(mapping1, mapping2);
    assert_ne!(mapping1, mapping3);

    let custom1 = SegmentMapping::custom([1, 2, 3, 4, 5, 6, 7, 8]);
    let custom2 = SegmentMapping::custom([1, 2, 3, 4, 5, 6, 7, 8]);
    let custom3 = SegmentMapping::custom([8, 7, 6, 5, 4, 3, 2, 1]);

    assert_eq!(custom1, custom2);
    assert_ne!(custom1, custom3);
    assert_ne!(mapping1, custom1);
}

#[cfg(test)]
mod integration_tests {
    // Note: These tests would require actual hardware to run
    // They are included as examples of how to use the API

    #[test]
    #[ignore] // Requires hardware
    fn test_uspibridge_segment_mapping_integration() {
        // This test would require a real PoKeys device and uSPIBridge
        // Example usage:
        /*
        let mut device = connect_to_device_with_serial(32223).unwrap();
        let slave_address = 0x42;
        
        // Set custom segment mapping
        let custom_mapping = [7, 6, 5, 4, 3, 2, 1, 0]; // Reversed
        let result = device.uspibridge_set_segment_mapping(slave_address, 0, &custom_mapping);
        assert!(result.is_ok());
        
        // Test the mapping
        let result = device.uspibridge_test_segment_mapping(slave_address, 0, 0xFF);
        assert!(result.is_ok());
        */
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_uspibridge_virtual_device_integration() {
        // This test would require a real PoKeys device and uSPIBridge
        // Example usage:
        /*
        let mut device = connect_to_device_with_serial(32223).unwrap();
        let slave_address = 0x42;
        
        // Display text on virtual device
        let result = device.uspibridge_virtual_text(slave_address, 0, "HELLO");
        assert!(result.is_ok());
        
        // Start scrolling effect
        let result = device.uspibridge_virtual_scroll(slave_address, 0, "SCROLLING TEXT", 500, true);
        assert!(result.is_ok());
        
        // Stop effect
        let result = device.uspibridge_virtual_stop(slave_address, 0);
        assert!(result.is_ok());
        */
    }
}
