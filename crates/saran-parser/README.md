# saran-parser

Parse YAML wrapper files and extract variable tokens from strings.

## Purpose

The `saran-parser` crate provides the parsing layer for the Saran CLI wrapper framework. It is responsible for:

1. **Token parsing** — Extract `$VAR_NAME` tokens from template strings
2. **YAML parsing & validation** — Deserialize YAML wrapper files into `WrapperDefinition` types and validate against schema

## Responsibilities

### Token Parsing ✅ Complete

- Extract `$VAR_NAME` variable references from strings using greedy regex matching
- Represent tokens with position information (start/end byte indices)
- Preserve literal text segments between and around tokens
- Report parsing errors with precise location information

**Tests:** 12 tests (TP-01 through TP-06, plus 6 additional)

**Public API:**

- `Token` — A parsed variable reference with name and position
- `ParsedTemplate` — Tokens and literals extracted from a string
- `parse_tokens()` — Extract all `$VAR_NAME` tokens from a string
- `TokenParsingError` — Custom error type for token parsing exceptions

### YAML Parsing & Validation ✅ Complete

Comprehensive YAML parsing and validation with 59 tests across 7 categories:

#### Top-Level & Foundation Validation (8 tests)

- Deserialize YAML wrapper files into `saran_types::WrapperDefinition`
- Validate wrapper name, version (SemVer), and commands structure
- Define `ValidationError` enum with detailed error reporting
- Implement `ValidationContext` to track declared names and collect errors

#### Field-Level Validation (36 tests)

- **Variable Declarations (7 tests):** Name format, duplicates, required/default mutual exclusivity, prefix conflicts
- **Commands & Actions (7 tests):** Command name format, action structure, executable validation
- **Optional Flags (12 tests):** Flag type, name format, enum values, bool+repeated conflict, duplicates
- **Positional Arguments (10 tests):** Arg type (str only), name format, duplicates, ordering (required after optional), prefix conflicts

#### Cross-Reference Validation (15 tests)

- **Variable References (6 tests):** Validate `$VAR_NAME` tokens in action args and var help text using token parser
- **Requires Section (9 tests):** Validate CLI requirements (cli/version fields, semver constraints, regex patterns with capture groups, duplicates)

**All Tests:** 71 tests total (12 Token Parsing + 8 Top-Level + 7 VD + 7 CA + 12 OF + 10 AR + 6 VR + 9 RE)

**Public API:**

- `validate_wrapper(yaml_str: &str)` — Main validation entry point
- `ValidationError` — Detailed error type with paths and context
- `ValidationContext` — Tracks state during validation

## Design Notes

**Token Matching Rules:**

- Pattern: `$` followed by `[A-Za-z_][A-Za-z0-9_]*`
- Greedy matching: longest valid identifier is always taken
- No escape mechanism: bare `$` followed by invalid characters produces an error
- No brace syntax: `${VAR}` is not supported in v1

**Validation Pipeline:**

1. **Top-Level** — Check wrapper name, version, commands structure
2. **Variables** — Validate var declarations, check for conflicts
3. **Commands & Actions** — Validate command/action structure, executables
4. **Flags & Arguments** — Validate optional flags and positional arguments
5. **Cross-References** — Check variable references in actions/help, validate requires section

**Error Reporting:**

- Path-based error locations (e.g., `vars[0].name`, `commands.list.actions[1]`)
- Detailed error context with expected vs. found values
- Multiple errors collected and returned together

**Dependencies:**

- `regex` — Compiled regex for token matching
- `thiserror` — Custom error types
- `serde_yaml` — YAML deserialization
- `semver` — SemVer constraint parsing
- `saran-types` — Type definitions
- `saran-core` — Error collection context

## Testing

Tests are located in `src/tests.rs` and organized by phase. Each test is tagged with its specification ID (e.g., `[TP-01]`) for traceability to the specification.

Run all tests:

```sh
cargo test -p saran-parser
```

Run with output:

```sh
cargo test -p saran-parser -- --nocapture --test-threads=1
```

Run specific test category:

```sh
# Token parsing tests
cargo test -p saran-parser test_parse

# YAML validation tests
cargo test -p saran-parser test_re  # Requires tests
cargo test -p saran-parser test_vr  # Variable reference tests
cargo test -p saran-parser test_ar  # Argument validation tests
```

## Example Usage

```rust
use saran_parser::validate_wrapper;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = fs::read_to_string("wrapper.yaml")?;

    match validate_wrapper(&yaml) {
        Ok(wrapper_def) => {
            println!("✓ Valid wrapper: {}", wrapper_def.name);
            println!("  Version: {}", wrapper_def.version);
            println!("  Commands: {}", wrapper_def.commands.len());
        }
        Err(errors) => {
            eprintln!("✗ Validation failed with {} errors:", errors.len());
            for (i, error) in errors.iter().enumerate() {
                eprintln!("  [{}] {}", i + 1, error);
            }
        }
    }

    Ok(())
}
```

## Wrapper Format Example

```yaml
name: "gh-pr"
version: "1.0.0"

vars:
  - name: OWNER
    required: true
    help: "Repository owner (e.g., $OWNER)"
  - name: REPO
    required: true

commands:
  list:
    args:
      - name: repo
        var_name: REPO
        type: str
        required: false
    actions:
      - gh: ["pr", "list", "--repo", "$OWNER/$REPO"]
        optional_flags:
          - name: --state
            type: enum
            values: ["open", "closed", "all"]

requires:
  - cli: gh
    version: ">=2.0.0 <3.0.0"
    version_probe: ["gh", "--version"]
    version_pattern: "gh version (\\d+\\.\\d+\\.\\d+)"
```
