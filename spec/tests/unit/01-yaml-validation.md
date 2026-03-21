# YAML Validation Unit Test Plan

## Overview

Unit tests for YAML schema validation defined in `saran-format.md`. These tests ensure that malformed wrapper files are rejected with descriptive errors before installation or execution.

**Total tests: 59**

## Types Under Test

| Type | Purpose |
|------|---------|
| `WrapperSchema` | Complete wrapper YAML structure validation |
| `ValidationError` | Structured error reporting for schema violations |

## Core Requirements to Test

### 1. Top-Level Structure Validation
- Required fields: `name`, `version`, `commands`
- Field types and format constraints
- Optional sections: `help`, `requires`, `vars`

### 2. Variable Declaration Validation (`vars:`)
- Name format, uniqueness, prefix conflicts
- Required/default mutual exclusivity
- No ambiguous optionality

### 3. Command Structure Validation (`commands:`)
- Command name format
- Required `actions` section
- Action structure and executable validation

### 4. Optional Flags Validation (`optional_flags:`)
- Flag name format and uniqueness
- Type constraints and enum values
- `passes_as` validation

### 5. Argument Validation (`args:`)
- Name and `var_name` format
- Type constraints, ordering rules
- Namespace conflicts with `vars:`

### 6. Variable Reference Validation
- `$VAR_NAME` syntax in action arrays and help text
- Reference resolution against declared names
- Context-specific rules (help vs action arrays)

### 7. `requires:` Section Validation
- CLI version constraint parsing
- Probe command and pattern validation
- Unique CLI names

## Test Specifications

### Top-Level Structure Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| TL-01 | Missing required `name` field | YAML without `name:` field | Validation error: `name` is missing |
| TL-02 | Empty `name` field | YAML with `name: ""` | Validation error: `name` is empty |
| TL-03 | Missing required `version` field | YAML without `version:` field | Validation error: `version` is missing |
| TL-04 | Invalid SemVer `version` | `version: "not-semver"` | Validation error: invalid SemVer 2.0.0 |
| TL-05 | Valid SemVer with prerelease | `version: "1.0.0-beta.1"` | Validation success |
| TL-06 | Missing required `commands` | YAML without `commands:` section | Validation error: `commands` is missing |
| TL-07 | Empty `commands` section | `commands: {}` (empty map) | Validation error: `commands` is empty |
| TL-08 | Valid top-level structure | All required fields with valid values | Validation success |

### Variable Declaration Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| VD-01 | Variable name invalid format | `name: "123VAR"` (starts with digit) | Validation error: invalid name format |
| VD-02 | Variable name with hyphen | `name: "VAR-NAME"` (contains hyphen) | Validation error: invalid name format |
| VD-03 | Duplicate variable names | Two `vars:` entries with same `name` | Validation error: duplicate variable name |
| VD-04 | Required and default both set | `required: true` and `default: "value"` | Validation error: mutually exclusive |
| VD-05 | Neither required nor default | No `required:` and no `default:` | Validation error: ambiguous optionality |
| VD-06 | Prefix name conflict | `VAR` and `VAR_SUFFIX` both declared | Validation error: variable name is prefix of another |
| VD-07 | Valid variable declarations | Properly formatted variables | Validation success |

### Command & Action Structure Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| CA-01 | Invalid command name format | Command name `"bad name"` (contains space) | Validation error: invalid command name |
| CA-02 | Missing `actions` in command | Command without `actions:` key | Validation error: command missing `actions` |
| CA-03 | Empty `actions` list | `actions: []` (empty array) | Validation error: `actions` is empty |
| CA-04 | Action with no executable key | `actions: [{}]` (empty map) | Validation error: action has no executable key |
| CA-05 | Action with extra keys | Action with `executable:`, `optional_flags:`, `extra:` | Validation error: only executable and optional_flags permitted |
| CA-06 | Invalid executable name | `actions: [{"/bin/ls": [...]}]` (absolute path) | Validation error: invalid executable name |
| CA-07 | Valid command structure | Properly formatted command with actions | Validation success |

### Optional Flag Validation Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| OF-01 | Missing flag `name` | `optional_flags: [{type: "str"}]` | Validation error: missing `name` |
| OF-02 | Missing flag `type` | `optional_flags: [{name: "--json"}]` | Validation error: missing `type` |
| OF-03 | Invalid flag type | `type: "float"` (not str/bool/int/enum) | Validation error: invalid type |
| OF-04 | Flag name doesn't start with `--` | `name: "json"` (single dash) | Validation error: must begin with `--` |
| OF-05 | Invalid characters in flag name | `name: "--bad_name"` (contains underscore) | Validation error: invalid characters |
| OF-06 | Enum type without `values` | `type: "enum"` (no values list) | Validation error: missing `values` |
| OF-07 | Empty `values` list for enum | `values: []` | Validation error: `values` is empty |
| OF-08 | Invalid enum value format | `values: ["bad value"]` (contains space) | Validation error: invalid enum value |
| OF-09 | Bool flag with `repeated: true` | `type: "bool", repeated: true` | Validation error: mutually exclusive |
| OF-10 | Duplicate flag names | Two flags with same `name` in same command | Validation error: duplicate flag name |
| OF-11 | `passes_as` contains `=` | `passes_as: "--repo=value"` | Validation error: cannot contain `=` |
| OF-12 | Valid optional flags | Properly formatted flags | Validation success |

### Argument Validation Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| AR-01 | Missing `name` in arg | `args: [{var_name: "FOO", type: "str"}]` | Validation error: missing `name` |
| AR-02 | Missing `var_name` in arg | `args: [{name: "foo", type: "str"}]` | Validation error: missing `var_name` |
| AR-03 | Missing `type` in arg | `args: [{name: "foo", var_name: "FOO"}]` | Validation error: missing `type` |
| AR-04 | Invalid arg type | `type: "int"` (not str) | Validation error: must be `str` |
| AR-05 | Invalid arg name format | `name: "bad name"` (contains space) | Validation error: invalid name format |
| AR-06 | Duplicate arg names | Two args with same `name` or `var_name` | Validation error: duplicate argument |
| AR-07 | Arg `var_name` conflicts with var | `var_name: "GH_REPO"` where `GH_REPO` in `vars:` | Validation error: namespace conflict |
| AR-08 | Required after optional | Optional arg before required arg in list | Validation error: required cannot follow optional |
| AR-09 | Prefix `var_name` conflict | `FOO` and `FOO_BAR` as `var_name` values | Validation error: `var_name` is prefix of another |
| AR-10 | Valid argument declarations | Properly formatted arguments | Validation success |

### Variable Reference Validation Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| VR-01 | Undeclared variable in action | `$UNDECLARED` in action array | Validation error: undeclared variable |
| VR-02 | Invalid `$` syntax in action | `"prefix$"` (bare `$`) in action | Validation error: invalid `$VAR_NAME` syntax |
| VR-03 | Digit after `$` in action | `"$1VAR"` in action | Validation error: invalid `$VAR_NAME` syntax |
| VR-04 | Arg reference in help text | `$PR_NUM` in help where `PR_NUM` only in `args:` | Validation error: args not valid in help |
| VR-05 | Undeclared variable in help | `$UNDECLARED` in help text | Validation error: undeclared variable in help |
| VR-06 | Valid variable references | Proper `$VAR_NAME` references | Validation success |

### `requires:` Section Validation Tests

| ID | Test Purpose | Test Case Description | Expected Result |
|----|-------------|----------------------|-----------------|
| RE-01 | Missing `cli` in requires | `requires: [{version: ">=1.0.0"}]` | Validation error: missing `cli` |
| RE-02 | Missing `version` in requires | `requires: [{cli: "gh"}]` | Validation error: missing `version` |
| RE-03 | Invalid semver constraint | `version: "not-semver"` | Validation error: invalid semver constraint |
| RE-04 | `version_probe` not array | `version_probe: "gh --version"` (string) | Validation error: must be array |
| RE-05 | Empty `version_probe` array | `version_probe: []` | Validation error: must be non-empty |
| RE-06 | Invalid regex in `version_pattern` | `version_pattern: "["` (invalid regex) | Validation error: invalid regex |
| RE-07 | `version_pattern` without capture | `version_pattern: "version \\S+"` (no capture) | Validation error: must have exactly one capture group |
| RE-08 | Duplicate CLI in requires | Two entries with same `cli` value | Validation error: duplicate CLI |
| RE-09 | Valid requires section | Properly formatted requires | Validation success |



## Implementation Considerations

### Error Reporting
- Each validation error should include:
  - Path to invalid element (e.g., `commands.list.actions[0].optional_flags[0].name`)
  - Specific rule violated
  - Suggested fix if possible

### Validation Order
Validate in logical order:
1. Basic structure (top-level fields)
2. Variable declarations
3. Command structure
4. Optional flags and arguments
5. Variable references
6. Cross-references and dependencies

### Performance
- Early exit on first error vs. collect all errors
- Consider lazy validation for large wrapper files
- Cache validation results for installed wrappers

## Dependencies
- YAML parsing library
- SemVer parsing for version validation
- Regex compilation for `version_pattern` validation
