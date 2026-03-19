# Saran CLI Reference

## Overview

The `saran` binary is both a wrapper executor and a management tool. When invoked directly as `saran`, it provides subcommands for installing, removing, and configuring wrappers. When invoked via a symlink (the multicall pattern), it executes the corresponding wrapper.

> **See also:** [`saran-format.md`](saran-format.md) for the wrapper YAML format, [`saran-env.md`](saran-env.md) for the `saran env` command.

---

## Multicall Execution Model

Saran uses the **multicall binary** pattern for zero-overhead wrapper invocation. When `saran install` registers a wrapper, it creates a symlink in `~/.local/share/saran/bin/` pointing back to the `saran` binary. When invoked via that symlink, `saran` reads `argv[0]` (the name it was called as), looks up the corresponding wrapper at `~/.local/share/saran/wrappers/<name>.yaml`, and executes it.

```
~/.local/share/saran/
  bin/
    gh-pr-ro      →  /usr/local/bin/saran   (symlink)
    gh-issue-ro   →  /usr/local/bin/saran   (symlink)
  wrappers/
    gh-pr-ro.yaml
    gh-issue-ro.yaml
  env.yaml
```

This means one `saran` binary serves all installed wrappers, with no per-wrapper compilation required.

> **PATH setup:** Add `~/.local/share/saran/bin` to your shell `PATH` to make installed wrappers directly invocable by name:
> ```bash
> export PATH="$HOME/.local/share/saran/bin:$PATH"
> ```

---

## `saran install`

Installs a Saran wrapper from a local file or a remote Git repository.

### Local install

```
saran install <path-to-yaml>
```

1. Validates the YAML file against the Saran format spec
2. Reads the `name:` field from the file
3. Copies the file to `~/.local/share/saran/wrappers/<name>.yaml`
4. Creates a symlink `~/.local/share/saran/bin/<name> → saran`

If a wrapper with the same name is already installed, `saran install` exits with an error. Use `--force` to overwrite.

```bash
saran install ./gh-pr-ro.yaml
saran install ./gh-pr-ro.yaml --force   # overwrite existing
```

### Remote install via Git

```
saran install --git <repo-url> [<path-within-repo>]
```

Clones or fetches the specified Git repository and installs wrapper(s) from it.

```bash
# Install all .yaml files found at the repo root
saran install --git github.com/myorg/saran-wrappers

# Install a specific file within the repo
saran install --git github.com/myorg/saran-wrappers gh-pr-ro.yaml

# Install from a subdirectory
saran install --git github.com/myorg/saran-wrappers wrappers/gh-pr-ro.yaml
```

**URL format:** The `--git` value is a repository URL. The following formats are accepted:

| Format | Example |
|---|---|
| Shorthand (GitHub assumed) | `github.com/myorg/myrepo` |
| Full HTTPS URL | `https://github.com/myorg/myrepo` |
| SSH URL | `git@github.com:myorg/myrepo.git` |

**Path argument:** If no path is specified, Saran looks for all `*.yaml` files at the repository root that are valid Saran wrapper files and installs them all. If a path is specified, only that file is installed.

**Version pinning:** An optional `@ref` suffix pins the install to a specific branch, tag, or commit SHA:

```bash
saran install --git github.com/myorg/saran-wrappers@v1.2.0
saran install --git github.com/myorg/saran-wrappers@main gh-pr-ro.yaml
```

If no `@ref` is specified, the default branch HEAD is used.

**Cloning behavior:** The repository is cloned into a temporary directory, the specified file(s) are copied to `~/.local/share/saran/wrappers/`, and the clone is discarded. Saran does not retain the full repository after install.

### Flags

| Flag | Description |
|---|---|
| `--force` | Overwrite an existing wrapper with the same name |
| `--git <url>` | Install from a remote Git repository |
| `--dry-run` | Validate and print what would be installed without making changes |

---

## `saran remove`

Uninstalls one or more installed wrappers.

```
saran remove <name> [<name> ...]
```

For each named wrapper:
1. Removes the symlink at `~/.local/share/saran/bin/<name>`
2. Removes the wrapper file at `~/.local/share/saran/wrappers/<name>.yaml`
3. Removes any per-wrapper entries for `<name>` from `~/.local/share/saran/env.yaml`

```bash
saran remove gh-pr-ro
saran remove gh-pr-ro gh-issue-ro   # remove multiple at once
```

If the named wrapper is not installed, `saran remove` exits with an error.

### Flags

| Flag | Description |
|---|---|
| `--keep-env` | Remove the wrapper files and symlink but preserve per-wrapper entries in `env.yaml` |
| `--dry-run` | Print what would be removed without making changes |

---

## `saran list`

Lists all currently installed wrappers.

```
$ saran list

gh-pr-ro      1.0.0   ~/.local/share/saran/wrappers/gh-pr-ro.yaml
gh-issue-ro   1.0.0   ⚠ gh 1.8.3 (requires >=2.0.0)   ~/.local/share/saran/wrappers/gh-issue-ro.yaml
greet         0.1.0   ~/.local/share/saran/wrappers/greet.yaml
```

The version shown is read from the `version:` field in each installed wrapper's YAML file — it reflects the wrapper definition version, not the underlying CLI's version. For wrappers installed via `--git`, the `@ref` tag pins which revision of the repository is fetched, but the displayed version always comes from the `version:` field inside the wrapper file itself.

If a wrapper's `requires:` constraints are not satisfied on the current system, each failing requirement is shown inline as a soft warning. If the version probe fails entirely (CLI not found or no version string matched), the warning reads `⚠ <cli> unknown (requires <constraint>)`. Wrappers with failed requirements remain invocable; the warning is informational only.

---

## `saran validate`

Validates a wrapper YAML file against the Saran format spec without installing it.

```
saran validate <path-to-yaml>
```

Exits with code `0` if the file is valid, non-zero with a descriptive error message if not. Useful for CI or pre-install checks.

Validation includes checking all `requires:` constraints against the current system. If any version probe fails or the detected version does not satisfy the declared constraint, `saran validate` exits non-zero with a clear diagnostic:

```bash
saran validate ./gh-pr-ro.yaml
# → OK: gh-pr-ro v1.0.0 (6 commands, 2 vars, 1 requirement checked)

saran validate ./gh-pr-ro.yaml
# → error: requires gh >=2.0.0, but found gh 1.8.3
#          Update gh or loosen the version constraint in the wrapper.

saran validate ./gh-pr-ro.yaml
# → error: requires gh >=2.0.0, but version probe failed: `gh --version` exited with code 127
#          Ensure `gh` is installed and available on PATH.
```

---

## Data Directory

All Saran state lives under `~/.local/share/saran/` by default:

```
~/.local/share/saran/
  bin/          Symlinks for installed wrappers (add to PATH)
  wrappers/     Installed wrapper YAML files
  env.yaml      Operator-managed variable configuration
```

The data directory can be overridden by setting the `SARAN_DATA_DIR` environment variable:

```bash
export SARAN_DATA_DIR=/opt/saran
```
