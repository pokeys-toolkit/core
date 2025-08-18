use pokeys_lib::{
    connect_to_device, enumerate_usb_devices, DeviceModel, PinFunction, PinModel, Result,
};
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_device_model_integration() -> Result<()> {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();
    let model_path = dir.path().join("TestDevice.yaml");

    // Create a test model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add pins with different capabilities
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string()],
            active: true,
        },
    );

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&model).unwrap();

    // Write to file
    fs::write(&model_path, yaml).unwrap();

    // Set the environment variable to use the test directory
    unsafe {
        std::env::set_var("POKEYS_MODEL_DIR", dir.path().to_string_lossy().to_string());
    }

    // This test will only run if a device is connected
    // Otherwise, it will be skipped
    if enumerate_usb_devices().unwrap_or(0) > 0 {
        let mut device = connect_to_device(0)?;

        // Get device data (this should load the model)
        device.get_device_data()?;

        // Override the model with our test model
        device.model = Some(model);

        // Test pin capability checking
        assert!(device.is_pin_capability_supported(1, "DigitalInput"));
        assert!(device.is_pin_capability_supported(1, "DigitalOutput"));
        assert!(!device.is_pin_capability_supported(1, "AnalogInput"));

        assert!(device.is_pin_capability_supported(2, "DigitalInput"));
        assert!(!device.is_pin_capability_supported(2, "DigitalOutput"));

        // Test getting pin capabilities
        let pin1_caps = device.get_pin_capabilities(1);
        assert_eq!(pin1_caps.len(), 2);
        assert!(pin1_caps.contains(&"DigitalInput".to_string()));
        assert!(pin1_caps.contains(&"DigitalOutput".to_string()));

        let pin2_caps = device.get_pin_capabilities(2);
        assert_eq!(pin2_caps.len(), 1);
        assert!(pin2_caps.contains(&"DigitalInput".to_string()));

        // Test setting pin functions
        // Pin 1 supports DigitalOutput, so this should succeed
        let result = device.set_pin_function(1, PinFunction::DigitalOutput);
        assert!(result.is_ok());

        // Pin 2 doesn't support DigitalOutput, so this should fail
        let result = device.set_pin_function(2, PinFunction::DigitalOutput);
        assert!(result.is_err());

        // Check that the error is the expected type
        if let Err(e) = result {
            assert!(format!("{e}").contains("does not support capability"));
        }
    }

    Ok(())
}
