# Saran

**Saran** is a tool for writing lightweight, declarative wrappers around existing CLIs. You describe
the wrapper in a YAML file, and Saran dynamically generates a `clap`-powered CLI that calls the
underlying executable with arguments set, restricted, or surfaced as the wrapper sees fit.

The primary use case is exposing a **read-only or otherwise restricted subset** of a CLI to an LLM
agent, preventing it from taking destructive actions while still giving it the access it needs.

> **Example:** wrap `gh` as `gh-pr-ro` — a CLI that only exposes the read-only subcommands of
> `gh pr` (`list`, `status`, `view`, `diff`, `checks`, `checkout`), with the repo fixed to a
> specific value the caller cannot override.

---

## How it works

1. Author a `.yaml` wrapper file following the [Saran format spec](spec/saran-format.md).
2. Install it with `saran install <file.yaml>` (or from a remote repo with `--git`).
3. Saran registers a symlink so the wrapper name is available directly on your `PATH`.
4. Invoke your wrapper like any other CLI — Saran resolves variables, assembles `argv`, and
   `execvp`s the underlying command. No shell. No metacharacters.

---

## Specification

| Document | Description |
|---|---|
| [`spec/saran-format.md`](spec/saran-format.md) | YAML wrapper format — the complete schema reference for authoring wrapper files |
| [`spec/saran-cli.md`](spec/saran-cli.md) | `saran` CLI reference — `install`, `remove`, `list`, `validate`, and the multicall model |
| [`spec/saran-env.md`](spec/saran-env.md) | `saran env` reference — variable resolution, `env.yaml` format, and security guidance |

---

## Examples

| File | Description |
|---|---|
| [`spec/examples/gh-pr-ro.yaml`](spec/examples/gh-pr-ro.yaml) | Read-only wrapper for `gh pr` — exposes `list`, `status`, `view`, `diff`, `checks`, `checkout` with no fixed repo scope |
| [`spec/examples/gh-pr-repo-ro.yaml`](spec/examples/gh-pr-repo-ro.yaml) | Repo-locked variant of `gh-pr-ro` — requires `GH_REPO` to be set via `saran env` |
| [`spec/examples/gh-issue-ro.yaml`](spec/examples/gh-issue-ro.yaml) | Read-only wrapper for `gh issue` — exposes `list`, `status`, `view` with no fixed repo scope |
| [`spec/examples/gh-issue-repo-ro.yaml`](spec/examples/gh-issue-repo-ro.yaml) | Repo-locked variant of `gh-issue-ro` — requires `GH_REPO` to be set via `saran env` |
| [`spec/examples/greet.yaml`](spec/examples/greet.yaml) | Minimal wrapper demonstrating positional arguments |

---

## Status

This branch (`specification`) contains the design specification only. No implementation exists yet.
