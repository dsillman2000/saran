//! Persistent state management for Saran.
//!
//! This crate handles reading and writing `env.yaml` and `quotas.yaml` files,
//! managing the data directory, and enforcing quotas.
//!
//! ## Responsibilities
//! - Store and retrieve environment variables in `~/.local/share/saran/env.yaml`
//! - Manage quota usage tracking in `~/.local/share/saran/quotas.yaml`
//! - Initialize quota entries when wrappers are installed
//! - Decrement quotas when actions are executed
//!
//! ## Key Types
//! - [`SaranState`] — Main entry point for state operations
//! - [`SaranEnvYaml`] — In-memory representation of `env.yaml`
//! - [`QuotasState`] — In-memory representation of `quotas.yaml`
//!
//! ## File Locations
//! - Data directory: `~/.local/share/saran/` (can be overridden by `SARAN_DATA_DIR` env var)
//! - Environment: `$SARAN_DATA_DIR/env.yaml` (falls back to `~/.local/share/saran/env.yaml`)
//! - Quotas: `$SARAN_DATA_DIR/quotas.yaml` (falls back to `~/.local/share/saran/quotas.yaml`)
//!
//! ## Concurrency
//! The crate assumes no concurrent execution of Saran wrappers.
//! No file locking is implemented per the specification.

pub mod env;
pub mod error;
pub mod quotas;
pub mod state;

pub use error::StateError;
pub use quotas::{QuotaStateEntry, QuotasState};
pub use state::SaranState;

// Re-export SaranEnvYaml from saran-core for convenience
pub use saran_core::SaranEnvYaml;

#[cfg(test)]
mod tests;
