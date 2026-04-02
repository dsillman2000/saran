//! Quota state management (`quotas.yaml`).

use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::StateError;
use crate::state::SaranState;

/// A single quota entry in `quotas.yaml`.
///
/// Tracks remaining executions and the configured limit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuotaStateEntry {
    /// Number of executions remaining.
    pub remaining: u32,
    /// Maximum allowed executions (from wrapper declaration or variable).
    pub limit: u32,
}

/// In-memory representation of `quotas.yaml`.
///
/// Outer map: wrapper name → inner map
/// Inner map: command name → quota entry
pub type QuotasState = HashMap<String, HashMap<String, QuotaStateEntry>>;

impl SaranState {
    /// Reads the current `quotas.yaml` from disk.
    ///
    /// If the file does not exist, returns an empty `QuotasState`.
    pub fn get_quotas(&self) -> Result<QuotasState, StateError> {
        self.ensure_data_dir()?;
        let path = self.quotas_path();

        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&path).map_err(StateError::Io)?;

        serde_yaml::from_str(&content).map_err(StateError::Yaml)
    }

    /// Writes the `quotas.yaml` to disk.
    ///
    /// Overwrites any existing file. Creates the data directory if needed.
    pub fn write_quotas(&self, quotas: &QuotasState) -> Result<(), StateError> {
        self.ensure_data_dir()?;
        let path = self.quotas_path();

        let yaml = serde_yaml::to_string(quotas).map_err(StateError::Yaml)?;

        fs::write(&path, yaml).map_err(StateError::Io)
    }

    /// Gets quota entries for a specific wrapper.
    ///
    /// Returns `None` if the wrapper has no quota entries.
    pub fn get_wrapper_quotas(
        &self,
        wrapper: &str,
    ) -> Result<Option<HashMap<String, QuotaStateEntry>>, StateError> {
        let quotas = self.get_quotas()?;
        Ok(quotas.get(wrapper).cloned())
    }

    /// Decrements the remaining count for a wrapper's command by 1.
    ///
    /// # Errors
    ///
    /// Returns `StateError::Quota` if:
    /// - The wrapper or command has no quota entry
    /// - The remaining count is already 0
    pub fn decrement_quota(&self, wrapper: &str, command: &str) -> Result<(), StateError> {
        let mut quotas = self.get_quotas()?;

        let wrapper_map = quotas.get_mut(wrapper).ok_or_else(|| {
            StateError::quota(format!("wrapper '{}' not found in quotas", wrapper))
        })?;

        let entry = wrapper_map.get_mut(command).ok_or_else(|| {
            StateError::quota(format!(
                "command '{}' not found in wrapper '{}'",
                command, wrapper
            ))
        })?;

        if entry.remaining == 0 {
            return Err(StateError::quota(format!(
                "quota exhausted for {}:{}",
                wrapper, command
            )));
        }

        entry.remaining -= 1;
        self.write_quotas(&quotas)
    }

    /// Resets all quotas for a wrapper to their limit values.
    ///
    /// If the wrapper has no quota entries, this is a no-op.
    pub fn reset_wrapper_quotas(&self, wrapper: &str) -> Result<(), StateError> {
        let mut quotas = self.get_quotas()?;

        if let Some(wrapper_map) = quotas.get_mut(wrapper) {
            for entry in wrapper_map.values_mut() {
                entry.remaining = entry.limit;
            }
            self.write_quotas(&quotas)
        } else {
            Ok(())
        }
    }

    /// Resets all quotas for all wrappers to their limit values.
    pub fn reset_all_quotas(&self) -> Result<(), StateError> {
        let mut quotas = self.get_quotas()?;

        for wrapper_map in quotas.values_mut() {
            for entry in wrapper_map.values_mut() {
                entry.remaining = entry.limit;
            }
        }

        self.write_quotas(&quotas)
    }
}
