# Saran Environment Configuration

## Overview

Saran maintains a shared environment configuration file at `~/.local/share/saran/env.yaml`. This file stores operator-managed variable values that are layered on top of wrapper defaults and the host environment when resolving `vars:` declarations at startup.

The `saran env` command is the primary interface for reading and writing this configuration.

> **See also:** [`saran-format.md`](saran-format.md) for how `vars:` are declared in a wrapper file, and [`saran-cli.md`](saran-cli.md) for other `saran` commands.

---

## Variable Resolution Chain

When a Saran wrapper starts, each variable declared in `vars:` is resolved in the following priority order. The **first source that provides a value wins**:

| Priority    | Source                              | How to set                              |
| ----------- | ----------------------------------- | --------------------------------------- |
| 1 (highest) | Per-wrapper namespace in `env.yaml` | `saran env <wrapper> VAR=value`         |
| 2           | Global namespace in `env.yaml`      | `saran env VAR=value`                   |
| 3           | Host environment                    | Shell profile, credential manager, etc. |
| 4 (lowest)  | `default:` in wrapper `vars:` entry | Declared in the wrapper YAML            |

If all four sources yield no value and the variable is declared `required: true`, Saran exits immediately with a descriptive error before executing any command:

```
error: required variable `GH_TOKEN` is not set.
       Set it in your host environment or via: saran env gh-pr.repo.ro GH_TOKEN=<value>
       Note: storing secrets in saran env is not recommended — use your host environment instead.
```

> **Security note:** Because Saran uses direct process invocation and callers can only supply values through `optional_flags`, resolved `vars:` values are guaranteed at invocation time — the caller cannot override them.

---

## `saran env` Command Reference

### Read: `saran env`

Prints all resolved variable values for all installed wrappers, annotated with the source that provided each value. Warns when a `required: true` variable has no value in any layer.

```
$ saran env

global:
  (none)

gh-pr.repo.ro:
  GH_TOKEN   ⚠  required — not set (set in host environment)
  GH_REPO    myorg/myrepo   [default]

gh-issue.ro:
  GH_TOKEN   ⚠  required — not set (set in host environment)
  GH_REPO    myorg/myrepo   [default]
```

### Read: `saran env <wrapper>`

Prints variable resolution state for a single installed wrapper only.

```
$ saran env gh-pr.repo.ro

gh-pr.repo.ro:
  GH_TOKEN   ⚠  required — not set (set in host environment)
  GH_REPO    myotherorg/myotherrepo   [per-wrapper]
```

### Write: `saran env [<wrapper>] VAR=value`

Sets a variable in the per-wrapper or global namespace of `env.yaml`.

```bash
# Per-wrapper: only affects gh-pr.repo.ro
saran env gh-pr.repo.ro GH_REPO=myotherorg/myotherrepo

# Global: affects all wrappers that declare GH_REPO
saran env GH_REPO=myotherorg/myotherrepo

# Multiple assignments in one command
saran env gh-pr.repo.ro GH_REPO=myotherorg/myotherrepo GH_PR_ID=99
```

When a per-wrapper value is set, it takes precedence over the global namespace, host environment, and wrapper default for that wrapper only.

### Unset: `saran env [<wrapper>] --unset VAR`

Removes a variable from the per-wrapper or global namespace, allowing the next lower-priority source to take effect.

```bash
saran env gh-pr.repo.ro --unset GH_REPO    # removes per-wrapper override; falls back to global/host/default
saran env --unset GH_REPO             # removes global override
```

---

## Source Annotations

`saran env` output annotates each value with its source:

| Annotation              | Meaning                                                 |
| ----------------------- | ------------------------------------------------------- |
| `[per-wrapper]`         | Set via `saran env <wrapper> VAR=value`                 |
| `[global]`              | Set via `saran env VAR=value`                           |
| `[host]`                | Resolved from the host environment                      |
| `[default]`             | Falling back to `default:` declared in the wrapper YAML |
| `⚠ required — not set` | `required: true` with no value in any layer             |

> **Note:** `saran env` never invokes any wrapper. It is a purely diagnostic and configuration tool.

---

## `env.yaml` Format

`~/.local/share/saran/env.yaml` is a YAML file with two top-level keys:

```yaml
global:
  GH_DEBUG: "1"

wrappers:
  gh-pr.repo.ro:
    GH_REPO: myotherorg/myotherrepo
  gh-issue.ro:
    GH_REPO: myotherorg/myotherrepo
```

### `global`

A map of variable names to string values. These are applied to every installed wrapper's resolution chain at priority 2 (below per-wrapper, above host).

### `wrappers`

A map of wrapper names to per-wrapper variable maps. Each wrapper's map is applied only when that wrapper is invoked, at priority 1 (highest).

---

## Security Guidance

`env.yaml` is stored as **plaintext** on disk. The following practices are strongly recommended:

- **Do not store secrets** (API tokens, passwords, credentials) in `env.yaml`. Any process with filesystem access to `~/.local/share/saran/` can read them.
- **For secrets**, declare the variable as `required: true` (no `default:`) in the wrapper's `vars:` block and set it in your host environment via your shell profile, a secrets manager (e.g. `pass`, `1Password CLI`, macOS Keychain), or a credential helper. This ensures secrets never touch saran's configuration files.
- **For non-secret context** (repository names, PR numbers, project identifiers), `saran env` is the appropriate tool. These values are not sensitive and benefit from the ergonomics of per-wrapper and global namespacing.

---

## Quota State Storage

Quota state is stored in `~/.local/share/saran/quotas.yaml`. This file tracks remaining operations per wrapper/command combination.

### `quotas.yaml` Format

```yaml
gh-pr-comment.pr.rw.quota:
  comment:
    remaining: 1
    limit: 1

glab-mr-note.mr.rw.quota:
  note:
    remaining: 3
    limit: 5
  resolve:
    remaining: 5
    limit: 5
```

### Quota Entry Fields

| Field       | Description                                                               |
| ----------- | ------------------------------------------------------------------------- |
| `remaining` | Number of executions remaining until reset                                |
| `limit`     | The configured maximum (from wrapper's `quotas:` declaration or variable) |

### Quota Behavior

- When a quota-guarded command is executed, `remaining` is decremented before the command runs
- If `remaining` is 0, the command is rejected with an error and the wrapper exits with code 68 (exceeds quota)
- `saran quotas reset <wrapper>` sets all `remaining` values back to their `limit` values
- Quota state persists across wrapper invocations until manually reset by the operator
