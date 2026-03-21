# Argument Assembly Unit Test Plan

## Overview

Unit tests for the argument assembly system defined in the "Argument Assembly" section of `saran-format.md`. These tests validate the construction of child process `argv` from actions, variables, positional arguments, and optional flags.

**Total tests: 19**

## Types Under Test

| Type | Purpose |
|------|---------|
| `ActionAssembly` | Represents a single action with executable and fixed args |
| `OptionalFlagValue` | Caller-supplied value for an optional flag |
| `ArgvBuilder` | Builds the final argv array for child process execution |
| `AssemblyContext` | Contains resolved vars, args, and optional flag values |

## Core Requirements to Test

### 1. Basic Argument Assembly
- Fixed action arguments are preserved exactly
- No shell interpolation or word splitting
- Each argument is a discrete element in argv

### 2. Variable Substitution in Actions
- `$VAR_NAME` references in action arrays are substituted
- Both `vars:` names and `args` `var_name` values are resolved
- Substitution happens at invocation time (after vars resolved at startup)

### 3. Optional Flag Appending
- Flags are appended after fixed arguments
- Order follows flag declaration order within each action
- Different flag types produce different argv patterns

### 4. Flag Type-Specific Behavior
- `type: str/int/enum` with `repeated: false` → `[name, value]`
- `type: str/int/enum` with `repeated: true` → `[name, value]` per occurrence
- `type: bool` → `[name]` only (no value)
- `passes_as` overrides flag name in argv

### 5. Multi-Action Execution
- Actions executed sequentially
- Non-zero exit code halts execution
- Each action gets its own argv assembly

## Test Specifications

### Basic Assembly Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| BA-01 | Fixed arguments preserved | Action: `["gh", "pr", "view"]` | argv: `["gh", "pr", "view"]` |
| BA-02 | Empty string argument | Action: `["gh", "", "pr"]` | argv: `["gh", "", "pr"]` |
| BA-03 | Whitespace in arguments | Action: `["gh", "pr with spaces"]` | argv: `["gh", "pr with spaces"]` |
| BA-04 | Special characters preserved | Action: `["gh", "pr", "--json", "{\"key\":\"value\"}"]` | argv preserves JSON exactly |

### Variable Substitution Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| VS-01 | Var substitution in action | Action: `["gh", "pr", "view", "$PR_NUM"]` with `PR_NUM="123"` | argv: `["gh", "pr", "view", "123"]` |
| VS-02 | Arg substitution in action | Action: `["gh", "pr", "view", "$PR_NUM"]` with arg `PR_NUM="456"` | argv: `["gh", "pr", "view", "456"]` |
| VS-03 | Mixed var and arg substitution | Action: `["gh", "pr", "view", "$REPO", "$PR_NUM"]` with var `REPO="org/repo"` and arg `PR_NUM="789"` | argv: `["gh", "pr", "view", "org/repo", "789"]` |
| VS-04 | Empty variable value | Action: `["gh", "pr", "view", "$EMPTY"]` with `EMPTY=""` | argv: `["gh", "pr", "view", ""]` |

### Optional Flag Appending Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| OFA-01 | Str flag appended | Action: `["gh", "pr", "view"]` with `--json` flag value `"title,body"` | argv: `["gh", "pr", "view", "--json", "title,body"]` |
| OFA-02 | Int flag appended | Action: `["tool", "run"]` with `--count` flag value `5` | argv: `["tool", "run", "--count", "5"]` |
| OFA-03 | Bool flag appended | Action: `["gh", "pr", "view"]` with `--verbose` flag (bool) | argv: `["gh", "pr", "view", "--verbose"]` |
| OFA-04 | Enum flag appended | Action: `["tool", "run"]` with `--format` flag value `"json"` | argv: `["tool", "run", "--format", "json"]` |

### Flag Type-Specific Behavior Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| FT-01 | Repeated str flag | Action: `["gh", "pr", "edit"]` with `--label` flag values `["bug", "enhancement"]` (repeated: true) | argv: `["gh", "pr", "edit", "--label", "bug", "--label", "enhancement"]` |
| FT-02 | Passes_as overrides name | Flag: `name: "--json", passes_as: "--format=json"` with value `"title"` | argv includes `["--format=json", "title"]` |
| FT-03 | Bool flag never has value | Action: `["gh", "pr", "view"]` with `--verbose` (bool) | argv: `["gh", "pr", "view", "--verbose"]` (not `["gh", "pr", "view", "--verbose", "true"]`) |
| FT-04 | Order follows declaration | Action with flags `--json` then `--verbose` declared | argv: `[..., "--json", "value", "--verbose"]` |

### Multi-Action Execution Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| MA-01 | Sequential action execution | Command with 2 actions: `["echo", "first"]` then `["echo", "second"]` | Both execute in order |
| MA-02 | Early halt on failure | Command with 3 actions, second fails | First executes, second fails, third never runs |
| MA-03 | Each action gets own argv | Two actions with same flag name but different values | Each action's argv contains its own flag values |

## Test Data Examples

### VS-01 (Var substitution in action)
```yaml
# Wrapper
vars:
  - name: PR_NUM
    required: false
    default: "123"
commands:
  view:
    actions:
      - gh: ["pr", "view", "$PR_NUM"]

# Expected argv for action
["gh", "pr", "view", "123"]
```

### OFA-01 (Str flag appended)
```yaml
# Wrapper
commands:
  view:
    actions:
      - gh:
          - "pr"
          - "view"
        optional_flags:
          - name: "--json"
            type: str

# Caller invocation: saran view --json "title,body"
# Expected argv
["gh", "pr", "view", "--json", "title,body"]
```

### FT-01 (Repeated str flag)
```yaml
# Wrapper
commands:
  edit:
    actions:
      - gh:
          - "pr"
          - "edit"
        optional_flags:
          - name: "--label"
            type: str
            repeated: true

# Caller invocation: saran edit --label bug --label enhancement
# Expected argv
["gh", "pr", "edit", "--label", "bug", "--label", "enhancement"]
```

## Out of Scope

These are handled by other test suites:

- YAML validation (schema rules for optional_flags, args, etc.)
- Variable resolution (priority chain for vars)
- `$VAR_NAME` token parsing (syntax, greedy matching)
- Process execution and I/O (stdout/stderr forwarding)
- CLI integration (clap parsing of caller invocation)

## Dependencies

- String manipulation for substitution
- Collections for building argv arrays
- Type conversions for flag values (int → string)