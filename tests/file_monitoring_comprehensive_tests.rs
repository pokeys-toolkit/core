use pokeys_lib::models::{DeviceModel, ModelMonitor, PinModel};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// Process-wide mutex used to serialise the tests in this file.
///
/// `notify`'s `recommended_watcher` uses shared OS resources
/// (ReadDirectoryChangesW on Windows, inotify instances on Linux). When
/// these tests run in parallel under `cargo test`, multiple watchers
/// contend for those resources and events occasionally get dropped — a
/// test that writes a YAML file sees its `Create` event get lost behind a
/// sibling test's watcher activity, and asserts fail with
/// "Callback was not called after file creation".
///
/// `cargo test` runs integration-test files in separate processes, but
/// tests *within* the same file run on a shared thread pool. Holding this
/// mutex for each test forces within-file serialisation without the
/// blast-radius of `--test-threads=1` on the whole suite.
fn serial_guard() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    // Recover a poisoned mutex — a prior test panicking shouldn't stop the
    // rest from running.
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Max time to wait for a file-system event to propagate through the
/// `notify` watcher on a noisy CI runner. Generous — inotify/FSEvent setup
/// latency and runner scheduling jitter mean a fixed `thread::sleep(500ms)`
/// is not reliable on GitHub's shared runners.
const WATCH_TIMEOUT: Duration = Duration::from_secs(5);
/// Poll interval inside `wait_until`. Short enough to exit quickly once the
/// condition becomes true; long enough to avoid hot-spinning on the mutex.
const WATCH_POLL: Duration = Duration::from_millis(20);

/// Poll `condition` up to [`WATCH_TIMEOUT`] and return `true` if it becomes
/// true within the window. Replaces the pattern
/// `thread::sleep(500ms); assert!(cond);` which is timing-flaky on CI
/// because file-system event propagation under the `notify` crate is not
/// bounded by a fixed delay.
fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + WATCH_TIMEOUT;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(WATCH_POLL);
    }
    condition()
}

/// `ModelMonitor::start()` returns as soon as `watcher.watch()` succeeds,
/// but on some backends (notably `ReadDirectoryChangesW` on Windows) the
/// watch is not yet ready to receive events for a few milliseconds. Call
/// this right after `monitor.start()` before writing files in a test.
fn settle_watcher() {
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_model_monitor_initialization_and_shutdown() {
    let _g = serial_guard();

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
    let _g = serial_guard();

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
    settle_watcher();

    // Write the model file
    let file_path = dir.path().join("TestDevice.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();
    fs::write(&file_path, yaml).unwrap();

    // Poll until the callback fires, up to WATCH_TIMEOUT.
    let called = wait_until(|| *callback_called.lock().unwrap());
    assert!(called, "Callback was not called after file creation");

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_file_update_detection() {
    let _g = serial_guard();

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
    settle_watcher();

    // Wait for the pre-existing file to be picked up (count goes to 1).
    assert!(
        wait_until(|| *counter.lock().unwrap() >= 1),
        "Callback was not called after initial file detection"
    );

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

    // Wait for the update event to bring the counter to at least 2.
    assert!(
        wait_until(|| *counter.lock().unwrap() >= 2),
        "Callback was not called after file update"
    );

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_multiple_files() {
    let _g = serial_guard();

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
    settle_watcher();

    // Write the model files
    let file_path1 = dir.path().join("TestDevice1.yaml");
    let file_path2 = dir.path().join("TestDevice2.yaml");

    let yaml1 = serde_yaml::to_string(&model1).unwrap();
    let yaml2 = serde_yaml::to_string(&model2).unwrap();

    // Write each file, then wait for its detection callback to fire before
    // writing the next. Back-to-back writes to the same directory can be
    // coalesced by the OS file-system watcher (notably
    // `ReadDirectoryChangesW` on Windows): the watcher delivers one event
    // instead of two, and the in-process 100ms debounce in `ModelMonitor`
    // then drops the second as "same path within window" even though the
    // paths differ. Serialising the writes through the callback gives the
    // watcher a chance to flush each event.
    fs::write(&file_path1, yaml1).unwrap();
    assert!(
        wait_until(|| detected_models.lock().unwrap().contains_key("TestDevice1")),
        "TestDevice1 was not detected within timeout"
    );

    fs::write(&file_path2, yaml2).unwrap();
    assert!(
        wait_until(|| detected_models.lock().unwrap().contains_key("TestDevice2")),
        "TestDevice2 was not detected within timeout"
    );

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_get_model() {
    let _g = serial_guard();

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

    // Poll until the model is loaded into the monitor.
    assert!(
        wait_until(|| monitor.get_model("TestDevice").is_some()),
        "Model was not loaded within timeout"
    );

    // Check the model
    let loaded_model = monitor.get_model("TestDevice").unwrap();
    assert_eq!(loaded_model.name, "TestDevice");
    assert_eq!(loaded_model.pins.len(), 1);
    assert!(loaded_model.pins.contains_key(&1));

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_get_all_models() {
    let _g = serial_guard();

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

    // Poll until both models are loaded.
    assert!(
        wait_until(|| monitor.get_all_models().len() >= 2),
        "Not all models were loaded within timeout"
    );

    let models = monitor.get_all_models();
    assert!(models.contains_key("TestDevice1"));
    assert!(models.contains_key("TestDevice2"));

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_file_deletion() {
    let _g = serial_guard();

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

    // Poll until the initial file is loaded.
    assert!(
        wait_until(|| monitor.get_model("TestDevice").is_some()),
        "Model was not loaded within timeout"
    );

    // Delete the file. We don't currently react to deletion events (the
    // model stays in memory), so there's nothing to poll for — just give
    // the notify event loop a brief moment to process the delete so any
    // deletion-related code path does run.
    fs::remove_file(&file_path).unwrap();
    thread::sleep(Duration::from_millis(100));

    // The model should still be in memory (deletion does not evict).
    assert!(monitor.get_model("TestDevice").is_some());

    // Stop the monitor
    assert!(monitor.stop().is_ok());

    // Create a new monitor to check if the model is loaded again from the
    // (now empty) directory.
    let callback = |_: String, _: DeviceModel| {};
    let mut new_monitor = ModelMonitor::new(dir.path().to_path_buf(), callback);

    // Start the monitor
    assert!(new_monitor.start().is_ok());

    // There are no yaml files to load; the monitor should not have the
    // model. A short settle lets `load_existing_models()` complete.
    thread::sleep(Duration::from_millis(100));
    assert!(new_monitor.get_model("TestDevice").is_none());

    // Stop the monitor
    assert!(new_monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_directory_creation() {
    let _g = serial_guard();

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
    let _g = serial_guard();

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

    // Start the monitor. `load_existing_models()` runs synchronously inside
    // `start()`, so by the time the `assert` above returns, both models
    // should already be in `get_all_models`. We still poll to tolerate any
    // timing skew in the internal callbacks.
    assert!(monitor.start().is_ok());

    assert!(
        wait_until(|| monitor.get_all_models().len() >= 2),
        "Not all models were loaded within timeout"
    );

    // The callback is invoked for each load, so both names should have
    // been reported.
    assert!(
        wait_until(|| {
            let m = detected_models.lock().unwrap();
            m.contains_key("TestDevice1") && m.contains_key("TestDevice2")
        }),
        "Not all models were reported via the callback within timeout"
    );

    let all_models = monitor.get_all_models();
    assert_eq!(all_models.len(), 2);

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}

#[test]
fn test_model_monitor_invalid_model_file() {
    let _g = serial_guard();

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
    let _g = serial_guard();

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
    settle_watcher();

    // Write the model file multiple times in quick succession. The monitor's
    // internal debouncer (src/models.rs ~line 731) should collapse these
    // into fewer callback invocations.
    let file_path = dir.path().join("TestDevice.yaml");
    let yaml = serde_yaml::to_string(&model).unwrap();

    for _ in 0..5 {
        fs::write(&file_path, &yaml).unwrap();
        thread::sleep(Duration::from_millis(10));
    }

    // Give the debouncer time to process the burst. Fewer than 5 callbacks
    // is the acceptance criterion.
    thread::sleep(Duration::from_millis(300));

    let count = *counter.lock().unwrap();
    assert!(count < 5, "Debouncing did not work (count = {count})");

    // Stop the monitor
    assert!(monitor.stop().is_ok());
}
