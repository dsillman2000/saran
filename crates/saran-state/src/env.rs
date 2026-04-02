//! Environment variable storage operations (`env.yaml`).

use std::collections::HashMap;
use std::fs;

use saran_core::SaranEnvYaml;

use crate::error::StateError;
use crate::state::SaranState;

impl SaranState {
    /// Reads the current `env.yaml` from disk.
    ///
    /// If the file does not exist, returns an empty `SaranEnvYaml`.
    pub fn get_env(&self) -> Result<SaranEnvYaml, StateError> {
        self.ensure_data_dir()?;
        let path = self.env_path();

        if !path.exists() {
            return Ok(SaranEnvYaml {
                global: HashMap::new(),
                wrappers: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&path).map_err(StateError::Io)?;

        SaranEnvYaml::from_yaml(&content)
            .map_err(|e| StateError::env(format!("failed to parse env.yaml: {}", e)))
    }

    /// Writes the `env.yaml` to disk.
    ///
    /// Overwrites any existing file. Creates the data directory if needed.
    pub fn write_env(&self, env: &SaranEnvYaml) -> Result<(), StateError> {
        self.ensure_data_dir()?;
        let path = self.env_path();

        let yaml = serde_yaml::to_string(env).map_err(StateError::Yaml)?;

        fs::write(&path, yaml).map_err(StateError::Io)
    }

    /// Sets a global variable in `env.yaml`.
    ///
    /// Reads the current file, updates the global map, and writes it back.
    pub fn set_global_var(&self, key: &str, value: &str) -> Result<(), StateError> {
        let mut env = self.get_env()?;
        env.global.insert(key.to_string(), value.to_string());
        self.write_env(&env)
    }

    /// Sets a per-wrapper variable in `env.yaml`.
    ///
    /// Reads the current file, updates the wrapper's map (creating it if needed),
    /// and writes it back.
    pub fn set_wrapper_var(&self, wrapper: &str, key: &str, value: &str) -> Result<(), StateError> {
        let mut env = self.get_env()?;
        let wrapper_map = env
            .wrappers
            .entry(wrapper.to_string())
            .or_insert_with(HashMap::new);
        wrapper_map.insert(key.to_string(), value.to_string());
        self.write_env(&env)
    }

    /// Removes a global variable from `env.yaml`.
    ///
    /// If the variable doesn't exist, this is a no-op.
    pub fn unset_global_var(&self, key: &str) -> Result<(), StateError> {
        let mut env = self.get_env()?;
        env.global.remove(key);
        self.write_env(&env)
    }

    /// Removes a per-wrapper variable from `env.yaml`.
    ///
    /// If the wrapper or variable doesn't exist, this is a no-op.
    pub fn unset_wrapper_var(&self, wrapper: &str, key: &str) -> Result<(), StateError> {
        let mut env = self.get_env()?;
        if let Some(wrapper_map) = env.wrappers.get_mut(wrapper) {
            wrapper_map.remove(key);
            // Clean up empty wrapper map
            if wrapper_map.is_empty() {
                env.wrappers.remove(wrapper);
            }
        }
        self.write_env(&env)
    }
}
