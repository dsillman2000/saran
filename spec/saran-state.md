# Saran State Management Specification

## Overview

The `saran-state` crate provides persistent state management for Saran: reading and writing `env.yaml` and `quotas.yaml` files, managing the data directory, and enforcing quota limits.

This crate is used by both the main `saran` CLI (for `saran env` and `saran quotas` commands) and generated wrapper binaries (for variable resolution and quota checking).

> **See also:** [`saran-env.md`](saran-env.md) for the variable resolution chain, [`saran-cli.md`](saran-cli.md) for the `saran env` and `saran quotas` commands.

---

## File Locations

| File               | Default Location                   |
| ------------------ | ---------------------------------- |
| Data directory     | `~/.local/share/saran/`            |
| Environment config | `~/.local/share/saran/env.yaml`    |
| Quota state        | `~/.local/share/saran/quotas.yaml` |

---

## Data Directory

The data directory is always `~/.local/share/saran/` on Unix systems and `%LOCALAPPDATA%\saran\` on Windows.

---

## env.yaml Structure

The `env.yaml` file stores operator-managed variable values for the resolution chain.

```yaml
global:
  GH_REPO: "myorg/myrepo"
  REDIS_HOST: "localhost"

wrappers:
  gh-pr.repo.ro:
    GH_REPO: "myorg/myrepo"
    GH_TOKEN: "ghp_xxxxx"

  redis-cli-info.db.ro:
    REDIS_HOST: "localhost"
    REDIS_PORT: "6379"
```

### Top-Level Keys

| Key        | Type                             | Description                       |
| ---------- | -------------------------------- | --------------------------------- |
| `global`   | Map<String, String>              | Variables applied to all wrappers |
| `wrappers` | Map<String, Map<String, String>> | Per-wrapper variable overrides    |

---

## quotas.yaml Structure

The `quotas.yaml` file tracks runtime quota state (remaining execution counts).

```yaml
gh-issue-create.repo.rw.quota:
  create:
    remaining: 5
    limit: 5

glab-mr-note.mr.rw.quota:
  create:
    remaining: 10
    limit: 10
```

### Top-Level Keys

Each key is a wrapper name. The value is a map of command names to `QuotaStateEntry`.

### Quota Entry Fields

| Field       | Type | Description                                 |
| ----------- | ---- | ------------------------------------------- |
| `remaining` | u32  | Executions remaining before quota exhausted |
| `limit`     | u32  | Maximum executions (set at initialization)  |

---

## Key Types

### `SaranState`

Main entry point for state operations. Constructed via `SaranState::new()` or `SaranState::new_with_dir(path)`.

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
```

### `QuotaStateEntry`

Individual quota entry:

```rust
pub struct QuotaStateEntry {
    pub remaining: u32,
    pub limit: u32,
}
```

### `StateError`

Unified error type with variants:

- `Io(String)` — File I/O errors
- `Yaml(String)` — YAML parsing/serialization errors
- `Env(String)` — Environment errors (missing data dir, invalid path)
- `Quota(String)` — Quota-specific errors (quota exhausted)

---

## Core Operations

### env.yaml Operations

| Operation     | Method                                             | Description                                  |
| ------------- | -------------------------------------------------- | -------------------------------------------- |
| Read          | `SaranState::read_env_yaml()`                      | Parse `env.yaml` into `SaranEnvYaml`         |
| Set global    | `SaranState::set_global_var(key, value)`           | Add/update variable in `global:` section     |
| Set wrapper   | `SaranState::set_wrapper_var(wrapper, key, value)` | Add/update variable in `wrappers.<wrapper>:` |
| Unset global  | `SaranState::unset_global_var(key)`                | Remove variable from `global:`               |
| Unset wrapper | `SaranState::unset_wrapper_var(wrapper, key)`      | Remove variable from `wrappers.<wrapper>:`   |

### quotas.yaml Operations

| Operation          | Method                                          | Description                                    |
| ------------------ | ----------------------------------------------- | ---------------------------------------------- |
| Read               | `SaranState::read_quotas_yaml()`                | Parse `quotas.yaml` into `QuotasState`         |
| Get wrapper quotas | `SaranState::get_wrapper_quotas(wrapper)`       | Get quota state for a specific wrapper         |
| Check & decrement  | `SaranState::decrement_quota(wrapper, command)` | Atomically check and decrement remaining count |
| Reset wrapper      | `SaranState::reset_wrapper_quotas(wrapper)`     | Reset all quotas for a wrapper to limits       |
| Reset all          | `SaranState::reset_all_quotas()`                | Reset quotas for all wrappers                  |

---

## Quota Initialization on Install

When `saran install` processes a wrapper with `quotas:` declarations:

1. Parse the wrapper's YAML to extract quota declarations
2. For each quota entry:
   - If `limit:` is a literal integer, use that value
   - If `limit:` references a variable (`$VAR_NAME`), resolve it from the environment
3. Create or replace entries in `quotas.yaml` with `remaining: <limit>` and `limit: <limit>`

---

## Integration Points

| Operation           | Caller             | When                                  |
| ------------------- | ------------------ | ------------------------------------- |
| `env.yaml` read     | `saran-core`       | Wrapper startup (variable resolution) |
| `env.yaml` write    | `saran` CLI        | `saran env set <wrapper> VAR=value`   |
| `quotas.yaml` read  | Generated wrappers | Before command execution              |
| `quotas.yaml` write | Generated wrappers | After successful command (decrement)  |
| `quotas.yaml` reset | `saran` CLI        | `saran quotas reset <wrapper>`        |

---

## Concurrency

**Wrappers are not expected to run concurrently on the same host.**

The `saran-state` crate does **not** implement file locking for `quotas.yaml`. If concurrent execution occurs, quota state may become inconsistent (race condition on read-modify-write).

> **Specification rationale:** The primary use case (LLM agents calling wrappers sequentially) does not require concurrent execution. Adding file locking would add complexity and latency to every quota check.

---

## Error Handling

All state operations return `Result<T, StateError>` for predictable error handling.

### Example Error Messages

| Error Type | Message                                                                           |
| ---------- | --------------------------------------------------------------------------------- |
| `Io`       | `failed to open env.yaml: No such file or directory`                              |
| `Yaml`     | `failed to parse env.yaml: did not find expected node`                            |
| `Env`      | `data directory must be absolute, got: relative/path`                             |
| `Env`      | `failed to resolve data directory: HOME not set`                                  |
| `Quota`    | `quota exhausted for command "create" in wrapper "gh-issue-create.repo.rw.quota"` |

---

## Dependencies

| Crate         | Purpose                            |
| ------------- | ---------------------------------- |
| `saran-core`  | `SaranEnvYaml` type (re-exported)  |
| `saran-types` | Core type definitions              |
| `serde_yaml`  | YAML serialization/deserialization |
| `thiserror`   | `StateError` enum derivation       |

---

## Testing

Tests for this crate are documented in [`spec/tests/unit/06-state-management.md`](tests/unit/06-state-management.md).

Test IDs follow the pattern:

- **W-##** — env.yaml write operations
- **U-##** — env.yaml unset operations
- **R-##** — env.yaml read operations
- **Q-##** — quotas.yaml read operations
- **D-##** — quota decrement operations
- **RS-##** — quota reset operations

Tests use temporary directories to avoid polluting the user's actual data directory.
