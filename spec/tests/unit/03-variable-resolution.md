# Variable Resolution Unit Test Plan

## Overview

Unit tests for the variable resolution system defined in `saran-env.md`. These tests validate the layered priority chain and the behavior of all types involved in declaring and resolving variables.

**Total tests: 14**

## Types Under Test

| Type            | Purpose                                                                   |
| --------------- | ------------------------------------------------------------------------- |
| `SaranVarDecl`  | Declares a single variable with name, required flag, and optional default |
| `SaranVarsDecl` | Collection of `SaranVarDecl` entries                                      |
| `SaranEnvVar`   | Represents a resolved variable with scope and value                       |
| `SaranEnvScope` | Enum tracking the source of a resolved value                              |
| `SaranEnv`      | HashMap of variable names to resolved variables                           |
| `SaranEnvYaml`  | Parsed structure of `env.yaml`                                            |
| `resolve_vars`  | Core resolution function implementing the priority chain                  |

## Core Requirements to Test

### 1. Priority Chain (Highest → Lowest)

- Per-wrapper → Global → Host → Default
- First source with value wins
- Lower sources never consulted

### 2. Error Conditions

- Required variable with no value → `missing_required`
- Optional variable with no value → excluded from result
- Empty string is valid value

### 3. Isolation

- Per-wrapper values only affect named wrapper
- Global values affect all wrappers

## Test Specifications

### Priority Chain Tests

| ID   | Test Purpose           | Test Case Description                                                       | Expected Result                                |
| ---- | ---------------------- | --------------------------------------------------------------------------- | ---------------------------------------------- |
| P-01 | Highest priority wins  | Variable has values in all four layers (per-wrapper, global, host, default) | Value from per-wrapper layer, scope=PerWrapper |
| P-02 | Fallback to global     | Variable missing from per-wrapper but present in global, host, and default  | Value from global layer, scope=Global          |
| P-03 | Fallback to host       | Variable missing from per-wrapper and global, present in host and default   | Value from host layer, scope=Host              |
| P-04 | Default as last resort | Variable only has a default value, absent from all other layers             | Value from default, scope=Default              |

### Edge Cases & Value Preservation

| ID   | Test Purpose                  | Test Case Description                                          | Expected Result                                           |
| ---- | ----------------------------- | -------------------------------------------------------------- | --------------------------------------------------------- |
| E-01 | Empty string is valid value   | Variable value is empty string "" (not null/absent)            | Resolves successfully with empty value, appropriate scope |
| E-02 | Special characters preserved  | Value contains newlines, quotes, Unicode, shell metacharacters | Value preserved exactly as provided, no transformations   |
| E-03 | Case-sensitive variable names | Variables `VAR` and `var` are distinct                         | Each resolves independently, no cross-contamination       |

### Error Conditions & Missing Values

| ID    | Test Purpose              | Test Case Description                                            | Expected Result                                                              |
| ----- | ------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| ER-01 | Required variable missing | Variable declared `required: true` with no value in any layer    | Variable name appears in `missing_required` list, excluded from resolved map |
| ER-02 | Optional variable omitted | Variable declared `required: false` with no value and no default | Variable excluded from resolved map entirely (not present with empty value)  |

### Isolation & Scope Tracking

| ID   | Test Purpose                | Test Case Description                                                  | Expected Result                                                                           |
| ---- | --------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| I-01 | Per-wrapper isolation       | Variable set for wrapper A in per-wrapper layer, not set for wrapper B | Wrapper A sees per-wrapper value (scope=PerWrapper), wrapper B sees next available source |
| I-02 | Global affects all wrappers | Variable set in global layer, no per-wrapper override                  | All wrappers see global value (scope=Global)                                              |
| I-03 | Scope correctly tracked     | Variable resolved from each possible source                            | Each resolved `SaranEnvVar` has correct `scope` field matching actual source              |

### Type Construction & Basic Parsing

| ID   | Test Purpose          | Test Case Description                                 | Expected Result                                             |
| ---- | --------------------- | ----------------------------------------------------- | ----------------------------------------------------------- |
| T-01 | Empty YAML parsing    | Empty or whitespace-only YAML document                | Parses successfully with empty global and wrappers sections |
| T-02 | Complete YAML parsing | YAML with both global and wrappers sections populated | Both sections parsed correctly with all key-value pairs     |

## Test Data Examples

### Priority Test P-01 (Highest priority wins)

```yaml
# env.yaml
global:
  GH_REPO: "org/global-repo"
wrappers:
  gh-pr.repo.ro:
    GH_REPO: "org/per-wrapper-repo"

# SaranVarDecl
name: GH_REPO, required: false, default: "org/default-repo"

# Host environment
GH_REPO="org/host-repo"

# Expected for wrapper "gh-pr.repo.ro"
resolved["GH_REPO"] = { value: "org/per-wrapper-repo", scope: PerWrapper }
```

### Error Test ER-01 (Required variable missing)

```yaml
# env.yaml (empty sections)

# SaranVarDecl
name: GH_TOKEN, required: true, default: None

# Host environment (empty)

# Expected
missing_required = ["GH_TOKEN"]
resolved = {}  # Empty map
```

### Isolation Test I-01 (Per-wrapper isolation)

```yaml
# env.yaml
wrappers:
  wrapper-a:
    TARGET: "value-for-a"
  # wrapper-b has no per-wrapper entry

# SaranVarDecl
name: TARGET, required: false, default: "default-value"

# Expected for wrapper-a
resolved["TARGET"] = { value: "value-for-a", scope: PerWrapper }

# Expected for wrapper-b
resolved["TARGET"] = { value: "default-value", scope: Default }
```

## Out of Scope

These are handled by other test suites:

- YAML validation (schema rules beyond basic parsing)
- `$VAR_NAME` substitution in action arrays
- Process execution and I/O
- CLI integration
- Version constraint checking

## Dependencies

Tests require standard Rust libraries for collections and type conversions. Additional dependencies (such as YAML parsing) will be selected during implementation.
