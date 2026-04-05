//! Main state management types and data directory resolution.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::StateError;

/// Main entry point for Saran state operations.
///
/// Provides methods for reading/writing environment variables and
/// managing quota state. All operations are scoped to the data directory
/// `~/.local/share/saran/` on Unix or `%LOCALAPPDATA%\saran\` on Windows.
#[derive(Debug, Clone)]
pub struct SaranState {
    data_dir: PathBuf,
}

impl SaranState {
    /// Creates a new `SaranState` instance using the data directory.
    ///
    /// # Data Directory
    ///
    /// Always uses `$HOME/.local/share/saran/` (Unix) or `%LOCALAPPDATA%\saran\` (Windows).
    /// The location cannot be customized.
    ///
    /// # Errors
    ///
    /// Returns `StateError::Env` if:
    /// - The home directory cannot be determined (e.g., `HOME` not set)
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

    /// Creates a new `SaranState` instance with a custom data directory.
    ///
    /// **For testing only.** Use `SaranState::new()` in production.
    #[cfg(test)]
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }
}

/// Resolves the Saran data directory path.
///
/// Always returns `~/.local/share/saran/` (Unix) or `%LOCALAPPDATA%\saran\` (Windows).
fn resolve_data_dir() -> Result<PathBuf, StateError> {
    // Get home directory
    let home_dir = get_home_dir()?;

    // Platform-specific default path
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
