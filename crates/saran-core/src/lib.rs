//! Runtime execution layer for generated Saran wrapper binaries.
//!
//! This crate provides the core runtime functionality that generated wrapper binaries depend on:
//! - Variable resolution from `env.yaml` with priority chain
//! - `$VAR_NAME` token substitution in strings
//! - Argument assembly for child process execution

use std::collections::HashMap;
use thiserror::Error;

pub use saran_parser::{parse_tokens, ParsedTemplate, Token};

// ============================================================================
// Phase 1: Variable Resolution Types
// ============================================================================

/// The source of a resolved variable value in the priority chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SaranEnvScope {
    /// Per-wrapper namespace in `env.yaml` (highest priority)
    PerWrapper,
    /// Global namespace in `env.yaml`
    Global,
    /// Host environment variables
    Host,
    /// Default value from wrapper `vars:` declaration (lowest priority)
    Default,
}

impl std::fmt::Display for SaranEnvScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaranEnvScope::PerWrapper => write!(f, "per-wrapper"),
            SaranEnvScope::Global => write!(f, "global"),
            SaranEnvScope::Host => write!(f, "host"),
            SaranEnvScope::Default => write!(f, "default"),
        }
    }
}

/// A resolved variable with its value and source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaranEnvVar {
    /// The variable's value
    pub value: String,
    /// Where the value came from
    pub scope: SaranEnvScope,
}

impl SaranEnvVar {
    /// Create a new resolved variable.
    pub fn new(value: String, scope: SaranEnvScope) -> Self {
        SaranEnvVar { value, scope }
    }
}

/// Collection of resolved variables: name → value + scope.
pub type SaranEnv = HashMap<String, SaranEnvVar>;

/// The structure of `env.yaml` as loaded from disk.
#[derive(Debug, Clone, Default)]
pub struct SaranEnvYaml {
    /// Global variables: affect all wrappers
    pub global: HashMap<String, String>,
    /// Per-wrapper variables: `wrapper_name` → `var_name` → `value`
    pub wrappers: HashMap<String, HashMap<String, String>>,
}

impl SaranEnvYaml {
    /// Parse `env.yaml` from a YAML string.
    ///
    /// Returns empty sections if the document is empty or missing sections.
    pub fn from_yaml(yaml_str: &str) -> Result<Self, SaranEnvYamlError> {
        if yaml_str.trim().is_empty() {
            return Ok(SaranEnvYaml {
                global: HashMap::new(),
                wrappers: HashMap::new(),
            });
        }

        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .map_err(|e| SaranEnvYamlError::ParseError(e.to_string()))?;

        let mut global = HashMap::new();
        let mut wrappers: HashMap<String, HashMap<String, String>> = HashMap::new();

        // Parse global section
        if let Some(global_section) = value.get("global") {
            if let Some(map) = global_section.as_mapping() {
                for (k, v) in map {
                    let key = k.as_str().ok_or(SaranEnvYamlError::InvalidKey)?.to_string();
                    let val = v
                        .as_str()
                        .ok_or(SaranEnvYamlError::InvalidValue)?
                        .to_string();
                    global.insert(key, val);
                }
            }
        }

        // Parse wrappers section
        if let Some(wrappers_section) = value.get("wrappers") {
            if let Some(map) = wrappers_section.as_mapping() {
                for (k, v) in map {
                    let wrapper_name = k.as_str().ok_or(SaranEnvYamlError::InvalidKey)?.to_string();
                    let wrapper_vars = if let Some(wrapper_map) = v.as_mapping() {
                        let mut vars = HashMap::new();
                        for (var_k, var_v) in wrapper_map {
                            let var_name = var_k
                                .as_str()
                                .ok_or(SaranEnvYamlError::InvalidKey)?
                                .to_string();
                            let var_val = var_v
                                .as_str()
                                .ok_or(SaranEnvYamlError::InvalidValue)?
                                .to_string();
                            vars.insert(var_name, var_val);
                        }
                        vars
                    } else {
                        HashMap::new()
                    };
                    wrappers.insert(wrapper_name, wrapper_vars);
                }
            }
        }

        Ok(SaranEnvYaml { global, wrappers })
    }
}

/// Errors that can occur when parsing `env.yaml`.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SaranEnvYamlError {
    /// YAML parsing failed
    #[error("Failed to parse env.yaml: {0}")]
    ParseError(String),

    /// A key in the YAML structure is not a string
    #[error("Invalid key in env.yaml: expected string")]
    InvalidKey,

    /// A value in the YAML structure is not a string
    #[error("Invalid value in env.yaml: expected string")]
    InvalidValue,
}

/// Errors that can occur during variable resolution.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum VariableResolutionError {
    /// One or more required variables have no value
    #[error("Missing required variables: {}", .0.join(", "))]
    MissingRequired(Vec<String>),
}

/// Result of resolving variables: the resolved map and any missing required variables.
#[derive(Debug, Clone)]
pub struct VariableResolutionResult {
    /// Resolved variables: name → value + scope
    pub resolved: SaranEnv,
    /// Names of required variables that were not resolved
    pub missing_required: Vec<String>,
}

/// Resolve variables according to the priority chain.
///
/// # Arguments
///
/// * `var_decls` - Variable declarations from the wrapper's `vars:` section
/// * `env_yaml` - Parsed `env.yaml` structure
/// * `wrapper_name` - Name of the wrapper being executed
/// * `host_env` - Host environment variables (typically `std::env::vars().collect()`)
///
/// # Returns
///
/// A `VariableResolutionResult` containing resolved variables and missing required variables.
/// Even if some required variables are missing, returns the result with those listed in
/// `missing_required`. The caller should check this field and error if needed.
///
/// # Priority Chain
///
/// 1. Per-wrapper namespace in `env.yaml`
/// 2. Global namespace in `env.yaml`
/// 3. Host environment
/// 4. `default:` from wrapper `vars:` declaration
///
/// The first source that provides a non-empty value is used. If all sources provide no value:
/// - Required variable: added to `missing_required` list
/// - Optional variable: excluded from result entirely
///
/// # Examples
///
/// ```ignore
/// use saran_core::{resolve_vars, SaranEnvYaml};
/// use saran_types::VarDecl;
///
/// let var_decls = vec![
///     VarDecl { name: "GH_REPO".to_string(), required: false, default: Some("org/default".to_string()), help: None },
/// ];
/// let env_yaml = SaranEnvYaml::from_yaml("global:\n  GH_REPO: org/global\n")?;
/// let host_env = std::collections::HashMap::new();
///
/// let result = resolve_vars(&var_decls, &env_yaml, "my-wrapper", &host_env);
/// assert!(result.missing_required.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn resolve_vars(
    var_decls: &[saran_types::VarDecl],
    env_yaml: &SaranEnvYaml,
    wrapper_name: &str,
    host_env: &HashMap<String, String>,
) -> VariableResolutionResult {
    let mut resolved = SaranEnv::new();
    let mut missing_required = Vec::new();

    for var_decl in var_decls {
        let var_name = &var_decl.name;

        // Try to resolve from priority chain: per-wrapper → global → host → default
        if let Some(value) = env_yaml
            .wrappers
            .get(wrapper_name)
            .and_then(|vars| vars.get(var_name))
        {
            resolved.insert(
                var_name.clone(),
                SaranEnvVar::new(value.clone(), SaranEnvScope::PerWrapper),
            );
        } else if let Some(value) = env_yaml.global.get(var_name) {
            resolved.insert(
                var_name.clone(),
                SaranEnvVar::new(value.clone(), SaranEnvScope::Global),
            );
        } else if let Some(value) = host_env.get(var_name) {
            resolved.insert(
                var_name.clone(),
                SaranEnvVar::new(value.clone(), SaranEnvScope::Host),
            );
        } else if let Some(default_value) = &var_decl.default {
            resolved.insert(
                var_name.clone(),
                SaranEnvVar::new(default_value.clone(), SaranEnvScope::Default),
            );
        } else if var_decl.required {
            // Required variable with no value
            missing_required.push(var_name.clone());
        }
        // Optional variable with no value: exclude from result (already not inserted)
    }

    VariableResolutionResult {
        resolved,
        missing_required,
    }
}

// ============================================================================
// Phase 2: Substitution Resolution Types
// ============================================================================

/// Context for resolving variable references during substitution.
///
/// Contains both startup-resolved variables and invocation-time caller arguments,
/// allowing different contexts (help vs action) to resolve references appropriately.
#[derive(Debug, Clone)]
pub struct ResolutionContext {
    /// Variables resolved from env.yaml at startup time
    pub resolved_vars: HashMap<String, String>,
    /// Arguments provided by caller at invocation time
    pub caller_args: HashMap<String, String>,
}

impl ResolutionContext {
    /// Create a new resolution context.
    pub fn new(
        resolved_vars: HashMap<String, String>,
        caller_args: HashMap<String, String>,
    ) -> Self {
        ResolutionContext {
            resolved_vars,
            caller_args,
        }
    }
}

/// Error types for substitution resolution.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SubstitutionError {
    /// A referenced variable does not exist
    #[error("Undeclared variable: ${}", .0)]
    UndeclaredVariable(String),
}

/// Resolve `$VAR_NAME` tokens in a template string within the action context.
///
/// Action context allows references to both `vars:` names and `args` var_names.
/// If a variable reference cannot be resolved, returns an error.
///
/// # Arguments
///
/// * `parsed` - Result of parsing the template string with `parse_tokens()`
/// * `context` - Resolution context with resolved vars and caller args
///
/// # Returns
///
/// The resolved string with all `$VAR_NAME` tokens replaced by their values,
/// or `SubstitutionError` if any variable reference cannot be resolved.
///
/// # Behavior
///
/// - No recursive substitution: if `$FOO="BAR"`, substitutes `"BAR"` not `BAR`'s value
/// - Empty values substituted literally
/// - Whitespace preserved exactly
/// - Dollar signs in values are not re-parsed
///
/// # Examples
///
/// ```ignore
/// use saran_core::{resolve_substitution, ResolutionContext};
/// use saran_parser::parse_tokens;
///
/// let parsed = parse_tokens("Deploy to $GH_REPO").unwrap();
/// let mut vars = HashMap::new();
/// vars.insert("GH_REPO".to_string(), "org/repo".to_string());
///
/// let context = ResolutionContext::new(vars, HashMap::new());
/// let result = resolve_substitution(&parsed, &context).unwrap();
/// assert_eq!(result, "Deploy to org/repo");
/// ```
pub fn resolve_substitution(
    parsed: &ParsedTemplate,
    context: &ResolutionContext,
) -> Result<String, SubstitutionError> {
    let mut result = String::new();

    // Special case: no tokens at all
    // In this case, literals contains the entire string as a single literal
    if parsed.tokens.is_empty() {
        if !parsed.literals.is_empty() {
            result.push_str(&parsed.literals[0].text);
        }
        return Ok(result);
    }

    // Structure of ParsedTemplate:
    // - literals[0] (if exists): text before first token (before=true)
    // - token[0]
    // - literals[1] (if exists): text after token[0] (before=false)
    // - token[1]
    // - ...
    // - literals[n]: text after last token (before=false)

    // Add the "before" literal if present
    if !parsed.literals.is_empty() && parsed.literals[0].before {
        result.push_str(&parsed.literals[0].text);
    }

    // Process tokens, adding their resolved values and interleaved literals
    for (i, token) in parsed.tokens.iter().enumerate() {
        // Resolve the token
        let value = context
            .resolved_vars
            .get(&token.var_name)
            .or_else(|| context.caller_args.get(&token.var_name))
            .ok_or_else(|| SubstitutionError::UndeclaredVariable(token.var_name.clone()))?;

        result.push_str(value);

        // Add the literal that follows this token (if it exists)
        // The literal after token[i] is at literals[i+1] if "before" was already used
        if !parsed.literals.is_empty() {
            // Literals are indexed: [0] = before, [1] = after token[0], [2] = after token[1], etc.
            let literal_idx = i + 1;
            if literal_idx < parsed.literals.len() {
                result.push_str(&parsed.literals[literal_idx].text);
            }
        }
    }

    Ok(result)
}

/// Resolve `$VAR_NAME` tokens in help text, tolerating missing values.
///
/// Help context allows references only to `vars:` names.
/// If a variable reference cannot be resolved, the literal `$VAR_NAME` is shown
/// instead of returning an error (per spec requirement).
///
/// # Arguments
///
/// * `parsed` - Result of parsing the help text with `parse_tokens()`
/// * `resolved_vars` - Variables resolved from env.yaml at startup
///
/// # Returns
///
/// The help text with resolved `$VAR_NAME` tokens replaced by their values,
/// or unresolved references shown literally.
///
/// # Behavior
///
/// - Unresolved references shown as literal `$VAR_NAME` (no error)
/// - No recursive substitution
/// - Empty values substituted literally
/// - Whitespace preserved exactly
///
/// # Examples
///
/// ```ignore
/// use saran_core::resolve_help_text;
/// use saran_parser::parse_tokens;
///
/// let parsed = parse_tokens("Operations for $GH_REPO").unwrap();
/// let mut vars = HashMap::new();
/// vars.insert("GH_REPO".to_string(), "org/repo".to_string());
///
/// let result = resolve_help_text(&parsed, &vars);
/// assert_eq!(result, "Operations for org/repo");
/// ```
pub fn resolve_help_text(
    parsed: &ParsedTemplate,
    resolved_vars: &HashMap<String, String>,
) -> String {
    let mut result = String::new();

    // Special case: no tokens at all
    // In this case, literals contains the entire string as a single literal
    if parsed.tokens.is_empty() {
        if !parsed.literals.is_empty() {
            result.push_str(&parsed.literals[0].text);
        }
        return result;
    }

    // Structure of ParsedTemplate:
    // - literals[0] (if exists): text before first token (before=true)
    // - token[0]
    // - literals[1] (if exists): text after token[0] (before=false)
    // - token[1]
    // - ...
    // - literals[n]: text after last token (before=false)

    // Add the "before" literal if present
    if !parsed.literals.is_empty() && parsed.literals[0].before {
        result.push_str(&parsed.literals[0].text);
    }

    // Process tokens, adding their resolved values and interleaved literals
    for (i, token) in parsed.tokens.iter().enumerate() {
        // Resolve the token, or show literal if not found
        if let Some(value) = resolved_vars.get(&token.var_name) {
            result.push_str(value);
        } else {
            // Show literal $VAR_NAME instead of erroring
            result.push('$');
            result.push_str(&token.var_name);
        }

        // Add the literal that follows this token (if it exists)
        // The literal after token[i] is at literals[i+1] if "before" was already used
        if !parsed.literals.is_empty() {
            // Literals are indexed: [0] = before, [1] = after token[0], [2] = after token[1], etc.
            let literal_idx = i + 1;
            if literal_idx < parsed.literals.len() {
                result.push_str(&parsed.literals[literal_idx].text);
            }
        }
    }

    result
}

// ============================================================================
// Phase 3: Argument Assembly Types
// ============================================================================

/// Representation of a single optional flag value provided by the caller.
///
/// Optional flags can have multiple values if `repeated: true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalFlagValue {
    /// A string value (including int and enum values as strings)
    String(String),
    /// Multiple string values (for repeated flags)
    Multiple(Vec<String>),
    /// A boolean flag (type: bool, no value)
    Bool,
}

impl OptionalFlagValue {
    /// Create a single string value
    pub fn string(value: String) -> Self {
        OptionalFlagValue::String(value)
    }

    /// Create multiple string values
    pub fn multiple(values: Vec<String>) -> Self {
        OptionalFlagValue::Multiple(values)
    }

    /// Create a boolean flag
    pub fn bool() -> Self {
        OptionalFlagValue::Bool
    }
}

/// Context for assembling command-line arguments.
///
/// Contains all the information needed to build an argv array:
/// - resolved variables (from env.yaml)
/// - positional argument values (from caller)
/// - optional flag values (from caller)
#[derive(Debug, Clone)]
pub struct AssemblyContext {
    /// Variables resolved from env.yaml at startup
    pub resolved_vars: HashMap<String, String>,
    /// Positional argument values (var_name -> value) provided by caller
    pub caller_args: HashMap<String, String>,
    /// Optional flag values (flag name -> value(s)) provided by caller
    pub optional_flags: HashMap<String, OptionalFlagValue>,
}

impl AssemblyContext {
    /// Create a new assembly context.
    pub fn new(
        resolved_vars: HashMap<String, String>,
        caller_args: HashMap<String, String>,
        optional_flags: HashMap<String, OptionalFlagValue>,
    ) -> Self {
        AssemblyContext {
            resolved_vars,
            caller_args,
            optional_flags,
        }
    }
}

/// Errors that can occur during argument assembly.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ArgvAssemblyError {
    /// A variable referenced in action args could not be resolved
    #[error("Cannot resolve variable in action arguments: ${}", .0)]
    UnresolvedVariable(String),
}

impl From<SubstitutionError> for ArgvAssemblyError {
    fn from(err: SubstitutionError) -> Self {
        match err {
            SubstitutionError::UndeclaredVariable(var_name) => {
                ArgvAssemblyError::UnresolvedVariable(var_name)
            }
        }
    }
}

/// Build the argv array for a child process command.
///
/// Assembles the complete argument list from:
/// - The action's executable name
/// - The action's fixed arguments (with variable substitution)
/// - Optional flags appended in declaration order
///
/// # Arguments
///
/// * `executable` - The executable name (e.g., "gh", "redis-cli")
/// * `action_args` - Fixed arguments from the action definition (may contain `$VAR_NAME` references)
/// * `optional_flags` - Definitions of optional flags (with declaration order)
/// * `context` - Assembly context with resolved variables and caller-provided values
/// * `flag_name_to_passes_as` - Map from flag name to its `passes_as` override (if any)
///
/// # Returns
///
/// The complete argv array suitable for `std::process::Command::new()`,
/// or `ArgvAssemblyError` if variable substitution fails.
///
/// # Behavior
///
/// 1. Adds the executable name as argv[0]
/// 2. Adds action args (with variable substitution applied)
/// 3. Appends optional flags in declaration order:
///    - Str/int/enum non-repeated flags: `[flag_name_or_passes_as, value]`
///    - Str/int/enum repeated flags: `[flag_name_or_passes_as, value1, flag_name_or_passes_as, value2]`
///    - Bool flags: `[flag_name_or_passes_as]` (no value)
/// 4. Omits optional flags not provided by caller
///
/// # Examples
///
/// ```ignore
/// use saran_core::{build_argv, AssemblyContext, OptionalFlagValue};
/// use saran_parser::parse_tokens;
/// use std::collections::HashMap;
///
/// let mut resolved_vars = HashMap::new();
/// resolved_vars.insert("PR_NUM".to_string(), "123".to_string());
///
/// let action_args = vec!["pr".to_string(), "view".to_string(), "$PR_NUM".to_string()];
/// let context = AssemblyContext::new(resolved_vars, HashMap::new(), HashMap::new());
///
/// let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();
/// assert_eq!(argv, vec!["gh", "pr", "view", "123"]);
/// ```
pub fn build_argv(
    executable: &str,
    action_args: &[String],
    optional_flags: &[saran_types::OptionalFlag],
    context: &AssemblyContext,
    flag_name_to_passes_as: &HashMap<String, String>,
) -> Result<Vec<String>, ArgvAssemblyError> {
    let mut argv = vec![executable.to_string()];

    // Process fixed action arguments with variable substitution
    for arg in action_args {
        let parsed = parse_tokens(arg)
            .map_err(|_| ArgvAssemblyError::UnresolvedVariable("parse error".to_string()))?;

        let resolved_arg = resolve_substitution(
            &parsed,
            &ResolutionContext::new(context.resolved_vars.clone(), context.caller_args.clone()),
        )?;

        argv.push(resolved_arg);
    }

    // Append optional flags in declaration order
    for flag in optional_flags {
        if let Some(flag_value) = context.optional_flags.get(&flag.name) {
            let flag_name = flag_name_to_passes_as
                .get(&flag.name)
                .map(|s| s.as_str())
                .unwrap_or(&flag.name);

            match flag_value {
                OptionalFlagValue::Bool => {
                    // Bool flags have no value, just the flag name
                    argv.push(flag_name.to_string());
                }
                OptionalFlagValue::String(value) => {
                    // Non-repeated str/int/enum: [flag_name, value]
                    argv.push(flag_name.to_string());
                    argv.push(value.clone());
                }
                OptionalFlagValue::Multiple(values) => {
                    // Repeated str/int/enum: [flag_name, value1, flag_name, value2, ...]
                    for value in values {
                        argv.push(flag_name.to_string());
                        argv.push(value.clone());
                    }
                }
            }
        }
    }

    Ok(argv)
}

#[cfg(test)]
mod tests;
