# saran-parser Developer Guide

## Context

The `saran-parser` crate is the parsing layer of the Saran CLI wrapper framework. It converts YAML wrapper files and template strings into structured data types.

## Current Implementation Status

**Token Parsing Responsibility:** ✅ Complete

- Extract `$VAR_NAME` tokens from strings
- 12 unit tests covering token extraction (6 TP tests + 6 additional parsing tests)

**YAML Parsing & Validation Responsibility:** ✅ Complete

- Deserialize YAML wrapper files into `WrapperDefinition`
- Validate YAML structure against the wrapper schema
- Provide detailed error messages with paths
- 59 validation tests organized in 7 categories:
  - **TL (Top-Level):** 8 tests for wrapper name, version, commands
  - **VD (Variable Declarations):** 7 tests for var name format, duplicates, conflicts
  - **CA (Commands & Actions):** 7 tests for command/action structure
  - **OF (Optional Flags):** 12 tests for flag type, format, enum values, bool+repeated
  - **AR (Arguments):** 10 tests for arg type (str only), format, ordering, conflicts
  - **VR (Variable References):** 6 tests for `$VAR_NAME` token validation in actions/help
  - **RE (Requires Section):** 9 tests for CLI requirements, semver, regex patterns

**Total Tests:** 71 (12 Token Parsing + 59 YAML Validation)

## Key Crate Structure

```
src/
├── lib.rs           # Token parser + YAML validation implementation (~1400 lines)
├── tests.rs         # 71 unit tests with spec ID tags (e.g., [TP-01], [VR-03])
tests/
└── fixtures/        # YAML test files organized by category
    ├── tl-all.yaml
    ├── vd-all.yaml
    ├── ca-all.yaml
    ├── of-all.yaml
    ├── ar-all.yaml
    ├── vr-all.yaml
    └── re-all.yaml
```

## Token Parsing Implementation

### Token Parser (`parse_tokens()`)

**Responsibility:** Extract all `$VAR_NAME` tokens from a string.

**Regex Pattern:** `\$([A-Za-z_][A-Za-z0-9_]*)`

- `$` — literal dollar sign
- `[A-Za-z_]` — must start with letter or underscore
- `[A-Za-z0-9_]*` — followed by zero or more alphanumeric chars or underscores
- Greedy: takes the longest valid match

**Behavior:**

- Returns `Ok(ParsedTemplate)` with all tokens and their positions
- Returns `Err(TokenParsingError)` if syntax is invalid (e.g., `$123`)
- Position tracking: byte indices within the input string
- Literal segments: text between/around tokens

**No validation scope:**

- Does NOT check if variables are declared
- Does NOT resolve variable values
- Does NOT validate YAML structure
- Does NOT perform substitution

### Error Type: `TokenParsingError`

Custom error type for token parsing failures. Should include:

- Error kind (e.g., `InvalidTokenStart`, `InvalidCharAfterDollar`)
- Position in string where error occurred
- The offending character (if applicable)

Uses `thiserror` for derivation.

## YAML Parsing & Validation Implementation

The validation pipeline is organized in layers:

```
parse_wrapper()
├── YAML deserialization (serde_yaml)
├── Top-level validation (wrapper name, version, commands)
├── Field-level validation (dependency order)
│   ├── Variable declarations
│   ├── Commands & actions
│   ├── Optional flags
│   └── Positional arguments
└── Cross-reference validation
    ├── Variable references (uses token parser)
    └── Requires section (CLI requirements)
```

### ValidationError Enum

All errors include a `path` field for precise location reporting:

```rust
pub enum ValidationError {
    MissingField { path: String, field: &'static str },
    InvalidFormat { path: String, reason: String },
    InvalidValue { path: String, expected: String, found: String },
    DuplicateKey { path: String, key: String },
    ConflictingFields { path: String, field1: &'static str, field2: &'static str },
    UndeclaredReference { path: String, var_name: String },
    SemverParseError { value: String, error: String },
    RegexCompileError { pattern: String, error: String },
}
```

**Path Format Examples:**

- `vars[0].name` — Field in indexed array
- `commands.list.actions[1].executable` — Nested structure
- `requires[2].version_pattern` — Requires section

### ValidationContext

Tracks state during validation:

```rust
pub struct ValidationContext {
    pub declared_vars: HashSet<String>,
    pub declared_commands: HashSet<String>,
    pub declared_args: HashMap<String, HashSet<String>>,
    pub errors: Vec<ValidationError>,
}
```

Used by validators to:

1. Track declared names to catch conflicts/duplicates
2. Collect errors without early exit
3. Enable cross-validator reference checking (e.g., VR validator checks against `declared_vars`)

### Validation Order

1. **Top-Level Validation** — Check wrapper name, version, commands structure

   - Name present and non-empty
   - Version valid SemVer
   - Commands present and non-empty

2. **Variables Validation** — Validate variable declarations

   - Name format matches `[A-Za-z_][A-Za-z0-9_]*`
   - No duplicates
   - `required` and `default` are mutually exclusive
   - No prefix conflicts (VAR and VAR_X can't both exist)
   - Populates `ctx.declared_vars`

3. **Commands & Actions Validation** — Validate command structure

   - Command name format valid
   - Actions present and non-empty
   - Each action has executable
   - Action has only allowed keys
   - Populates `ctx.declared_commands`

4. **Optional Flags Validation** — Validate flags per command

   - Name and type present
   - Type is str/bool/int/enum
   - Name starts with `--` and has valid chars
   - Enum has values
   - Bool can't have repeated=true
   - No duplicate names per command

5. **Arguments Validation** — Validate positional args per command

   - Name, var_name, type present
   - Type must be `str`
   - Name format valid
   - No duplicates
   - var_name doesn't conflict with declared vars
   - Required args don't follow optional
   - No prefix conflicts in var_names
   - Populates `ctx.declared_args[command_name]`

6. **Variable References Validation** — Validate token references

   - Uses token parser to extract `$VAR_NAME` tokens
   - Checks action arg tokens against declared variables
   - Checks var help text tokens against declared variables
   - Reports invalid token syntax via token parser errors

7. **Requires Section Validation** — Validate CLI requirements
   - `cli` field present (changed to `Option<String>` for validation)
   - `version` field present (changed to `Option<String>` for validation)
   - Version constraint valid SemVer format
   - `version_probe` is array if present, non-empty
   - `version_pattern` is valid regex with exactly 1 capture group
   - No duplicate cli names

### Format Validation Helpers

Helper functions used throughout validators:

- `validate_var_name_format(name: &str)` — Checks `[A-Za-z_][A-Za-z0-9_]*`
- `validate_command_name_format(name: &str)` — Checks alphanumeric + hyphens
- `validate_flag_name_format(name: &str)` — Checks must start with `--`, valid chars
- `validate_semver(version: &str)` — SemVer 2.0.0 compliance
- `validate_semver_constraint(constraint: &str)` — Parses `>=1.0.0 <2.0.0` format
- `validate_regex(pattern: &str)` — Compiles to `Regex`
- `validate_regex_capture_groups(pattern: &str)` — Checks for exactly 1 capture group
- `check_prefix_conflicts(names: &[String])` — Returns conflicting pairs

### Test Fixtures

Fixtures are consolidated YAML files with named test cases:

```yaml
test-case-name:
  name: "test-wrapper"
  version: "1.0.0"
  # ... rest of wrapper definition
```

Loaded via helper:

```rust
let yaml = load_fixture("vr-all.yaml", "vr-01-undeclared-var-in-action");
let result = validate_wrapper(&yaml);
```

## Implementation Guidelines

### Adding a New Validator

1. **Define the validation function:**

   ```rust
   fn validate_xyz(xyz: &[Item], ctx: &mut ValidationContext) -> Vec<ValidationError> {
       let mut errors = Vec::new();
       // validation logic
       errors
   }
   ```

2. **Add to validate_wrapper() pipeline:**

   ```rust
   context.errors.extend(validate_xyz(&wrapper.xyz, &mut context));
   ```

3. **Create test fixture file (if new category):**

   - `tests/fixtures/xyz-all.yaml`
   - Named test cases: `xyz-01-description`, `xyz-02-description`, ...

4. **Implement tests in src/tests.rs:**

   - Use `saran_test!("XYZ-01", test_name, { ... })` macro
   - Load fixture: `load_fixture("xyz-all.yaml", "xyz-01-description")`
   - Call `validate_wrapper(&yaml)` and assert on errors

5. **Document in to-do.md:**
   - Add Task 2X-Y entries
   - Link to spec section
   - Describe test cases

### Testing Strategy

Each test function is tagged with a spec ID:

```rust
saran_test!("VR-03", test_vr_digit_after_dollar, {
    let yaml = load_fixture("vr-all.yaml", "vr-03-digit-after-dollar");
    let result = validate_wrapper(&yaml);

    assert!(result.is_err(), "Expected validation to fail");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidFormat { .. })));
});
```

This allows automated tracing from test code back to specification.

## Key Type Changes from saran-types

**CliRequirement Fields:** Changed `cli` and `version` from `String` to `Option<String>` to enable validation of missing required fields (RE-01, RE-02 tests).

```rust
pub struct CliRequirement {
    #[serde(default)]
    pub cli: Option<String>,  // Was: pub cli: String

    #[serde(default)]
    pub version: Option<String>,  // Was: pub version: String

    #[serde(default)]
    pub version_probe: Option<Vec<String>>,

    #[serde(default)]
    pub version_pattern: Option<String>,
}
```

## Specification References

- **Token Parsing:** `spec/tests/unit/02-token-parsing.md`
- **YAML Validation:** `spec/tests/unit/01-yaml-validation.md`
- **Wrapper Format:** `spec/saran-format.md`
- **Type Definitions:** `crates/saran-types/src/lib.rs`
- **Implementation Plan:** `crates/saran-parser/to-do.md`

## Development Workflow

1. Read the relevant spec section and test cases
2. Implement the validator function in `src/lib.rs` with doc comments
3. Create/update test fixture in `tests/fixtures/`
4. Implement tests in `src/tests.rs` with spec ID tags
5. Run: `cargo test -p saran-parser`
6. Verify all tests pass and clippy is clean
7. Run: `pre-commit run --all-files`
8. Update `to-do.md` with completion status

## Common Pitfalls

- **Validation scope creep:** Each validator should focus on its specific domain
- **Forgetting context population:** Be sure to populate `ctx.declared_*` for later validators
- **Early exit:** Collect ALL errors before returning; don't exit on first error
- **Path precision:** Always include the full path to the problematic field
- **Type handling:** Remember `CliRequirement` fields are `Option<String>` now

## Troubleshooting Common Validation Issues

**Tests failing with "Expected MissingField error":**

- Check fixture YAML has the proper structure
- Verify `serde(default)` is applied to optional fields in types
- Ensure validator checks `is_none()` not `is_empty()`

**Clippy complaining about &Vec:**

- Change signature to use `&[T]` slice instead of `&Vec<T>`

**Tests flagged with "doc list item without indentation":**

- Use `*` instead of `-` for bullet points in doc comments
- Ensure proper indentation for bullet lists

**Validator not being called:**

- Verify `validate_wrapper()` includes call to validator
- Check errors are being collected: `context.errors.extend(...)`

## Code Generation Integration

The **saran-codegen** crate will:

- Accept validated `WrapperDefinition` from this crate's `validate_wrapper()` function
- Generate Rust source code with clap-powered CLI
- Create standalone executable binaries per wrapper

**Contract:**

- Input: Result of `validate_wrapper()` (validated `WrapperDefinition`)
- Output: Rust source code (clap app + subcommands)
- All validation is complete before codegen; no re-validation in codegen

**Assumptions:**

- Codegen will use `saran-types` directly for type information
- All validation responsibilities remain in saran-parser (this crate)
- No blockers identified for integration
