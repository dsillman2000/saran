//! Token parsing for Saran wrapper templates.
//!
//! This module provides pure string parsing to extract `$VAR_NAME` tokens from template
//! strings. It is responsible for identifying variable references and preserving literal text.
//!
//! # Token Matching Rules
//!
//! - Pattern: `$` followed by `[A-Za-z_][A-Za-z0-9_]*`
//! - Greedy matching: the longest valid identifier is always taken
//! - No escape mechanism: bare `$` followed by invalid characters produces an error
//! - No brace syntax: `${VAR}` is not supported in v1
//!
//! # Example
//!
//! ```ignore
//! use saran_parser::parse_tokens;
//!
//! let result = parse_tokens("prefix-$VAR_NAME-suffix")?;
//! assert_eq!(result.tokens.len(), 1);
//! assert_eq!(result.tokens[0].var_name, "VAR_NAME");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use regex::Regex;
use semver::{Version, VersionReq};
use std::fmt;
use thiserror::Error;

/// A parsed variable reference: `$VAR_NAME`.
///
/// Stores the variable name and its position within the source string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The variable name (without the `$` prefix).
    pub var_name: String,
    /// Starting byte position in the source string (inclusive).
    pub start: usize,
    /// Ending byte position in the source string (exclusive).
    pub end: usize,
}

impl Token {
    /// Create a new token with the given variable name and position.
    pub fn new(var_name: String, start: usize, end: usize) -> Self {
        Token {
            var_name,
            start,
            end,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${} [{}:{}]", self.var_name, self.start, self.end)
    }
}

/// A literal text segment from a parsed template.
///
/// Represents the text between and around variable tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The literal text content.
    pub text: String,
    /// True if this literal appears before any tokens (or at the start).
    pub before: bool,
}

/// Result of parsing a template string for variable tokens.
///
/// Contains all tokens found and the literal text segments that remain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTemplate {
    /// All `$VAR_NAME` tokens found in the string, in order.
    pub tokens: Vec<Token>,
    /// Literal text segments (between and around tokens).
    pub literals: Vec<Literal>,
}

/// Errors that can occur during token parsing.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TokenParsingError {
    /// A `$` was followed by an invalid character for starting a variable name.
    ///
    /// Variable names must start with `[A-Za-z_]`, not digits or special characters.
    #[error("Invalid character after '$' at position {position}: '{char}' is not a valid variable name start")]
    InvalidCharAfterDollar {
        /// Byte position in the source string where the `$` was found.
        position: usize,
        /// The character that followed the `$`.
        char: char,
    },

    /// A `$` appeared at the end of the string with no following character.
    #[error("Incomplete variable reference: '$' at end of string (position {position})")]
    IncompleteVariableReference {
        /// Byte position where the `$` was found.
        position: usize,
    },
}

/// Parse `$VAR_NAME` tokens from a template string.
///
/// Extracts all variable references matching the pattern `$[A-Za-z_][A-Za-z0-9_]*` from
/// the input string. Uses greedy matching: the longest valid identifier is always taken.
///
/// # Arguments
///
/// * `input` - The template string to parse.
///
/// # Returns
///
/// * `Ok(ParsedTemplate)` - Successfully parsed tokens and literals.
/// * `Err(TokenParsingError)` - If the string contains invalid token syntax.
///
/// # Examples
///
/// Basic variable reference:
/// ```ignore
/// let result = parse_tokens("$GH_REPO")?;
/// assert_eq!(result.tokens[0].var_name, "GH_REPO");
/// ```
///
/// Multiple tokens:
/// ```ignore
/// let result = parse_tokens("$FOO$BAR")?;
/// assert_eq!(result.tokens.len(), 2);
/// assert_eq!(result.tokens[0].var_name, "FOO");
/// assert_eq!(result.tokens[1].var_name, "BAR");
/// ```
///
/// Mixed literals and variables:
/// ```ignore
/// let result = parse_tokens("prefix-$VAR-suffix")?;
/// assert_eq!(result.tokens[0].var_name, "VAR");
/// // Literals preserve the prefix and suffix text
/// ```
pub fn parse_tokens(input: &str) -> Result<ParsedTemplate, TokenParsingError> {
    let regex = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").expect("regex pattern is valid");

    let mut tokens = Vec::new();
    let mut literals = Vec::new();
    let mut last_end = 0;

    // Find all regex matches (valid tokens)
    for cap in regex.captures_iter(input) {
        let full_match = cap.get(0).expect("captures include full match");
        let start = full_match.start();
        let end = full_match.end();
        let var_name = cap.get(1).expect("captures include group 1").as_str();

        // Preserve literal text before this token
        if start > last_end {
            let before_text = input[last_end..start].to_string();
            literals.push(Literal {
                text: before_text,
                before: literals.is_empty(),
            });
        }

        tokens.push(Token {
            var_name: var_name.to_string(),
            start,
            end,
        });

        last_end = end;
    }

    // Check for invalid `$` characters (not matching the regex pattern)
    for (idx, ch) in input.chars().enumerate() {
        if ch == '$' {
            // Check if this `$` is part of a valid token
            let is_valid_token = tokens.iter().any(|t| t.start <= idx && idx < t.end);

            if !is_valid_token {
                // This `$` is not part of any valid token, so it's an error
                if idx + 1 < input.len() {
                    // There's a character after the `$`
                    let next_char = input[idx + 1..].chars().next().expect("char exists");
                    return Err(TokenParsingError::InvalidCharAfterDollar {
                        position: idx,
                        char: next_char,
                    });
                } else {
                    // `$` is at the end of the string
                    return Err(TokenParsingError::IncompleteVariableReference { position: idx });
                }
            }
        }
    }

    // Preserve remaining literal text after the last token
    if last_end < input.len() {
        let after_text = input[last_end..].to_string();
        literals.push(Literal {
            text: after_text,
            before: false,
        });
    } else if tokens.is_empty() && input.is_empty() {
        // Empty input: one empty literal
        literals.push(Literal {
            text: String::new(),
            before: true,
        });
    }

    Ok(ParsedTemplate { tokens, literals })
}

/// Structured error reporting for YAML wrapper validation.
///
/// Each variant includes the path to the problematic field (e.g., `commands.pr.optional_flags[0].name`)
/// to help users locate and fix issues in their wrapper YAML files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A required field is missing from the YAML structure.
    MissingField {
        /// Path to the missing field (e.g., `"name"`, `"commands.pr.actions"`)
        path: String,
        /// Name of the missing field
        field: &'static str,
    },

    /// A field value does not conform to the expected format.
    InvalidFormat {
        /// Path to the field with invalid format
        path: String,
        /// Explanation of what format is expected and why the value is invalid
        reason: String,
    },

    /// A field contains a value that is invalid for its context.
    InvalidValue {
        /// Path to the invalid value
        path: String,
        /// What was expected
        expected: String,
        /// What was actually found
        found: String,
    },

    /// A key appears multiple times in a map where uniqueness is required.
    DuplicateKey {
        /// Path to the duplicate key
        path: String,
        /// The key that was duplicated
        key: String,
    },

    /// Two fields that should be mutually exclusive are both set.
    ConflictingFields {
        /// Path to the conflicting section
        path: String,
        /// First conflicting field
        field1: &'static str,
        /// Second conflicting field
        field2: &'static str,
    },

    /// A variable or command reference appears but is not declared.
    UndeclaredReference {
        /// Path to the reference
        path: String,
        /// The name of the undeclared variable/command
        var_name: String,
    },

    /// A SemVer string failed to parse.
    SemverParseError {
        /// The invalid SemVer string
        value: String,
        /// Error description
        error: String,
    },

    /// A regex pattern failed to compile.
    RegexCompileError {
        /// The invalid regex pattern
        pattern: String,
        /// Error description
        error: String,
    },

    /// A validation rule was violated (e.g., required arg after optional arg).
    CrossReferenceViolation {
        /// Path to the problematic section
        path: String,
        /// Description of what rule was violated
        reason: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField { path, field } => {
                write!(f, "Missing required field '{}' at '{}'", field, path)
            }
            ValidationError::InvalidFormat { path, reason } => {
                write!(f, "Invalid format at '{}': {}", path, reason)
            }
            ValidationError::InvalidValue {
                path,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Invalid value at '{}': expected {}, found {}",
                    path, expected, found
                )
            }
            ValidationError::DuplicateKey { path, key } => {
                write!(f, "Duplicate key '{}' at '{}'", key, path)
            }
            ValidationError::ConflictingFields {
                path,
                field1,
                field2,
            } => {
                write!(
                    f,
                    "Conflicting fields '{}' and '{}' at '{}': must be mutually exclusive",
                    field1, field2, path
                )
            }
            ValidationError::UndeclaredReference { path, var_name } => {
                write!(f, "Undeclared reference '${}' at '{}'", var_name, path)
            }
            ValidationError::SemverParseError { value, error } => {
                write!(f, "Invalid SemVer '{}': {}", value, error)
            }
            ValidationError::RegexCompileError { pattern, error } => {
                write!(f, "Invalid regex '{}': {}", pattern, error)
            }
            ValidationError::CrossReferenceViolation { path, reason } => {
                write!(f, "Validation violation at '{}': {}", path, reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Context for collecting validation errors and tracking declared names.
///
/// Used during the validation pipeline to collect all errors at once
/// (rather than failing on first error) and to track which variables,
/// commands, and arguments have been declared.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Set of declared variable names (from `vars:` section)
    pub declared_vars: std::collections::HashSet<String>,
    /// Set of declared command names (from `commands:` section)
    pub declared_commands: std::collections::HashSet<String>,
    /// Map of declared argument var_names per command
    pub declared_args: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// Collected validation errors
    pub errors: Vec<ValidationError>,
}

impl ValidationContext {
    /// Create a new empty validation context.
    pub fn new() -> Self {
        ValidationContext {
            declared_vars: std::collections::HashSet::new(),
            declared_commands: std::collections::HashSet::new(),
            declared_args: std::collections::HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Add a validation error to the context.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Check if any errors have been collected.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Consume the context and return collected errors, or Ok if none.
    pub fn collect_errors(self) -> Result<(), Vec<ValidationError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Format Validation Helpers
// ============================================================================

/// Validate that a variable name matches the required format: `[A-Za-z_][A-Za-z0-9_]*`
///
/// # Arguments
/// * `name` - The variable name to validate
///
/// # Returns
/// * `Ok(())` if the name is valid
/// * `Err(String)` with a description of the format problem otherwise
pub fn validate_var_name_format(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("variable name cannot be empty".to_string());
    }

    // Check first character: must be letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return Err(format!(
            "variable name must start with a letter or underscore, found '{}'",
            first_char
        ));
    }

    // Check remaining characters: must be alphanumeric or underscore
    for ch in name.chars().skip(1) {
        if !ch.is_alphanumeric() && ch != '_' {
            return Err(format!(
                "variable name contains invalid character '{}': only alphanumeric and underscore allowed",
                ch
            ));
        }
    }

    Ok(())
}

/// Validate that a command name matches the required format.
///
/// Valid format: alphanumeric characters and hyphens, must not start or end with hyphen.
///
/// # Arguments
/// * `name` - The command name to validate
///
/// # Returns
/// * `Ok(())` if valid, `Err(String)` otherwise
pub fn validate_command_name_format(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("command name cannot be empty".to_string());
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err("command name cannot start or end with hyphen".to_string());
    }

    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != '-' {
            return Err(format!(
                "command name contains invalid character '{}': only alphanumeric and hyphen allowed",
                ch
            ));
        }
    }

    Ok(())
}

/// Validate that a flag name matches the required format.
///
/// Valid format: starts with `--`, followed by alphanumeric characters and hyphens.
///
/// # Arguments
/// * `name` - The flag name to validate
///
/// # Returns
/// * `Ok(())` if valid, `Err(String)` otherwise
pub fn validate_flag_name_format(name: &str) -> Result<(), String> {
    if !name.starts_with("--") {
        return Err("flag name must start with '--'".to_string());
    }

    let suffix = &name[2..];
    if suffix.is_empty() {
        return Err("flag name cannot be only '--'".to_string());
    }

    // First character after '--' must be alphanumeric
    let first_char = suffix.chars().next().unwrap();
    if !first_char.is_alphanumeric() {
        return Err(format!(
            "flag name must start with '--' followed by alphanumeric character, found '{}'",
            first_char
        ));
    }

    // Remaining characters: alphanumeric and hyphen
    for ch in suffix.chars().skip(1) {
        if !ch.is_alphanumeric() && ch != '-' {
            return Err(format!(
                "flag name contains invalid character '{}': only alphanumeric and hyphen allowed",
                ch
            ));
        }
    }

    Ok(())
}

/// Validate that a version string conforms to SemVer 2.0.0.
///
/// Uses the `semver` crate for robust parsing that handles all edge cases
/// including prerelease identifiers, metadata, and proper format validation.
///
/// # Arguments
/// * `version` - The version string to validate
///
/// # Returns
/// * `Ok(())` if valid SemVer, `Err(String)` otherwise
///
/// # Examples
///
/// ```ignore
/// assert!(validate_semver("1.0.0").is_ok());
/// assert!(validate_semver("1.0.0-beta.1").is_ok());
/// assert!(validate_semver("1.0.0+build.123").is_ok());
/// assert!(validate_semver("not-a-version").is_err());
/// ```
pub fn validate_semver(version: &str) -> Result<(), String> {
    Version::parse(version).map_err(|e| e.to_string())?;
    Ok(())
}

/// Validate a SemVer version constraint string (e.g., `>=1.0.0, <2.0.0`).
///
/// Uses the `semver` crate's `VersionReq` for constraint parsing.
/// Supports the standard Cargo-style version requirement format with operators:
/// `=`, `>`, `>=`, `<`, `<=`, `~`, `^`, `*`, and comma-separated comparisons.
///
/// # Arguments
/// * `constraint` - The constraint string to validate
///
/// # Returns
/// * `Ok(())` if valid, `Err(String)` otherwise
///
/// # Examples
///
/// ```ignore
/// assert!(validate_semver_constraint(">=1.0.0").is_ok());
/// assert!(validate_semver_constraint(">=1.0.0, <2.0.0").is_ok());
/// assert!(validate_semver_constraint("~1.2.3").is_ok());
/// assert!(validate_semver_constraint("invalid").is_err());
/// ```
pub fn validate_semver_constraint(constraint: &str) -> Result<(), String> {
    if constraint.is_empty() {
        return Err("version constraint cannot be empty".to_string());
    }

    VersionReq::parse(constraint).map_err(|e| e.to_string())?;
    Ok(())
}

/// Validate that a regex pattern compiles successfully and contains exactly one capture group.
///
/// # Arguments
/// * `pattern` - The regex pattern to validate
///
/// # Returns
/// * `Ok(())` if valid and has one capture group, `Err(String)` otherwise
pub fn validate_regex(pattern: &str) -> Result<(), String> {
    match Regex::new(pattern) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("regex compilation failed: {}", e)),
    }
}

/// Validate that a regex pattern has exactly one capture group.
///
/// # Arguments
/// * `pattern` - The regex pattern to check
///
/// # Returns
/// * `Ok(())` if the pattern has exactly one capture group, `Err(String)` otherwise
pub fn validate_regex_capture_groups(pattern: &str) -> Result<(), String> {
    match Regex::new(pattern) {
        Ok(re) => {
            let num_captures = re.captures_len();
            if num_captures != 2 {
                // captures_len() includes the full match (group 0), so 2 means 1 capture group
                Err(format!(
                    "regex must have exactly 1 capture group, found {}",
                    num_captures - 1
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("regex compilation failed: {}", e)),
    }
}

/// Check for prefix conflicts in a list of names.
///
/// Returns pairs of names where one is a prefix of the other (e.g., `VAR` and `VAR_SUFFIX`).
/// This is a HARD ERROR in variable names and argument var_names.
///
/// # Arguments
/// * `names` - List of names to check
///
/// # Returns
/// Vector of (name1, name2) pairs where name1 is a prefix of name2
pub fn check_prefix_conflicts(names: &[String]) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();

    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            let a = &names[i];
            let b = &names[j];

            // Check if one is a prefix of the other (requires separator after prefix)
            if a.len() < b.len() && b.starts_with(a) && b.chars().nth(a.len()) == Some('_') {
                conflicts.push((a.clone(), b.clone()));
            } else if b.len() < a.len() && a.starts_with(b) && a.chars().nth(b.len()) == Some('_') {
                conflicts.push((b.clone(), a.clone()));
            }
        }
    }

    conflicts
}

// ============================================================================
// Phase 2B: Context-Aware Validation
// ============================================================================

/// Validate all variable declarations in the `vars:` section.
///
/// Checks:
/// - Each var name matches format [A-Za-z_][A-Za-z0-9_]*
/// - No duplicate var names
/// - required and default are mutually exclusive
/// - Neither required nor default is specified (ambiguous optionality)
/// - No prefix conflicts (e.g., VAR and VAR_SUFFIX)
///
/// Updates context.errors and populates context.declared_vars
pub fn validate_vars(vars: &[saran_types::VarDecl], context: &mut ValidationContext) {
    let mut var_names = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for (index, var) in vars.iter().enumerate() {
        // Check for duplicates before validating
        if seen_names.contains(&var.name) {
            context.add_error(ValidationError::DuplicateKey {
                path: "vars".to_string(),
                key: var.name.clone(),
            });
        }
        seen_names.insert(var.name.clone());

        validate_var_declaration(var, index, context);
        var_names.push(var.name.clone());
    }

    // Check for prefix conflicts
    let conflicts = check_prefix_conflicts(&var_names);
    for (prefix, longer) in conflicts {
        context.add_error(ValidationError::CrossReferenceViolation {
            path: "vars".to_string(),
            reason: format!("variable name '{}' is a prefix of '{}'", prefix, longer),
        });
    }
}

/// Validate a single variable declaration.
fn validate_var_declaration(
    var: &saran_types::VarDecl,
    index: usize,
    context: &mut ValidationContext,
) {
    let path = format!("vars[{}]", index);

    // Check name format
    if let Err(e) = validate_var_name_format(&var.name) {
        context.add_error(ValidationError::InvalidFormat {
            path: format!("{}.name", path),
            reason: e,
        });
    }

    // Check required and default are mutually exclusive
    if var.required && var.default.is_some() {
        context.add_error(ValidationError::ConflictingFields {
            path: path.clone(),
            field1: "required",
            field2: "default",
        });
    }

    // Check that either required or default is set (not both, not neither)
    if !var.required && var.default.is_none() {
        context.add_error(ValidationError::CrossReferenceViolation {
            path: path.clone(),
            reason: "variable must either have required: true or a default value (ambiguous optionality)"
                .to_string(),
        });
    }

    // Add to declared vars if no format errors
    if validate_var_name_format(&var.name).is_ok() {
        context.declared_vars.insert(var.name.clone());
    }
}

/// Validate all commands in the `commands:` section.
///
/// Checks:
/// - Each command name is a valid format (alphanumeric and hyphens, no leading/trailing)
/// - Each command has `actions` section that is non-empty
/// - All actions have valid structure
///
/// Updates context.errors and populates context.declared_commands
pub fn validate_commands(
    commands: &std::collections::BTreeMap<String, saran_types::Command>,
    context: &mut ValidationContext,
) {
    // Commands are part of deserialization, so BTreeMap iteration is safe
    for (cmd_name, command) in commands {
        // Validate command name format
        if let Err(e) = validate_command_name_format(cmd_name) {
            context.add_error(ValidationError::InvalidFormat {
                path: format!("commands.{}", cmd_name),
                reason: e,
            });
            continue;
        }

        // Check that actions exist and are non-empty
        if command.actions.is_empty() {
            context.add_error(ValidationError::MissingField {
                path: format!("commands.{}", cmd_name),
                field: "actions",
            });
            continue;
        }

        // Validate each action in this command
        for (action_index, action) in command.actions.iter().enumerate() {
            validate_action_structure(action, cmd_name, action_index, context);
        }

        // Add to declared commands if no errors
        if validate_command_name_format(cmd_name).is_ok() && !command.actions.is_empty() {
            context.declared_commands.insert(cmd_name.clone());
        }
    }
}

/// Validate a single action's structure.
fn validate_action_structure(
    action: &saran_types::Action,
    command_name: &str,
    action_index: usize,
    context: &mut ValidationContext,
) {
    let path = format!("commands.{}.actions[{}]", command_name, action_index);

    // Validate executable name (no absolute paths, no special chars)
    if let Err(e) = validate_executable_name(&action.executable) {
        context.add_error(ValidationError::InvalidFormat {
            path: format!("{}.executable", path),
            reason: e,
        });
    }
}

/// Validate an executable name (no absolute paths, no special chars).
fn validate_executable_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("executable name cannot be empty".to_string());
    }

    // Check for absolute path
    if name.starts_with('/') {
        return Err("executable name cannot be an absolute path".to_string());
    }

    // Check for parent directory reference
    if name.contains("..") {
        return Err("executable name cannot contain parent directory reference (..)".to_string());
    }

    // Check for path separators
    if name.contains('/') {
        return Err("executable name cannot contain path separators".to_string());
    }

    Ok(())
}

/// Validate all optional flags in all actions across all commands.
pub fn validate_optional_flags_all(
    commands: &std::collections::BTreeMap<String, saran_types::Command>,
    context: &mut ValidationContext,
) {
    for (cmd_name, command) in commands {
        for (action_index, action) in command.actions.iter().enumerate() {
            validate_optional_flags_in_action(
                cmd_name,
                action_index,
                &action.optional_flags,
                context,
            );
        }
    }
}

/// Validate all optional flags in a single action.
fn validate_optional_flags_in_action(
    command_name: &str,
    action_index: usize,
    flags: &[saran_types::OptionalFlag],
    context: &mut ValidationContext,
) {
    let mut flag_names = std::collections::HashSet::new();

    for (flag_index, flag) in flags.iter().enumerate() {
        let path = format!(
            "commands.{}.actions[{}].optional_flags[{}]",
            command_name, action_index, flag_index
        );

        // Check that name and type are present
        if flag.name.is_empty() {
            context.add_error(ValidationError::MissingField {
                path: path.clone(),
                field: "name",
            });
        }

        if flag.flag_type.is_empty() {
            context.add_error(ValidationError::MissingField {
                path: path.clone(),
                field: "flag_type",
            });
        }

        // Validate flag name format
        if !flag.name.is_empty() {
            if let Err(e) = validate_flag_name_format(&flag.name) {
                context.add_error(ValidationError::InvalidFormat {
                    path: format!("{}.name", path),
                    reason: e,
                });
            } else {
                // Check for duplicate flag names
                if flag_names.contains(&flag.name) {
                    context.add_error(ValidationError::DuplicateKey {
                        path: format!(
                            "commands.{}.actions[{}].optional_flags",
                            command_name, action_index
                        ),
                        key: flag.name.clone(),
                    });
                }
                flag_names.insert(flag.name.clone());
            }
        }

        // Validate flag type and related constraints
        if !flag.flag_type.is_empty() {
            match flag.flag_type.as_str() {
                "str" | "int" | "bool" | "enum" => {
                    // Valid types
                    // Check enum-specific constraints
                    if flag.flag_type == "enum" {
                        if flag.values.is_empty() {
                            context.add_error(ValidationError::MissingField {
                                path: format!("{}.values", path),
                                field: "values",
                            });
                        } else {
                            // Validate enum values
                            for (val_index, val) in flag.values.iter().enumerate() {
                                if let Err(e) = validate_enum_value(val) {
                                    context.add_error(ValidationError::InvalidFormat {
                                        path: format!("{}.values[{}]", path, val_index),
                                        reason: e,
                                    });
                                }
                            }
                        }
                    }

                    // Check bool/repeated mutual exclusivity
                    if flag.flag_type == "bool" && flag.repeated {
                        context.add_error(ValidationError::ConflictingFields {
                            path: path.clone(),
                            field1: "type",
                            field2: "repeated",
                        });
                    }
                }
                _ => {
                    context.add_error(ValidationError::InvalidValue {
                        path: format!("{}.flag_type", path),
                        expected: "one of: str, int, bool, enum".to_string(),
                        found: flag.flag_type.clone(),
                    });
                }
            }
        }

        // Validate passes_as (must not contain `=`)
        if let Some(passes_as) = &flag.passes_as {
            if passes_as.contains('=') {
                context.add_error(ValidationError::InvalidFormat {
                    path: format!("{}.passes_as", path),
                    reason: "passes_as cannot contain '=' character".to_string(),
                });
            }
        }
    }
}

/// Validate an enum value format: [a-z0-9][a-z0-9_-]*
fn validate_enum_value(val: &str) -> Result<(), String> {
    if val.is_empty() {
        return Err("enum value cannot be empty".to_string());
    }

    // First char must be alphanumeric
    let first = val.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return Err(format!(
            "enum value must start with alphanumeric character, found '{}'",
            first
        ));
    }

    // Remaining chars must be alphanumeric, underscore, or hyphen
    for ch in val.chars().skip(1) {
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' {
            return Err(format!(
                "enum value contains invalid character '{}': only alphanumeric, underscore, and hyphen allowed",
                ch
            ));
        }
    }

    Ok(())
}

/// Validate all positional arguments in all commands.
pub fn validate_args_all(
    commands: &std::collections::BTreeMap<String, saran_types::Command>,
    context: &mut ValidationContext,
) {
    for (cmd_name, command) in commands {
        validate_args(cmd_name, &command.args, context);
    }
}

/// Validate all positional arguments in a command.
fn validate_args(
    command_name: &str,
    args: &[saran_types::PositionalArg],
    context: &mut ValidationContext,
) {
    let mut arg_names = std::collections::HashSet::new();
    let mut arg_var_names = Vec::new();
    let mut seen_arg_var_names = std::collections::HashSet::new();

    for (index, arg) in args.iter().enumerate() {
        let path = format!("commands.{}.args[{}]", command_name, index);

        // Check required fields
        if arg.name.is_empty() {
            context.add_error(ValidationError::MissingField {
                path: path.clone(),
                field: "name",
            });
        }

        if arg.var_name.is_empty() {
            context.add_error(ValidationError::MissingField {
                path: path.clone(),
                field: "var_name",
            });
        }

        if arg.arg_type.is_empty() || arg.arg_type != "str" {
            context.add_error(ValidationError::InvalidValue {
                path: format!("{}.arg_type", path),
                expected: "str".to_string(),
                found: arg.arg_type.clone(),
            });
        }

        // Validate arg name format
        if !arg.name.is_empty() {
            if let Err(e) = validate_arg_name_format(&arg.name) {
                context.add_error(ValidationError::InvalidFormat {
                    path: format!("{}.name", path),
                    reason: e,
                });
            } else {
                // Check for duplicate arg names
                if arg_names.contains(&arg.name) {
                    context.add_error(ValidationError::DuplicateKey {
                        path: format!("commands.{}.args", command_name),
                        key: arg.name.clone(),
                    });
                }
                arg_names.insert(arg.name.clone());
            }
        }

        // Validate var_name format
        if !arg.var_name.is_empty() {
            if let Err(e) = validate_var_name_format(&arg.var_name) {
                context.add_error(ValidationError::InvalidFormat {
                    path: format!("{}.var_name", path),
                    reason: e,
                });
            } else {
                // Check for duplicate var_names
                if seen_arg_var_names.contains(&arg.var_name) {
                    context.add_error(ValidationError::DuplicateKey {
                        path: format!("commands.{}.args", command_name),
                        key: arg.var_name.clone(),
                    });
                }
                seen_arg_var_names.insert(arg.var_name.clone());
                arg_var_names.push(arg.var_name.clone());

                // Check for conflict with declared vars
                if context.declared_vars.contains(&arg.var_name) {
                    context.add_error(ValidationError::CrossReferenceViolation {
                        path: format!("{}.var_name", path),
                        reason: format!(
                            "argument var_name '{}' conflicts with declared variable",
                            arg.var_name
                        ),
                    });
                }
            }
        }
    }

    // Check arg ordering (required args cannot follow optional)
    check_arg_ordering(args, command_name, context);

    // Check for prefix conflicts in var_names
    let conflicts = check_prefix_conflicts(&arg_var_names);
    for (prefix, longer) in conflicts {
        context.add_error(ValidationError::CrossReferenceViolation {
            path: format!("commands.{}.args", command_name),
            reason: format!("argument var_name '{}' is a prefix of '{}'", prefix, longer),
        });
    }
}

/// Validate arg name format: [a-z0-9-]+ (no leading/trailing hyphens)
fn validate_arg_name_format(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("argument name cannot be empty".to_string());
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err("argument name cannot start or end with hyphen".to_string());
    }

    for ch in name.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '-' {
            return Err(format!(
                "argument name contains invalid character '{}': only alphanumeric and hyphen allowed",
                ch
            ));
        }
    }

    Ok(())
}

/// Check that required arguments don't follow optional ones.
fn check_arg_ordering(
    args: &[saran_types::PositionalArg],
    command_name: &str,
    context: &mut ValidationContext,
) {
    let mut seen_optional = false;

    for (index, arg) in args.iter().enumerate() {
        if arg.required {
            if seen_optional {
                context.add_error(ValidationError::CrossReferenceViolation {
                    path: format!("commands.{}.args[{}]", command_name, index),
                    reason: "required argument cannot follow optional argument".to_string(),
                });
            }
        } else {
            seen_optional = true;
        }
    }
}

// ============================================================================
// Top-Level Validation
// ============================================================================

/// Validate top-level wrapper structure.
///
/// Checks required fields: `name`, `version`, `commands`.
///
/// # Arguments
/// * `wrapper` - The wrapper definition to validate
///
/// # Returns
/// Vector of validation errors (empty if all checks pass)
pub fn validate_top_level(wrapper: &saran_types::WrapperDefinition) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check name is present and non-empty (TL-01, TL-02)
    if wrapper.name.is_empty() {
        errors.push(ValidationError::MissingField {
            path: "".to_string(),
            field: "name",
        });
    }

    // Check version is present and valid SemVer (TL-03, TL-04, TL-05)
    if wrapper.version.is_empty() {
        errors.push(ValidationError::MissingField {
            path: "".to_string(),
            field: "version",
        });
    } else if let Err(e) = validate_semver(&wrapper.version) {
        errors.push(ValidationError::SemverParseError {
            value: wrapper.version.clone(),
            error: e,
        });
    }

    // Check commands is present and non-empty (TL-06, TL-07)
    if wrapper.commands.is_empty() {
        errors.push(ValidationError::MissingField {
            path: "".to_string(),
            field: "commands",
        });
    }

    errors
}

// ============================================================================
// Phase 2C: Variable References Validation
// ============================================================================

/// Validate all `$VAR_NAME` token references in action args and variable help text.
///
/// Checks:
/// * Action args can only reference variables declared in `vars:` section
/// * Variable help text can only reference variables declared in `vars:` section
/// * Token syntax must be valid (`$VAR_NAME` format)
///
/// Uses Phase 1's `parse_tokens()` to extract and validate token syntax.
fn validate_variable_references(
    wrapper: &saran_types::WrapperDefinition,
    ctx: &ValidationContext,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check all action arg references
    for (cmd_name, command) in &wrapper.commands {
        for (action_idx, action) in command.actions.iter().enumerate() {
            for (arg_idx, arg) in action.args.iter().enumerate() {
                // Parse tokens from the arg string
                match parse_tokens(arg) {
                    Ok(parsed) => {
                        // Check each token against declared vars
                        for token in &parsed.tokens {
                            if !ctx.declared_vars.contains(&token.var_name) {
                                errors.push(ValidationError::UndeclaredReference {
                                    path: format!(
                                        "commands.{}.actions[{}].args[{}]",
                                        cmd_name, action_idx, arg_idx
                                    ),
                                    var_name: token.var_name.clone(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Token parsing failed - invalid syntax
                        errors.push(ValidationError::InvalidFormat {
                            path: format!(
                                "commands.{}.actions[{}].args[{}]",
                                cmd_name, action_idx, arg_idx
                            ),
                            reason: format!("Invalid $VAR_NAME syntax: {}", e),
                        });
                    }
                }
            }
        }
    }

    // Check all variable help text references
    if !wrapper.vars.is_empty() {
        for (var_idx, var_decl) in wrapper.vars.iter().enumerate() {
            if let Some(help_text) = &var_decl.help {
                // Parse tokens from help text
                match parse_tokens(help_text) {
                    Ok(parsed) => {
                        // Check each token against declared vars
                        // (Simpler design: help text can only reference vars, not args)
                        for token in &parsed.tokens {
                            if !ctx.declared_vars.contains(&token.var_name) {
                                errors.push(ValidationError::UndeclaredReference {
                                    path: format!("vars[{}].help", var_idx),
                                    var_name: token.var_name.clone(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Token parsing failed - invalid syntax
                        errors.push(ValidationError::InvalidFormat {
                            path: format!("vars[{}].help", var_idx),
                            reason: format!("Invalid $VAR_NAME syntax: {}", e),
                        });
                    }
                }
            }
        }
    }

    errors
}

// ============================================================================
// Phase 2C: Requires Section Validation
// ============================================================================

/// Validate the `requires:` section of a wrapper definition.
///
/// Checks:
/// - All required fields present (`cli`, `version`)
/// - Version constraint is valid SemVer format
/// - Version probe is an array (if present) and non-empty
/// - Version pattern is valid regex with exactly 1 capture group (if present)
/// - No duplicate CLI names
///
/// Reuses Phase 2A helpers: `validate_semver_constraint()`, `validate_regex()`,
/// `validate_regex_capture_groups()`.
fn validate_requires(requires: &[saran_types::CliRequirement]) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Early exit if requires section is empty
    if requires.is_empty() {
        return errors;
    }

    // Track seen CLI names for duplicate detection
    let mut seen_clis = std::collections::HashSet::new();

    for (idx, req) in requires.iter().enumerate() {
        let path = format!("requires[{}]", idx);

        // RE-01: Check cli field is present
        if req.cli.is_none() || req.cli.as_ref().map(|s| s.is_empty()).unwrap_or(false) {
            errors.push(ValidationError::MissingField {
                path: path.clone(),
                field: "cli",
            });
        } else if let Some(cli) = &req.cli {
            // RE-08: Check for duplicate CLI names
            if !seen_clis.insert(cli.clone()) {
                errors.push(ValidationError::DuplicateKey {
                    path: path.clone(),
                    key: cli.clone(),
                });
            }
        }

        // RE-02: Check version field is present
        if req.version.is_none() || req.version.as_ref().map(|s| s.is_empty()).unwrap_or(false) {
            errors.push(ValidationError::MissingField {
                path: path.clone(),
                field: "version",
            });
            continue; // Skip further validation if version is missing
        }

        // RE-03: Validate version constraint format
        if let Some(version) = &req.version {
            if let Err(e) = validate_semver_constraint(version) {
                errors.push(ValidationError::SemverParseError {
                    value: version.clone(),
                    error: e,
                });
            }
        }

        // RE-04, RE-05: Validate version_probe if present
        if let Some(probe) = &req.version_probe {
            if probe.is_empty() {
                errors.push(ValidationError::InvalidFormat {
                    path: format!("{}.version_probe", path),
                    reason: "must be non-empty array".to_string(),
                });
            }
        }

        // RE-06, RE-07: Validate version_pattern if present
        if let Some(pattern) = &req.version_pattern {
            // RE-06: Check regex is valid
            if let Err(e) = validate_regex(pattern) {
                errors.push(ValidationError::RegexCompileError {
                    pattern: pattern.clone(),
                    error: e,
                });
            } else {
                // RE-07: Check for exactly 1 capture group
                if let Err(e) = validate_regex_capture_groups(pattern) {
                    errors.push(ValidationError::InvalidValue {
                        path: format!("{}.version_pattern", path),
                        expected: "exactly 1 capture group".to_string(),
                        found: e,
                    });
                }
            }
        }
    }

    errors
}

pub fn validate_wrapper(
    yaml_str: &str,
) -> Result<saran_types::WrapperDefinition, Vec<ValidationError>> {
    // Parse YAML into WrapperDefinition
    let wrapper: saran_types::WrapperDefinition = match serde_yaml::from_str(yaml_str) {
        Ok(w) => w,
        Err(e) => {
            return Err(vec![ValidationError::InvalidFormat {
                path: "".to_string(),
                reason: format!("Failed to parse YAML: {}", e),
            }]);
        }
    };

    let mut context = ValidationContext::new();

    // Phase 2A validators: top-level structure
    context.errors.extend(validate_top_level(&wrapper));

    // Phase 2B validators in dependency order
    // 2B.1: Variable declarations
    validate_vars(&wrapper.vars, &mut context);

    // 2B.2: Commands and actions
    validate_commands(&wrapper.commands, &mut context);

    // 2B.3: Optional flags
    validate_optional_flags_all(&wrapper.commands, &mut context);

    // 2B.4: Positional arguments
    validate_args_all(&wrapper.commands, &mut context);

    // Phase 2C validators
    // 2C.1: Variable references
    context
        .errors
        .extend(validate_variable_references(&wrapper, &context));

    // 2C.2: Requires section
    context.errors.extend(validate_requires(&wrapper.requires));

    context.collect_errors().map(|_| wrapper)
}

#[cfg(test)]
mod tests;
