# Token Parsing Unit Test Plan

## Overview

Pure string parsing to find `$VAR_NAME` tokens in strings. This is a **pure function** with no dependencies - it only examines input strings and identifies variable references.

**Total tests: 6**

## Types Under Test

| Type | Purpose |
|------|---------|
| `Token` | Represents a parsed variable reference with name and position |
| `ParsedTemplate` | Contains tokens and literal segments from a string |
| `TokenParser` | Pure function that extracts tokens from strings |

## Core Requirements to Test

### 1. Token Parsing Rules
- `$` followed by greedy `[A-Za-z_][A-Za-z0-9_]*` match
- Substitution ends at first non-matching character or end of string
- No `${VAR}` brace syntax support in v1
- No escape mechanism for literal `$`

## Test Specifications

### Token Parsing Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| TP-01 | Basic variable reference | String `"$GH_REPO"` contains a single variable | Parses token `GH_REPO` spanning positions 0-8 |
| TP-02 | Greedy matching stops at non-identifier | String `"$GH_REPO/"` has literal suffix | Token `GH_REPO`, trailing `/` remains literal text |
| TP-03 | Multiple adjacent references | String `"$FOO$BAR"` with no separator | Two tokens: `FOO` (0-4), `BAR` (4-8) |
| TP-04 | Case-sensitive parsing | Strings `"$Var"` and `"$VAR"` | Distinct tokens `Var` and `VAR` |
| TP-05 | Mixed literals and variables | String `"prefix-$VAR-suffix"` | Token `VAR` with literals `prefix-` and `-suffix` preserved |
| TP-06 | Greedy matching takes maximal valid identifier | String `"$VARsuffix"` where `suffix` are valid identifier chars | Token `VARsuffix` (entire valid sequence, not just `VAR`) |

## Test Data Examples

### TP-02 (Greedy matching stops at non-identifier)
```rust
// Input: "$GH_REPO/"
let result = parse_tokens("$GH_REPO/");

// Expected: `/` is not valid identifier char, so greedy stops before it
assert_eq!(result.tokens, vec![
    Token { var_name: "GH_REPO", start: 0, end: 8 }
]);
assert_eq!(result.literals, vec![
    Literal { text: "", before: true },
    Literal { text: "/", after: true }
]);
```

### TP-06 (Greedy matching takes maximal identifier)
```rust
// Input: "$VARsuffix" (all chars are valid identifier chars)
let result = parse_tokens("$VARsuffix");

// Expected: Entire "VARsuffix" is valid identifier, so greedy takes it all
// Note: This would be a validation error if both VAR and VARsuffix were declared
assert_eq!(result.tokens, vec![
    Token { var_name: "VARsuffix", start: 0, end: 9 }
]);
// No literal suffix
```

## Implementation Considerations

### Error Messages (Syntax Errors)
Should include:
- Position in string where syntax error occurred
- Specific reason (bare `$`, digit after `$`, etc.)

### Performance
- Pre-compile regex: `\$([A-Za-z_][A-Za-z0-9_]*)`
- Cache parsed templates for repeated strings
- Zero-copy parsing where possible (return string slices)

## Out of Scope

- Value resolution (looking up variable values)
- Validation (checking if variables are declared)
- Context-specific rules (help vs action arrays)
- Substitution (replacing tokens with values)

## Dependencies

- Regex compilation for token extraction
- String slicing for position tracking
- Collections for returning token lists