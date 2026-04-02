//! Error types for the `saran-state` crate.

use std::io;
use thiserror::Error;

/// Unified error type for all state management operations.
#[derive(Error, Debug)]
pub enum StateError {
    /// I/O error reading or writing state files.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// YAML parsing or serialization error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Environment-specific error (missing data directory, invalid variable name).
    #[error("environment error: {0}")]
    Env(String),

    /// Quota-specific error (quota exhausted, entry not found).
    #[error("quota error: {0}")]
    Quota(String),
}

impl StateError {
    /// Creates an environment error with a formatted message.
    pub fn env(msg: impl Into<String>) -> Self {
        StateError::Env(msg.into())
    }

    /// Creates a quota error with a formatted message.
    pub fn quota(msg: impl Into<String>) -> Self {
        StateError::Quota(msg.into())
    }
}
