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

/// Main validation entry point.
///
/// Deserializes YAML and validates the complete wrapper definition.
/// Returns all errors collected during validation, or the validated `WrapperDefinition`.
///
/// # Arguments
/// * `yaml_str` - The YAML content to parse and validate
///
/// # Returns
/// * `Ok(WrapperDefinition)` if validation passes
/// * `Err(Vec<ValidationError>)` containing all validation errors
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

    // Run validation
    let errors = validate_top_level(&wrapper);

    if errors.is_empty() {
        Ok(wrapper)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests;
