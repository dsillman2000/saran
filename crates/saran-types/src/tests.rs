#[cfg(test)]
mod tests {
    use crate::{
        Action, BTreeMap, OptionalFlag, PositionalArg, QuotaLimit, VarDecl, WrapperDefinition,
    };
    use serde_yaml;

    #[test]
    fn test_wrapper_definition_roundtrip() {
        let wrapper = WrapperDefinition {
            name: "test-wrapper".to_string(),
            version: "1.0.0".to_string(),
            help: Some("A test wrapper".to_string()),
            requires: vec![],
            vars: vec![],
            quotas: vec![],
            commands: BTreeMap::new(),
        };

        let serialized = serde_yaml::to_string(&wrapper).unwrap();
        let deserialized: WrapperDefinition = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(wrapper, deserialized);
    }

    #[test]
    fn test_var_decl_required_vs_default() {
        let required = VarDecl {
            name: "REQUIRED".to_string(),
            required: true,
            default: None,
            help: None,
        };
        assert!(required.required);
        assert_eq!(required.default, None);

        let optional = VarDecl {
            name: "OPTIONAL".to_string(),
            required: false,
            default: Some("default_value".to_string()),
            help: None,
        };
        assert!(!optional.required);
        assert_eq!(optional.default, Some("default_value".to_string()));
    }

    #[test]
    fn test_optional_flag_type() {
        let str_flag = OptionalFlag {
            name: "--json".to_string(),
            flag_type: "str".to_string(),
            repeated: false,
            help: None,
            passes_as: None,
            values: vec![],
        };
        assert_eq!(str_flag.flag_type, "str");

        let enum_flag = OptionalFlag {
            name: "--state".to_string(),
            flag_type: "enum".to_string(),
            repeated: false,
            help: None,
            passes_as: None,
            values: vec!["open".to_string(), "closed".to_string()],
        };
        assert_eq!(enum_flag.flag_type, "enum");
        assert!(!enum_flag.values.is_empty());
    }

    #[test]
    fn test_action_with_substitution() {
        let action = Action {
            executable: "gh".to_string(),
            args: vec![
                "pr".to_string(),
                "list".to_string(),
                "-R".to_string(),
                "$GH_REPO".to_string(),
            ],
            optional_flags: vec![],
        };
        assert_eq!(action.executable, "gh");
        assert!(action.args.iter().any(|arg| arg.contains("$GH_REPO")));
    }

    #[test]
    fn test_quota_limit_literal_vs_variable() {
        let literal = QuotaLimit::Literal(5);
        assert_eq!(literal, QuotaLimit::Literal(5));

        let variable = QuotaLimit::Variable("$GH_PR_COMMENT_QUOTA".to_string());
        assert_eq!(
            variable,
            QuotaLimit::Variable("$GH_PR_COMMENT_QUOTA".to_string())
        );
    }

    #[test]
    fn test_positional_arg_construction() {
        let arg = PositionalArg {
            name: "repo".to_string(),
            var_name: "REPO".to_string(),
            arg_type: "str".to_string(),
            required: true,
            help: None,
        };
        assert_eq!(arg.arg_type, "str");
        assert_eq!(arg.required, true);
    }

    // ============================================================================
    // Action Deserialization Error Cases
    // ============================================================================

    #[test]
    fn test_action_deserialize_multiple_executable_keys() {
        let yaml = r#"
gh: [pr, list]
redis: [get, key]
"#;
        let result: Result<Action, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("exactly one executable key"),
            "Expected error about multiple keys, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_action_deserialize_no_executable_key() {
        let yaml = r#"
optional_flags: []
"#;
        let result: Result<Action, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("executable key"),
            "Expected error about missing executable key, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_action_deserialize_non_array_executable_value() {
        let yaml = r#"
gh: "not an array"
"#;
        let result: Result<Action, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // serde_yaml returns "invalid type: string" error when trying to deserialize as sequence
        assert!(
            err_msg.contains("invalid type") && err_msg.contains("sequence"),
            "Expected type mismatch error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_action_deserialize_object_executable_value() {
        let yaml = r#"
gh:
  nested: true
"#;
        let result: Result<Action, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // serde_yaml returns "invalid type: map" error when trying to deserialize as sequence
        assert!(
            err_msg.contains("invalid type") && err_msg.contains("sequence"),
            "Expected type mismatch error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_action_deserialize_with_optional_flags() {
        let yaml = r#"
gh: [pr, list]
optional_flags:
  - name: --draft
    flag_type: bool
"#;
        let result: Result<Action, _> = serde_yaml::from_str(yaml);
        if let Err(e) = &result {
            eprintln!("Error: {}", e);
        }
        assert!(
            result.is_ok(),
            "Failed to deserialize Action with optional_flags"
        );
        let action = result.unwrap();
        assert_eq!(action.executable, "gh");
        assert_eq!(action.args, vec!["pr", "list"]);
        assert_eq!(action.optional_flags.len(), 1);
        assert_eq!(action.optional_flags[0].name, "--draft");
        assert_eq!(action.optional_flags[0].flag_type, "bool");
    }

    #[test]
    fn test_action_deserialize_invalid_optional_flags_not_list() {
        let yaml = r#"
gh: [pr, list]
optional_flags: "not a list"
"#;
        let result: Result<Action, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        // The error will come from optional_flags deserialization
    }

    // ============================================================================
    // Empty Collections in WrapperDefinition
    // ============================================================================

    #[test]
    fn test_wrapper_definition_empty_collections() {
        let wrapper = WrapperDefinition {
            name: "empty-wrapper".to_string(),
            version: "1.0.0".to_string(),
            help: None,
            requires: vec![],
            vars: vec![],
            quotas: vec![],
            commands: BTreeMap::new(),
        };

        assert!(wrapper.requires.is_empty());
        assert!(wrapper.vars.is_empty());
        assert!(wrapper.quotas.is_empty());
        assert!(wrapper.commands.is_empty());
    }

    #[test]
    fn test_wrapper_definition_empty_collections_roundtrip() {
        let wrapper = WrapperDefinition {
            name: "empty-wrapper".to_string(),
            version: "1.0.0".to_string(),
            help: None,
            requires: vec![],
            vars: vec![],
            quotas: vec![],
            commands: BTreeMap::new(),
        };

        let serialized = serde_yaml::to_string(&wrapper).unwrap();
        let deserialized: WrapperDefinition = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(wrapper, deserialized);
    }

    #[test]
    fn test_wrapper_definition_omitted_collections() {
        // YAML with no requires, vars, or quotas sections
        let yaml = r#"
name: minimal-wrapper
version: 1.0.0
commands:
  test:
    help: A test command
    actions:
      - echo: ["hello"]
"#;
        let result: Result<WrapperDefinition, _> = serde_yaml::from_str(yaml);
        assert!(result.is_ok());
        let wrapper = result.unwrap();
        assert!(wrapper.requires.is_empty());
        assert!(wrapper.vars.is_empty());
        assert!(wrapper.quotas.is_empty());
    }

    #[test]
    fn test_wrapper_definition_with_all_fields_but_empty_collections() {
        let wrapper = WrapperDefinition {
            name: "full-empty-wrapper".to_string(),
            version: "1.0.0".to_string(),
            help: Some("A wrapper with all fields but empty collections".to_string()),
            requires: vec![],
            vars: vec![],
            quotas: vec![],
            commands: BTreeMap::new(),
        };

        let serialized = serde_yaml::to_string(&wrapper).unwrap();
        let deserialized: WrapperDefinition = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(wrapper, deserialized);
        assert_eq!(
            deserialized.help,
            Some("A wrapper with all fields but empty collections".to_string())
        );
    }

    #[test]
    fn test_wrapper_definition_empty_commands_non_empty_vars() {
        let wrapper = WrapperDefinition {
            name: "vars-only".to_string(),
            version: "1.0.0".to_string(),
            help: None,
            requires: vec![],
            vars: vec![VarDecl {
                name: "TEST_VAR".to_string(),
                required: true,
                default: None,
                help: None,
            }],
            quotas: vec![],
            commands: BTreeMap::new(),
        };

        assert_eq!(wrapper.vars.len(), 1);
        assert!(wrapper.commands.is_empty());
        assert!(wrapper.requires.is_empty());
        assert!(wrapper.quotas.is_empty());
    }
}
