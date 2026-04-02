# saran-state

Persistent state management for Saran: `env.yaml` and `quotas.yaml` file I/O.

## Overview

The `saran-state` crate provides the programmatic API for:

- Reading and writing environment variables to `env.yaml`
- Reading and modifying quota state in `quotas.yaml`
- Data directory discovery and management

This crate is used by:

- The main `saran` CLI — for `saran env` and `saran quotas` commands
- Generated wrapper binaries — to check and decrement quotas at runtime

## Key Types

### `SaranState`

Main entry point for state operations:

```rust
let state = SaranState::new()?;

// Set a global variable
state.set_global_var("GH_REPO", "myorg/myrepo")?;

// Set a per-wrapper variable
state.set_wrapper_var("gh-pr.ro", "GH_TOKEN", "gho_xxx")?;

// Read environment
let env = state.get_env()?;
assert_eq!(env.global.get("GH_REPO"), Some(&"myorg/myrepo".to_string()));

// Check quotas
let quotas = state.get_quotas()?;

// Decrement a quota
state.decrement_quota("gh-pr.ro", "comment")?;
```

### `SaranEnvYaml`

In-memory representation of `env.yaml`:

```rust
pub struct SaranEnvYaml {
    pub global: HashMap<String, String>,
    pub wrappers: HashMap<String, HashMap<String, String>>,
}
```

### `QuotasState`

In-memory representation of `quotas.yaml`:

```rust
pub type QuotasState = HashMap<String, HashMap<String, QuotaStateEntry>>;

pub struct QuotaStateEntry {
    pub remaining: u32,
    pub limit: u32,
}
```

## Data Directory

The data directory is determined as follows:

1. `SARAN_DATA_DIR` environment variable (if set)
2. Default: `$HOME/.local/share/saran/` (Unix) or `%LOCALAPPDATA%\saran\` (Windows)

## Error Handling

All errors use the `StateError` enum:

```rust
pub enum StateError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    Env(String),
    Quota(String),
}
```

## Dependencies

- `saran-core` — `SaranEnvYaml` type (re-exported)
- `saran-types` — Type definitions
- `serde_yaml` — YAML serialization
- `thiserror` — Error handling
- `anyhow` — Contextual error handling

## Testing

Run tests with:

```bash
cargo test -p saran-state
```

All 24 unit tests pass, covering:

- env.yaml read/write/unset operations
- quotas.yaml read/decrement/reset operations
