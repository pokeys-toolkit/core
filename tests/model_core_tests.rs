use pokeys_lib::models::{DeviceModel, PinModel};
use std::collections::HashMap;

#[test]
fn test_device_model_creation() {
    // Create a model
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };

    // Add pins
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
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: true,
        },
    );

    // Check model properties
    assert_eq!(model.name, "TestModel");
    assert_eq!(model.pins.len(), 2);

    // Check pin 1
    let pin1 = model.pins.get(&1).unwrap();
    assert_eq!(pin1.capabilities.len(), 2);
    assert!(pin1.capabilities.contains(&"DigitalInput".to_string()));
    assert!(pin1.capabilities.contains(&"DigitalOutput".to_string()));
    assert!(pin1.active);

    // Check pin 2
    let pin2 = model.pins.get(&2).unwrap();
    assert_eq!(pin2.capabilities.len(), 2);
    assert!(pin2.capabilities.contains(&"DigitalInput".to_string()));
    assert!(pin2.capabilities.contains(&"AnalogInput".to_string()));
    assert!(pin2.active);
}

#[test]
fn test_device_model_validation() {
    // Create a valid model
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };

    // Add pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Validate the model
    assert!(model.validate().is_ok());

    // Test empty name
    let mut invalid_model = model.clone();
    invalid_model.name = "".to_string();
    assert!(invalid_model.validate().is_err());

    // Test empty pins
    let invalid_model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };
    assert!(invalid_model.validate().is_err());

    // Test pin with no capabilities
    let mut invalid_model = model.clone();
    invalid_model.pins.insert(
        2,
        PinModel {
            capabilities: vec![],
            active: true,
        },
    );
    assert!(invalid_model.validate().is_err());
}

#[test]
fn test_pin_capability_checking() {
    // Create a model
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };

    // Add pins
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
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: true,
        },
    );

    // Check supported capabilities
    assert!(model.is_pin_capability_supported(1, "DigitalInput"));
    assert!(model.is_pin_capability_supported(1, "DigitalOutput"));
    assert!(!model.is_pin_capability_supported(1, "AnalogInput"));

    assert!(model.is_pin_capability_supported(2, "DigitalInput"));
    assert!(model.is_pin_capability_supported(2, "AnalogInput"));
    assert!(!model.is_pin_capability_supported(2, "DigitalOutput"));

    // Check non-existent pin
    assert!(!model.is_pin_capability_supported(3, "DigitalInput"));

    // Check get_pin_capabilities
    let pin1_caps = model.get_pin_capabilities(1);
    assert_eq!(pin1_caps.len(), 2);
    assert!(pin1_caps.contains(&"DigitalInput".to_string()));
    assert!(pin1_caps.contains(&"DigitalOutput".to_string()));

    // Check non-existent pin
    let pin3_caps = model.get_pin_capabilities(3);
    assert_eq!(pin3_caps.len(), 0);
}

#[test]
fn test_related_capabilities() {
    // Create a model with encoder pins
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };

    // Add encoder pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "Encoder_1A".to_string()],
            active: true,
        },
    );

    model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "Encoder_1B".to_string()],
            active: true,
        },
    );

    // Validate the model
    assert!(model.validate().is_ok());

    // Check related capabilities
    let related = model.get_related_capabilities(1, "Encoder_1A");
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].0, "Encoder_1B");
    assert_eq!(related[0].1, 2);

    // Check validate_pin_capability
    assert!(model.validate_pin_capability(1, "Encoder_1A").is_ok());
    assert!(model.validate_pin_capability(2, "Encoder_1B").is_ok());

    // Test with missing related capability
    let mut invalid_model = model.clone();
    invalid_model.pins.get_mut(&2).unwrap().capabilities = vec!["DigitalInput".to_string()];

    // The model should fail validation due to our enhanced validation
    assert!(invalid_model.validate().is_err());
}

#[test]
fn test_matrix_keyboard_validation() {
    // Create a model with matrix keyboard pins
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };

    // Add matrix keyboard pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec![
                "DigitalInput".to_string(),
                "MatrixKeyboard_Row1".to_string(),
            ],
            active: true,
        },
    );

    model.pins.insert(
        2,
        PinModel {
            capabilities: vec![
                "DigitalInput".to_string(),
                "MatrixKeyboard_Col1".to_string(),
            ],
            active: true,
        },
    );

    // Validate the model
    assert!(model.validate().is_ok());

    // Check related capabilities
    let related = model.get_related_capabilities(1, "MatrixKeyboard_Row1");
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].0, "MatrixKeyboard_Col1");
    assert_eq!(related[0].1, 2);

    // Test with missing columns
    let mut invalid_model = model.clone();
    invalid_model.pins.remove(&2);

    // The model should fail validation
    assert!(invalid_model.validate().is_err());
}

#[test]
fn test_pwm_channel_validation() {
    // Create a model with PWM pins
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };

    // Add PWM pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "PWM_1".to_string()],
            active: true,
        },
    );

    model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "PWM_2".to_string()],
            active: true,
        },
    );

    // Validate the model
    assert!(model.validate().is_ok());

    // Test with non-sequential PWM channels
    let mut invalid_model = model.clone();
    invalid_model.pins.get_mut(&2).unwrap().capabilities =
        vec!["DigitalInput".to_string(), "PWM_3".to_string()];

    // The model should fail validation
    assert!(invalid_model.validate().is_err());
}

#[test]
fn test_inactive_pins() {
    // Create a model with inactive pins
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
        pins: HashMap::new(),
    };

    // Add pins
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
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: false,
        },
    );

    // Validate the model
    assert!(model.validate().is_ok());

    // Check capability validation
    assert!(model.validate_pin_capability(1, "DigitalInput").is_ok());
    assert!(model.validate_pin_capability(1, "DigitalOutput").is_ok());

    // Inactive pins should still validate their capabilities
    assert!(model.validate_pin_capability(2, "DigitalInput").is_ok());
    assert!(model.validate_pin_capability(2, "AnalogInput").is_ok());

    // But related capabilities should fail if the related pin is inactive
    let mut encoder_model = DeviceModel {
        name: "EncoderModel".to_string(),
        pins: HashMap::new(),
    };

    encoder_model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "Encoder_1A".to_string()],
            active: true,
        },
    );

    encoder_model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "Encoder_1B".to_string()],
            active: false,
        },
    );

    // The model should validate
    assert!(encoder_model.validate().is_ok());

    // But validate_pin_capability should fail for Encoder_1A because Encoder_1B is inactive
    assert!(
        encoder_model
            .validate_pin_capability(1, "Encoder_1A")
            .is_err()
    );
}
