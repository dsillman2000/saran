# saran-state Developer Guide

## Context

The `saran-state` crate handles persistent state management for Saran: reading/writing `env.yaml` and `quotas.yaml` files, managing the data directory, and enforcing quotas. This crate is used by both the main `saran` CLI and generated wrapper binaries.

## Current Implementation Status

**Phases 2-4: Complete** — env.yaml and quotas.yaml operations implemented

- ✅ Workspace integration (Cargo.toml updated)
- ✅ Basic crate structure (lib.rs, modules)
- ✅ Module implementations (env, quotas, error, state)
- ✅ Tests (24 unit tests, all passing)

## Design Decisions

| Decision       | Value                   | Rationale                                     |
| -------------- | ----------------------- | --------------------------------------------- |
| Single crate   | `saran-state`           | Both env and quotas need data dir access      |
| File locking   | None                    | Concurrent execution not supported (per spec) |
| Error handling | `thiserror`             | Consistent with other saran crates            |
| Storage format | YAML                    | Human-readable, matches existing spec         |
| Data directory | `~/.local/share/saran/` | Can be overridden by `SARAN_DATA_DIR` env var |

## Module Structure

```
src/
├── lib.rs          # Re-exports and crate documentation
├── error.rs        # Error types (StateError, EnvError, QuotaError)
├── state.rs        # Main SaranState struct, data directory resolution
├── env.rs          # env.yaml operations (SaranEnvYaml)
└── quotas.rs       # quotas.yaml operations (QuotasState, QuotaEntry)
```

## Data Directory Resolution

The data directory is determined as follows (highest priority first):

1. `SARAN_DATA_DIR` environment variable (if set)
2. Default: `$HOME/.local/share/saran/` (Unix), `%LOCALAPPDATA%\saran\` (Windows)

Functions must use `SaranState::data_dir()` or `SaranState::new()` to ensure consistent path resolution.

## Key Types

### `SaranState`

Main entry point for state operations. Provides methods for:

- Reading/writing environment variables
- Reading/decrementing/resetting quotas
- Data directory management

### `SaranEnvYaml`

In-memory representation of `env.yaml` with two top-level maps:

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
```

### `QuotaStateEntry`

Individual quota entry with remaining count and limit (runtime state, not declaration):

```rust
pub struct QuotaStateEntry {
    pub remaining: u32,
    pub limit: u32,
}
```

## Error Handling

All errors are unified under `StateError` enum with variants:

- `Io` — File I/O errors
- `Yaml` — YAML parsing/serialization errors
- `Env` — Environment-specific errors (e.g., missing data dir)
- `Quota` — Quota-specific errors (e.g., quota exhausted)

## Concurrency

The crate assumes **no concurrent execution** of Saran wrappers. No file locking is implemented per the specification.

## Testing Strategy

Tests should follow the unit test specifications in `spec/tests/unit/06-state-management.md`:

- **W-01 through W-05** — env.yaml write operations
- **R-01 through R-04** — env.yaml read operations
- **U-01 through U-04** — env.yaml unset operations
- **Q-01 through Q-04** — quotas.yaml read operations
- **D-01 through D-04** — quota decrement operations
- **RS-01 through RS-03** — quota reset operations

Tests should use `tempfile` crate to avoid polluting the user's actual data directory.

## Integration Points

| Operation           | Who Calls          | When                     |
| ------------------- | ------------------ | ------------------------ |
| `env.yaml` read     | `saran-core`       | Wrapper startup          |
| `env.yaml` write    | `saran` CLI        | `saran env set`          |
| `quotas.yaml` read  | Generated wrappers | Before command execution |
| `quotas.yaml` write | Generated wrappers | After successful command |
| `quotas.yaml` reset | `saran` CLI        | `saran quotas reset`     |

## Quota Initialization on Install

When `saran install` runs:

1. Parse wrapper's `quotas:` declaration from YAML
2. Resolve variable references in quota limits
3. Create/replace entries in `quotas.yaml` with `remaining: limit`

This logic will be implemented in a future phase.

## Dependencies

- **saran-core** — `SaranEnvYaml` type (re-exported)
- **saran-types** — Type definitions
- **serde_yaml** — YAML serialization
- **thiserror** — Error type derivation
- **anyhow** — Convenient error handling in CLI context

## Development Workflow

### Adding a New Function

1. Determine which module it belongs in (env, quotas, state)
2. Add function with appropriate error handling
3. Add unit test following spec ID pattern
4. Run tests: `cargo test -p saran-state`

### Debugging Test Failures

```bash
# Run specific test with output
cargo test -p saran-state test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test -p saran-state test_name
```

## Future Considerations

- **Caching** — Could cache file reads for performance
- **Validation** — Validate env.yaml structure on write
- **Migration** — Versioned state files for backward compatibility
