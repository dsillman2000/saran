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
}
