# saran-core

Runtime execution layer for generated Saran wrapper binaries.

This crate provides the core runtime functionality that generated wrapper binaries depend on for variable resolution, token substitution, and command-line argument assembly.

## Responsibilities

### 1. Variable Resolution

Resolve variables from `env.yaml` with a four-layer priority chain:

1. **Per-wrapper namespace** in `env.yaml` (highest priority)
2. **Global namespace** in `env.yaml`
3. **Host environment variables**
4. **Default values** from wrapper `vars:` declaration (lowest priority)

**Public API:**

- `resolve_vars()` — Main function implementing the priority chain
- `SaranEnvYaml::from_yaml()` — Parse `env.yaml` from string
- `SaranEnv` — Type alias for resolved variables: `HashMap<String, SaranEnvVar>`
- `SaranEnvVar` — A resolved variable with its value and source scope
- `VariableResolutionError` — Returned when required variables are missing

**Behavior:**

- Empty string values are valid and preserved (not skipped)
- Required variables without any value are collected in `missing_required` list
- Optional variables with no value are excluded from the result entirely
- Special characters and Unicode are preserved exactly
- Variable names are case-sensitive

### 2. Substitution Resolution

Resolve `$VAR_NAME` tokens in strings, supporting two distinct contexts with different behavior:

#### Action Context

Allows substitution of both `vars:` names and positional argument values.
Used when building command-line arguments for child process execution.

**Public API:**

- `resolve_substitution()` — Resolve tokens in action argument context
- `ResolutionContext` — Contains resolved vars and caller arguments
- `SubstitutionError` — Returned if a referenced variable cannot be resolved

#### Help Context

Allows substitution of only `vars:` names, tolerates missing values by showing literal `$VAR_NAME`.
Used when generating help text for the CLI.

**Public API:**

- `resolve_help_text()` — Resolve tokens in help text context
- Returns unresolved references as literal `$VAR_NAME` (no error)

**Behavior:**

- No recursive substitution: if `$FOO="BAR"`, substitutes `"BAR"` only
- Empty values substituted literally
- Whitespace preserved exactly
- Dollar signs in values are not re-parsed

### 3. Argument Assembly

Build the complete argv array for executing a child process, assembling:

1. The executable name
2. Fixed action arguments (with variable substitution)
3. Optional flags (in declaration order, with caller-provided values)

**Public API:**

- `build_argv()` — Main function assembling the complete argv array
- `AssemblyContext` — Contains resolved vars, caller args, and optional flag values
- `OptionalFlagValue` — Represents a flag value: `String`, `Multiple` (for repeated flags), or `Bool`
- `ArgvAssemblyError` — Returned if variable substitution in action args fails

**Behavior:**

- Fixed arguments are preserved exactly (no shell interpolation or word splitting)
- Variable substitution applied to action arguments
- Optional flags appended after fixed arguments in declaration order
- Non-repeated str/int/enum flags: `[flag_name, value]`
- Repeated str/int/enum flags: `[flag_name, value1, flag_name, value2, ...]`
- Bool flags: `[flag_name]` (no value)
- `passes_as` override supported for custom flag names in argv

## Type Organization

All types are re-exported from `lib.rs` for public API clarity:

### Variable Resolution

- `SaranEnvScope`, `SaranEnvVar`, `SaranEnv`, `SaranEnvYaml`
- `VariableResolutionResult`, `VariableResolutionError`, `SaranEnvYamlError`
- Function: `resolve_vars()`

### Substitution Resolution

- `ResolutionContext`, `SubstitutionError`
- Functions: `resolve_substitution()`, `resolve_help_text()`
- Re-exported: `parse_tokens`, `ParsedTemplate`, `Token` (from `saran-parser`)

### Argument Assembly

- `AssemblyContext`, `OptionalFlagValue`, `ArgvAssemblyError`
- Function: `build_argv()`

## Test Coverage

Total: **45 unit tests** across all functionality areas.

| Area                    | Tests | Spec                                            |
| ----------------------- | ----- | ----------------------------------------------- |
| Variable Resolution     | 16    | `spec/tests/unit/03-variable-resolution.md`     |
| Substitution Resolution | 10    | `spec/tests/unit/04-substitution-resolution.md` |
| Argument Assembly       | 19    | `spec/tests/unit/05-argument-assembly.md`       |

All tests use the `saran_test!` macro for spec ID tracking and pass with zero warnings.

## Running Tests

```bash
cargo test -p saran-core
```

## Dependencies

- `saran-types` — Wrapper definition types
- `saran-parser` — Token parsing and type definitions
- `thiserror` — Error type derivation
- `serde_yaml` — YAML deserialization

## Integration with Generated Code

Generated wrapper binaries follow this execution flow:

1. **At startup** — Parse `env.yaml` and call `resolve_vars()` to get all resolved variables
2. **Per command** — Parse positional arguments and optional flags from the CLI
3. **Per action** — Call `build_argv()` to assemble the child process argv
4. **Execution** — Spawn child process with assembled argv via `std::process::Command`

All variable and flag resolution is deterministic and happens at compile time or invocation time, with no dynamic YAML parsing at runtime.
