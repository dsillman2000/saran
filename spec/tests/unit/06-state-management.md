# State Management Unit Test Plan

## Overview

Unit tests for the state management system defined in `saran-env.md`. These tests validate the reading, writing, and modification of `env.yaml` and `quotas.yaml` files via the `saran-state` crate.

**Total tests: 28**

> **Prerequisite:** Review [`saran-env.md`](../../saran-env.md) for the `env.yaml` and `quotas.yaml` formats, priority chains, and quota behavior.

## Types Under Test

| Type                   | Purpose                                                     |
| ---------------------- | ----------------------------------------------------------- |
| `SaranState`           | Main state manager for data directory and file I/O          |
| `SaranEnvYaml`         | Parsed structure of `env.yaml` (global + wrappers sections) |
| `QuotasState`          | Parsed structure of `quotas.yaml`                           |
| `QuotaEntry`           | Single quota: remaining + limit for a command               |
| `set_global_var`       | Write a variable to the global namespace                    |
| `set_wrapper_var`      | Write a variable to a per-wrapper namespace                 |
| `unset_global_var`     | Remove a variable from the global namespace                 |
| `unset_wrapper_var`    | Remove a variable from a per-wrapper namespace              |
| `get_quotas`           | Read quota state for a wrapper                              |
| `decrement_quota`      | Atomically decrement remaining count                        |
| `reset_wrapper_quotas` | Reset all quotas for a wrapper to limits                    |
| `reset_all_quotas`     | Reset quotas for all wrappers                               |

## Core Requirements to Test

### 1. env.yaml Operations

- Set global variable persists to file
- Set wrapper variable persists to file
- Unset removes variable from file
- Multiple operations in sequence work correctly
- File is created if it doesn't exist

### 2. quotas.yaml Operations

- Reading quota state returns correct remaining/limit values
- Decrementing reduces remaining by 1
- Decrementing at zero returns not-allowed
- Reset restores remaining to limit
- Reset all clears all wrapper quotas

### 3. Integration with Installation

- Quota limits initialized from wrapper YAML on install
- Existing quotas replaced on reinstall
- Variable reference resolution in quota limits

---

## Test Specifications

### env.yaml Write Operations

| ID   | Test Purpose              | Test Case Description                                     | Expected Result                                    |
| ---- | ------------------------- | --------------------------------------------------------- | -------------------------------------------------- |
| W-01 | Set global variable       | Call `set_global_var("GH_REPO", "org/repo")`              | Variable persisted in `global:` section            |
| W-02 | Set wrapper variable      | Call `set_wrapper_var("gh-pr.ro", "GH_REPO", "org/repo")` | Variable persisted in `wrappers.gh-pr.ro:` section |
| W-03 | Set multiple vars at once | Set GH_REPO and GH_TOKEN in same wrapper                  | Both persisted, correct nesting                    |
| W-04 | Overwrite existing value  | Set GH_REPO twice with different values                   | Second value overwrites first                      |
| W-05 | File created if missing   | Write to non-existent data directory                      | `env.yaml` created with correct content            |

### env.yaml Unset Operations

| ID   | Test Purpose               | Test Case Description                           | Expected Result                       |
| ---- | -------------------------- | ----------------------------------------------- | ------------------------------------- |
| U-01 | Unset global variable      | Unset a global var that exists                  | Variable removed from `global:`       |
| U-02 | Unset wrapper variable     | Unset a per-wrapper var that exists             | Variable removed from wrapper section |
| U-03 | Unset non-existent var     | Unset a variable that doesn't exist             | No error, file unchanged              |
| U-04 | Unset cascades to fallback | Unset per-wrapper, then resolve var for wrapper | Falls through to global/host/default  |

### env.yaml Read Operations

| ID   | Test Purpose            | Test Case Description            | Expected Result                   |
| ---- | ----------------------- | -------------------------------- | --------------------------------- |
| R-01 | Empty env.yaml          | Parse empty file                 | Returns empty global and wrappers |
| R-02 | Only global section     | Parse file with only `global:`   | Wrappers section empty            |
| R-03 | Only wrappers section   | Parse file with only `wrappers:` | Global section empty              |
| R-04 | Both sections populated | Parse complete env.yaml          | Both sections parsed correctly    |

### quotas.yaml Read Operations

| ID   | Test Purpose              | Test Case Description                        | Expected Result             |
| ---- | ------------------------- | -------------------------------------------- | --------------------------- |
| Q-01 | Read wrapper quotas       | Get quotas for wrapper with multiple actions | All action quotas returned  |
| Q-02 | Read single action        | Get quota for specific action in wrapper     | Correct remaining and limit |
| Q-03 | Read non-existent wrapper | Get quotas for wrapper not in file           | Empty quotas returned       |
| Q-04 | Empty quotas.yaml         | Parse empty file                             | Empty map returned          |

### quotas.yaml Decrement Operations

| ID   | Test Purpose                  | Test Case Description               | Expected Result               |
| ---- | ----------------------------- | ----------------------------------- | ----------------------------- |
| D-01 | Decrement from positive       | Decrement with remaining=3          | remaining=2 returned, saved   |
| D-02 | Decrement to zero             | Decrement from remaining=1          | remaining=0 returned          |
| D-03 | Decrement at zero fails       | Decrement when remaining=0          | `allowed: false`, remaining=0 |
| D-04 | Decrement non-existent action | Decrement action not in quotas.yaml | Error or empty                |

### quotas.yaml Reset Operations

| ID    | Test Purpose               | Test Case Description         | Expected Result                      |
| ----- | -------------------------- | ----------------------------- | ------------------------------------ |
| RS-01 | Reset single wrapper       | Reset quotas for one wrapper  | All remaining=limit for that wrapper |
| RS-02 | Reset all wrappers         | Reset quotas for all wrappers | All remaining=limit for all wrappers |
| RS-03 | Reset non-existent wrapper | Reset wrapper not in file     | No error, no changes                 |

### Data Directory Resolution

| ID    | Test Purpose                          | Test Case Description                            | Expected Result                                                    |
| ----- | ------------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------ |
| SD-01 | Default path resolved from `$HOME`    | `$HOME` is set                                   | `data_dir()` returns `$HOME/.local/share/saran`                    |
| SD-02 | Missing `HOME` fails                  | Unset `HOME`                                     | `SaranState::new()` returns `Err` containing "HOME"                |
| SD-03 | `ensure_data_dir` creates missing dir | Path points at a deeply-nested non-existent path | After `ensure_data_dir()`, the full directory hierarchy is created |

---

## Implementation Notes

### Test Setup

Each test should:

1. Create a temporary data directory
2. Initialize `SaranState` with that directory
3. Perform operations
4. Verify file contents and state manager responses
5. Clean up temp directory

### File Format Compatibility

Tests should verify that:

- Written YAML is valid and parseable by other tools
- Existing `env.yaml` format is preserved (backward compatible)
- Quota state survives round-trip (write → read)

### Error Handling

Tests should verify:

- Invalid YAML in existing files produces clear errors
- Missing data directory produces actionable error
- Permission errors are handled gracefully
