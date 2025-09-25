use pokeys_lib::models::{DeviceModel, PinModel, get_default_model_dir, load_model};
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_device_model_creation() {
    // Create a simple model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
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
    assert_eq!(model.name, "TestDevice");
    assert_eq!(model.pins.len(), 2);
    assert!(model.pins.contains_key(&1));
    assert!(model.pins.contains_key(&2));

    // Check pin capabilities
    let pin1 = model.pins.get(&1).unwrap();
    assert_eq!(pin1.capabilities.len(), 2);
    assert!(pin1.capabilities.contains(&"DigitalInput".to_string()));
    assert!(pin1.capabilities.contains(&"DigitalOutput".to_string()));

    let pin2 = model.pins.get(&2).unwrap();
    assert_eq!(pin2.capabilities.len(), 2);
    assert!(pin2.capabilities.contains(&"DigitalInput".to_string()));
    assert!(pin2.capabilities.contains(&"AnalogInput".to_string()));
}

#[test]
fn test_device_model_validation() {
    // Create a valid model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
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

    // Validate the model
    assert!(model.validate().is_ok());

    // Test empty name
    let mut invalid_model = model.clone();
    invalid_model.name = "".to_string();
    assert!(invalid_model.validate().is_err());

    // Test empty pins
    let invalid_model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };
    assert!(invalid_model.validate().is_err());

    // Test pin with no capabilities
    let mut invalid_model = model.clone();
    invalid_model.pins.insert(
        3,
        PinModel {
            capabilities: vec![],
            active: true,
        },
    );
    assert!(invalid_model.validate().is_err());
}

#[test]
fn test_yaml_serialization() {
    // Create a model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
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

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&model).unwrap();

    // Deserialize from YAML
    let deserialized: DeviceModel = serde_yaml::from_str(&yaml).unwrap();

    // Check that the models are equal
    assert_eq!(model.name, deserialized.name);
    assert_eq!(model.pins.len(), deserialized.pins.len());

    for (pin_num, pin) in &model.pins {
        let deserialized_pin = deserialized.pins.get(pin_num).unwrap();
        assert_eq!(pin.capabilities, deserialized_pin.capabilities);
        assert_eq!(pin.active, deserialized_pin.active);
    }
}

#[test]
fn test_model_file_loading() {
    // Create a temporary directory
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("TestDevice.yaml");

    // Create a model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
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

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&model).unwrap();

    // Write to file
    fs::write(&file_path, yaml).unwrap();

    // Load the model
    let loaded_model = DeviceModel::from_file(&file_path).unwrap();

    // Check that the models are equal
    assert_eq!(model.name, loaded_model.name);
    assert_eq!(model.pins.len(), loaded_model.pins.len());

    for (pin_num, pin) in &model.pins {
        let loaded_pin = loaded_model.pins.get(pin_num).unwrap();
        assert_eq!(pin.capabilities, loaded_pin.capabilities);
        assert_eq!(pin.active, loaded_pin.active);
    }
}

#[test]
fn test_related_capabilities() {
    // Create a model with encoder pins
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
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

    // Test related capabilities
    let related = model.get_related_capabilities(1, "Encoder_1A");
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].0, "Encoder_1B");
    assert_eq!(related[0].1, 2);

    // Test missing related capability
    let mut invalid_model = model.clone();
    invalid_model.pins.get_mut(&2).unwrap().capabilities = vec!["DigitalInput".to_string()];

    // The model should fail validation with our enhanced validation
    assert!(invalid_model.validate().is_err());

    // And there should be no related capabilities
    let related = invalid_model.get_related_capabilities(1, "Encoder_1A");
    assert_eq!(related.len(), 0);
}

#[test]
fn test_pin_capability_validation() {
    // Create a model with various capabilities
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
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: true,
        },
    );

    // Test supported capabilities
    assert!(model.is_pin_capability_supported(1, "DigitalInput"));
    assert!(model.is_pin_capability_supported(1, "DigitalOutput"));
    assert!(!model.is_pin_capability_supported(1, "AnalogInput"));

    assert!(model.is_pin_capability_supported(2, "DigitalInput"));
    assert!(model.is_pin_capability_supported(2, "AnalogInput"));
    assert!(!model.is_pin_capability_supported(2, "DigitalOutput"));

    // Test non-existent pin
    assert!(!model.is_pin_capability_supported(3, "DigitalInput"));

    // Test validate_pin_capability
    assert!(model.validate_pin_capability(1, "DigitalInput").is_ok());
    assert!(model.validate_pin_capability(1, "DigitalOutput").is_ok());
    assert!(model.validate_pin_capability(2, "DigitalInput").is_ok());
    assert!(model.validate_pin_capability(2, "AnalogInput").is_ok());

    // Test unsupported capabilities
    assert!(model.validate_pin_capability(1, "AnalogInput").is_err());
    assert!(model.validate_pin_capability(2, "DigitalOutput").is_err());
    assert!(model.validate_pin_capability(3, "DigitalInput").is_err());
}

#[test]
fn test_get_pin_capabilities() {
    // Create a model with various capabilities
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
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: true,
        },
    );

    // Test get_pin_capabilities
    let pin1_caps = model.get_pin_capabilities(1);
    assert_eq!(pin1_caps.len(), 2);
    assert!(pin1_caps.contains(&"DigitalInput".to_string()));
    assert!(pin1_caps.contains(&"DigitalOutput".to_string()));

    let pin2_caps = model.get_pin_capabilities(2);
    assert_eq!(pin2_caps.len(), 2);
    assert!(pin2_caps.contains(&"DigitalInput".to_string()));
    assert!(pin2_caps.contains(&"AnalogInput".to_string()));

    // Test non-existent pin
    let pin3_caps = model.get_pin_capabilities(3);
    assert_eq!(pin3_caps.len(), 0);
}

#[test]
fn test_default_model_dir() {
    // Just check that it returns a path
    let dir = get_default_model_dir();
    assert!(dir.to_string_lossy().contains(".config/pokeys/models"));
}

#[test]
fn test_load_model_with_custom_dir() {
    // Create a temporary directory
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("TestDevice.yaml");

    // Create a model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&model).unwrap();

    // Write to file
    fs::write(&file_path, yaml).unwrap();

    // Load the model with custom directory
    let loaded_model = load_model("TestDevice", Some(dir.path())).unwrap();

    // Check that the models are equal
    assert_eq!(model.name, loaded_model.name);
    assert_eq!(model.pins.len(), loaded_model.pins.len());
}
#[test]
fn test_enhanced_validation() {
    // Create a model with encoder pins
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
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

    // Test validate_pin_capability with encoder pins
    assert!(model.validate_pin_capability(1, "Encoder_1A").is_ok());
    assert!(model.validate_pin_capability(2, "Encoder_1B").is_ok());

    // Test with missing related capability
    let mut invalid_model = model.clone();
    invalid_model.pins.get_mut(&2).unwrap().capabilities = vec!["DigitalInput".to_string()];

    // The model should fail validation with our enhanced validation
    assert!(invalid_model.validate().is_err());

    // Create a new model with encoder pins but missing the B pin
    let mut test_model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add encoder pins
    test_model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "Encoder_1A".to_string()],
            active: true,
        },
    );

    // This model is missing the Encoder_1B pin
    // validate_pin_capability should fail for Encoder_1A
    assert!(test_model.validate_pin_capability(1, "Encoder_1A").is_err());

    // Test with inactive related pin
    let mut inactive_model = model.clone();
    inactive_model.pins.get_mut(&2).unwrap().active = false;

    // The model should still validate
    assert!(inactive_model.validate().is_ok());

    // But validate_pin_capability should fail
    assert!(
        inactive_model
            .validate_pin_capability(1, "Encoder_1A")
            .is_err()
    );

    // Test matrix keyboard validation
    let mut matrix_model = DeviceModel {
        name: "MatrixKeyboard".to_string(),
        pins: HashMap::new(),
    };

    // Add matrix keyboard pins
    matrix_model.pins.insert(
        1,
        PinModel {
            capabilities: vec![
                "DigitalInput".to_string(),
                "MatrixKeyboard_Row1".to_string(),
            ],
            active: true,
        },
    );

    matrix_model.pins.insert(
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
    assert!(matrix_model.validate().is_ok());

    // Test validate_pin_capability with matrix keyboard pins
    assert!(
        matrix_model
            .validate_pin_capability(1, "MatrixKeyboard_Row1")
            .is_ok()
    );
    assert!(
        matrix_model
            .validate_pin_capability(2, "MatrixKeyboard_Col1")
            .is_ok()
    );

    // Test with missing columns
    let mut invalid_matrix = matrix_model.clone();
    invalid_matrix.pins.remove(&2);

    // The model should fail validation
    assert!(invalid_matrix.validate().is_err());

    // And validate_pin_capability should fail
    assert!(
        invalid_matrix
            .validate_pin_capability(1, "MatrixKeyboard_Row1")
            .is_err()
    );

    // Test PWM channel validation
    let mut pwm_model = DeviceModel {
        name: "PWMDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add PWM pins
    pwm_model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "PWM_1".to_string()],
            active: true,
        },
    );

    pwm_model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "PWM_2".to_string()],
            active: true,
        },
    );

    // Validate the model
    assert!(pwm_model.validate().is_ok());

    // Test with non-sequential PWM channels
    let mut invalid_pwm = pwm_model.clone();
    invalid_pwm.pins.get_mut(&2).unwrap().capabilities =
        vec!["DigitalInput".to_string(), "PWM_3".to_string()];

    // The model should fail validation
    assert!(invalid_pwm.validate().is_err());
}
