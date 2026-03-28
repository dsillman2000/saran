//! Core type definitions for the Saran CLI wrapper framework.
//!
//! This crate provides canonical Rust representations of wrapper YAML files.
//! No parsing, validation, or code generation logic is present here — only data structures.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A complete Saran wrapper definition, representing a single YAML wrapper file.
///
/// This is the in-memory representation created by parsing a wrapper YAML file.
/// It captures all metadata needed to validate, generate code, and install the wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WrapperDefinition {
    /// The name of the generated CLI binary (e.g., `gh-pr.repo.ro`).
    /// Must be non-empty.
    pub name: String,

    /// Semantic version of this wrapper (e.g., `"1.0.0"`).
    /// Must conform to SemVer 2.0.0.
    pub version: String,

    /// Optional top-level help/about text shown by `--help`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,

    /// Optional version constraints on external CLIs required for this wrapper to function.
    #[serde(default)]
    pub requires: Vec<CliRequirement>,

    /// Environment variable declarations this wrapper depends on.
    /// Variables are resolved at startup from the env.yaml priority chain.
    #[serde(default)]
    pub vars: Vec<VarDecl>,

    /// Quota declarations that bound write command executions.
    #[serde(default)]
    pub quotas: Vec<QuotaEntry>,

    /// Named subcommands exposed by the wrapper.
    /// Each command name becomes a clap subcommand.
    pub commands: BTreeMap<String, Command>,
}

/// A version constraint on an external CLI dependency.
///
/// Example:
/// ```yaml
/// - cli: gh
///   version: ">=2.0.0"
///   version_probe: [git, --version]
///   version_pattern: "git version (\\S+)"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliRequirement {
    /// The executable name to check (e.g., `gh`, `redis-cli`).
    /// Must satisfy `[a-z0-9_-]+`.
    pub cli: String,

    /// A semver constraint string (e.g., `">=2.0.0"`, `">=1.0.0 <3.0.0"`).
    /// Supports operators: `>=`, `>`, `<=`, `<`, `=`.
    pub version: String,

    /// Override the command used to query the CLI's version.
    /// Defaults to `[cli, --version]`.
    #[serde(default)]
    pub version_probe: Option<Vec<String>>,

    /// Regex with one capture group to extract version from probe output.
    /// Defaults to matching first `\d+\.\d+\.\d+[\w.-]*`.
    #[serde(default)]
    pub version_pattern: Option<String>,
}

/// An environment variable declaration.
///
/// Variables declared here are resolved at wrapper startup from a 4-tier priority chain:
/// 1. Per-wrapper namespace in `~/.local/share/saran/env.yaml`
/// 2. Global namespace in `~/.local/share/saran/env.yaml`
/// 3. Host environment
/// 4. Default value (if provided)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VarDecl {
    /// The environment variable name (e.g., `GH_REPO`, `REDIS_HOST`).
    /// Must satisfy `[A-Za-z_][A-Za-z0-9_]*`.
    pub name: String,

    /// Whether this variable is required to be set by the resolution chain.
    /// Mutually exclusive with `default`.
    #[serde(default)]
    pub required: bool,

    /// A literal string fallback if no higher-priority source provides the variable.
    /// Mutually exclusive with `required: true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Help text describing this variable (shown in `saran env` output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// A quota declaration that bounds write command executions.
///
/// Quota-guarded commands are only valid in `.rw` wrappers.
/// Quota state is stored in `~/.local/share/saran/quotas.yaml` and reset with `saran quotas reset`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuotaEntry {
    /// The name of the command to quota (must match a key in `commands:`).
    pub command: String,

    /// The maximum number of executions allowed.
    /// Can be a literal integer or a `$VAR_NAME` reference to a variable.
    pub limit: QuotaLimit,
}

/// The limit for a quota, either a literal integer or a variable reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum QuotaLimit {
    /// Literal integer limit
    Literal(u32),
    /// Reference to a variable (e.g., `$GH_PR_COMMENT_QUOTA`)
    Variable(String),
}

/// A subcommand definition.
///
/// Each key in the `commands:` map becomes a subcommand in the generated clap CLI.
///
/// Example:
/// ```yaml
/// commands:
///   list:
///     help: "List pull requests"
///     args: [...]
///     actions:
///       - gh: [pr, list, -R, "$GH_REPO"]
///         optional_flags: [...]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Command {
    /// Help text for this subcommand (shown in `--help` output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,

    /// Positional arguments the caller may supply when invoking this subcommand.
    /// Arguments are declared in the order clap expects them on the command line.
    #[serde(default)]
    pub args: Vec<PositionalArg>,

    /// Ordered list of process invocations to execute when this subcommand is called.
    /// If any action exits non-zero, execution halts immediately.
    pub actions: Vec<Action>,
}

/// A positional argument definition.
///
/// Positional args are collected from the caller and injected into actions via `$VAR_NAME` substitution.
///
/// Example:
/// ```yaml
/// args:
///   - name: repo
///     var_name: REPO
///     type: str
///     required: true
///     help: "Repository in OWNER/REPO format"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PositionalArg {
    /// The display name for this positional argument in clap's usage and help output.
    /// Must consist only of lowercase ASCII letters, digits, and hyphens.
    pub name: String,

    /// The substitution variable name referenced in actions as `$VAR_NAME`.
    /// Must satisfy `[A-Za-z_][A-Za-z0-9_]*`.
    pub var_name: String,

    /// The type of this argument. In v1, only `str` is supported.
    #[serde(default = "default_arg_type")]
    pub arg_type: String,

    /// Whether the caller must supply this argument.
    /// Defaults to `true`. Required positionals must not appear after optional ones.
    #[serde(default = "default_true")]
    pub required: bool,

    /// Help text describing this argument (shown in clap's `--help` output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// A process invocation to execute as part of a command.
///
/// Each action is a map with one required key (the executable name and its fixed argument array)
/// and one optional key (`optional_flags`).
///
/// Example:
/// ```yaml
/// actions:
///   - gh: [pr, list, -R, "$GH_REPO"]
///     optional_flags:
///       - name: --json
///         type: str
///         help: "JSON output fields"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    /// The executable name (e.g., `gh`, `redis-cli`) and its fixed argument array.
    /// The executable is resolved via PATH; it must not contain `/` or `..`.
    /// Each element in the array may contain `$VAR_NAME` substitution references.
    pub executable: String,

    /// The fixed argument array for this executable.
    /// Each element is passed as a discrete argument to the child process.
    /// Elements may contain `$VAR_NAME` references to `vars:` or `args:` var_names.
    pub args: Vec<String>,

    /// Flags that the caller may optionally supply when invoking this action.
    /// When supplied, flags are appended to the action's argument array at execution time.
    #[serde(default)]
    pub optional_flags: Vec<OptionalFlag>,
}

/// An optional flag definition.
///
/// Optional flags are exposed to the caller via clap. When supplied, they are appended
/// to the action's argument array at execution time.
///
/// Example:
/// ```yaml
/// - name: --json
///   type: str
///   help: "JSON output fields"
///   passes_as: --output-format
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptionalFlag {
    /// The flag name as it appears in the generated saran CLI (e.g., `--json`).
    /// Must begin with `--` and contain only lowercase letters, digits, and hyphens.
    pub name: String,

    /// The value type for this flag. Supports: `str`, `bool`, `int`, `enum`.
    pub flag_type: String,

    /// Allow this flag to be supplied multiple times by the caller.
    /// Defaults to `false`. Mutually exclusive with `type: bool`.
    #[serde(default)]
    pub repeated: bool,

    /// Help text for this flag (shown in clap's `--help` output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,

    /// The flag name passed to the underlying CLI if different from `name`.
    /// Must begin with `--` and must not contain `=`.
    /// Defaults to `name` if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passes_as: Option<String>,

    /// For `type: enum`, the list of allowed values.
    /// Each value must satisfy `[a-z0-9][a-z0-9_-]*`.
    #[serde(default)]
    pub values: Vec<String>,
}

// Default helper functions for serde defaults
fn default_arg_type() -> String {
    "str".to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests;
