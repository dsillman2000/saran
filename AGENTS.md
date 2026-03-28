# Saran Project Guide

## Overview

**Saran** is a specification-driven, test-driven implementation of a CLI wrapper framework. It allows users to write declarative YAML wrappers around existing CLI tools, generating `clap`-powered binaries that expose only restricted subsets of the underlying CLI.

The project is structured as a multi-crate Rust workspace, implementing the core logic across specialized modules for parsing, validation, and code generation.

The primary use case is providing safe, read-only CLI access to LLM agents while preventing destructive actions.

## Project Philosophy

This project follows a **specification-driven** development methodology within a Rust implementation framework:

1. **Specifications** are the source of truth — all behavior is defined in `spec/*.md`
2. **Test specifications** define expected behavior before implementation
3. **Rust Implementation** - Features are implemented across six specialized crates: `saran`, `saran-core`, `saran-parser`, `saran-validation`, `saran-codegen`, and `saran-types`.
4. **Examples** demonstrate real-world wrapper configurations

**Do not implement without a corresponding specification.**

## Key Directories

| Directory | Purpose |
|-----------|---------|
| `spec/` | All specification documents |
| `crates/` | Core Rust implementation (six-crate design) |
| `spec/saran-format.md` | YAML wrapper schema and execution model |
| `spec/saran-cli.md` | User-facing CLI commands |
| `spec/saran-env.md` | Environment variable resolution |
| `spec/saran-conventions.md` | Wrapper naming conventions |
| `spec/saran-codegen.md` | YAML → Rust code generation |
| `spec/tests/unit/` | Unit test specifications (108 tests) |
| `spec/tests/integration/` | Integration test scenarios |
| `spec/examples/` | Example wrapper configurations |

## Specification Index

See [`spec/INDEX.md`](spec/INDEX.md) for the complete specification map, test coverage matrix, and data flow diagrams.

### Core Specifications

- **saran-format.md** — YAML wrapper file schema (commands, vars, quotas, actions)
- **saran-cli.md** — CLI commands (install, remove, list, validate, env)
- **saran-env.md** — Variable resolution chain and env.yaml format
- **saran-conventions.md** — Naming scheme: `[cli-fragment].[scope].[ro|rw][.quota]`
- **saran-codegen.md** — How YAML becomes executable Rust code

### Test Specifications

#### Unit Tests (spec/tests/unit/)

| File | Tests | Coverage |
|------|-------|----------|
| 01-yaml-validation.md | 59 | YAML schema validation, error messages |
| 02-token-parsing.md | 6 | `$VAR_NAME` token extraction |
| 03-variable-resolution.md | 14 | Env.yaml priority chain resolution |
| 04-substitution-resolution.md | 10 | Variable substitution in strings |
| 05-argument-assembly.md | 19 | Child process argv construction |

**Total: 108 unit tests**

#### Integration Tests (spec/tests/integration/)

- `scenarios/ro.yaml` — 11 scenarios for Redis read-only wrapper execution

## Variable Resolution Chain

Variables resolve in priority order (highest → lowest):

1. Per-wrapper namespace in `~/.local/share/saran/env.yaml`
2. Global namespace in `~/.local/share/saran/env.yaml`
3. Host environment
4. Default value in wrapper YAML

## Example Wrappers

Located in `spec/examples/`:

- `gh/` — GitHub CLI wrappers (gh-pr, gh-issue, gh-run, gh-release, gh-search)
- `glab/` — GitLab CLI wrappers (glab-mr, glab-issue, glab-ci, glab-release)
- `redis-cli/` — Redis CLI wrappers
- `greet.yaml` — Minimal example

Example wrapper naming: `gh-pr.repo.ro.yaml` means:
- `gh-pr` — wraps `gh pr`
- `.repo` — repo-scoped (requires GH_REPO)
- `.ro` — read-only

## Implementation Workflow

1. **Find the spec** — Check `spec/` for the relevant specification
2. **Find the test spec** — Check `spec/tests/unit/` or `spec/tests/integration/`
3. **Implement** — Write code that passes the test specifications
4. **Verify** — Run tests; all must pass

## Build/Run Commands

Check `README.md` or search for existing build scripts. If none exist, ask the user.

## Memory Usage (CRITICAL)

**One line, detailed** - Keep each memory on a single line to avoid git conflicts. Be detailed but concise. Include file references where applicable (e.g., "See: path/to/file.py").

- `memory_recall()` at session START and before answering any questions
- **NEVER** use `memory_remember()` automatically - only when user explicitly asks to remember something
- If user asks to remember: store as patterns, decisions, learnings, preferences, blockers, or context
- If new info contradicts existing memory: ask user before using `memory_forget()` + `memory_remember()`
- **End of session**: If significant patterns, decisions, or learnings were discovered, ask user: "Would you like me to remember [specific thing]?"

**Use memory_recall freely. NEVER memory_remember automatically.**

### Memory Types

| Type | Use For | Example |
|------|---------|---------|
| decision | Architecture/design choices | "Using Drizzle ORM over Prisma for type safety. See: src/db/schema.ts" |
| learning | Codebase discoveries | "Auth tokens stored in httpOnly cookies, not localStorage. See: src/auth/session.ts" |
| preference | User/project preferences | "User prefers functional components over class components" |
| blocker | Known issues | "Websocket reconnection fails on Safari - tracking in issue #42" |
| context | Feature/system info | "Payment integration uses Stripe in test mode. API keys in .env.local" |
| pattern | Code patterns | "All API routes follow /api/v1/[resource]/[action] pattern. See: src/routes/" |

### Memory Scopes

Use scopes to organize memories logically:

| Scope | Use For |
|-------|---------|
| `project` | Project-wide decisions and patterns |
| `user` | User-specific preferences |
| `auth` | Authentication/authorization context |
| `api` | API design decisions |
| `database` | Database schema and query patterns |
| `testing` | Testing strategies and known issues |
| `deployment` | Deployment and infrastructure notes |

### Example Memory Workflow

```
# At session start - always recall first
memory_recall()                           # Get all memories
memory_recall(scope="project")            # Get project-specific memories
memory_recall(type="blocker")             # Check for known blockers

# When user explicitly asks to remember
User: "Remember that we decided to use Redis for session storage"
memory_remember(
  type="decision",
  scope="project",
  content="Using Redis for session storage instead of database sessions. Config in src/lib/redis.ts"
)

# When updating existing memory
User: "Actually we switched from Redis to database sessions"
memory_update(
  type="decision",
  scope="project",
  content="Using database sessions (switched from Redis). See: src/lib/session.ts"
)

# When removing outdated memory
memory_forget(
  type="blocker",
  scope="testing",
  reason="Issue #42 was fixed in PR #58"
)

# Discovering all stored context
memory_list()  # Returns all scopes and types in use
```

## Tool Calling

**ALWAYS USE PARALLEL TOOLS WHEN APPLICABLE**.

When multiple independent operations are needed, batch them together:

```
# Good - parallel reads
Read file1.rs, file2.rs, file3.rs in parallel

# Good - parallel memory operations
memory_recall(scope="auth") + memory_recall(scope="api") in parallel

# Bad - sequential when parallel is possible
Read file1.rs
Read file2.rs
Read file3.rs
```

## Important Constraints

- Each wrapper compiles to a standalone binary
- No runtime YAML parsing or routing
- Generated code uses pinned `clap` for reliability
- Non-shell exec (std::process::Command) — no metacharacter interpretation
- Forced environment variables override inherited values
- Discrete argv elements (no word splitting)
