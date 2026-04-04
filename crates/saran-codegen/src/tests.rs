use crate::*;

#[test]
fn test_codegen_error_display() {
    let err = CodegenError::InvalidWrapperDefinition("test error".to_string());
    assert_eq!(err.to_string(), "Invalid wrapper definition: test error");
}

#[test]
fn test_codegen_error_template_syntax() {
    let err = CodegenError::TemplateSyntaxError("unclosed brace".to_string());
    assert_eq!(err.to_string(), "Template syntax error: unclosed brace");
}

#[test]
fn test_codegen_error_internal() {
    let err = CodegenError::InternalError("unexpected state".to_string());
    assert_eq!(err.to_string(), "Internal codegen error: unexpected state");
}

#[test]
fn test_generate_not_implemented() {
    let wrapper_def = saran_types::WrapperDefinition {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![],
        quotas: vec![],
        commands: Default::default(),
    };

    let result = generate(&wrapper_def);
    assert!(result.is_err());
    match result {
        Err(CodegenError::InternalError(msg)) => {
            assert!(msg.contains("not yet implemented"));
        }
        _ => panic!("Expected InternalError"),
    }
}

// M2.2 Tests: Variable Declaration Function Generation

#[test]
fn test_codegen_var_decls_empty() {
    let wrapper_def = saran_types::WrapperDefinition {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![],
        quotas: vec![],
        commands: Default::default(),
    };

    let result = generate_var_declarations(&wrapper_def);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("fn get_var_declarations() -> Vec<VarDecl>"));
    assert!(code.contains("vec![\n    ]\n"));
}

#[test]
fn test_codegen_var_decls_required() {
    let wrapper_def = saran_types::WrapperDefinition {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![
            saran_types::VarDecl {
                name: "GH_REPO".to_string(),
                required: true,
                default: None,
                help: None,
            },
            saran_types::VarDecl {
                name: "GH_TOKEN".to_string(),
                required: true,
                default: None,
                help: None,
            },
        ],
        quotas: vec![],
        commands: Default::default(),
    };

    let result = generate_var_declarations(&wrapper_def);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("fn get_var_declarations() -> Vec<VarDecl>"));
    assert!(code.contains("name: \"GH_REPO\".to_string()"));
    assert!(code.contains("name: \"GH_TOKEN\".to_string()"));
    assert!(code.contains("required: true"));
    assert!(code.contains("default: None"));
}

#[test]
fn test_codegen_var_decls_with_defaults() {
    let wrapper_def = saran_types::WrapperDefinition {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![
            saran_types::VarDecl {
                name: "TIMEOUT".to_string(),
                required: false,
                default: Some("30".to_string()),
                help: None,
            },
            saran_types::VarDecl {
                name: "RETRIES".to_string(),
                required: false,
                default: Some("3".to_string()),
                help: None,
            },
        ],
        quotas: vec![],
        commands: Default::default(),
    };

    let result = generate_var_declarations(&wrapper_def);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("name: \"TIMEOUT\".to_string()"));
    assert!(code.contains("name: \"RETRIES\".to_string()"));
    assert!(code.contains("required: false"));
    assert!(code.contains("default: Some(\"30\".to_string())"));
    assert!(code.contains("default: Some(\"3\".to_string())"));
}

#[test]
fn test_codegen_var_decls_mixed() {
    let wrapper_def = saran_types::WrapperDefinition {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![
            saran_types::VarDecl {
                name: "API_KEY".to_string(),
                required: true,
                default: None,
                help: None,
            },
            saran_types::VarDecl {
                name: "LOG_LEVEL".to_string(),
                required: false,
                default: Some("INFO".to_string()),
                help: None,
            },
            saran_types::VarDecl {
                name: "ENDPOINT".to_string(),
                required: true,
                default: None,
                help: None,
            },
        ],
        quotas: vec![],
        commands: Default::default(),
    };

    let result = generate_var_declarations(&wrapper_def);
    assert!(result.is_ok());

    let code = result.unwrap();

    // Check required variables
    assert!(code.contains("name: \"API_KEY\".to_string()"));
    assert!(code.contains("name: \"ENDPOINT\".to_string()"));

    // Check optional variable with default
    assert!(code.contains("name: \"LOG_LEVEL\".to_string()"));
    assert!(code.contains("default: Some(\"INFO\".to_string())"));

    // Verify order is preserved
    let api_key_pos = code.find("API_KEY").expect("API_KEY not found");
    let log_level_pos = code.find("LOG_LEVEL").expect("LOG_LEVEL not found");
    let endpoint_pos = code.find("ENDPOINT").expect("ENDPOINT not found");

    assert!(api_key_pos < log_level_pos, "Variables not in order");
    assert!(log_level_pos < endpoint_pos, "Variables not in order");
}
