use pokeys_lib::model_manager::ModelManager;
use pokeys_lib::models::{DeviceModel, PinModel};
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_model_manager_initialization() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let manager = ModelManager::new(Some(dir.path().to_path_buf()));
    assert!(manager.is_ok(), "Failed to create model manager");

    // Check that the model directory was created
    assert!(dir.path().exists());
    assert!(dir.path().is_dir());

    // Check that the default models were copied
    let default_models = [
        "PoKeys56U.yaml",
        "PoKeys57U.yaml",
        "PoKeys56E.yaml",
        "PoKeys57E.yaml",
    ];

    for model_file in &default_models {
        let file_path = dir.path().join(model_file);
        assert!(
            file_path.exists(),
            "Default model file {model_file} was not copied"
        );
    }
}

#[test]
fn test_model_manager_create_model() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let mut manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Create a model
    let mut pins = HashMap::new();
    pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: true,
        },
    );

    // Create the model
    assert!(manager.create_model("TestModel", pins).is_ok());

    // Check that the model file was created
    let file_path = dir.path().join("TestModel.yaml");
    assert!(file_path.exists(), "Model file was not created");

    // Check that the model was loaded
    let model = manager.get_model("TestModel");
    assert!(model.is_some(), "Model was not loaded");

    let model = model.unwrap();
    assert_eq!(model.name, "TestModel");
    assert_eq!(model.pins.len(), 2);
    assert!(model.pins.contains_key(&1));
    assert!(model.pins.contains_key(&2));
}

#[test]
fn test_model_manager_copy_model() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let mut manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Create a model
    let mut pins = HashMap::new();
    pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Create the model
    assert!(manager.create_model("SourceModel", pins).is_ok());

    // Copy the model
    assert!(manager.copy_model("SourceModel", "TargetModel").is_ok());

    // Check that the target model file was created
    let file_path = dir.path().join("TargetModel.yaml");
    assert!(file_path.exists(), "Target model file was not created");

    // Check that the target model was loaded
    let model = manager.get_model("TargetModel");
    assert!(model.is_some(), "Target model was not loaded");

    let model = model.unwrap();
    assert_eq!(model.name, "TargetModel");
    assert_eq!(model.pins.len(), 1);
    assert!(model.pins.contains_key(&1));

    // Check that copying a non-existent model fails
    assert!(manager.copy_model("NonExistentModel", "NewModel").is_err());
}

#[test]
fn test_model_manager_delete_model() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let mut manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Create a model
    let mut pins = HashMap::new();
    pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Create the model
    assert!(manager.create_model("TestModel", pins).is_ok());

    // Check that the model file was created
    let file_path = dir.path().join("TestModel.yaml");
    assert!(file_path.exists(), "Model file was not created");

    // Delete the model
    assert!(manager.delete_model("TestModel").is_ok());

    // Check that the model file was deleted
    assert!(!file_path.exists(), "Model file was not deleted");

    // Check that the model was removed from memory
    assert!(
        manager.get_model("TestModel").is_none(),
        "Model was not removed from memory"
    );

    // Check that deleting a non-existent model succeeds (idempotent)
    assert!(manager.delete_model("NonExistentModel").is_ok());
}

#[test]
fn test_model_manager_reload_models() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model
    let mut model = DeviceModel {
        name: "TestModel".to_string(),
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

    // Write the model file directly
    let file_path = dir.path().join("TestModel.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Create a model manager
    let mut manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Check that the model was loaded
    let loaded_model = manager.get_model("TestModel");
    assert!(loaded_model.is_some(), "Model was not loaded");

    // Delete the model file directly
    fs::remove_file(&file_path).unwrap();

    // The model should still be in memory
    assert!(
        manager.get_model("TestModel").is_some(),
        "Model was removed from memory"
    );

    // Reload models
    assert!(manager.reload_models().is_ok());

    // The model should now be gone
    assert!(
        manager.get_model("TestModel").is_none(),
        "Model was not removed after reload"
    );

    // Create a new model file
    let mut new_model = DeviceModel {
        name: "NewModel".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
    new_model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string()],
            active: true,
        },
    );

    // Write the new model file
    let new_file_path = dir.path().join("NewModel.yaml");
    let yaml = serde_yaml::to_string(&new_model).unwrap();
    fs::write(&new_file_path, yaml).unwrap();

    // Reload models
    assert!(manager.reload_models().is_ok());

    // The new model should be loaded
    assert!(
        manager.get_model("NewModel").is_some(),
        "New model was not loaded"
    );
}

#[test]
fn test_model_manager_validate_model() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let mut manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Create a valid model
    let mut pins = HashMap::new();
    pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Create the model
    assert!(manager.create_model("ValidModel", pins).is_ok());

    // Validate the model
    assert!(manager.validate_model("ValidModel").is_ok());

    // Create an invalid model
    let mut pins = HashMap::new();
    pins.insert(
        1,
        PinModel {
            capabilities: vec![],
            active: true,
        },
    );

    // Create the model (this should fail validation)
    assert!(manager.create_model("InvalidModel", pins).is_err());

    // Check that validating a non-existent model fails
    assert!(manager.validate_model("NonExistentModel").is_err());
}

#[test]
fn test_model_manager_save_model() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let mut manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Create a model
    let mut pins = HashMap::new();
    pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "DigitalOutput".to_string()],
            active: true,
        },
    );

    // Create the model
    assert!(manager.create_model("TestModel", pins).is_ok());

    // Get the model
    let mut model = manager.get_model("TestModel").unwrap().clone();

    // Modify the model
    model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalInput".to_string(), "AnalogInput".to_string()],
            active: true,
        },
    );

    // Save the model
    assert!(manager.save_model(&model).is_ok());

    // Reload models
    assert!(manager.reload_models().is_ok());

    // Check that the model was updated
    let loaded_model = manager.get_model("TestModel").unwrap();
    assert_eq!(loaded_model.pins.len(), 2);
    assert!(loaded_model.pins.contains_key(&1));
    assert!(loaded_model.pins.contains_key(&2));
}

#[test]
fn test_model_manager_get_model_dir() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Check that the model directory is correct
    assert_eq!(manager.get_model_dir(), dir.path());

    // Create a model manager with the default directory
    let manager = ModelManager::new(None).unwrap();

    // Check that the model directory is the default
    let default_dir = pokeys_lib::models::get_default_model_dir();
    assert_eq!(manager.get_model_dir(), default_dir);
}

#[test]
fn test_model_manager_get_all_models() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model manager
    let mut manager = ModelManager::new(Some(dir.path().to_path_buf())).unwrap();

    // Create models
    let mut pins1 = HashMap::new();
    pins1.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string()],
            active: true,
        },
    );

    let mut pins2 = HashMap::new();
    pins2.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalOutput".to_string()],
            active: true,
        },
    );

    // Create the models
    assert!(manager.create_model("Model1", pins1).is_ok());
    assert!(manager.create_model("Model2", pins2).is_ok());

    // Get all models
    let models = manager.get_all_models();

    // Check that both models are present
    assert!(models.contains_key("Model1"));
    assert!(models.contains_key("Model2"));

    // Check that the default models are also present
    assert!(models.contains_key("PoKeys56U"));
    assert!(models.contains_key("PoKeys57U"));
    assert!(models.contains_key("PoKeys56E"));
    assert!(models.contains_key("PoKeys57E"));
}
