//! Main state management types and data directory resolution.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::StateError;

/// Main entry point for Saran state operations.
///
/// Provides methods for reading/writing environment variables and
/// managing quota state. All operations are scoped to the data directory
/// determined by `SARAN_DATA_DIR` environment variable or default location.
#[derive(Debug, Clone)]
pub struct SaranState {
    data_dir: PathBuf,
}

impl SaranState {
    /// Creates a new `SaranState` instance using the configured data directory.
    ///
    /// # Data Directory Resolution
    ///
    /// 1. `SARAN_DATA_DIR` environment variable (if set)
    /// 2. Default: `$HOME/.local/share/saran/` (Unix) or `%LOCALAPPDATA%\saran\` (Windows)
    ///
    /// # Errors
    ///
    /// Returns `StateError::Env` if:
    /// - The data directory cannot be determined (e.g., `HOME` not set)
    /// - The data directory cannot be created
    pub fn new() -> Result<Self, StateError> {
        let data_dir = resolve_data_dir()?;
        Ok(Self { data_dir })
    }

    /// Returns the path to the data directory.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Returns the path to `env.yaml` within the data directory.
    pub fn env_path(&self) -> PathBuf {
        self.data_dir.join("env.yaml")
    }

    /// Returns the path to `quotas.yaml` within the data directory.
    pub fn quotas_path(&self) -> PathBuf {
        self.data_dir.join("quotas.yaml")
    }

    /// Ensures the data directory exists, creating it if necessary.
    ///
    /// Called internally before file operations that require the directory.
    pub fn ensure_data_dir(&self) -> Result<(), StateError> {
        if !self.data_dir.exists() {
            fs::create_dir_all(&self.data_dir)
                .map_err(|e| StateError::env(format!("failed to create data directory: {}", e)))?;
        }
        Ok(())
    }
}

/// Resolves the Saran data directory path.
///
/// See `SaranState::new()` for resolution priority.
fn resolve_data_dir() -> Result<PathBuf, StateError> {
    // 1. SARAN_DATA_DIR environment variable
    if let Ok(custom_dir) = env::var("SARAN_DATA_DIR") {
        let path = PathBuf::from(custom_dir);
        if path.is_absolute() {
            return Ok(path);
        } else {
            return Err(StateError::env("SARAN_DATA_DIR must be an absolute path"));
        }
    }

    // 2. Default platform-specific location
    let home_dir = get_home_dir()?;
    #[cfg(target_family = "unix")]
    let default_path = home_dir.join(".local/share/saran");
    #[cfg(target_family = "windows")]
    let default_path = home_dir.join("AppData/Local/saran");

    Ok(default_path)
}

/// Returns the user's home directory.
///
/// # Errors
///
/// Returns `StateError::Env` if the home directory cannot be determined.
fn get_home_dir() -> Result<PathBuf, StateError> {
    #[cfg(target_family = "unix")]
    let home_var = "HOME";
    #[cfg(target_family = "windows")]
    let home_var = "USERPROFILE";

    env::var(home_var)
        .map(PathBuf::from)
        .map_err(|_| StateError::env(format!("{} environment variable not set", home_var)))
}
