# Saran Wrapper Format Specification

## Overview

A **Saran file** is a YAML document that defines a restricted CLI wrapper around an existing CLI tool. Saran reads this file and uses `clap` to dynamically generate a new CLI binary that:

- Exposes only the subcommands declared in the file (allowlist model)
- Resolves a declared set of environment variables from a layered configuration chain before each invocation
- Executes the underlying CLI via non-shell exec (no shell interpolation, no glob expansion, no metacharacter interpretation)
- Appends declared optional flags safely when provided by the caller

The primary use case is exposing a minimal, safe subset of a CLI for consumption by an LLM agent, preventing destructive or out-of-scope operations.

Throughout this document, **caller** refers to the entity invoking the generated saran CLI — typically an LLM agent, but may also be a human operator or automation script.

---

## File Naming

Saran wrapper files are standard YAML documents. The recommended extension is `.yaml` (e.g. `gh-pr-review.yaml`). There is no enforced naming convention, but a descriptive name reflecting the wrapped tool and scope is encouraged.

---

## Execution Model

Every action in a Saran wrapper's `actions:` list is executed as a **direct process invocation** (equivalent to `execvp`), not via a shell. This means:

- Each element of an action's argument array is passed as a discrete argument to the target process — no word splitting, no glob expansion, no shell metacharacter interpretation
- The process environment is constructed by Saran: it inherits the ambient environment and then force-sets all variables resolved from the `vars:` declaration chain, overwriting any pre-existing values
- Each action may declare its own `optional_flags:`. When those flags are supplied by the caller, they are appended to that action's argument array at execution time
- Actions in a command's `actions:` list are executed sequentially. If any action exits with a non-zero code, execution halts immediately and that exit code is returned. If all succeed, Saran exits with code `0`.

---

## Top-Level Structure

```yaml
name: <string>             # Required. The name of the generated CLI binary.
version: <string>          # Required. Semantic version of this wrapper (e.g. "1.0.0").
help: <string>             # Optional. Top-level help/about text shown by --help.
requires:                  # Optional. Version constraints on external CLIs.
  - ...
vars:                      # Optional. Declares environment variables this wrapper depends on.
  - ...
commands:                  # Required. Named subcommands exposed by the wrapper.
  <command-name>:
    ...
```

### `name`
**Required.** The name of the wrapper CLI. Used as the program name in `clap`'s help output.

### `version`
**Required.** The semantic version of this wrapper in `MAJOR.MINOR.PATCH` format (e.g. `"1.0.0"`). Must conform to [SemVer 2.0.0](https://semver.org/). Exposed via `--version` in the generated CLI and displayed by `saran list`.

> **Note:** This version tracks the wrapper definition itself, not the underlying CLI being wrapped. Increment it when the wrapper's commands, flags, or vars change in a meaningful way.

### `help`
**Optional.** A short description of the wrapper shown in top-level `--help` output.

### `requires`
**Optional.** A list of version constraints on external CLIs that must be satisfied for the wrapper to function correctly. Each entry declares one CLI dependency with its expected version range. Saran checks these constraints at validate time (hard error) and at list time (soft warning).

```yaml
requires:
  - cli: gh
    version: ">=2.0.0"
  - cli: git
    version: ">=2.40.0 <3.0.0"
    version_probe: [git, --version]
    version_pattern: "git version (\\S+)"
```

#### `requires` entry fields

| Field | Required | Description |
|---|---|---|
| `cli` | ✅ | The executable name to check. Used as a human-readable label in diagnostics and as the default probe target (`[<cli>, --version]`). Must satisfy `[a-z0-9_-]+`. |
| `version` | ✅ | A semver constraint string. Supports `>=`, `>`, `<=`, `<`, `=` operators. Multiple constraints may be space-separated to form a range (e.g. `">=1.0.0 <3.0.0"`). Must match at least one resolved version of the CLI on the user's system. |
| `version_probe` | — | Override the command used to query the CLI's version. An array of strings executed via non-shell exec. Defaults to `[<cli>, --version]`. Both stdout and stderr are captured and searched for a version string. |
| `version_pattern` | — | A regex with exactly one capture group used to extract the version string from the probe output. Defaults to matching the first occurrence of `\d+\.\d+\.\d+[\w.-]*` anywhere in stdout or stderr. |

#### Version constraint syntax

The `version` field accepts standard semver comparison operators:

| Constraint | Meaning |
|---|---|
| `">=2.0.0"` | Version 2.0.0 or higher |
| `">2.0.0"` | Strictly greater than 2.0.0 |
| `"<=2.0.0"` | Version 2.0.0 or lower |
| `"<2.0.0"` | Strictly less than 2.0.0 |
| `"=2.0.0"` | Exactly version 2.0.0 |
| `">=1.0.0 <3.0.0"` | At least 1.0.0 and below 3.0.0 (space-separated AND) |

#### Unknown version handling

If the probe command fails (executable not in PATH, non-zero exit, or no version string matched), the version is treated as `unknown`. Enforcement behavior:

- `saran validate` — hard error: prints the probe failure reason and exits non-zero
- `saran list` — soft warning: shows `⚠ unknown (requires <constraint>)` next to the wrapper



### `vars`
**Optional.** A list of environment variable declarations this wrapper depends on. Each entry declares one variable, its optionality, and an optional default. Variables declared here are resolved at startup from a layered configuration chain and injected into the child process environment before every invocation. The caller cannot influence these values.

```yaml
vars:
  - name: GH_TOKEN
    required: true
    help: "GitHub auth token — must be set in the host environment"
  - name: GH_REPO
    default: myorg/myrepo
    help: "Target repository in OWNER/REPO format"
```

#### `vars` entry fields

| Field | Required | Description |
|---|---|---|
| `name` | ✅ | The environment variable name. Must satisfy `[A-Za-z_][A-Za-z0-9_]*`. |
| `required` | ✅ | `true` if the variable must be provided by the resolution chain; `false` if the `default:` is sufficient. Mutually exclusive with `default:`. |
| `default` | — | A literal string value used as a fallback if no higher-priority source provides the variable. Mutually exclusive with `required: true`. |
| `help` | — | A short description shown in `saran env` output. |

> **Note:** `required: true` and `default:` are mutually exclusive. A var with `required: true` must be set somewhere in the resolution chain — if it is not, Saran exits at startup with a descriptive error. A var with a `default:` is always satisfiable and never causes a startup error.

#### Variable resolution chain

When Saran starts, each declared variable is resolved in the following order. The first source that provides a value wins:

1. **Per-wrapper namespace** in `~/.local/share/saran/env.yaml` (set via `saran env <wrapper-name> VAR=value`)
2. **Global namespace** in `~/.local/share/saran/env.yaml` (set via `saran env VAR=value`)
3. **Host environment** (the ambient environment of the process that launched Saran)
4. **`default:`** declared in the `vars:` entry

If all four sources yield no value and the variable is `required: true`, Saran exits immediately with an error:

```
error: required variable `GH_TOKEN` is not set.
       Set it in your host environment or via: [`saran env gh-pr-review GH_TOKEN=<value>`](saran-env.md)
       Note: storing secrets in saran env is not recommended — use your host environment instead.
```

> See [saran-env.md](saran-env.md) for the full `saran env` command reference.

> **Security note:** Because Saran uses direct process invocation and callers can only supply values through `optional_flags`, resolved `vars:` values are guaranteed at invocation time — the caller cannot override them. The host environment contributes to resolution *before* Saran launches, not during invocation.
>
> **Warning:** `~/.local/share/saran/env.yaml` is stored as plaintext. Do not use `saran env` to store secrets such as API tokens or credentials. For secrets, declare the variable as `required: true` (no `default:`) and set it in your host environment via your shell profile, a credential manager, or a secrets tool. This keeps secrets out of saran's configuration files entirely.

### `commands`
**Required.** A map of subcommand names to their definitions. Each key becomes a subcommand in the generated `clap` CLI. See [Command Definition](#command-definition) below.

---

## Command Definition

```yaml
commands:
  <command-name>:
    help: <string>             # Optional. Per-command help text.
    args:                      # Optional. Positional arguments the caller must/may supply.
      - ...
    actions:                   # Required. Ordered list of process invocations to execute.
      - <executable>: [...]    # Required. Executable key with its fixed argument array.
        optional_flags:        # Optional. Flags the caller may pass to this specific action.
          - ...
```

### `help`
**Optional.** A short description of this subcommand, shown in `--help` output for the wrapper.

### `args`
**Optional.** A list of positional arguments the caller may supply when invoking this subcommand. Positional args are declared in the order `clap` will expect them on the command line. Their values are injected into `actions` entries via `$VAR_NAME` substitution.

See [Positional Argument Definition](#positional-argument-definition) below.

### `actions`
**Required.** An ordered list of process invocations to execute when this subcommand is called. Each entry is a YAML map with one required key (the executable name, whose value is the fixed argument array) and one optional key (`optional_flags:`). Actions are executed sequentially; if any action exits non-zero, execution halts immediately.

```yaml
# Single action, no caller flags
actions:
  - gh: [pr, status, -R, "$GH_REPO"]

# Single action with caller flags
actions:
  - gh: [pr, view, "$PR_REF", -R, "$GH_REPO"]
    optional_flags:
      - name: --comments
        type: bool
        help: "Include pull request comments in the output"
      - name: --json
        type: str
        help: "Comma-separated fields for JSON output"

# Multiple actions, flags localized to the action that uses them
actions:
  - gh: [pr, view, "$PR_REF", -R, "$GH_REPO"]
    optional_flags:
      - name: --json
        type: str
        help: "Comma-separated fields for JSON output"
  - printf: ["---\nDone.\n"]
```

The executable key in each entry must satisfy the same validity rules as any executable name: no `/`, no `..`, no absolute paths. It is resolved via the process's `PATH`.

`clap` aggregates all `optional_flags` from all action entries across the command to build a single unified argument parser for the caller. Each flag is then routed to the specific action that declared it at execution time. Flag names must be unique across all action entries in a command — two actions in the same command may not declare a flag with the same `name`.

Values from `vars:` and caller-supplied positional arguments (declared in `args:`) may be referenced in any action's argument array using `$VAR_NAME` syntax. Saran uses a **unified substitution namespace**: both `vars` names and `args` `var_name` values share the same `$VAR_NAME` reference syntax. Substitution is performed at invocation time, after all `vars:` have been resolved at startup and the caller's positional values have been parsed.

**Substitution parsing rule:** A `$VAR_NAME` token is parsed by matching `$` followed by a greedy sequence of characters satisfying `[A-Za-z_][A-Za-z0-9_]*`. Substitution ends at the first character outside that set, or at end of string. For example, `"$GH_REPO"` substitutes the full string with the value of `GH_REPO`; `"$GH_REPO/"` substitutes only `$GH_REPO` and leaves the trailing `/` literal. A Rust implementor should use the regex `\$([A-Za-z_][A-Za-z0-9_]*)` to locate all references within each element.

> **Note:** `$VAR_NAME` references in action argument arrays are only valid for names declared in `vars:` or as a `var_name` in the command's `args:` block. References to undeclared names are a validation error.

#### `$VAR_NAME` substitution rules

- A variable reference begins with `$` and continues with the longest sequence of ASCII letters, digits, and underscores (`[A-Za-z0-9_]+`). The first character after `$` must be a letter or underscore (not a digit).
- Substitution is performed left-to-right within each string element. Adjacent references like `$FOO$BAR` are resolved as two separate substitutions: `$FOO` terminates at the second `$`.
- A literal `$` that does not start a recognized `$VAR_NAME` pattern (e.g. a trailing `$` or `$1`) is a validation error — there is no escape syntax. Values that need a literal dollar sign must be placed in `vars:` with a literal default and referenced by name.
- `${VAR_NAME}` brace syntax is **not** supported in v1; use plain `$VAR_NAME` only.
- `vars:` names and `args` `var_name` values share one namespace — they must be unique within a command. A collision is a validation error.

> **Resolution timing:** `$VAR_NAME` in action argument arrays resolves against already-resolved `vars:` values (computed at startup) and caller-supplied `args` values (available at invocation). There is no re-resolution against the host environment at invocation time.

#### `$VAR_NAME` in `help:` strings

`$VAR_NAME` interpolation is supported in **any `help:` field** in the YAML document — top-level, command-level, `args` entries, `optional_flags` entries, and `vars` entries. This allows help text to reflect environment-specific values (e.g. the actual repo name) rather than hard-coded placeholders.

```yaml
vars:
  - name: GH_REPO
    default: myorg/myrepo
    help: "Target repository in OWNER/REPO format"

help: "Read-only gh pr operations for $GH_REPO"

commands:
  list:
    help: "List pull requests in $GH_REPO"
    ...
```

**Scope restriction:** `$VAR_NAME` references in `help:` strings are limited to **top-level `vars:` names only**. `args` `var_name` values are not valid in help strings — they are command-scoped positional values that are not yet known when help text is rendered at startup.

**Resolution timing:** `help:` string interpolation resolves at startup, immediately after `vars:` resolution. If a required var has not been set in any resolution layer at help-display time, the literal `$VAR_NAME` token is shown in its place (no error is raised — the missing-var runtime error is reported separately).

**Syntax rules:** The same parsing rule applies as for action arrays — `$VAR_NAME` is a greedy match of `[A-Za-z_][A-Za-z0-9_]*` after the `$`. A bare `$` or `$` followed by a digit in a `help:` string is a validation error.

---

## Optional Flag Definition

`optional_flags:` is declared as a sibling key to the executable key within an `actions:` entry. Each flag defined here is exposed to the caller via `clap` and, when supplied, is appended to that specific action's argument array.

```yaml
optional_flags:
  - name: <string>         # Required. The flag name as exposed in the saran CLI (e.g. --json).
    type: <type>           # Required. One of: str, bool, int, enum.
    repeated: <bool>       # Optional. Allow the flag to be supplied multiple times. Defaults to false.
    help: <string>         # Optional. Description shown in --help for this flag.
    passes_as: <string>    # Optional. The flag name passed to the underlying CLI. Defaults to `name`.
```

### `name`
**Required.** The flag name as it appears in the generated `clap` CLI. Must begin with `--`. The remainder must consist only of lowercase ASCII letters (`a–z`), digits (`0–9`), and hyphens (`-`), with no consecutive hyphens and no trailing hyphen (e.g., `--json`, `--failed-only`). Underscores and spaces are not valid. This matches `clap`'s long-flag name constraints.

### `type`
**Required.** The value type for this flag. Saran supports four types:

| Type | `clap` mapping | Child argv behavior |
|------|----------------|---------------------|
| `str` | `ArgAction::Set` | Appends two elements: the flag name (or `passes_as`) and the caller-supplied string value. |
| `bool` | `ArgAction::SetTrue` | A presence flag — no value is accepted. Appends one element: the flag name (or `passes_as`) only. |
| `int` | `ArgAction::Set` + integer parser | Accepts an integer value; `clap` rejects non-integer input before invocation. Appends two elements: the flag name (or `passes_as`) and the integer rendered as a decimal string. |
| `enum` | `ArgAction::Set` + `PossibleValues` | Accepts only one of the values listed in the required sibling field `values:`. `clap` rejects anything outside that list before invocation. Appends two elements: the flag name (or `passes_as`) and the caller-supplied value. |

When `type: enum` is used, a `values:` field is **required** on the same flag entry:

```yaml
- name: --state
  type: enum
  values: [open, closed, merged, all]
  help: "Filter by PR state (default: open)"

- name: --limit
  type: int
  help: "Maximum number of results to return (default: 30)"

- name: --color
  type: enum
  values: [always, never, auto]
  help: "Whether to use color in output (default: auto)"
```

`values:` must be a non-empty list of non-empty strings. Each value must satisfy `[a-z0-9][a-z0-9_-]*` (lowercase, no spaces). `values:` is a validation error on any type other than `enum`.

### `repeated`
**Optional.** Defaults to `false`. When `true`, the flag may be supplied multiple times by the caller. Each occurrence is collected independently and appended to the action's argument array in the order supplied.

- Maps to `ArgAction::Append` in `clap` instead of `ArgAction::Set`.
- Assembly: for each value the caller supplied, append `[passes_as ?? name, value]` — so `--label bug --label enhancement` produces `[--label, bug, --label, enhancement]` in the child argv.
- Composes with `type: enum`: each individual occurrence is independently validated against `values:`. This is the recommended pattern for flags that accept multiple constrained values.
- `repeated: true` combined with `type: bool` is a **validation error** — bool flags carry no value and repetition is meaningless.

```yaml
# repeated str — any string accepted each time
- name: --label
  type: str
  repeated: true
  help: "Filter by label (may be specified multiple times)"

# repeated enum — each occurrence must be one of the listed values
- name: --label
  type: enum
  values: [bug, enhancement, question, documentation]
  repeated: true
  help: "Filter by label (may be specified multiple times)"
```

### `help`
**Optional.** A short description of the flag, shown in `clap`'s `--help` output for the subcommand.

### `passes_as`
**Optional.** When present, this value is used as the flag name in the underlying CLI invocation instead of `name`. This allows the saran-facing interface to use a different (e.g., more descriptive) name than the underlying CLI's flag.

- Must begin with `--`.
- Must **not** contain `=`. An `=` in `passes_as` would cause a `type: bool` flag to inject a key=value pair (e.g. `--repo=evil/repo`) as a single argument, potentially overriding a fixed argument in the action's array in CLIs that accept `--flag=value` syntax. Saran rejects this at validation time.

```yaml
- name: --fields
  passes_as: --json
  type: str
  help: "Comma-separated list of fields to include in JSON output"
```

When the caller passes `--fields title,body`, Saran appends `--json` and `title,body` to that action's argument array.

---

## Positional Argument Definition

```yaml
args:
  - name: <string>        # Required. The positional name shown in clap help/usage (e.g. "name").
    var_name: <string>    # Required. The substitution variable referenced in action via $VAR_NAME.
    type: str             # Required. Must be `str` in v1.
    required: <bool>      # Optional. Whether the caller must supply this argument. Defaults to true.
    help: <string>        # Optional. Description shown in --help for this subcommand.
```

### `name`
**Required.** The display name for this positional argument in `clap`'s usage and help output (e.g. `<name>`). Must consist only of lowercase ASCII letters (`a–z`), digits (`0–9`), and hyphens (`-`).

### `var_name`
**Required.** The substitution variable name. Must satisfy `[A-Za-z_][A-Za-z0-9_]*`. Referenced as `$VAR_NAME` in the command's `actions` entries. Must be unique across all `vars:` names and other `args` `var_name` values within the same command.

### `type`
**Required.** Must be `str` in v1. Positional arguments always receive a single string value from the caller.

### `required`
**Optional.** Defaults to `true`. When `true`, `clap` will error if the caller omits this argument. When `false`, the positional is optional; if omitted, its `$VAR_NAME` reference in `actions` entries resolves to an empty string.

### `help`
**Optional.** A short description shown in `clap`'s `--help` output.

### Ordering
Positionals are registered with `clap` in the order they appear in the `args:` list. Required positionals must not appear after optional ones — this is a validation error (it would make the optional positional unreachable).

```yaml
# Valid: required before optional
args:
  - name: repo
    var_name: REPO
    type: str
    required: true
  - name: ref
    var_name: REF
    type: str
    required: false

# Invalid: optional before required
args:
  - name: ref
    var_name: REF
    type: str
    required: false
  - name: repo       # ← validation error: required after optional
    var_name: REPO
    type: str
    required: true
```

---

## Argument Assembly

> **Clarification on two distinct models:** The assembly described below refers to constructing the **child process argv** — the argument vector passed to each action's underlying CLI via non-shell exec. This is separate from the **`clap` parsing model** that Saran builds for its own CLI. Saran uses `clap` to parse the *caller's* invocation of the wrapper (subcommand selection and optional flag values); the result of that parse is then used to build each child process argv via the rules below.

When a subcommand is invoked, Saran assembles and executes the actions as follows:

1. Parse the caller's invocation with `clap`: collect the selected subcommand, any positional arg values (from `args:`), and any optional flag values (aggregated from all `optional_flags:` entries across all actions in the command)
2. For each action entry in `actions:`, in declaration order:
   a. Build the argument array: perform `$VAR_NAME` substitution on each element, resolving against both resolved `vars:` values (computed at startup) and `args` `var_name` values (supplied by caller at runtime)
   b. Append any caller-supplied optional flags that belong to this action:
      - If `type: str`, `int`, or `enum` and `repeated: false` (default): append `[passes_as ?? name, value]` (for `int`, value is rendered as a decimal string)
      - If `type: str`, `int`, or `enum` and `repeated: true`: for each supplied value in order, append `[passes_as ?? name, value]`
      - If `type: bool`: append `[passes_as ?? name]`
   c. Execute the resulting array via non-shell exec with the forced environment
   d. If the action exits non-zero, halt immediately and return that exit code to the caller
3. If all actions succeed, exit with code `0`

> **Notation:** `passes_as ?? name` is null-coalesce shorthand — use `passes_as` if it is declared on the flag, otherwise fall back to `name`. In practice: if the flag definition includes `passes_as`, the underlying CLI sees that string; if not, it sees `name` unchanged.

Optional flags are always appended **after** each action's fixed args. Ordering among optional flags follows their declaration order within each action's `optional_flags:` list.

---

## Process I/O and Exit Code

Saran must pass through each child process's I/O and exit status transparently:

- **stdout** — each action's stdout is connected directly to Saran's stdout (no buffering or transformation). Output from sequential actions is streamed in order.
- **stderr** — each action's stderr is connected directly to Saran's stderr (no buffering or transformation).
- **stdin** — Saran's stdin is connected to each action in sequence. When one action completes, the next action receives stdin.
- **Exit code** — if any action in the sequence exits non-zero, Saran halts and returns that exit code exactly. If an action is terminated by a signal, Saran exits with `128 + signal_number` (Unix convention). If all actions succeed, Saran exits with `0`.

This ensures that callers (including LLM agents) can determine success or failure from Saran's exit code and consume the child process's output without modification.

---

## Validation Rules

Saran must reject a malformed wrapper file with a descriptive error. The following are validation errors:

- `name` is missing or empty
- `version` is missing, empty, or not a valid SemVer 2.0.0 string (must match `MAJOR.MINOR.PATCH` with optional pre-release/build metadata, e.g. `"1.0.0"`, `"0.2.1-beta.1"`)
- `commands` is missing or empty
- A command is missing `actions`
- `actions` is empty
- An `actions` entry has no executable key (the map is empty)
- An `actions` entry has more than two keys (only the executable key and `optional_flags` are permitted)
- An `actions` entry's executable key is not a valid executable name
- A "valid executable name" is a non-empty string containing no path separator (`/`) and no null byte. Absolute paths (beginning with `/`) and relative path traversal (containing `..` or `/`) are rejected. The executable is resolved via the process's `PATH` after var substitution.
- A `requires:` entry is missing `cli` or `version`
- A `requires:` entry's `version` is not a valid semver constraint string
- A `requires:` `version_probe` is not a non-empty array of strings
- A `requires:` `version_pattern` does not compile as a valid regex or does not contain exactly one capture group
- A `requires:` `version_probe` or `version_pattern` is declared without `version` (orphaned override)
- Two or more `requires:` entries share the same `cli` value
- An `optional_flags` entry is missing `name` or `type`
- An `optional_flags` `type` is not one of `str`, `bool`, `int`, `enum`
- An `optional_flags` entry has `type: enum` but is missing `values:`, or `values:` is empty
- An `optional_flags` entry has `type: bool` and `repeated: true` (mutually exclusive)
- An `optional_flags` `values:` entry is empty or contains characters outside `[a-z0-9_-]`
- An `optional_flags` `name` does not begin with `--`
- An `optional_flags` `name` contains characters outside `[a-z0-9-]` after the `--` prefix, or has consecutive/trailing hyphens
- A `commands` key contains characters outside `[a-z0-9-]`
- An `optional_flags` `passes_as` does not begin with `--`
- An `optional_flags` `passes_as` contains `=`
- Two or more `optional_flags` entries across all actions in the same command share the same `name` (flag names must be unique per command, regardless of which action they belong to)
- A `vars:` entry is missing `name`
- A `vars:` entry has both `required: true` and a `default:` value (mutually exclusive)
- A `vars:` entry has neither `required: true` nor a `default:` (ambiguous optionality — one must be specified)
- Two or more `vars:` entries share the same `name`
- A `vars:` `name` does not satisfy `[A-Za-z_][A-Za-z0-9_]*`
- A `$VAR_NAME` reference in an `actions` entry's argument array resolves to neither a `vars:` name nor an `args` `var_name`
- A `$VAR_NAME` pattern in an `actions` entry's argument array uses invalid syntax (e.g. a bare trailing `$`, or `$` followed by a digit)
- A `$VAR_NAME` reference in any `help:` string resolves to a name not declared in top-level `vars:`
- A `$VAR_NAME` pattern in any `help:` string uses invalid syntax (e.g. a bare trailing `$`, or `$` followed by a digit)
- An `args` entry is missing `name`, `var_name`, or `type`
- An `args` `type` is not `str`
- An `args` `var_name` conflicts with a `vars:` name or another `args` `var_name` in the same command
- An `args` `name` contains characters outside `[a-z0-9-]`
- Two or more `args` entries within the same command share the same `name` or `var_name`
- A required `args` entry appears after an optional one in the same command
- A `required: true` `vars:` entry has no value in any resolution layer at startup (runtime error, not a parse error)

---

## Default Behavior (No Subcommand)

If the generated CLI is invoked with no subcommand — or with `--help` — it prints the top-level help text (populated from `name` and `help`) and the list of available subcommands, then exits successfully. This mirrors standard `clap` behavior for multi-subcommand CLIs.

Invoking with an unrecognized subcommand is an error; `clap` will print an error message and exit with a non-zero status.

---

## Design Patterns

### Scope-locked vs. open wrappers

A common decision when authoring a wrapper is whether to fix a resource scope (e.g. a specific repository) into the wrapper itself, or to leave it open.

**Open wrapper** — omit the scoping variable entirely. The caller (or ambient environment) controls scope through the underlying CLI's own mechanisms. Suitable when the wrapper is shared across multiple projects.

```yaml
# gh-pr-ro: no GH_REPO var, no -R flag — works against any repo
commands:
  list:
    actions:
      - gh: [pr, list]
```

**Scope-locked wrapper** — declare the scoping variable as `required: true` with no `default:`. The wrapper will refuse to start unless the variable is set in the environment or `saran env`. Suitable for project-specific wrappers that should never accidentally operate against the wrong resource.

```yaml
# gh-pr-repo-ro: GH_REPO must be set — always operates on the declared repo
vars:
  - name: GH_REPO
    required: true
    help: "Target repository in OWNER/REPO format"
commands:
  list:
    help: "List pull requests in $GH_REPO"
    actions:
      - gh: [pr, list, -R, "$GH_REPO"]
```

> **Note:** There is intentionally no "optional scoping" model — a var with no `required:` and no `default:` is a validation error. Scope is either fixed (required) or absent (not declared). This keeps the wrapper's contract unambiguous.

---

## Non-Goals (v1)

> This document covers the Saran wrapper YAML format only. See [saran-cli.md](saran-cli.md) for the Saran CLI reference (install, remove, env).

The following are explicitly out of scope for the initial version:

- **Denylist mode** — subcommands not declared are simply not exposed; there is no way to say "allow everything except X"
- **Inter-action data passing** — stdout of one action cannot be piped as stdin to the next; actions share only the process environment
- **Conditional action execution** — all `actions` entries always run in sequence; there are no `if`/`when` guards
- **`$VAR_NAME` references in `optional_flags` values** — flag values are passed through as-is from the caller
- **Type validation beyond str/bool/int/enum** — no regex patterns, float types, or complex constraints
- **`default:` values referencing other variables** — `default:` must be a literal string; no `$VAR_NAME` substitution within defaults
