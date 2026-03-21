# Substitution Resolution Unit Test Plan

## Overview

Resolving `$VAR_NAME` tokens to values. This depends on:
- **Token parsing** (to identify what to substitute)
- **Variable resolution** (to get values for `vars:` names)
- **Caller arguments** (to get values for `args` names)

**Total tests: 10**

## Types Under Test

| Type | Purpose |
|------|---------|
| `SubstitutionResolver` | Resolves parsed tokens against available values |
| `ResolutionContext` | Contains resolved vars and caller args |
| `SubstitutionResult` | Result of substitution with resolved string |

## Core Requirements to Test

### 1. Resolution Rules
- References must match declared `vars:` names or `args` `var_name` values
- Resolution timing: `vars:` resolved at startup, `args` at invocation

### 2. Context-Specific Rules
- **Action arrays**: Can reference both `vars:` and `args` names
- **Help strings**: Can only reference top-level `vars:` names
- **Help timing**: Resolved at startup (before `args` are known)

## Test Specifications

### Value Resolution Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| VR-01 | Valid variable reference resolves | Reference `$GH_REPO` where `GH_REPO` declared in `vars:` | Resolves to variable's value |
| VR-02 | Valid argument reference resolves | Reference `$PR_NUM` where `PR_NUM` declared in command `args:` | Resolves to argument's value (at invocation time) |

### Context-Specific Behavior Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| CS-01 | Help strings accept variable references | Help text `"Operations for $GH_REPO"` with `GH_REPO` in `vars:` | Resolves to variable value at startup |
| CS-02 | Help resolution tolerates missing values | Help text `"Repo: $GH_REPO"` where `GH_REPO` unresolved at startup | Literal `$GH_REPO` shown (no error, per spec) |
| CS-03 | Help with multiple variable references | Text `"Repo: $REPO, PR: $PR"` with one resolved, one not | Mixed resolution: resolved var substituted, unresolved shown literally |

### Edge Cases & Value Handling Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| EC-01 | Empty variable value substitutes | Reference `$EMPTY_VAR` where variable value is empty string `""` | Substitutes empty string (not omitted) |
| EC-02 | Whitespace in values preserved | Reference `$VAR` where value contains spaces, tabs, newlines | Value preserved exactly, no trimming or normalization |
| EC-03 | Dollar sign in value not re-parsed | Reference `$VAR` where value is `"text$more"` | Substitutes literal `$`, no recursive parsing |
| EC-04 | No recursive substitution | String `"$FOO"` where `FOO="BAR"` and `BAR="value"` | Substitutes `"BAR"` only, does not look up `BAR` |
| EC-05 | Large variable values | Reference `$VAR` where value is 64KB string | Substitutes successfully (no artificial size limits) |

## Test Data Examples

### VR-01 (Valid variable reference resolves)
```rust
// Mock inputs
let tokens = parse_tokens("$GH_REPO"); // From token parsing
let resolved_vars = HashMap::from([("GH_REPO", "org/repo")]); // From variable resolution
let caller_args = HashMap::new(); // No args in this test

// Resolution
let result = resolve_substitution(tokens, &resolved_vars, &caller_args);

// Expected
assert_eq!(result, "org/repo");
```

### CS-02 (Help resolution tolerates missing values)
```rust
// vars: GH_REPO has no value at startup (required but not set)
// help: "Operations for $GH_REPO"
let tokens = parse_tokens("Operations for $GH_REPO");
let resolved_vars = HashMap::new(); // No resolved vars at startup
let caller_args = HashMap::new(); // Help doesn't use args

let resolved = resolve_help_text(tokens, &resolved_vars);

// Expected (per spec: literal $VAR shown, no error)
assert_eq!(resolved, "Operations for $GH_REPO");
```

### EC-04 (No recursive substitution)
```rust
// Mock: FOO="BAR", but BAR is also a variable with value "value"
let tokens = parse_tokens("$FOO");
let resolved_vars = HashMap::from([("FOO", "BAR")]);
// Note: BAR is NOT in resolved_vars - we don't do recursive lookup
let caller_args = HashMap::new();

let result = resolve_substitution(tokens, &resolved_vars, &caller_args);

// Expected: Substitutes "BAR", doesn't look up BAR's value
assert_eq!(result, "BAR");
```

## Mock Requirements

These tests require mocking:

1. **Token parsing output** (from `token-parsing.md` tests)
2. **Resolved variables** (from `variable-resolution.md` tests)  
3. **Caller arguments** (from clap parsing, not yet tested)

## Implementation Considerations

### Error Handling
- Undeclared references are validation errors (handled in YAML validation)
- Missing values at runtime handled per context (help vs action)

### Performance
- Cache resolution results for repeated templates
- Lazy resolution for help text with unresolved variables

## Out of Scope

- Token parsing (handled in `token-parsing.md`)
- Variable resolution (handled in `variable-resolution.md`)
- YAML validation (handled in `yaml-validation.md`)
- Argument assembly (handled in `argument-assembly.md`)

## Dependencies

- Token parsing types and functions
- Variable resolution types and values
- String manipulation for substitution
- Hash maps for value lookup