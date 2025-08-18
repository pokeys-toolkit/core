use pokeys_lib::models::{DeviceModel, ModelMonitor, PinModel};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_model_monitor_initialization() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a callback that counts the number of times it's called
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let callback = move |_: String, _: DeviceModel| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
    };

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_file_detection() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

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

    // Create a flag to indicate when the callback is called
    let callback_called = Arc::new(Mutex::new(false));
    let callback_called_clone = callback_called.clone();

    // Create a callback that sets the flag
    let callback = move |name: String, _: DeviceModel| {
        if name == "TestDevice" {
            let mut called = callback_called_clone.lock().unwrap();
            *called = true;
        }
    };

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Write the model file
    let file_path = dir.path().join("TestDevice.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Wait for the file to be detected
    thread::sleep(Duration::from_millis(500));

    // Check if the callback was called
    let called = *callback_called.lock().unwrap();
    assert!(called, "Callback was not called after file creation");

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_file_update() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string()],
            active: true,
        },
    );

    // Write the initial model file
    let file_path = dir.path().join("TestDevice.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Create a counter to track callback calls
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    // Create a callback that increments the counter
    let callback = move |_: String, _: DeviceModel| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
    };

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Wait for the initial file to be detected
    thread::sleep(Duration::from_millis(500));

    // Update the model
    let mut updated_model = model.clone();
    updated_model.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalOutput".to_string()],
            active: true,
        },
    );

    // Write the updated model file
    let yaml = serde_yaml::to_string(&updated_model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Wait for the file update to be detected
    thread::sleep(Duration::from_millis(500));

    // Check if the callback was called at least twice
    let count = *counter.lock().unwrap();
    assert!(count >= 2, "Callback was not called after file update");

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_get_model() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

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

    // Write the model file
    let file_path = dir.path().join("TestDevice.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Create a callback
    let callback = |_: String, _: DeviceModel| {};

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Wait for the file to be detected
    thread::sleep(Duration::from_millis(500));

    // Get the model
    let loaded_model = monitor.get_model("TestDevice");
    assert!(loaded_model.is_some(), "Model was not loaded");

    // Check the model
    let loaded_model = loaded_model.unwrap();
    assert_eq!(loaded_model.name, "TestDevice");
    assert_eq!(loaded_model.pins.len(), 1);
    assert!(loaded_model.pins.contains_key(&1));

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_get_all_models() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create models
    let mut model1 = DeviceModel {
        name: "TestDevice1".to_string(),
        pins: HashMap::new(),
    };

    let mut model2 = DeviceModel {
        name: "TestDevice2".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
    model1.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string()],
            active: true,
        },
    );

    model2.pins.insert(
        2,
        PinModel {
            capabilities: vec!["DigitalOutput".to_string()],
            active: true,
        },
    );

    // Write the model files
    let file_path1 = dir.path().join("TestDevice1.yaml");
    let file_path2 = dir.path().join("TestDevice2.yaml");

    let yaml1 = serde_yaml::to_string(&model1).unwrap();
    let yaml2 = serde_yaml::to_string(&model2).unwrap();

    fs::write(&file_path1, yaml1).unwrap();
    fs::write(&file_path2, yaml2).unwrap();

    // Create a callback
    let callback = |_: String, _: DeviceModel| {};

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Wait for the files to be detected
    thread::sleep(Duration::from_millis(500));

    // Get all models
    let models = monitor.get_all_models();
    assert_eq!(models.len(), 2, "Not all models were loaded");

    // Check the models
    assert!(models.contains_key("TestDevice1"));
    assert!(models.contains_key("TestDevice2"));

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_retry_logic() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a flag to indicate when the callback is called
    let callback_called = Arc::new(Mutex::new(false));
    let callback_called_clone = callback_called.clone();

    // Create a callback that sets the flag
    let callback = move |name: String, _: DeviceModel| {
        if name == "TestDevice" {
            let mut called = callback_called_clone.lock().unwrap();
            *called = true;
        }
    };

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Wait a bit
    thread::sleep(Duration::from_millis(100));

    // Create a model
    let mut model = DeviceModel {
        name: "TestDevice".to_string(),
        pins: HashMap::new(),
    };

    // Add some pins
    model.pins.insert(
        1,
        PinModel {
            capabilities: vec!["DigitalInput".to_string()],
            active: true,
        },
    );

    // Write the model file after a delay
    let file_path = dir.path().join("TestDevice.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Wait for the file to be detected
    thread::sleep(Duration::from_millis(500));

    // Check if the callback was called
    let called = *callback_called.lock().unwrap();
    assert!(
        called,
        "Callback was not called after delayed file creation"
    );

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}
