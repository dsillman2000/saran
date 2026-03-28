# Saran Wrapper Naming Conventions

This document describes the recommended naming convention for Saran wrapper files and the
`name:` field within them. Following this convention makes wrapper intent legible from the
filename alone without opening the file.

---

## Format

```
[cli-fragment].[scope].[ro|rw][.quota]
```

The four parts are:

| Part           | Required | Description                                                               |
| -------------- | -------- | ------------------------------------------------------------------------- |
| `cli-fragment` | Yes      | Identifies the CLI and subcommand(s) being wrapped                        |
| `scope`        | No       | Names the most-derived resource that is operator-fixed via `vars:`        |
| `ro` or `rw`   | Yes      | Declares the access level granted to the caller                           |
| `quota`        | No       | Present only when one or more `commands:` are quota-bounded via `quotas:` |

The separator between all parts is `.` (dot). Wrapper filenames use this name directly with a
`.yaml` extension appended: `<name>.yaml`.

> **Note:** The `name:` field in the YAML and the filename (without `.yaml`) must always match.

---

## `cli-fragment`

The CLI fragment identifies the underlying CLI binary and the relevant subcommand or operation
group being wrapped. Use `-` (hyphen) as the separator within this segment, following the
natural naming of the underlying CLI.

**Examples:**

| CLI command         | Fragment            |
| ------------------- | ------------------- |
| `gh pr`             | `gh-pr`             |
| `gh issue`          | `gh-issue`          |
| `gh run`            | `gh-run`            |
| `gh release`        | `gh-release`        |
| `gh search`         | `gh-search`         |
| `gh pr comment`     | `gh-pr-comment`     |
| `gh issue create`   | `gh-issue-create`   |
| `gh issue comment`  | `gh-issue-comment`  |
| `glab mr`           | `glab-mr`           |
| `glab issue`        | `glab-issue`        |
| `glab issue create` | `glab-issue-create` |
| `glab issue note`   | `glab-issue-note`   |
| `glab ci`           | `glab-ci`           |
| `glab release`      | `glab-release`      |

When a wrapper focuses on a specific write operation (relevant for `-rw` wrappers), the
write operation name is appended to the fragment rather than placed elsewhere in the name.
This keeps the fragment self-contained: `gh-issue-create.repo.rw.quota` wraps `gh issue`
with `create` as the quota-guarded write command.

---

## `scope`

The scope token names the **most-derived resource that is fixed by the operator** via `vars:`.
It is omitted entirely when no resources are operator-fixed (i.e. the wrapper is ambient —
the caller's environment determines the target).

**Scope implies a resource hierarchy.** A wrapper scoped to `issue` has both the repository
and the issue number fixed — `repo` is implied and does not need to appear in the name.
Only name the deepest fixed resource.

**Common scope tokens:**

| Token      | Variables implied                                   | Example                                                             |
| ---------- | --------------------------------------------------- | ------------------------------------------------------------------- |
| _(absent)_ | none fixed                                          | `gh-pr.ro`, `glab-mr.ro`                                            |
| `repo`     | `GH_REPO` / `GLAB_REPO`                             | `gh-pr.repo.ro`, `glab-mr.repo.ro`                                  |
| `pr`       | `GH_REPO` + `GH_PR`                                 | `gh-pr-comment.pr.rw.quota`                                         |
| `issue`    | `GH_REPO` + `GH_ISSUE` / `GLAB_REPO` + `GLAB_ISSUE` | `gh-issue-comment.issue.rw.quota`, `glab-issue-note.issue.rw.quota` |
| `branch`   | `GLAB_REPO` + `GLAB_BRANCH`                         | `glab-ci.branch.ro`                                                 |
| `run`      | `GH_REPO` + `GH_RUN_ID`                             | `gh-run-view.run.ro` _(hypothetical)_                               |
| `key`      | `REDIS_KEY` _(plus host/port/db)_                   | `redis-cli-string-set.key.rw.quota`                                 |
| `prefix`   | `REDIS_KEY_PREFIX` _(plus host/port/db)_            | `redis-cli-key-meta.prefix.ro`                                      |
| `db`       | `REDIS_DB` _(plus host/port)_                       | `redis-cli-info.db.ro`                                              |

Scope tokens are always lowercase and match a natural resource name, not a variable name.

---

## `ro` / `rw`

Declares the access level the wrapper grants to its caller.

| Suffix | Meaning                                                        |
| ------ | -------------------------------------------------------------- |
| `.ro`  | Read-only. No commands in the wrapper mutate state.            |
| `.rw`  | Read-write. At least one command in the wrapper mutates state. |

`.ro` and `.rw` are **always present** — never omit them. A wrapper with no scope token
still carries `.ro` or `.rw`: `gh-pr.ro`, `gh-run.ro`.

---

## `quota`

`.quota` is appended when the wrapper declares a `quotas:` block that bounds one or more
write commands. It is only meaningful on `.rw` wrappers: `.ro.quota` is not a valid
combination.

`.quota` signals to operators that:

1. The wrapper allows writes but they are bounded.
2. A quota variable (e.g. `GH_ISSUE_CREATE_QUOTA`) must be configured in `saran env`.
3. Quota state must be reset between sessions with `saran quotas reset <name>`.

---

## Examples

| Name                              | Interpretation                                                                  |
| --------------------------------- | ------------------------------------------------------------------------------- |
| `gh-pr.ro`                        | `gh pr`, ambient scope, read-only                                               |
| `gh-pr.repo.ro`                   | `gh pr`, repo fixed by operator, read-only                                      |
| `gh-pr-comment.pr.rw.quota`       | `gh pr comment`, repo+PR fixed by operator, writes allowed, quota-bounded       |
| `gh-issue.ro`                     | `gh issue`, ambient scope, read-only                                            |
| `gh-issue.repo.ro`                | `gh issue`, repo fixed by operator, read-only                                   |
| `gh-issue-create.repo.rw.quota`   | `gh issue create`, repo fixed by operator, writes allowed, quota-bounded        |
| `gh-issue-comment.issue.rw.quota` | `gh issue comment`, repo+issue fixed by operator, writes allowed, quota-bounded |
| `gh-release.repo.ro`              | `gh release`, repo fixed by operator, read-only                                 |
| `gh-run.ro`                       | `gh run`, ambient scope, read-only                                              |
| `gh-run.repo.ro`                  | `gh run`, repo fixed by operator, read-only                                     |
| `gh-search.repo.ro`               | `gh search`, repo fixed by operator, read-only                                  |
| `glab-mr.ro`                      | `glab mr`, ambient scope, read-only                                             |
| `glab-mr.repo.ro`                 | `glab mr`, repo fixed by operator, read-only                                    |
| `glab-issue.ro`                   | `glab issue`, ambient scope, read-only                                          |
| `glab-issue.repo.ro`              | `glab issue`, repo fixed by operator, read-only                                 |
| `glab-issue-create.repo.rw.quota` | `glab issue create`, repo fixed by operator, writes allowed, quota-bounded      |
| `glab-issue-note.issue.rw.quota`  | `glab issue note`, repo+issue fixed by operator, writes allowed, quota-bounded  |
| `glab-ci.ro`                      | `glab ci`, ambient scope, read-only                                             |
| `glab-ci.repo.ro`                 | `glab ci`, repo fixed by operator, read-only                                    |
| `glab-ci.branch.ro`               | `glab ci`, repo+branch fixed by operator, read-only                             |
| `glab-release.repo.ro`            | `glab release`, repo fixed by operator, read-only                               |

---

## File placement

Wrapper files are grouped into subdirectories by the CLI they wrap:

```
spec/examples/
  gh/                  # GitHub CLI wrappers
    gh-pr.ro.yaml
    gh-pr.repo.ro.yaml
    ...
  glab/                # GitLab CLI wrappers
    glab-mr.repo.ro.yaml
    ...
  greet.yaml           # Generic / illustrative examples at root
```

---

## See also

- [`spec/saran-format.md`](saran-format.md) — schema for authoring wrapper files
- [`spec/saran-env.md`](saran-env.md) — how `vars:` and `saran env` work
- [`spec/saran-cli.md`](saran-cli.md) — `saran quotas` command reference
