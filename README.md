# Saran

**Saran** is a tool for writing lightweight, declarative wrappers around existing CLIs. You describe
the wrapper in a YAML file, and Saran dynamically generates a `clap`-powered CLI that calls the
underlying executable with arguments set, restricted, or surfaced as the wrapper sees fit.

The primary use case is exposing a **read-only or otherwise restricted subset** of a CLI to an LLM
agent, preventing it from taking destructive actions while still giving it the access it needs.

> **Example:** wrap `gh` as `gh-pr.repo.ro` — a CLI that only exposes the read-only subcommands of
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
| [`spec/saran-format.md`](spec/saran-format.md) | YAML wrapper format — the complete schema reference for authoring wrapper files, including `vars`, `quotas`, and `commands` |
| [`spec/saran-cli.md`](spec/saran-cli.md) | `saran` CLI reference — `install`, `remove`, `list`, `validate`, `quotas`, and the multicall model |
| [`spec/saran-env.md`](spec/saran-env.md) | `saran env` reference — variable resolution, `env.yaml` format, and security guidance |
| [`spec/saran-conventions.md`](spec/saran-conventions.md) | Naming conventions for wrapper files — the `[cli-fragment].[scope].[ro\|rw][.quota]` scheme |

---

## Examples

Examples are organized by the CLI they wrap under `spec/examples/`.

### `spec/examples/gh/` — GitHub CLI wrappers

#### `gh pr`

| File | Description |
|---|---|
| [`spec/examples/gh/gh-pr.ro.yaml`](spec/examples/gh/gh-pr.ro.yaml) | Read-only wrapper for `gh pr` — exposes `list`, `status`, `view`, `diff`, `checks`, `checkout` with no fixed repo scope |
| [`spec/examples/gh/gh-pr.repo.ro.yaml`](spec/examples/gh/gh-pr.repo.ro.yaml) | Repo-locked variant — requires `GH_REPO` to be set via `saran env` |
| [`spec/examples/gh/gh-pr-comment.pr.rw.quota.yaml`](spec/examples/gh/gh-pr-comment.pr.rw.quota.yaml) | PR- and repo-locked wrapper with read-only commands plus a quota-guarded `comment` — `GH_REPO`, `GH_PR`, and `GH_PR_COMMENT_QUOTA` configured via `saran env` |

#### `gh issue`

| File | Description |
|---|---|
| [`spec/examples/gh/gh-issue.ro.yaml`](spec/examples/gh/gh-issue.ro.yaml) | Read-only wrapper for `gh issue` — exposes `list`, `status`, `view` with no fixed repo scope |
| [`spec/examples/gh/gh-issue.repo.ro.yaml`](spec/examples/gh/gh-issue.repo.ro.yaml) | Repo-locked variant — requires `GH_REPO` to be set via `saran env` |
| [`spec/examples/gh/gh-issue-create.repo.rw.quota.yaml`](spec/examples/gh/gh-issue-create.repo.rw.quota.yaml) | Repo-locked wrapper with read-only commands plus a quota-guarded `create` — `GH_REPO` and `GH_ISSUE_CREATE_QUOTA` configured via `saran env` |
| [`spec/examples/gh/gh-issue-comment.issue.rw.quota.yaml`](spec/examples/gh/gh-issue-comment.issue.rw.quota.yaml) | Issue- and repo-locked wrapper with read-only commands plus a quota-guarded `comment` — `GH_REPO`, `GH_ISSUE`, and `GH_ISSUE_COMMENT_QUOTA` configured via `saran env` |

#### `gh run`

| File | Description |
|---|---|
| [`spec/examples/gh/gh-run.ro.yaml`](spec/examples/gh/gh-run.ro.yaml) | Read-only wrapper for `gh run` — exposes `list`, `view`, `watch`, `download` with no fixed repo scope |
| [`spec/examples/gh/gh-run.repo.ro.yaml`](spec/examples/gh/gh-run.repo.ro.yaml) | Repo-locked variant — requires `GH_REPO` to be set via `saran env` |

#### `gh release`

| File | Description |
|---|---|
| [`spec/examples/gh/gh-release.repo.ro.yaml`](spec/examples/gh/gh-release.repo.ro.yaml) | Repo-locked read-only wrapper for `gh release` — exposes `list`, `view`, `download`; `view` and `download` require an explicit tag |

#### `gh search`

| File | Description |
|---|---|
| [`spec/examples/gh/gh-search.repo.ro.yaml`](spec/examples/gh/gh-search.repo.ro.yaml) | Repo-scoped read-only wrapper for `gh search` — exposes `issues`, `prs`, `commits`, `code`, all filtered to `GH_REPO` via `--repo` |

### `spec/examples/glab/` — GitLab CLI wrappers

#### `glab mr`

| File | Description |
|---|---|
| [`spec/examples/glab/glab-mr.ro.yaml`](spec/examples/glab/glab-mr.ro.yaml) | Read-only wrapper for `glab mr` — exposes `list`, `view`, `diff`, `issues`, `approvers` with no fixed repo scope; `--repo` available per-invocation |
| [`spec/examples/glab/glab-mr.repo.ro.yaml`](spec/examples/glab/glab-mr.repo.ro.yaml) | Repo-locked variant — requires `GLAB_REPO` to be set via `saran env` |
| [`spec/examples/glab/glab-mr-note.mr.rw.quota.yaml`](spec/examples/glab/glab-mr-note.mr.rw.quota.yaml) | MR- and repo-locked wrapper with `list`, fixed-MR `view` and `diff`, plus quota-guarded `note`, `resolve`, and `unresolve` — `GLAB_REPO`, `GLAB_MR`, and `GLAB_MR_NOTE_QUOTA` configured via `saran env` |

#### `glab issue`

| File | Description |
|---|---|
| [`spec/examples/glab/glab-issue.ro.yaml`](spec/examples/glab/glab-issue.ro.yaml) | Read-only wrapper for `glab issue` — exposes `list`, `view`, `board-view` with no fixed repo scope |
| [`spec/examples/glab/glab-issue.repo.ro.yaml`](spec/examples/glab/glab-issue.repo.ro.yaml) | Repo-locked variant — requires `GLAB_REPO` to be set via `saran env` |
| [`spec/examples/glab/glab-issue-create.repo.rw.quota.yaml`](spec/examples/glab/glab-issue-create.repo.rw.quota.yaml) | Repo-locked wrapper with read-only commands plus a quota-guarded `create` — `GLAB_REPO` and `GLAB_ISSUE_CREATE_QUOTA` configured via `saran env` |
| [`spec/examples/glab/glab-issue-note.issue.rw.quota.yaml`](spec/examples/glab/glab-issue-note.issue.rw.quota.yaml) | Issue- and repo-locked wrapper with `list`, fixed-issue `view`, plus a quota-guarded `note` — `GLAB_REPO`, `GLAB_ISSUE`, and `GLAB_ISSUE_NOTE_QUOTA` configured via `saran env` |

#### `glab ci`

| File | Description |
|---|---|
| [`spec/examples/glab/glab-ci.ro.yaml`](spec/examples/glab/glab-ci.ro.yaml) | Read-only wrapper for `glab ci` — exposes `list`, `get`, `status`, `trace`, `artifact` with no fixed repo scope |
| [`spec/examples/glab/glab-ci.repo.ro.yaml`](spec/examples/glab/glab-ci.repo.ro.yaml) | Repo-locked variant — requires `GLAB_REPO` to be set via `saran env` |
| [`spec/examples/glab/glab-ci.branch.ro.yaml`](spec/examples/glab/glab-ci.branch.ro.yaml) | Branch-locked variant — fixes both `GLAB_REPO` and `GLAB_BRANCH`; `--branch`/`--ref` removed from optional flags since they are baked in |

#### `glab release`

| File | Description |
|---|---|
| [`spec/examples/glab/glab-release.repo.ro.yaml`](spec/examples/glab/glab-release.repo.ro.yaml) | Repo-locked read-only wrapper for `glab release` — exposes `list`, `view`, `download`; tag is optional on all commands (omit to target latest release) |

### `spec/examples/` — General examples

| File | Description |
|---|---|
| [`spec/examples/greet.yaml`](spec/examples/greet.yaml) | Minimal wrapper demonstrating positional arguments |

---

## Status

This branch (`specification`) contains the design specification only. No implementation exists yet.
