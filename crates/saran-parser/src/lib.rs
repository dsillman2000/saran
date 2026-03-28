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
        Token { var_name, start, end }
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
    let mut chars = input.chars().enumerate();
    while let Some((idx, ch)) = chars.next() {
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

#[cfg(test)]
mod tests;
