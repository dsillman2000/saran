# saran-parser

Parse YAML wrapper files and extract variable tokens from strings.

## Purpose

The `saran-parser` crate provides the parsing layer for the Saran CLI wrapper framework. It is responsible for:

1. **Token parsing** — Extract `$VAR_NAME` tokens from template strings
2. **YAML parsing** — Deserialize YAML wrapper files into `WrapperDefinition` types

## Responsibilities

### Phase 1: Token Parsing (Current)

- Extract `$VAR_NAME` variable references from strings using greedy regex matching
- Represent tokens with position information (start/end byte indices)
- Preserve literal text segments between and around tokens
- Report parsing errors with precise location information

**Public API:**
- `Token` — A parsed variable reference with name and position
- `ParsedTemplate` — Tokens and literals extracted from a string
- `parse_tokens()` — Extract all `$VAR_NAME` tokens from a string
- `TokenParsingError` — Custom error type for token parsing exceptions

### Phase 2: YAML Parsing (Future)

- Deserialize YAML wrapper files into `saran_types::WrapperDefinition`
- Validate YAML structure against the wrapper schema
- Provide clear error messages for malformed input

## Design Notes

**Token Matching Rules:**
- Pattern: `$` followed by `[A-Za-z_][A-Za-z0-9_]*`
- Greedy matching: longest valid identifier is always taken
- No escape mechanism: bare `$` followed by invalid characters produces an error
- No brace syntax: `${VAR}` is not supported in v1

**Dependencies:**
- `regex` — Compiled regex for token matching
- `thiserror` — Custom error types
- `saran-types` — Type definitions

## Testing

Tests are located in `src/tests.rs` and organized by phase. Each test is tagged with its specification ID (e.g., `// [TP-01]`) for traceability to the specification.

Run tests with:
```sh
cargo test -p saran-parser
```
