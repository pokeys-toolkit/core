//! Model management API
//!
//! This module provides a high-level API for managing device models.
//! It allows users to list, create, edit, validate, and apply models to devices.

use crate::error::{PoKeysError, Result};
use crate::models::{
    copy_default_models_to_user_dir, get_default_model_dir, DeviceModel, PinModel,
};
use log::{info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Model manager for device models
pub struct ModelManager {
    /// Directory containing model files
    model_dir: PathBuf,

    /// Loaded models
    models: HashMap<String, DeviceModel>,
}

impl ModelManager {
    /// Create a new model manager
    ///
    /// # Arguments
    ///
    /// * `model_dir` - Optional custom directory for model files
    ///
    /// # Returns
    ///
    /// * `Result<ModelManager>` - The new model manager or an error
    pub fn new(model_dir: Option<PathBuf>) -> Result<Self> {
        let dir = model_dir.unwrap_or_else(get_default_model_dir);

        // Create the directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| {
                PoKeysError::ModelDirCreateError(dir.to_string_lossy().to_string(), e.to_string())
            })?;
        }

        // Copy default models to the user's directory
        copy_default_models_to_user_dir(Some(&dir))?;

        let mut manager = Self {
            model_dir: dir,
            models: HashMap::new(),
        };

        // Load all models from the directory
        manager.reload_models()?;

        Ok(manager)
    }

    /// Reload all models from the model directory
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if models were loaded successfully, an error otherwise
    pub fn reload_models(&mut self) -> Result<()> {
        self.models.clear();

        // Read all YAML files in the directory
        let entries = fs::read_dir(&self.model_dir).map_err(|e| {
            PoKeysError::ModelDirReadError(
                self.model_dir.to_string_lossy().to_string(),
                e.to_string(),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PoKeysError::ModelDirReadError(
                    self.model_dir.to_string_lossy().to_string(),
                    e.to_string(),
                )
            })?;

            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "yaml") {
                if let Some(file_name) = path.file_stem() {
                    let model_name = file_name.to_string_lossy().to_string();

                    match DeviceModel::from_file(&path) {
                        Ok(model) => {
                            self.models.insert(model_name, model);
                        }
                        Err(e) => {
                            warn!("Failed to load model from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        info!(
            "Loaded {} models from {}",
            self.models.len(),
            self.model_dir.display()
        );
        Ok(())
    }

    /// Get a model by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the model
    ///
    /// # Returns
    ///
    /// * `Option<&DeviceModel>` - The model if found, None otherwise
    pub fn get_model(&self, name: &str) -> Option<&DeviceModel> {
        self.models.get(name)
    }

    /// Get a mutable model by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the model
    ///
    /// # Returns
    ///
    /// * `Option<&mut DeviceModel>` - The model if found, None otherwise
    pub fn get_model_mut(&mut self, name: &str) -> Option<&mut DeviceModel> {
        self.models.get_mut(name)
    }

    /// Get all loaded models
    ///
    /// # Returns
    ///
    /// * `&HashMap<String, DeviceModel>` - Map of model names to models
    pub fn get_all_models(&self) -> &HashMap<String, DeviceModel> {
        &self.models
    }

    /// Create a new model
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the model
    /// * `pins` - Map of pin numbers to pin models
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the model was created successfully, an error otherwise
    pub fn create_model(&mut self, name: &str, pins: HashMap<u8, PinModel>) -> Result<()> {
        // Create the model
        let model = DeviceModel {
            name: name.to_string(),
            pins,
        };

        // Validate the model
        model.validate()?;

        // Save the model to a file
        self.save_model(&model)?;

        // Add the model to the map
        self.models.insert(name.to_string(), model);

        Ok(())
    }

    /// Save a model to a file
    ///
    /// # Arguments
    ///
    /// * `model` - The model to save
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the model was saved successfully, an error otherwise
    pub fn save_model(&self, model: &DeviceModel) -> Result<()> {
        let file_path = self.model_dir.join(format!("{}.yaml", model.name));

        // Serialize the model to YAML
        let yaml = serde_yaml::to_string(model).map_err(|e| {
            PoKeysError::ModelParseError(file_path.to_string_lossy().to_string(), e.to_string())
        })?;

        // Write the YAML to a file
        fs::write(&file_path, yaml).map_err(|e| {
            PoKeysError::ModelLoadError(file_path.to_string_lossy().to_string(), e.to_string())
        })?;

        info!("Saved model {} to {}", model.name, file_path.display());
        Ok(())
    }

    /// Delete a model
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the model
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the model was deleted successfully, an error otherwise
    pub fn delete_model(&mut self, name: &str) -> Result<()> {
        // Remove the model from the map
        self.models.remove(name);

        // Delete the model file
        let file_path = self.model_dir.join(format!("{}.yaml", name));

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| {
                PoKeysError::ModelLoadError(file_path.to_string_lossy().to_string(), e.to_string())
            })?;

            info!("Deleted model {}", name);
        }

        Ok(())
    }

    /// Create a copy of a model with a new name
    ///
    /// # Arguments
    ///
    /// * `source_name` - The name of the source model
    /// * `target_name` - The name of the target model
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the model was copied successfully, an error otherwise
    pub fn copy_model(&mut self, source_name: &str, target_name: &str) -> Result<()> {
        // Get the source model
        let source_model = self.get_model(source_name).ok_or_else(|| {
            PoKeysError::ModelLoadError(source_name.to_string(), "Model not found".to_string())
        })?;

        // Create a copy of the model with the new name
        let mut target_model = source_model.clone();
        target_model.name = target_name.to_string();

        // Save the target model
        self.save_model(&target_model)?;

        // Add the target model to the map
        self.models.insert(target_name.to_string(), target_model);

        Ok(())
    }

    /// Validate a model
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the model
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the model is valid, an error otherwise
    pub fn validate_model(&self, name: &str) -> Result<()> {
        // Get the model
        let model = self.get_model(name).ok_or_else(|| {
            PoKeysError::ModelLoadError(name.to_string(), "Model not found".to_string())
        })?;

        // Validate the model
        model.validate()
    }

    /// Get the model directory
    ///
    /// # Returns
    ///
    /// * `&Path` - The model directory
    pub fn get_model_dir(&self) -> &Path {
        &self.model_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_model_manager() {
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

        // Get the model
        let model = manager.get_model("TestModel");
        assert!(model.is_some());

        let model = model.unwrap();
        assert_eq!(model.name, "TestModel");
        assert_eq!(model.pins.len(), 2);

        // Copy the model
        assert!(manager.copy_model("TestModel", "TestModel2").is_ok());

        // Get the copied model
        let model = manager.get_model("TestModel2");
        assert!(model.is_some());

        let model = model.unwrap();
        assert_eq!(model.name, "TestModel2");
        assert_eq!(model.pins.len(), 2);

        // Delete the model
        assert!(manager.delete_model("TestModel").is_ok());

        // Check that the model was deleted
        assert!(manager.get_model("TestModel").is_none());

        // Check that the copied model still exists
        assert!(manager.get_model("TestModel2").is_some());

        // Reload models
        assert!(manager.reload_models().is_ok());

        // Check that the copied model was reloaded
        assert!(manager.get_model("TestModel2").is_some());
    }
}
