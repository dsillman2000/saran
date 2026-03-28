# saran-parser Developer Guide

## Context

The `saran-parser` crate is the parsing layer of the Saran CLI wrapper framework. It converts YAML wrapper files and template strings into structured data types.

## Current Implementation Status

**Phase 1 (Token Parsing):** ✅ In Development

- Extract `$VAR_NAME` tokens from strings
- 6 unit tests from spec/tests/unit/02-token-parsing.md

**Phase 2 (YAML Parsing):** ⏳ Future

- Deserialize YAML wrapper files
- 59 unit tests from spec/tests/unit/01-yaml-validation.md

## Key Crate Structure

```
src/
├── lib.rs           # Token parser implementation with doc comments
├── tests.rs         # Unit tests with spec ID tags (e.g., [TP-01])
```

## Implementation Guidelines

### Token Parser (`parse_tokens()`)

**Purpose:** Extract all `$VAR_NAME` tokens from a string.

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

Use `thiserror` for derivation.

### Testing Strategy

Each test function is tagged with a spec ID:

```rust
#[test]
fn test_parse_single_variable_reference() {
    // [TP-01] Basic variable reference: "$GH_REPO" is a single token
    ...
}
```

This allows automated tracing from test code back to specification.

## Specification References

- **Token Parsing Spec:** `spec/tests/unit/02-token-parsing.md`
- **Format Spec:** `spec/saran-format.md`
- **Type Definitions:** `crates/saran-types/src/lib.rs`

## Development Workflow

1. Read the relevant spec section
2. Implement the function/type in `src/lib.rs` with doc comments
3. Implement corresponding tests in `src/tests.rs` with spec ID tags
4. Run: `cargo test -p saran-parser`
5. Ensure all tests pass before moving to next phase

## Common Pitfalls

- **Off-by-one errors:** Position tracking should use byte indices, not char indices
- **Forgetting literals:** `ParsedTemplate` must preserve text between tokens, not just token list
- **Incomplete error info:** Error should include position and context, not just "invalid token"
- **Validation scope creep:** Token parser does NOT validate; that's a future phase

## Next Steps (After Phase 1)

Phase 2 will implement YAML parsing:

- `parse_wrapper_yaml(yaml_string)` — deserialize to `WrapperDefinition`
- Validation against the wrapper schema
- 59 unit tests covering edge cases and error conditions
