use pokeys_lib::models::{DeviceModel, ModelMonitor, PinModel};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_model_monitor_initialization_and_shutdown() {
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

    // Start again to ensure it can be restarted
    assert!(monitor.start().is_ok());

    // Stop again
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_file_creation_detection() {
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
fn test_model_monitor_file_update_detection() {
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
fn test_model_monitor_multiple_files() {
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

    // Create a map to track which models were detected
    let detected_models = Arc::new(Mutex::new(HashMap::<String, bool>::new()));
    let detected_models_clone = detected_models.clone();

    // Create a callback that marks models as detected
    let callback = move |name: String, _: DeviceModel| {
        let mut models = detected_models_clone.lock().unwrap();
        models.insert(name, true);
    };

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Write the model files
    let file_path1 = dir.path().join("TestDevice1.yaml");
    let file_path2 = dir.path().join("TestDevice2.yaml");

    let yaml1 = serde_yaml::to_string(&model1).unwrap();
    let yaml2 = serde_yaml::to_string(&model2).unwrap();

    fs::write(&file_path1, yaml1).unwrap();
    fs::write(&file_path2, yaml2).unwrap();

    // Wait for the files to be detected
    thread::sleep(Duration::from_millis(500));

    // Check if both models were detected
    let models = detected_models.lock().unwrap();
    assert!(
        models.contains_key("TestDevice1"),
        "TestDevice1 was not detected"
    );
    assert!(
        models.contains_key("TestDevice2"),
        "TestDevice2 was not detected"
    );

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
fn test_model_monitor_file_deletion() {
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

    // Check that the model was loaded
    assert!(monitor.get_model("TestDevice").is_some());

    // Delete the file
    fs::remove_file(&file_path).unwrap();

    // Wait for the deletion to be detected
    thread::sleep(Duration::from_millis(500));

    // The model should still be in memory
    assert!(monitor.get_model("TestDevice").is_some());

    // Stop the monitor
    assert!(monitor.stop().is_ok());

    // Create a new monitor to check if the model is loaded again
    let callback = |_: String, _: DeviceModel| {};
    let mut new_monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(new_monitor.start().is_ok());

    // Wait for any files to be detected
    thread::sleep(Duration::from_millis(500));

    // The model should not be loaded
    assert!(new_monitor.get_model("TestDevice").is_none());

    // Stop the monitor
    assert!(new_monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_directory_creation() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create a non-existent subdirectory path
    let subdir_path = dir.path().join("non_existent_dir");

    // Create a callback
    let callback = |_: String, _: DeviceModel| {};

    // Create a model monitor with the non-existent directory
    let mut monitor = ModelMonitor::new(subdir_path.clone(), callback);

    // Start the monitor - this should create the directory
    assert!(monitor.start().is_ok());

    // Check that the directory was created
    assert!(subdir_path.exists());
    assert!(subdir_path.is_dir());

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_load_existing_models() {
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

    // Write the model files before creating the monitor
    let file_path1 = dir.path().join("TestDevice1.yaml");
    let file_path2 = dir.path().join("TestDevice2.yaml");

    let yaml1 = serde_yaml::to_string(&model1).unwrap();
    let yaml2 = serde_yaml::to_string(&model2).unwrap();

    fs::write(&file_path1, yaml1).unwrap();
    fs::write(&file_path2, yaml2).unwrap();

    // Create a map to track which models were detected
    let detected_models = Arc::new(Mutex::new(HashMap::<String, bool>::new()));
    let detected_models_clone = detected_models.clone();

    // Create a callback that marks models as detected
    let callback = move |name: String, _: DeviceModel| {
        let mut models = detected_models_clone.lock().unwrap();
        models.insert(name, true);
    };

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Wait for the files to be detected
    thread::sleep(Duration::from_millis(500));

    // Check if both models were detected
    let models = detected_models.lock().unwrap();
    assert!(
        models.contains_key("TestDevice1"),
        "TestDevice1 was not detected"
    );
    assert!(
        models.contains_key("TestDevice2"),
        "TestDevice2 was not detected"
    );

    // Get all models
    let all_models = monitor.get_all_models();
    assert_eq!(all_models.len(), 2, "Not all models were loaded");

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_invalid_model_file() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();

    // Create an invalid YAML file
    let file_path = dir.path().join("InvalidModel.yaml");
    fs::write(&file_path, "this is not valid yaml").unwrap();

    // Create a callback
    let callback = |_: String, _: DeviceModel| {};

    // Create a model monitor
    let mut monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(monitor.start().is_ok());

    // Wait for the file to be detected
    thread::sleep(Duration::from_millis(500));

    // The invalid model should not be loaded
    assert!(monitor.get_model("InvalidModel").is_none());

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_debouncing() {
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

    // Write the model file multiple times in quick succession
    let file_path = dir.path().join("TestDevice.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();

    for _ in 0..5 {
        fs::write(&file_path, &yaml).unwrap();
        thread::sleep(Duration::from_millis(10));
    }

    // Wait for the file to be detected
    thread::sleep(Duration::from_millis(500));

    // Check if the callback was called fewer times than the number of writes
    let count = *counter.lock().unwrap();
    assert!(count < 5, "Debouncing did not work");

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}
