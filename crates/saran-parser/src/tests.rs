//! Unit tests for token parsing.
//!
//! Each test is tagged with its specification ID (e.g., [TP-01]) for traceability
//! to spec/tests/unit/02-token-parsing.md.

use saran_test::saran_test;

use crate::{parse_tokens, TokenParsingError};

saran_test!("TP-01", test_parse_single_variable_reference, {
    let result = parse_tokens("$GH_REPO").expect("should parse successfully");

    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].var_name, "GH_REPO");
    assert_eq!(result.tokens[0].start, 0);
    assert_eq!(result.tokens[0].end, 8);
});

saran_test!("TP-02", test_greedy_matching_stops_at_non_identifier, {
    let result = parse_tokens("$GH_REPO/").expect("should parse successfully");

    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].var_name, "GH_REPO");
    assert_eq!(result.tokens[0].start, 0);
    assert_eq!(result.tokens[0].end, 8);

    // The "/" should be preserved in literals
    assert!(result.literals.iter().any(|l| l.text.contains("/")));
});

saran_test!("TP-03", test_multiple_adjacent_references, {
    let result = parse_tokens("$FOO$BAR").expect("should parse successfully");

    assert_eq!(result.tokens.len(), 2);
    assert_eq!(result.tokens[0].var_name, "FOO");
    assert_eq!(result.tokens[0].start, 0);
    assert_eq!(result.tokens[0].end, 4);

    assert_eq!(result.tokens[1].var_name, "BAR");
    assert_eq!(result.tokens[1].start, 4);
    assert_eq!(result.tokens[1].end, 8);
});

saran_test!("TP-04", test_case_sensitive_parsing, {
    let result_var = parse_tokens("$Var").expect("should parse successfully");
    assert_eq!(result_var.tokens[0].var_name, "Var");

    let result_var_upper = parse_tokens("$VAR").expect("should parse successfully");
    assert_eq!(result_var_upper.tokens[0].var_name, "VAR");

    // Ensure they are different
    assert_ne!(
        result_var.tokens[0].var_name,
        result_var_upper.tokens[0].var_name
    );
});

saran_test!("TP-05", test_mixed_literals_and_variables, {
    let result = parse_tokens("prefix-$VAR-suffix").expect("should parse successfully");

    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].var_name, "VAR");
    assert_eq!(result.tokens[0].start, 7);
    assert_eq!(result.tokens[0].end, 11);

    // Check literals: "prefix-" before token and "-suffix" after
    let before_literal = result
        .literals
        .iter()
        .find(|l| l.before)
        .map(|l| l.text.clone());
    assert_eq!(before_literal, Some("prefix-".to_string()));

    let after_literal = result
        .literals
        .iter()
        .find(|l| !l.before)
        .map(|l| l.text.clone());
    assert_eq!(after_literal, Some("-suffix".to_string()));
});

saran_test!("TP-06", test_greedy_matching_takes_maximal_identifier, {
    let result = parse_tokens("$VARsuffix").expect("should parse successfully");

    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].var_name, "VARsuffix");
    assert_eq!(result.tokens[0].start, 0);
    assert_eq!(result.tokens[0].end, 10);

    // No literal suffix since the entire string was consumed as one token
    let after_literal = result.literals.iter().find(|l| !l.before);
    assert!(after_literal.is_none() || after_literal.unwrap().text.is_empty());
});

// Additional edge cases and error conditions (not from spec)

#[test]
fn test_dollar_at_end_of_string() {
    let result = parse_tokens("hello$");
    assert!(result.is_err());
    match result {
        Err(TokenParsingError::IncompleteVariableReference { position }) => {
            assert_eq!(position, 5);
        }
        _ => panic!("expected IncompleteVariableReference error"),
    }
}

#[test]
fn test_dollar_followed_by_digit() {
    let result = parse_tokens("$123");
    assert!(result.is_err());
    match result {
        Err(TokenParsingError::InvalidCharAfterDollar { position, char }) => {
            assert_eq!(position, 0);
            assert_eq!(char, '1');
        }
        _ => panic!("expected InvalidCharAfterDollar error"),
    }
}

#[test]
fn test_dollar_followed_by_special_char() {
    let result = parse_tokens("prefix-$#suffix");
    assert!(result.is_err());
    match result {
        Err(TokenParsingError::InvalidCharAfterDollar { position, char }) => {
            assert_eq!(position, 7);
            assert_eq!(char, '#');
        }
        _ => panic!("expected InvalidCharAfterDollar error"),
    }
}

#[test]
fn test_valid_underscore_start() {
    let result = parse_tokens("$_PRIVATE").expect("should parse successfully");
    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].var_name, "_PRIVATE");
}

#[test]
fn test_empty_string() {
    let result = parse_tokens("").expect("should parse successfully");
    assert_eq!(result.tokens.len(), 0);
    // Empty input should have empty literals
    assert!(result.literals.iter().any(|l| l.text.is_empty()));
}

#[test]
fn test_only_literals() {
    let result = parse_tokens("hello world").expect("should parse successfully");
    assert_eq!(result.tokens.len(), 0);
    assert!(result
        .literals
        .iter()
        .any(|l| l.text.contains("hello world")));
}

// ============================================================================
// Phase 2A: Top-Level Validation Tests (TL-01 through TL-08)
// ============================================================================

use crate::{validate_wrapper, ValidationError};

saran_test!("TL-01", test_missing_name_field, {
    let yaml = include_str!("../tests/fixtures/tl-01-missing-name.yaml");
    let result = validate_wrapper(yaml);

    assert!(result.is_err(), "Expected validation to fail");
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "Expected exactly one error");

    match &errors[0] {
        ValidationError::InvalidFormat { reason, .. } => {
            assert!(
                reason.contains("name"),
                "Expected error related to missing 'name' field"
            );
        }
        _ => panic!("Expected InvalidFormat error, got: {:?}", errors[0]),
    }
});

saran_test!("TL-02", test_empty_name_field, {
    let yaml = include_str!("../tests/fixtures/tl-02-empty-name.yaml");
    let result = validate_wrapper(yaml);

    assert!(result.is_err(), "Expected validation to fail");
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "Expected exactly one error");

    match &errors[0] {
        ValidationError::MissingField { field, .. } => {
            assert_eq!(*field, "name", "Expected missing 'name' field error");
        }
        _ => panic!("Expected MissingField error, got: {:?}", errors[0]),
    }
});

saran_test!("TL-03", test_missing_version_field, {
    let yaml = include_str!("../tests/fixtures/tl-03-missing-version.yaml");
    let result = validate_wrapper(yaml);

    assert!(result.is_err(), "Expected validation to fail");
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "Expected exactly one error");

    match &errors[0] {
        ValidationError::InvalidFormat { reason, .. } => {
            assert!(
                reason.contains("version"),
                "Expected error related to missing 'version' field"
            );
        }
        _ => panic!("Expected InvalidFormat error, got: {:?}", errors[0]),
    }
});

saran_test!("TL-04", test_invalid_semver, {
    let yaml = include_str!("../tests/fixtures/tl-04-invalid-semver.yaml");
    let result = validate_wrapper(yaml);

    assert!(result.is_err(), "Expected validation to fail");
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "Expected exactly one error");

    match &errors[0] {
        ValidationError::SemverParseError { value, .. } => {
            assert_eq!(value, "not-semver", "Expected SemVer parse error");
        }
        _ => panic!("Expected SemverParseError, got: {:?}", errors[0]),
    }
});

saran_test!("TL-05", test_valid_semver_with_prerelease, {
    let yaml = include_str!("../tests/fixtures/tl-05-valid-prerelease.yaml");
    let result = validate_wrapper(yaml);

    assert!(
        result.is_ok(),
        "Expected validation to succeed, got: {:?}",
        result
    );
    let wrapper = result.unwrap();
    assert_eq!(wrapper.name, "test-wrapper");
    assert_eq!(wrapper.version, "1.0.0-beta.1");
});

saran_test!("TL-06", test_missing_commands_section, {
    let yaml = include_str!("../tests/fixtures/tl-06-missing-commands.yaml");
    let result = validate_wrapper(yaml);

    assert!(result.is_err(), "Expected validation to fail");
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "Expected exactly one error");

    match &errors[0] {
        ValidationError::InvalidFormat { reason, .. } => {
            assert!(
                reason.contains("commands"),
                "Expected error related to missing 'commands' field"
            );
        }
        _ => panic!("Expected InvalidFormat error, got: {:?}", errors[0]),
    }
});

saran_test!("TL-07", test_empty_commands_section, {
    let yaml = include_str!("../tests/fixtures/tl-07-empty-commands.yaml");
    let result = validate_wrapper(yaml);

    assert!(result.is_err(), "Expected validation to fail");
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1, "Expected exactly one error");

    match &errors[0] {
        ValidationError::MissingField { field, .. } => {
            assert_eq!(
                *field, "commands",
                "Expected missing 'commands' field error"
            );
        }
        _ => panic!("Expected MissingField error, got: {:?}", errors[0]),
    }
});

saran_test!("TL-08", test_valid_minimal_wrapper, {
    let yaml = include_str!("../tests/fixtures/tl-08-valid-minimal.yaml");
    let result = validate_wrapper(yaml);

    assert!(
        result.is_ok(),
        "Expected validation to succeed, got: {:?}",
        result
    );
    let wrapper = result.unwrap();
    assert_eq!(wrapper.name, "test-wrapper");
    assert_eq!(wrapper.version, "1.0.0");
    assert!(
        !wrapper.commands.is_empty(),
        "Expected at least one command"
    );
});
