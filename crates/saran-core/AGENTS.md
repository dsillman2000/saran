# saran-core Developer Guide

## Context

The `saran-core` crate is the runtime execution layer for generated Saran wrapper binaries. It provides three core capabilities: variable resolution from `env.yaml`, token substitution in strings, and command-line argument assembly for child process execution.

## Current Implementation Status

**All functionality complete with comprehensive test coverage:**

- ✅ Variable Resolution — 16 unit tests
- ✅ Substitution Resolution — 10 unit tests
- ✅ Argument Assembly — 19 unit tests
- **Total:** 45 tests, all passing, zero warnings

## Key Crate Structure

```
src/
├── lib.rs         # All types and functions (~650 lines)
└── tests.rs       # 45 unit tests with spec ID tags (~970 lines)

README.md           # Crate responsibilities and public API documentation
AGENTS.md          # This file
Cargo.toml         # Dependencies
```

## Module Organization

The crate is organized as a single module (`lib.rs`) with three conceptual sections:

### 1. Variable Resolution (Lines 1-256)

**Types:**

- `SaranEnvScope` — Enum for priority chain positions
- `SaranEnvVar` — Resolved variable with value and source
- `SaranEnv` — Type alias: `HashMap<String, SaranEnvVar>`
- `SaranEnvYaml` — Parsed `env.yaml` structure
- `VariableResolutionResult` — Result type with resolved map and missing_required
- Error types: `SaranEnvYamlError`, `VariableResolutionError`

**Functions:**

- `SaranEnvYaml::from_yaml()` — Parse YAML string to structure
- `resolve_vars()` — Main function implementing 4-layer priority chain

**Tests (16 total):**

- P-01 through P-04: Priority chain behavior
- E-01 through E-03: Edge cases (empty strings, special chars, case sensitivity)
- ER-01, ER-02: Error conditions (required vars, optional omission)
- I-01 through I-03: Isolation and scope tracking
- T-01, T-02: YAML parsing
- MULTI, MIXED: Additional comprehensive tests

### 2. Substitution Resolution (Lines 290-435)

**Types:**

- `ResolutionContext` — Contains resolved vars and caller args
- `SubstitutionError` — Error for unresolved variables

**Functions:**

- `resolve_substitution()` — Token substitution for action context
- `resolve_help_text()` — Token substitution for help context (tolerates missing values)

**Re-exports:**

- `parse_tokens`, `ParsedTemplate`, `Token` from `saran-parser`

**Tests (10 total):**

- VR-01, VR-02: Value resolution (vars and args)
- CS-01, CS-02, CS-03: Context-specific behavior
- EC-01 through EC-05: Edge cases (empty values, whitespace, dollar signs, recursion, size)

### 3. Argument Assembly (Lines 437-648)

**Types:**

- `OptionalFlagValue` — Enum: String, Multiple, Bool
- `AssemblyContext` — Contains resolved vars, caller args, optional flags
- `ArgvAssemblyError` — Error type for assembly failures

**Functions:**

- `build_argv()` — Main function assembling complete argv array
- `From<SubstitutionError> for ArgvAssemblyError` — Error conversion

**Tests (19 total):**

- BA-01 through BA-04: Basic assembly (fixed args, empty strings, whitespace, special chars)
- VS-01 through VS-04: Variable substitution in actions
- OFA-01 through OFA-04: Optional flag appending
- FT-01 through FT-04: Flag type-specific behavior
- MA-01 through MA-03: Multi-action execution

## Specification Compliance

Each implementation section corresponds to a test specification document:

| Section                 | Spec File                                       | Tests | Status      |
| ----------------------- | ----------------------------------------------- | ----- | ----------- |
| Variable Resolution     | `spec/tests/unit/03-variable-resolution.md`     | 16    | ✅ Complete |
| Substitution Resolution | `spec/tests/unit/04-substitution-resolution.md` | 10    | ✅ Complete |
| Argument Assembly       | `spec/tests/unit/05-argument-assembly.md`       | 19    | ✅ Complete |

All tests use the `saran_test!` macro for spec ID tagging, enabling automated tracing from test code to specification.

## Type Design Principles

### 1. **Layered Composition**

Types build upon each other:

- `SaranEnvVar` wraps a value with scope information
- `ResolutionContext` contains resolved vars and caller args
- `AssemblyContext` extends ResolutionContext with optional flags

### 2. **Error Conversion**

Error types implement `From` for automatic conversion when using `?` operator:

- `SubstitutionError` → `ArgvAssemblyError` (via `From` impl)
- Transparent error propagation in `build_argv()`

### 3. **No Public Mutability**

All types are immutable after construction. No setter methods or interior mutability.

### 4. **Owned Strings**

Uses `String` and `HashMap` (not `&str` or references) because:

- Types must be owned (not borrowed from context)
- Allows independent construction for testing
- Matches parsed data ownership model

## Implementation Details

### Variable Resolution Algorithm

```
for each var_decl:
  check per-wrapper namespace (env.yaml)
  if found: add to resolved with PerWrapper scope
  else check global namespace (env.yaml)
  if found: add to resolved with Global scope
  else check host environment
  if found: add to resolved with Host scope
  else check default value in var_decl
  if found: add to resolved with Default scope
  else if required: add to missing_required list
  else: skip (optional with no value)
```

### ParsedTemplate Structure

When parsing templates for substitution:

- Tokens: list of `$VAR_NAME` references with byte positions
- Literals: text segments between/around tokens

**Key insight:** For input with no tokens (e.g., "pr"), literals contains the entire string as a single Literal with `before=false`.

**Reconstruction algorithm:**

1. If no tokens: return the single literal text
2. Otherwise:
   - Add literals[0] (before-text) if `before=true`
   - For each token: add resolved value, then literals[i+1]

### Argv Assembly Flow

```
build_argv():
1. Create argv with executable as first element
2. For each action arg:
   - Parse for tokens
   - Substitute variables/args
   - Add to argv
3. For each optional flag (in declaration order):
   - If provided by caller:
     - Get pass_as override (if any)
     - Append based on flag type:
       * str/int/enum non-repeated: [flag_name, value]
       * str/int/enum repeated: [flag_name, v1, flag_name, v2, ...]
       * bool: [flag_name]
```

## Testing Strategy

All 45 tests follow a consistent pattern:

```rust
saran_test!("SPEC-ID", test_function_name, {
    // Arrange: Set up inputs
    let input = ...;

    // Act: Call function
    let result = function(&input).unwrap();

    // Assert: Verify output
    assert_eq!(result, expected);
});
```

**Spec ID Format:**

- `P-01` through `P-04`: Priority chain
- `E-01` through `E-03`: Edge cases (variable resolution)
- `ER-01`, `ER-02`: Error conditions
- `I-01` through `I-03`: Isolation and scope
- `T-01`, `T-02`: Type construction
- `MULTI`, `MIXED`: Additional tests
- `VR-01`, `VR-02`: Value resolution (substitution)
- `CS-01` through `CS-03`: Context-specific behavior
- `EC-01` through `EC-05`: Edge cases (substitution)
- `BA-01` through `BA-04`: Basic assembly
- `VS-01` through `VS-04`: Variable substitution (assembly)
- `OFA-01` through `OFA-04`: Optional flag appending
- `FT-01` through `FT-04`: Flag type-specific behavior
- `MA-01` through `MA-03`: Multi-action execution

## Dependencies

### Direct Dependencies

- **saran-types** (workspace) — `VarDecl`, `OptionalFlag` types
- **saran-parser** (workspace) — `parse_tokens`, `ParsedTemplate`, `Token`
- **thiserror** — Error type derivation (`#[derive(Error)]`)
- **serde_yaml** — YAML deserialization

### Why No circular dependencies?

- `saran-core` depends on `saran-types` and `saran-parser` only
- No other crates should depend on `saran-core` for type definitions
- Only `saran-codegen` and generated binaries call into `saran-core`

## Common Development Tasks

### Adding a New Test

1. Identify which section (variable resolution, substitution, assembly)
2. Choose a spec ID from the appropriate range
3. Write test in `src/tests.rs` with `saran_test!` macro:

```rust
saran_test!("SPEC-ID", test_name, {
    // Test code here
});
```

4. Run: `cargo test -p saran-core`

### Updating Type Documentation

All types have doc comments. When updating behavior:

1. Update the type doc comment to reflect the behavior
2. Update the README.md "Responsibilities" section
3. Run: `cargo doc -p saran-core --open`

### Debugging a Test Failure

```bash
# Run single test with output
cargo test -p saran-core test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test -p saran-core test_name

# Check clippy warnings
cargo clippy -p saran-core
```

## Future Considerations

### 1. **Performance**

Current implementation prioritizes correctness over performance:

- Uses `HashMap` lookups (O(1) average case)
- String cloning in `resolve_substitution` and `build_argv`
- No caching of parsed templates

If needed, could optimize:

- Cache `ParsedTemplate` results for frequently used strings
- Use reference counting for large strings
- Pre-build common argv patterns

### 2. **Error Messages**

Error messages are intentionally simple. Could enhance:

- Add position information for substitution errors
- Provide suggestions for common mistakes
- Better context in argv assembly errors

### 3. **Extended Functionality**

Out of scope but possible future additions:

- Variable validation (e.g., ensure GH_REPO matches org/repo pattern)
- Hook functions before/after action execution
- Environment variable expansion beyond $VAR_NAME

## Code Quality Standards

- **100% test coverage** for all public functions
- **Zero clippy warnings** — `cargo clippy -p saran-core` must be clean
- **Doc comments** on all public types and functions
- **Specification compliance** — all tests tagged with spec IDs

## Troubleshooting

### Tests fail with "parse error" in resolve_substitution

The `parse_tokens` call returned an error. Check if the input string contains invalid token syntax (e.g., `$123` or `$`).

### Argv assembly produces unexpected order

Remember: action args come first, then optional flags. Optional flags are appended in the order they were declared in the wrapper definition.

### ParsedTemplate has unexpected literals

When a string has no tokens (e.g., "plain text"), there's ONE literal with `before=false`. When string starts with a token (e.g., "$VAR text"), literals is EMPTY (token position info used instead).

## Integration Testing

`saran-core` is tested at the unit level. Integration testing happens in:

- `saran-codegen` — tests generated code that calls `saran-core`
- Generated wrapper binaries — end-to-end testing with real processes
