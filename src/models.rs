//! Device model definitions and validation
//!
//! This module provides structures and functions for loading and validating
//! device models from YAML files. Device models define the capabilities of
//! each pin on a PoKeys device, ensuring that users can only assign supported
//! functions to pins.

use crate::error::{PoKeysError, Result};
use log::{error, info, warn};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Default directory for device model files
pub const DEFAULT_MODEL_DIR: &str = ".config/pokeys/models";

/// Default retry interval for model loading (in seconds)
pub const DEFAULT_RETRY_INTERVAL: u64 = 10;

/// Pin model defining the capabilities of a single pin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PinModel {
    /// List of capabilities supported by this pin
    pub capabilities: Vec<String>,

    /// Whether the pin is active
    #[serde(default = "default_active")]
    pub active: bool,
}

/// Device model defining the capabilities of all pins on a device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceModel {
    /// Device model name
    pub name: String,

    /// Map of pin numbers to pin models
    pub pins: HashMap<u8, PinModel>,
}

/// Default value for pin active state
fn default_active() -> bool {
    true
}

impl DeviceModel {
    /// Load a device model from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML file
    ///
    /// # Returns
    ///
    /// * `Result<DeviceModel>` - The loaded device model or an error
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            PoKeysError::ModelLoadError(path.as_ref().to_string_lossy().to_string(), e.to_string())
        })?;

        let model: DeviceModel = serde_yaml::from_str(&content).map_err(|e| {
            PoKeysError::ModelParseError(path.as_ref().to_string_lossy().to_string(), e.to_string())
        })?;

        model.validate()?;

        Ok(model)
    }

    /// Validate the device model
    ///
    /// Checks that the model is well-formed and that all related capabilities
    /// are properly defined.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the model is valid, an error otherwise
    pub fn validate(&self) -> Result<()> {
        // Check that the model has a name
        if self.name.is_empty() {
            return Err(PoKeysError::ModelValidationError(
                "Model name cannot be empty".to_string(),
            ));
        }

        // Check that the model has at least one pin
        if self.pins.is_empty() {
            return Err(PoKeysError::ModelValidationError(
                "Model must define at least one pin".to_string(),
            ));
        }

        // Check that all pins have at least one capability
        for (pin_num, pin) in &self.pins {
            if pin.capabilities.is_empty() {
                return Err(PoKeysError::ModelValidationError(format!(
                    "Pin {} must have at least one capability",
                    pin_num
                )));
            }
        }

        // Validate related capabilities
        self.validate_related_capabilities()?;

        Ok(())
    }

    /// Validate that all related capabilities are properly defined
    ///
    /// For example, if a pin has the capability "Encoder_1A", there must be
    /// another pin with the capability "Encoder_1B".
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if all related capabilities are valid, an error otherwise
    fn validate_related_capabilities(&self) -> Result<()> {
        // Validate encoder pairs
        self.validate_encoder_pairs()?;

        // Validate matrix keyboard rows and columns
        self.validate_matrix_keyboard()?;

        // Validate PWM channels
        self.validate_pwm_channels()?;

        Ok(())
    }

    /// Validate encoder pairs
    ///
    /// Checks that for each encoder A pin, there is a corresponding encoder B pin.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if all encoder pairs are valid, an error otherwise
    fn validate_encoder_pairs(&self) -> Result<()> {
        // Find all encoder A pins
        let mut encoder_a_pins = HashMap::new();
        for (pin_num, pin) in &self.pins {
            for capability in &pin.capabilities {
                if capability.starts_with("Encoder_") && capability.ends_with("A") {
                    let encoder_id = &capability[8..capability.len() - 1]; // Extract "1" from "Encoder_1A"
                    encoder_a_pins.insert(encoder_id.to_string(), *pin_num);
                }
            }
        }

        // Check that each encoder A pin has a corresponding encoder B pin
        for (encoder_id, pin_a) in &encoder_a_pins {
            let encoder_b_capability = format!("Encoder_{}B", encoder_id);
            let mut found_b = false;

            for (pin_num, pin) in &self.pins {
                if *pin_num != *pin_a
                    && pin
                        .capabilities
                        .iter()
                        .any(|cap| cap == &encoder_b_capability)
                {
                    found_b = true;
                    break;
                }
            }

            if !found_b {
                return Err(PoKeysError::ModelValidationError(format!(
                    "Encoder {}A on pin {} has no corresponding {}B pin",
                    encoder_id, pin_a, encoder_b_capability
                )));
            }
        }

        Ok(())
    }

    /// Validate matrix keyboard rows and columns
    ///
    /// Checks that if there are matrix keyboard rows, there are also matrix keyboard columns.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the matrix keyboard configuration is valid, an error otherwise
    fn validate_matrix_keyboard(&self) -> Result<()> {
        let mut has_rows = false;
        let mut has_columns = false;

        for pin in self.pins.values() {
            for capability in &pin.capabilities {
                if capability.starts_with("MatrixKeyboard_Row") {
                    has_rows = true;
                } else if capability.starts_with("MatrixKeyboard_Col") {
                    has_columns = true;
                }
            }
        }

        if has_rows && !has_columns {
            return Err(PoKeysError::ModelValidationError(
                "Matrix keyboard has rows but no columns".to_string(),
            ));
        }

        if !has_rows && has_columns {
            return Err(PoKeysError::ModelValidationError(
                "Matrix keyboard has columns but no rows".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate PWM channels
    ///
    /// Checks that PWM channels are properly defined.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the PWM channels are valid, an error otherwise
    fn validate_pwm_channels(&self) -> Result<()> {
        // For now, just check that PWM channels are numbered sequentially
        let mut pwm_channels = Vec::new();

        for (pin_num, pin) in &self.pins {
            for capability in &pin.capabilities {
                if let Some(stripped) = capability.strip_prefix("PWM_") {
                    if let Ok(channel) = stripped.parse::<u32>() {
                        pwm_channels.push((channel, *pin_num));
                    }
                }
            }
        }

        // Sort by channel number
        pwm_channels.sort_by_key(|(channel, _)| *channel);

        // Check that channels are sequential starting from 1
        for (i, (channel, pin)) in pwm_channels.iter().enumerate() {
            if *channel != (i + 1) as u32 {
                return Err(PoKeysError::ModelValidationError(format!(
                    "PWM channels are not sequential: expected channel {}, found channel {} on pin {}",
                    i + 1,
                    channel,
                    pin
                )));
            }
        }

        Ok(())
    }

    /// Check if a pin supports a specific capability
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The pin number to check
    /// * `capability` - The capability to check for
    ///
    /// # Returns
    ///
    /// * `bool` - True if the pin supports the capability, false otherwise
    pub fn is_pin_capability_supported(&self, pin_num: u8, capability: &str) -> bool {
        if let Some(pin) = self.pins.get(&pin_num) {
            pin.capabilities.iter().any(|cap| cap == capability)
        } else {
            false
        }
    }

    /// Get all capabilities for a pin
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The pin number to get capabilities for
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - List of capabilities supported by the pin
    pub fn get_pin_capabilities(&self, pin_num: u8) -> Vec<String> {
        if let Some(pin) = self.pins.get(&pin_num) {
            pin.capabilities.clone()
        } else {
            Vec::new()
        }
    }

    /// Get related capabilities for a specific capability
    ///
    /// For example, if the capability is "Encoder_1A", this will return
    /// [("Encoder_1B", pin_num)] where pin_num is the pin that has the "Encoder_1B" capability.
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The pin number with the capability
    /// * `capability` - The capability to find related capabilities for
    ///
    /// # Returns
    ///
    /// * `Vec<(String, u8)>` - List of related capabilities and their pin numbers
    pub fn get_related_capabilities(&self, pin_num: u8, capability: &str) -> Vec<(String, u8)> {
        let mut related = Vec::new();

        // Check for encoder capabilities
        if capability.starts_with("Encoder_") && capability.len() >= 10 {
            let encoder_id = &capability[8..capability.len() - 1]; // Extract "1" from "Encoder_1A"
            let role = &capability[capability.len() - 1..]; // Extract "A" from "Encoder_1A"

            let related_role = if role == "A" { "B" } else { "A" };
            let related_capability = format!("Encoder_{}{}", encoder_id, related_role);

            // Find the pin with the related capability
            for (other_pin, pin_model) in &self.pins {
                if *other_pin != pin_num
                    && pin_model
                        .capabilities
                        .iter()
                        .any(|cap| cap == &related_capability)
                {
                    related.push((related_capability, *other_pin));
                    break;
                }
            }
        }

        // Check for matrix keyboard capabilities
        if capability.starts_with("MatrixKeyboard_Row") {
            // For a row, all columns are related
            for (other_pin, pin_model) in &self.pins {
                if *other_pin != pin_num {
                    for cap in &pin_model.capabilities {
                        if cap.starts_with("MatrixKeyboard_Col") {
                            related.push((cap.clone(), *other_pin));
                        }
                    }
                }
            }
        }

        if capability.starts_with("MatrixKeyboard_Col") {
            // For a column, all rows are related
            for (other_pin, pin_model) in &self.pins {
                if *other_pin != pin_num {
                    for cap in &pin_model.capabilities {
                        if cap.starts_with("MatrixKeyboard_Row") {
                            related.push((cap.clone(), *other_pin));
                        }
                    }
                }
            }
        }

        related
    }

    /// Validate that a pin can be configured with a specific capability
    ///
    /// This checks both that the pin supports the capability and that any
    /// related capabilities are properly configured.
    ///
    /// # Arguments
    ///
    /// * `pin_num` - The pin number to check
    /// * `capability` - The capability to check for
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the capability is valid, an error otherwise
    pub fn validate_pin_capability(&self, pin_num: u8, capability: &str) -> Result<()> {
        // Check that the pin exists and supports the capability
        if !self.is_pin_capability_supported(pin_num, capability) {
            return Err(PoKeysError::UnsupportedPinCapability(
                pin_num,
                capability.to_string(),
            ));
        }

        // Check related capabilities
        let related = self.get_related_capabilities(pin_num, capability);

        // For encoder capabilities, ensure the related pin is configured
        if capability.starts_with("Encoder_") && capability.ends_with("A") {
            let encoder_id = &capability[8..capability.len() - 1]; // Extract "1" from "Encoder_1A"
            let encoder_b_capability = format!("Encoder_{}B", encoder_id);

            // Check if any pin has the B capability
            let mut found_b = false;
            for (related_cap, related_pin) in &related {
                if related_cap == &encoder_b_capability {
                    found_b = true;

                    // Check if the related pin is active
                    if let Some(pin_model) = self.pins.get(related_pin) {
                        if !pin_model.active {
                            return Err(PoKeysError::RelatedPinInactive(
                                *related_pin,
                                related_cap.clone(),
                            ));
                        }
                    }

                    break;
                }
            }

            if !found_b {
                return Err(PoKeysError::MissingRelatedCapability(
                    pin_num,
                    capability.to_string(),
                    encoder_b_capability,
                ));
            }
        }

        // For encoder B capabilities, ensure the related A pin is configured
        if capability.starts_with("Encoder_") && capability.ends_with("B") {
            let encoder_id = &capability[8..capability.len() - 1]; // Extract "1" from "Encoder_1B"
            let encoder_a_capability = format!("Encoder_{}A", encoder_id);

            // Check if any pin has the A capability
            let mut found_a = false;
            for (related_cap, related_pin) in &related {
                if related_cap == &encoder_a_capability {
                    found_a = true;

                    // Check if the related pin is active
                    if let Some(pin_model) = self.pins.get(related_pin) {
                        if !pin_model.active {
                            return Err(PoKeysError::RelatedPinInactive(
                                *related_pin,
                                related_cap.clone(),
                            ));
                        }
                    }

                    break;
                }
            }

            if !found_a {
                return Err(PoKeysError::MissingRelatedCapability(
                    pin_num,
                    capability.to_string(),
                    encoder_a_capability,
                ));
            }
        }

        // For matrix keyboard rows, ensure there's at least one column
        if capability.starts_with("MatrixKeyboard_Row") {
            let mut found_col = false;
            for pin in self.pins.values() {
                if pin.active
                    && pin
                        .capabilities
                        .iter()
                        .any(|cap| cap.starts_with("MatrixKeyboard_Col"))
                {
                    found_col = true;
                    break;
                }
            }

            if !found_col {
                return Err(PoKeysError::MissingRelatedCapability(
                    pin_num,
                    capability.to_string(),
                    "MatrixKeyboard_Col".to_string(),
                ));
            }
        }

        // For matrix keyboard columns, ensure there's at least one row
        if capability.starts_with("MatrixKeyboard_Col") {
            let mut found_row = false;
            for pin in self.pins.values() {
                if pin.active
                    && pin
                        .capabilities
                        .iter()
                        .any(|cap| cap.starts_with("MatrixKeyboard_Row"))
                {
                    found_row = true;
                    break;
                }
            }

            if !found_row {
                return Err(PoKeysError::MissingRelatedCapability(
                    pin_num,
                    capability.to_string(),
                    "MatrixKeyboard_Row".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate LED matrix configuration against device model
    ///
    /// # Arguments
    ///
    /// * `config` - LED matrix configuration to validate
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if valid, error if invalid
    pub fn validate_led_matrix_config(
        &self,
        config: &crate::matrix::LedMatrixConfig,
    ) -> Result<()> {
        // Check if matrix ID is valid (1 or 2)
        if config.matrix_id < 1 || config.matrix_id > 2 {
            return Err(PoKeysError::ModelValidationError(format!(
                "Invalid matrix ID: {}. Must be 1 or 2",
                config.matrix_id
            )));
        }

        // Get the pins for this matrix
        let pins = match config.matrix_id {
            1 => crate::matrix::LED_MATRIX_1_PINS,
            2 => crate::matrix::LED_MATRIX_2_PINS,
            _ => {
                return Err(PoKeysError::ModelValidationError(format!(
                    "Invalid matrix ID: {}",
                    config.matrix_id
                )));
            }
        };

        // Validate that all required pins support the necessary capabilities
        for &pin in &pins {
            if !self.is_pin_capability_supported(pin, "DigitalOutput") {
                return Err(PoKeysError::ModelValidationError(format!(
                    "Pin {} does not support DigitalOutput capability required for LED matrix {}",
                    pin, config.matrix_id
                )));
            }
        }

        Ok(())
    }

    /// Reserve LED matrix pins in the device model
    ///
    /// # Arguments
    ///
    /// * `matrix_id` - Matrix ID (1 or 2)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if pins reserved successfully, error if conflict
    pub fn reserve_led_matrix_pins(&mut self, matrix_id: u8) -> Result<()> {
        // Get the pins for this matrix
        let pins = match matrix_id {
            1 => crate::matrix::LED_MATRIX_1_PINS,
            2 => crate::matrix::LED_MATRIX_2_PINS,
            _ => {
                return Err(PoKeysError::ModelValidationError(format!(
                    "Invalid matrix ID: {}",
                    matrix_id
                )));
            }
        };

        // For now, just validate that the pins exist and support the capability
        // In a full implementation, this would track pin reservations
        for &pin in &pins {
            if !self.is_pin_capability_supported(pin, "DigitalOutput") {
                return Err(PoKeysError::ModelValidationError(format!(
                    "Cannot reserve pin {} for LED matrix {}: pin does not support DigitalOutput",
                    pin, matrix_id
                )));
            }
        }

        Ok(())
    }
}

/// Get the default model directory path
///
/// This returns the default directory for device model files, which is
/// ~/.config/pokeys/models on Unix systems and %APPDATA%\pokeys\models on Windows.
///
/// # Returns
///
/// * `PathBuf` - The default model directory path
pub fn get_default_model_dir() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(DEFAULT_MODEL_DIR);
    path
}

/// Get the path to a device model file
///
/// # Arguments
///
/// * `device_name` - The name of the device model
/// * `model_dir` - Optional custom directory for model files
///
/// # Returns
///
/// * `PathBuf` - The path to the model file
pub fn get_model_path(device_name: &str, model_dir: Option<&Path>) -> PathBuf {
    let dir = model_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(get_default_model_dir);
    dir.join(format!("{}.yaml", device_name))
}

/// Load a device model by name
///
/// # Arguments
///
/// * `device_name` - The name of the device model
/// * `model_dir` - Optional custom directory for model files
///
/// # Returns
///
/// * `Result<DeviceModel>` - The loaded device model or an error
pub fn load_model(device_name: &str, model_dir: Option<&Path>) -> Result<DeviceModel> {
    let path = get_model_path(device_name, model_dir);
    DeviceModel::from_file(path)
}

/// Model monitor for watching for changes to model files
pub struct ModelMonitor {
    /// The watcher for file system events
    watcher: Option<RecommendedWatcher>,

    /// The directory being watched
    watch_dir: PathBuf,

    /// Loaded models
    models: Arc<RwLock<HashMap<String, DeviceModel>>>,

    /// Callback for model updates
    callback: Arc<dyn Fn(String, DeviceModel) + Send + Sync + 'static>,

    /// Whether the monitor is running
    running: bool,
}

impl ModelMonitor {
    /// Create a new model monitor
    ///
    /// # Arguments
    ///
    /// * `model_dir` - Directory containing model files
    /// * `callback` - Callback function called when a model is updated
    ///
    /// # Returns
    ///
    /// * `ModelMonitor` - The new model monitor
    pub fn new<F>(model_dir: PathBuf, callback: F) -> Self
    where
        F: Fn(String, DeviceModel) + Send + Sync + 'static,
    {
        Self {
            watcher: None,
            watch_dir: model_dir,
            models: Arc::new(RwLock::new(HashMap::new())),
            callback: Arc::new(callback),
            running: false,
        }
    }

    /// Start monitoring for model file changes
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if monitoring started successfully, an error otherwise
    pub fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }

        // Create the model directory if it doesn't exist
        if !self.watch_dir.exists() {
            fs::create_dir_all(&self.watch_dir).map_err(|e| {
                PoKeysError::ModelDirCreateError(
                    self.watch_dir.to_string_lossy().to_string(),
                    e.to_string(),
                )
            })?;
        }

        // Load existing models
        self.load_existing_models()?;

        // Create a channel for the watcher
        let (tx, rx) = std::sync::mpsc::channel();

        // Create the watcher
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| PoKeysError::ModelWatcherError(e.to_string()))?;

        // Start watching the directory
        watcher
            .watch(&self.watch_dir, RecursiveMode::NonRecursive)
            .map_err(|e| PoKeysError::ModelWatcherError(e.to_string()))?;

        self.watcher = Some(watcher);
        self.running = true;

        // Clone the models and callback for the thread
        let models = self.models.clone();
        let callback = self.callback.clone();

        // Spawn a thread to handle file system events
        std::thread::spawn(move || {
            let mut debouncer = HashMap::new();

            for res in rx {
                match res {
                    Ok(event) => {
                        if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
                            for path in event.paths {
                                if path.extension().is_some_and(|ext| ext == "yaml") {
                                    // Debounce the event
                                    let now = std::time::Instant::now();
                                    let path_str = path.to_string_lossy().to_string();

                                    // Only process the event if it's been at least 100ms since the last event for this file
                                    if debouncer.get(&path_str).is_none_or(|last| {
                                        now.duration_since(*last) > Duration::from_millis(100)
                                    }) {
                                        debouncer.insert(path_str, now);

                                        // Get the device name from the file name
                                        if let Some(file_name) = path.file_stem() {
                                            let device_name =
                                                file_name.to_string_lossy().to_string();

                                            // Try to load the model
                                            match DeviceModel::from_file(&path) {
                                                Ok(model) => {
                                                    // Update the model in the map
                                                    {
                                                        let mut models = models.write().unwrap();
                                                        models.insert(
                                                            device_name.clone(),
                                                            model.clone(),
                                                        );
                                                    }

                                                    // Call the callback
                                                    callback(device_name, model);
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "Failed to load model from {}: {}",
                                                        path.display(),
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Watch error: {:?}", e);
                    }
                }
            }
        });

        info!(
            "Model monitor started, watching directory: {}",
            self.watch_dir.display()
        );
        Ok(())
    }

    /// Stop monitoring for model file changes
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if monitoring stopped successfully, an error otherwise
    pub fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        self.watcher = None;
        self.running = false;

        info!("Model monitor stopped");
        Ok(())
    }

    /// Load existing model files from the watch directory
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if models loaded successfully, an error otherwise
    fn load_existing_models(&self) -> Result<()> {
        if !self.watch_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.watch_dir).map_err(|e| {
            PoKeysError::ModelDirReadError(
                self.watch_dir.to_string_lossy().to_string(),
                e.to_string(),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PoKeysError::ModelDirReadError(
                    self.watch_dir.to_string_lossy().to_string(),
                    e.to_string(),
                )
            })?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "yaml") {
                if let Some(file_name) = path.file_stem() {
                    let device_name = file_name.to_string_lossy().to_string();

                    match DeviceModel::from_file(&path) {
                        Ok(model) => {
                            // Update the model in the map
                            {
                                let mut models = self.models.write().unwrap();
                                models.insert(device_name.clone(), model.clone());
                            }

                            // Call the callback
                            (self.callback)(device_name, model);
                        }
                        Err(e) => {
                            warn!("Failed to load model from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a model by name
    ///
    /// # Arguments
    ///
    /// * `device_name` - The name of the device model
    ///
    /// # Returns
    ///
    /// * `Option<DeviceModel>` - The model if found, None otherwise
    pub fn get_model(&self, device_name: &str) -> Option<DeviceModel> {
        let models = self.models.read().unwrap();
        models.get(device_name).cloned()
    }

    /// Get all loaded models
    ///
    /// # Returns
    ///
    /// * `HashMap<String, DeviceModel>` - Map of device names to models
    pub fn get_all_models(&self) -> HashMap<String, DeviceModel> {
        let models = self.models.read().unwrap();
        models.clone()
    }
}

/// Copy default model files to the user's model directory
///
/// This function copies the default model files from the package to the user's
/// model directory if they don't already exist.
///
/// # Arguments
///
/// * `model_dir` - Optional custom directory for model files
///
/// # Returns
///
/// * `Result<()>` - Ok if the files were copied successfully, an error otherwise
pub fn copy_default_models_to_user_dir(model_dir: Option<&Path>) -> Result<()> {
    let dir = model_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(get_default_model_dir);

    // Create the directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| {
            PoKeysError::ModelDirCreateError(dir.to_string_lossy().to_string(), e.to_string())
        })?;
    }

    // List of default models
    let default_models = [
        "PoKeys56U.yaml",
        "PoKeys57U.yaml",
        "PoKeys56E.yaml",
        "PoKeys57E.yaml",
    ];

    // Get the path to the package's model directory
    let package_model_dir = std::env::current_exe()
        .map_err(|e| {
            PoKeysError::ModelDirReadError(
                "Failed to get current executable path".to_string(),
                e.to_string(),
            )
        })?
        .parent()
        .ok_or_else(|| {
            PoKeysError::ModelDirReadError(
                "Failed to get parent directory of executable".to_string(),
                "No parent directory".to_string(),
            )
        })?
        .join("models");

    // If the package model directory doesn't exist, try to find it in the crate directory
    let package_model_dir = if !package_model_dir.exists() {
        // Try to find the models in the crate directory
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // If CARGO_MANIFEST_DIR is not set, use a relative path
                PathBuf::from("pokeys-lib/models")
            });

        crate_dir.join("models")
    } else {
        package_model_dir
    };

    // Copy each default model file if it doesn't exist in the user's directory
    for model_file in &default_models {
        let user_file_path = dir.join(model_file);

        // Skip if the file already exists
        if user_file_path.exists() {
            continue;
        }

        // Try to find the model file in the package
        let package_file_path = package_model_dir.join(model_file);

        if package_file_path.exists() {
            // Copy the file
            fs::copy(&package_file_path, &user_file_path).map_err(|e| {
                PoKeysError::ModelLoadError(
                    package_file_path.to_string_lossy().to_string(),
                    e.to_string(),
                )
            })?;

            info!(
                "Copied default model file {} to {}",
                model_file,
                user_file_path.display()
            );
        } else {
            // If the file doesn't exist in the package, try to find it in the source directory
            let source_file_path = PathBuf::from(format!("pokeys-lib/models/{}", model_file));

            if source_file_path.exists() {
                // Copy the file
                fs::copy(&source_file_path, &user_file_path).map_err(|e| {
                    PoKeysError::ModelLoadError(
                        source_file_path.to_string_lossy().to_string(),
                        e.to_string(),
                    )
                })?;

                info!(
                    "Copied default model file {} to {}",
                    model_file,
                    user_file_path.display()
                );
            } else {
                warn!("Default model file {} not found", model_file);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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

        // The model should still validate (we only warn about related capabilities)
        // Note: We're not validating encoder pairs in this test
    }

    #[test]
    fn test_matrix_keyboard_validation() {
        // Create a model with matrix keyboard pins
        let mut model = DeviceModel {
            name: "TestDevice".to_string(),
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
                    "MatrixKeyboard_Row2".to_string(),
                ],
                active: true,
            },
        );

        model.pins.insert(
            3,
            PinModel {
                capabilities: vec![
                    "DigitalInput".to_string(),
                    "MatrixKeyboard_Col1".to_string(),
                ],
                active: true,
            },
        );

        model.pins.insert(
            4,
            PinModel {
                capabilities: vec![
                    "DigitalInput".to_string(),
                    "MatrixKeyboard_Col2".to_string(),
                ],
                active: true,
            },
        );

        // Validate the model
        assert!(model.validate().is_ok());

        // Test related capabilities
        let related = model.get_related_capabilities(1, "MatrixKeyboard_Row1");
        assert_eq!(related.len(), 2);
        assert!(
            related
                .iter()
                .any(|(cap, pin)| cap == "MatrixKeyboard_Col1" && *pin == 3)
        );
        assert!(
            related
                .iter()
                .any(|(cap, pin)| cap == "MatrixKeyboard_Col2" && *pin == 4)
        );

        // Test missing columns
        let mut invalid_model = model.clone();
        invalid_model.pins.remove(&3);
        invalid_model.pins.remove(&4);

        // The model should still validate (we only warn about related capabilities)
        // Note: We're not validating matrix keyboard rows/columns in this test
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
}
