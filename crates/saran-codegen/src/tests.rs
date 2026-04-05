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

// M2.1 Tests: CLI Structure Generation

#[test]
fn test_codegen_cli_single_command() {
    let mut commands = std::collections::BTreeMap::new();
    commands.insert(
        "list".to_string(),
        saran_types::Command {
            help: Some("List items".to_string()),
            args: vec![],
            actions: vec![saran_types::Action {
                executable: "ls".to_string(),
                args: vec!["-la".to_string()],
                optional_flags: vec![],
            }],
        },
    );

    let wrapper_def = saran_types::WrapperDefinition {
        name: "my-cli".to_string(),
        version: "1.0.0".to_string(),
        help: Some("My CLI".to_string()),
        requires: vec![],
        vars: vec![],
        quotas: vec![],
        commands,
    };

    let result = generate_cli_struct(&wrapper_def);
    assert!(result.is_ok());

    let (cli_code, enum_code) = result.unwrap();

    // Check CLI struct
    assert!(cli_code.contains("#[derive(Parser)]"));
    assert!(cli_code.contains("#[command(name = \"my-cli\")]"));
    assert!(cli_code.contains("#[command(version = \"1.0.0\")]"));
    assert!(cli_code.contains("struct Cli"));

    // Check Commands enum
    assert!(enum_code.contains("#[derive(Subcommand)]"));
    assert!(enum_code.contains("enum Commands"));
    assert!(enum_code.contains("/// List items"));
    assert!(enum_code.contains("List"));
}

#[test]
fn test_codegen_cli_multiple_commands() {
    let mut commands = std::collections::BTreeMap::new();
    commands.insert(
        "list".to_string(),
        saran_types::Command {
            help: Some("List items".to_string()),
            args: vec![],
            actions: vec![saran_types::Action {
                executable: "ls".to_string(),
                args: vec![],
                optional_flags: vec![],
            }],
        },
    );
    commands.insert(
        "create".to_string(),
        saran_types::Command {
            help: Some("Create item".to_string()),
            args: vec![],
            actions: vec![saran_types::Action {
                executable: "touch".to_string(),
                args: vec![],
                optional_flags: vec![],
            }],
        },
    );

    let wrapper_def = saran_types::WrapperDefinition {
        name: "my-cli".to_string(),
        version: "1.0.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![],
        quotas: vec![],
        commands,
    };

    let result = generate_cli_struct(&wrapper_def);
    assert!(result.is_ok());

    let (_cli_code, enum_code) = result.unwrap();

    // Check both commands are present
    assert!(enum_code.contains("Create"));
    assert!(enum_code.contains("List"));

    // Verify order (BTreeMap sorts lexicographically)
    let create_pos = enum_code.find("Create").expect("Create not found");
    let list_pos = enum_code.find("List").expect("List not found");
    assert!(create_pos < list_pos, "Commands should be sorted");
}

#[test]
fn test_codegen_cli_command_with_help() {
    let mut commands = std::collections::BTreeMap::new();
    commands.insert(
        "ping".to_string(),
        saran_types::Command {
            help: Some("Ping Redis at $REDIS_HOST:$REDIS_PORT".to_string()),
            args: vec![],
            actions: vec![saran_types::Action {
                executable: "redis-cli".to_string(),
                args: vec!["PING".to_string()],
                optional_flags: vec![],
            }],
        },
    );

    let wrapper_def = saran_types::WrapperDefinition {
        name: "redis-cli.db.ro".to_string(),
        version: "1.0.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![],
        quotas: vec![],
        commands,
    };

    let result = generate_cli_struct(&wrapper_def);
    assert!(result.is_ok());

    let (_cli_code, enum_code) = result.unwrap();

    // Check help text is included (with variable substitution tokens)
    assert!(enum_code.contains("Ping Redis at $REDIS_HOST:$REDIS_PORT"));
}

#[test]
fn test_codegen_cli_duplicate_command_names() {
    // This test verifies that duplicate arg names in a single command are caught
    let mut commands = std::collections::BTreeMap::new();
    commands.insert(
        "test".to_string(),
        saran_types::Command {
            help: None,
            args: vec![
                saran_types::PositionalArg {
                    name: "file".to_string(),
                    var_name: "FILE".to_string(),
                    arg_type: "str".to_string(),
                    required: true,
                    help: None,
                },
                saran_types::PositionalArg {
                    name: "file".to_string(), // Duplicate name
                    var_name: "FILE2".to_string(),
                    arg_type: "str".to_string(),
                    required: true,
                    help: None,
                },
            ],
            actions: vec![saran_types::Action {
                executable: "cat".to_string(),
                args: vec![],
                optional_flags: vec![],
            }],
        },
    );

    let wrapper_def = saran_types::WrapperDefinition {
        name: "test-cli".to_string(),
        version: "1.0.0".to_string(),
        help: None,
        requires: vec![],
        vars: vec![],
        quotas: vec![],
        commands,
    };

    let result = generate_cli_struct(&wrapper_def);
    assert!(result.is_err());

    match result {
        Err(CodegenError::InvalidWrapperDefinition(msg)) => {
            assert!(msg.contains("Duplicate argument name"));
        }
        _ => panic!("Expected InvalidWrapperDefinition error"),
    }
}

#[test]
fn test_codegen_cli_help_text_substitution() {
    let mut commands = std::collections::BTreeMap::new();
    commands.insert(
        "list".to_string(),
        saran_types::Command {
            help: Some("List PRs in \"$GH_REPO\"".to_string()),
            args: vec![saran_types::PositionalArg {
                name: "state".to_string(),
                var_name: "STATE".to_string(),
                arg_type: "str".to_string(),
                required: false,
                help: Some("Filter by state (open, closed, all)".to_string()),
            }],
            actions: vec![saran_types::Action {
                executable: "gh".to_string(),
                args: vec!["pr".to_string(), "list".to_string()],
                optional_flags: vec![saran_types::OptionalFlag {
                    name: "--json".to_string(),
                    flag_type: "str".to_string(),
                    repeated: false,
                    help: Some("Output as JSON with fields".to_string()),
                    passes_as: None,
                    values: vec![],
                }],
            }],
        },
    );

    let wrapper_def = saran_types::WrapperDefinition {
        name: "gh-pr.repo.ro".to_string(),
        version: "1.0.0".to_string(),
        help: Some("GitHub PR operations".to_string()),
        requires: vec![],
        vars: vec![],
        quotas: vec![],
        commands,
    };

    let result = generate_cli_struct(&wrapper_def);
    assert!(result.is_ok());

    let (cli_code, enum_code) = result.unwrap();

    // Check CLI about text
    assert!(cli_code.contains("GitHub PR operations"));

    // Check command help with escaped quotes
    assert!(enum_code.contains("List PRs in \\\"$GH_REPO\\\""));

    // Check positional arg help
    assert!(enum_code.contains("Filter by state"));

    // Check optional flag help
    assert!(enum_code.contains("Output as JSON with fields"));
}
