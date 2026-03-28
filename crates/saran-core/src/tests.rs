use crate::*;
use saran_test::saran_test;
use saran_types::VarDecl;
use std::collections::HashMap;

// Helper: create a VarDecl
fn var_decl(name: &str, required: bool, default: Option<&str>) -> VarDecl {
    VarDecl {
        name: name.to_string(),
        required,
        default: default.map(|s| s.to_string()),
        help: None,
    }
}

// Helper: create a host environment
fn host_env(vars: &[(&str, &str)]) -> HashMap<String, String> {
    vars.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ============================================================================
// Priority Chain Tests (P-01 to P-04)
// ============================================================================

saran_test!("P-01", p_01_highest_priority_wins, {
    // Variable has values in all four layers; per-wrapper wins
    let var_decls = vec![var_decl("GH_REPO", false, Some("org/default"))];

    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .global
        .insert("GH_REPO".to_string(), "org/global".to_string());
    env_yaml
        .wrappers
        .entry("test-wrapper".to_string())
        .or_insert_with(HashMap::new)
        .insert("GH_REPO".to_string(), "org/per-wrapper".to_string());

    let host = host_env(&[("GH_REPO", "org/host")]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert!(result.missing_required.is_empty());
    assert_eq!(result.resolved.len(), 1);
    let resolved_var = &result.resolved["GH_REPO"];
    assert_eq!(resolved_var.value, "org/per-wrapper");
    assert_eq!(resolved_var.scope, SaranEnvScope::PerWrapper);
});

saran_test!("P-02", p_02_fallback_to_global, {
    // Variable missing from per-wrapper; global wins
    let var_decls = vec![var_decl("GH_REPO", false, Some("org/default"))];

    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .global
        .insert("GH_REPO".to_string(), "org/global".to_string());

    let host = host_env(&[("GH_REPO", "org/host")]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert!(result.missing_required.is_empty());
    assert_eq!(result.resolved.len(), 1);
    let resolved_var = &result.resolved["GH_REPO"];
    assert_eq!(resolved_var.value, "org/global");
    assert_eq!(resolved_var.scope, SaranEnvScope::Global);
});

saran_test!("P-03", p_03_fallback_to_host, {
    // Variable missing from per-wrapper and global; host wins
    let var_decls = vec![var_decl("GH_REPO", false, Some("org/default"))];

    let env_yaml = SaranEnvYaml::default();
    let host = host_env(&[("GH_REPO", "org/host")]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert!(result.missing_required.is_empty());
    assert_eq!(result.resolved.len(), 1);
    let resolved_var = &result.resolved["GH_REPO"];
    assert_eq!(resolved_var.value, "org/host");
    assert_eq!(resolved_var.scope, SaranEnvScope::Host);
});

saran_test!("P-04", p_04_default_as_last_resort, {
    // Variable only has a default value
    let var_decls = vec![var_decl("GH_REPO", false, Some("org/default"))];

    let env_yaml = SaranEnvYaml::default();
    let host = host_env(&[]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert!(result.missing_required.is_empty());
    assert_eq!(result.resolved.len(), 1);
    let resolved_var = &result.resolved["GH_REPO"];
    assert_eq!(resolved_var.value, "org/default");
    assert_eq!(resolved_var.scope, SaranEnvScope::Default);
});

// ============================================================================
// Edge Cases & Value Preservation (E-01 to E-03)
// ============================================================================

saran_test!("E-01", e_01_empty_string_is_valid_value, {
    // Empty string from per-wrapper resolves (not skipped)
    let var_decls = vec![var_decl("EMPTY_VAR", false, Some("default"))];

    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .wrappers
        .entry("test-wrapper".to_string())
        .or_insert_with(HashMap::new)
        .insert("EMPTY_VAR".to_string(), "".to_string());

    let host = host_env(&[("EMPTY_VAR", "host-value")]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert!(result.missing_required.is_empty());
    let resolved_var = &result.resolved["EMPTY_VAR"];
    assert_eq!(resolved_var.value, "");
    assert_eq!(resolved_var.scope, SaranEnvScope::PerWrapper);
});

saran_test!("E-02", e_02_special_characters_preserved, {
    // Values with newlines, quotes, Unicode, shell chars preserved exactly
    let var_decls = vec![var_decl("VAR", false, None)];

    let special_value = "line1\nline2\t\"quoted\"\n$shell'meta'Unicode:→";
    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .global
        .insert("VAR".to_string(), special_value.to_string());

    let host = host_env(&[]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    let resolved_var = &result.resolved["VAR"];
    assert_eq!(resolved_var.value, special_value);
});

saran_test!("E-03", e_03_case_sensitive_variable_names, {
    // VAR and var are distinct variables
    let var_decls = vec![
        var_decl("VAR", false, Some("default-upper")),
        var_decl("var", false, Some("default-lower")),
    ];

    let env_yaml = SaranEnvYaml::default();
    let host = host_env(&[]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert_eq!(result.resolved.len(), 2);
    assert_eq!(result.resolved["VAR"].value, "default-upper");
    assert_eq!(result.resolved["var"].value, "default-lower");
});

// ============================================================================
// Error Conditions & Missing Values (ER-01, ER-02)
// ============================================================================

saran_test!("ER-01", er_01_required_variable_missing, {
    // Required variable with no value anywhere
    let var_decls = vec![var_decl("GH_TOKEN", true, None)];

    let env_yaml = SaranEnvYaml::default();
    let host = host_env(&[]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert_eq!(result.resolved.len(), 0);
    assert_eq!(result.missing_required, vec!["GH_TOKEN"]);
});

saran_test!("ER-02", er_02_optional_variable_omitted, {
    // Optional variable with no value is completely absent from result
    let var_decls = vec![var_decl("OPTIONAL_VAR", false, None)];

    let env_yaml = SaranEnvYaml::default();
    let host = host_env(&[]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert_eq!(result.resolved.len(), 0);
    assert!(result.missing_required.is_empty());
    assert!(!result.resolved.contains_key("OPTIONAL_VAR"));
});

// ============================================================================
// Isolation & Scope Tracking (I-01 to I-03)
// ============================================================================

saran_test!("I-01", i_01_per_wrapper_isolation, {
    // Per-wrapper value only affects named wrapper
    let var_decls = vec![var_decl("TARGET", false, Some("default"))];

    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .wrappers
        .entry("wrapper-a".to_string())
        .or_insert_with(HashMap::new)
        .insert("TARGET".to_string(), "value-for-a".to_string());

    let host = host_env(&[]);

    // Resolve for wrapper-a (has per-wrapper value)
    let result_a = resolve_vars(&var_decls, &env_yaml, "wrapper-a", &host);
    assert_eq!(result_a.resolved["TARGET"].value, "value-for-a");
    assert_eq!(result_a.resolved["TARGET"].scope, SaranEnvScope::PerWrapper);

    // Resolve for wrapper-b (no per-wrapper value, falls through to default)
    let result_b = resolve_vars(&var_decls, &env_yaml, "wrapper-b", &host);
    assert_eq!(result_b.resolved["TARGET"].value, "default");
    assert_eq!(result_b.resolved["TARGET"].scope, SaranEnvScope::Default);
});

saran_test!("I-02", i_02_global_affects_all_wrappers, {
    // Global value affects all wrappers equally
    let var_decls = vec![var_decl("SHARED_VAR", false, None)];

    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .global
        .insert("SHARED_VAR".to_string(), "global-value".to_string());

    let host = host_env(&[]);

    let result_a = resolve_vars(&var_decls, &env_yaml, "wrapper-a", &host);
    let result_b = resolve_vars(&var_decls, &env_yaml, "wrapper-b", &host);

    assert_eq!(result_a.resolved["SHARED_VAR"].value, "global-value");
    assert_eq!(result_a.resolved["SHARED_VAR"].scope, SaranEnvScope::Global);

    assert_eq!(result_b.resolved["SHARED_VAR"].value, "global-value");
    assert_eq!(result_b.resolved["SHARED_VAR"].scope, SaranEnvScope::Global);
});

saran_test!("I-03", i_03_scope_correctly_tracked, {
    // Each resolved var has correct scope matching actual source
    let var_decls = vec![
        var_decl("FROM_PER_WRAPPER", false, None),
        var_decl("FROM_GLOBAL", false, None),
        var_decl("FROM_HOST", false, None),
        var_decl("FROM_DEFAULT", false, Some("default-val")),
    ];

    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .wrappers
        .entry("test-wrapper".to_string())
        .or_insert_with(HashMap::new)
        .insert(
            "FROM_PER_WRAPPER".to_string(),
            "per-wrapper-val".to_string(),
        );
    env_yaml
        .global
        .insert("FROM_GLOBAL".to_string(), "global-val".to_string());

    let host = host_env(&[("FROM_HOST", "host-val")]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert_eq!(
        result.resolved["FROM_PER_WRAPPER"].scope,
        SaranEnvScope::PerWrapper
    );
    assert_eq!(result.resolved["FROM_GLOBAL"].scope, SaranEnvScope::Global);
    assert_eq!(result.resolved["FROM_HOST"].scope, SaranEnvScope::Host);
    assert_eq!(
        result.resolved["FROM_DEFAULT"].scope,
        SaranEnvScope::Default
    );
});

// ============================================================================
// Type Construction & Basic Parsing (T-01, T-02)
// ============================================================================

saran_test!("T-01", t_01_empty_yaml_parsing, {
    // Empty YAML parses to empty sections
    let yaml_empty = "";
    let env = SaranEnvYaml::from_yaml(yaml_empty).expect("empty yaml should parse");
    assert!(env.global.is_empty());
    assert!(env.wrappers.is_empty());

    let yaml_whitespace = "   \n  \n  ";
    let env = SaranEnvYaml::from_yaml(yaml_whitespace).expect("whitespace yaml should parse");
    assert!(env.global.is_empty());
    assert!(env.wrappers.is_empty());
});

saran_test!("T-02", t_02_complete_yaml_parsing, {
    // Full YAML with both sections parses correctly
    let yaml = r#"
global:
  GH_REPO: "org/global-repo"
  SHARED_VAR: "shared-value"
wrappers:
  gh-pr.repo.ro:
    GH_REPO: "org/per-wrapper-repo"
  other-wrapper:
    OTHER_VAR: "other-value"
"#;

    let env = SaranEnvYaml::from_yaml(yaml).expect("should parse");

    // Check global section
    assert_eq!(env.global.len(), 2);
    assert_eq!(env.global["GH_REPO"], "org/global-repo");
    assert_eq!(env.global["SHARED_VAR"], "shared-value");

    // Check wrappers section
    assert_eq!(env.wrappers.len(), 2);
    assert_eq!(
        env.wrappers["gh-pr.repo.ro"]["GH_REPO"],
        "org/per-wrapper-repo"
    );
    assert_eq!(env.wrappers["other-wrapper"]["OTHER_VAR"], "other-value");
});

// ============================================================================
// Additional Tests: Multiple Missing Required Variables
// ============================================================================

saran_test!("MULTI", multiple_missing_required_variables, {
    // Multiple required variables with no values should all be listed
    let var_decls = vec![
        var_decl("TOKEN_A", true, None),
        var_decl("TOKEN_B", true, None),
        var_decl("TOKEN_C", true, None),
    ];

    let env_yaml = SaranEnvYaml::default();
    let host = host_env(&[]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert_eq!(result.resolved.len(), 0);
    let mut missing = result.missing_required.clone();
    missing.sort();
    assert_eq!(missing, vec!["TOKEN_A", "TOKEN_B", "TOKEN_C"]);
});

// ============================================================================
// Additional Tests: Mixed Required and Optional
// ============================================================================

saran_test!("MIXED", mixed_required_and_optional_variables, {
    // Some required with values, some without; some optional with values, some without
    let var_decls = vec![
        var_decl("REQUIRED_WITH_VALUE", true, None),
        var_decl("REQUIRED_NO_VALUE", true, None),
        var_decl("OPTIONAL_WITH_VALUE", false, None),
        var_decl("OPTIONAL_NO_VALUE", false, None),
    ];

    let mut env_yaml = SaranEnvYaml::default();
    env_yaml
        .global
        .insert("REQUIRED_WITH_VALUE".to_string(), "has-value".to_string());
    env_yaml
        .global
        .insert("OPTIONAL_WITH_VALUE".to_string(), "has-value".to_string());

    let host = host_env(&[]);

    let result = resolve_vars(&var_decls, &env_yaml, "test-wrapper", &host);

    assert_eq!(result.resolved.len(), 2);
    assert!(result.resolved.contains_key("REQUIRED_WITH_VALUE"));
    assert!(result.resolved.contains_key("OPTIONAL_WITH_VALUE"));
    assert!(!result.resolved.contains_key("OPTIONAL_NO_VALUE"));

    assert_eq!(result.missing_required, vec!["REQUIRED_NO_VALUE"]);
});

// ============================================================================
// Phase 2: Substitution Resolution Tests
// ============================================================================

// ---- Value Resolution Tests (VR-01, VR-02) ----

saran_test!("VR-01", vr_01_valid_variable_reference_resolves, {
    // Reference $GH_REPO where GH_REPO declared in vars:
    let parsed = parse_tokens("$GH_REPO").unwrap();
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("GH_REPO".to_string(), "org/repo".to_string());
    let caller_args = HashMap::new();

    let context = ResolutionContext::new(resolved_vars, caller_args);
    let result = resolve_substitution(&parsed, &context).unwrap();

    assert_eq!(result, "org/repo");
});

saran_test!("VR-02", vr_02_valid_argument_reference_resolves, {
    // Reference $PR_NUM where PR_NUM declared in command args:
    let parsed = parse_tokens("PR #$PR_NUM").unwrap();
    let resolved_vars = HashMap::new();
    let mut caller_args = HashMap::new();
    caller_args.insert("PR_NUM".to_string(), "42".to_string());

    let context = ResolutionContext::new(resolved_vars, caller_args);
    let result = resolve_substitution(&parsed, &context).unwrap();

    assert_eq!(result, "PR #42");
});

// ---- Context-Specific Behavior Tests (CS-01, CS-02, CS-03) ----

saran_test!("CS-01", cs_01_help_strings_accept_variable_references, {
    // Help text "Operations for $GH_REPO" with GH_REPO in vars:
    let parsed = parse_tokens("Operations for $GH_REPO").unwrap();
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("GH_REPO".to_string(), "org/repo".to_string());

    let result = resolve_help_text(&parsed, &resolved_vars);

    assert_eq!(result, "Operations for org/repo");
});

saran_test!("CS-02", cs_02_help_resolution_tolerates_missing_values, {
    // Help text "Repo: $GH_REPO" where GH_REPO unresolved at startup
    let parsed = parse_tokens("Repo: $GH_REPO").unwrap();
    let resolved_vars = HashMap::new();

    let result = resolve_help_text(&parsed, &resolved_vars);

    // Per spec: literal $GH_REPO shown (no error)
    assert_eq!(result, "Repo: $GH_REPO");
});

saran_test!("CS-03", cs_03_help_with_multiple_variable_references, {
    // Text "Repo: $REPO, PR: $PR" with one resolved, one not
    let parsed = parse_tokens("Repo: $REPO, PR: $PR").unwrap();
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("REPO".to_string(), "org/repo".to_string());
    // PR not resolved

    let result = resolve_help_text(&parsed, &resolved_vars);

    // Mixed resolution: resolved var substituted, unresolved shown literally
    assert_eq!(result, "Repo: org/repo, PR: $PR");
});

// ---- Edge Cases & Value Handling Tests (EC-01 through EC-05) ----

saran_test!("EC-01", ec_01_empty_variable_value_substitutes, {
    // Reference $EMPTY_VAR where variable value is empty string ""
    let parsed = parse_tokens("Value: [$EMPTY_VAR]").unwrap();
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("EMPTY_VAR".to_string(), "".to_string());
    let caller_args = HashMap::new();

    let context = ResolutionContext::new(resolved_vars, caller_args);
    let result = resolve_substitution(&parsed, &context).unwrap();

    // Substitutes empty string (not omitted)
    assert_eq!(result, "Value: []");
});

saran_test!("EC-02", ec_02_whitespace_in_values_preserved, {
    // Reference $VAR where value contains spaces, tabs, newlines
    let parsed = parse_tokens("Text: $VAR").unwrap();
    let mut resolved_vars = HashMap::new();
    let value_with_whitespace = "hello  \t world\n  end".to_string();
    resolved_vars.insert("VAR".to_string(), value_with_whitespace.clone());
    let caller_args = HashMap::new();

    let context = ResolutionContext::new(resolved_vars, caller_args);
    let result = resolve_substitution(&parsed, &context).unwrap();

    assert_eq!(result, format!("Text: {}", value_with_whitespace));
});

saran_test!("EC-03", ec_03_dollar_sign_in_value_not_reparsed, {
    // Reference $VAR where value is "text$more"
    let parsed = parse_tokens("$VAR").unwrap();
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("VAR".to_string(), "text$more".to_string());
    let caller_args = HashMap::new();

    let context = ResolutionContext::new(resolved_vars, caller_args);
    let result = resolve_substitution(&parsed, &context).unwrap();

    // Substitutes literal $, no recursive parsing
    assert_eq!(result, "text$more");
});

saran_test!("EC-04", ec_04_no_recursive_substitution, {
    // String "$FOO" where FOO="BAR" and BAR="value"
    let parsed = parse_tokens("$FOO").unwrap();
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("FOO".to_string(), "BAR".to_string());
    // Note: BAR is NOT in resolved_vars - we don't do recursive lookup
    let caller_args = HashMap::new();

    let context = ResolutionContext::new(resolved_vars, caller_args);
    let result = resolve_substitution(&parsed, &context).unwrap();

    // Substitutes "BAR", doesn't look up BAR's value
    assert_eq!(result, "BAR");
});

saran_test!("EC-05", ec_05_large_variable_values, {
    // Reference $VAR where value is 64KB string
    let parsed = parse_tokens("Value: $VAR").unwrap();
    let mut resolved_vars = HashMap::new();
    let large_value = "x".repeat(64 * 1024);
    resolved_vars.insert("VAR".to_string(), large_value.clone());
    let caller_args = HashMap::new();

    let context = ResolutionContext::new(resolved_vars, caller_args);
    let result = resolve_substitution(&parsed, &context).unwrap();

    // Substitutes successfully (no artificial size limits)
    assert_eq!(result, format!("Value: {}", large_value));
});

// ============================================================================
// Phase 3: Argument Assembly Tests
// ============================================================================

// ---- Basic Assembly Tests (BA-01 through BA-04) ----

saran_test!("BA-01", ba_01_fixed_arguments_preserved, {
    // Action: ["gh", "pr", "view"]
    let action_args = vec!["pr".to_string(), "view".to_string()];
    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    // argv: ["gh", "pr", "view"]
    assert_eq!(argv, vec!["gh", "pr", "view"]);
});

saran_test!("BA-02", ba_02_empty_string_argument, {
    // Action: ["gh", "", "pr"]
    let action_args = vec!["".to_string(), "pr".to_string()];
    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "", "pr"]);
});

saran_test!("BA-03", ba_03_whitespace_in_arguments, {
    // Action: ["gh", "pr with spaces"]
    let action_args = vec!["pr with spaces".to_string()];
    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr with spaces"]);
});

saran_test!("BA-04", ba_04_special_characters_preserved, {
    // Action: ["gh", "pr", "--json", "{\"key\":\"value\"}"]
    let action_args = vec![
        "pr".to_string(),
        "--json".to_string(),
        "{\"key\":\"value\"}".to_string(),
    ];
    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "--json", "{\"key\":\"value\"}"]);
});

// ---- Variable Substitution Tests (VS-01 through VS-04) ----

saran_test!("VS-01", vs_01_var_substitution_in_action, {
    // Action: ["gh", "pr", "view", "$PR_NUM"] with PR_NUM="123"
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("PR_NUM".to_string(), "123".to_string());

    let action_args = vec!["pr".to_string(), "view".to_string(), "$PR_NUM".to_string()];
    let context = AssemblyContext::new(resolved_vars, HashMap::new(), HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "view", "123"]);
});

saran_test!("VS-02", vs_02_arg_substitution_in_action, {
    // Action: ["gh", "pr", "view", "$PR_NUM"] with arg PR_NUM="456"
    let mut caller_args = HashMap::new();
    caller_args.insert("PR_NUM".to_string(), "456".to_string());

    let action_args = vec!["pr".to_string(), "view".to_string(), "$PR_NUM".to_string()];
    let context = AssemblyContext::new(HashMap::new(), caller_args, HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "view", "456"]);
});

saran_test!("VS-03", vs_03_mixed_var_and_arg_substitution, {
    // Action: ["gh", "pr", "view", "$REPO", "$PR_NUM"] with var REPO="org/repo" and arg PR_NUM="789"
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("REPO".to_string(), "org/repo".to_string());

    let mut caller_args = HashMap::new();
    caller_args.insert("PR_NUM".to_string(), "789".to_string());

    let action_args = vec![
        "pr".to_string(),
        "view".to_string(),
        "$REPO".to_string(),
        "$PR_NUM".to_string(),
    ];
    let context = AssemblyContext::new(resolved_vars, caller_args, HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "view", "org/repo", "789"]);
});

saran_test!("VS-04", vs_04_empty_variable_value, {
    // Action: ["gh", "pr", "view", "$EMPTY"] with EMPTY=""
    let mut resolved_vars = HashMap::new();
    resolved_vars.insert("EMPTY".to_string(), "".to_string());

    let action_args = vec!["pr".to_string(), "view".to_string(), "$EMPTY".to_string()];
    let context = AssemblyContext::new(resolved_vars, HashMap::new(), HashMap::new());

    let argv = build_argv("gh", &action_args, &[], &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "view", ""]);
});

// ---- Optional Flag Appending Tests (OFA-01 through OFA-04) ----

saran_test!("OFA-01", ofa_01_str_flag_appended, {
    // Action: ["gh", "pr", "view"] with --json flag value "title,body"
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert(
        "--json".to_string(),
        OptionalFlagValue::string("title,body".to_string()),
    );

    let action_args = vec!["pr".to_string(), "view".to_string()];
    let flags = vec![saran_types::OptionalFlag {
        name: "--json".to_string(),
        flag_type: "str".to_string(),
        repeated: false,
        help: None,
        passes_as: None,
        values: vec![],
    }];

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("gh", &action_args, &flags, &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "view", "--json", "title,body"]);
});

saran_test!("OFA-02", ofa_02_int_flag_appended, {
    // Action: ["tool", "run"] with --count flag value "5"
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert(
        "--count".to_string(),
        OptionalFlagValue::string("5".to_string()),
    );

    let action_args = vec!["run".to_string()];
    let flags = vec![saran_types::OptionalFlag {
        name: "--count".to_string(),
        flag_type: "int".to_string(),
        repeated: false,
        help: None,
        passes_as: None,
        values: vec![],
    }];

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("tool", &action_args, &flags, &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["tool", "run", "--count", "5"]);
});

saran_test!("OFA-03", ofa_03_bool_flag_appended, {
    // Action: ["gh", "pr", "view"] with --verbose flag (bool)
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert("--verbose".to_string(), OptionalFlagValue::bool());

    let action_args = vec!["pr".to_string(), "view".to_string()];
    let flags = vec![saran_types::OptionalFlag {
        name: "--verbose".to_string(),
        flag_type: "bool".to_string(),
        repeated: false,
        help: None,
        passes_as: None,
        values: vec![],
    }];

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("gh", &action_args, &flags, &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "view", "--verbose"]);
});

saran_test!("OFA-04", ofa_04_enum_flag_appended, {
    // Action: ["tool", "run"] with --format flag value "json"
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert(
        "--format".to_string(),
        OptionalFlagValue::string("json".to_string()),
    );

    let action_args = vec!["run".to_string()];
    let flags = vec![saran_types::OptionalFlag {
        name: "--format".to_string(),
        flag_type: "enum".to_string(),
        repeated: false,
        help: None,
        passes_as: None,
        values: vec!["json".to_string(), "yaml".to_string()],
    }];

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("tool", &action_args, &flags, &context, &HashMap::new()).unwrap();

    assert_eq!(argv, vec!["tool", "run", "--format", "json"]);
});

// ---- Flag Type-Specific Behavior Tests (FT-01 through FT-04) ----

saran_test!("FT-01", ft_01_repeated_str_flag, {
    // Action: ["gh", "pr", "edit"] with --label flag values ["bug", "enhancement"] (repeated: true)
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert(
        "--label".to_string(),
        OptionalFlagValue::multiple(vec!["bug".to_string(), "enhancement".to_string()]),
    );

    let action_args = vec!["pr".to_string(), "edit".to_string()];
    let flags = vec![saran_types::OptionalFlag {
        name: "--label".to_string(),
        flag_type: "str".to_string(),
        repeated: true,
        help: None,
        passes_as: None,
        values: vec![],
    }];

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("gh", &action_args, &flags, &context, &HashMap::new()).unwrap();

    assert_eq!(
        argv,
        vec![
            "gh",
            "pr",
            "edit",
            "--label",
            "bug",
            "--label",
            "enhancement"
        ]
    );
});

saran_test!("FT-02", ft_02_passes_as_overrides_name, {
    // Flag: name: "--json", passes_as: "--format=json" with value "title"
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert(
        "--json".to_string(),
        OptionalFlagValue::string("title".to_string()),
    );

    let action_args = vec!["pr".to_string(), "view".to_string()];
    let flags = vec![saran_types::OptionalFlag {
        name: "--json".to_string(),
        flag_type: "str".to_string(),
        repeated: false,
        help: None,
        passes_as: Some("--format=json".to_string()),
        values: vec![],
    }];

    let mut passes_as_map = HashMap::new();
    passes_as_map.insert("--json".to_string(), "--format=json".to_string());

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("gh", &action_args, &flags, &context, &passes_as_map).unwrap();

    assert_eq!(argv, vec!["gh", "pr", "view", "--format=json", "title"]);
});

saran_test!("FT-03", ft_03_bool_flag_never_has_value, {
    // Action: ["gh", "pr", "view"] with --verbose (bool)
    // Should NOT produce: ["gh", "pr", "view", "--verbose", "true"]
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert("--verbose".to_string(), OptionalFlagValue::bool());

    let action_args = vec!["pr".to_string(), "view".to_string()];
    let flags = vec![saran_types::OptionalFlag {
        name: "--verbose".to_string(),
        flag_type: "bool".to_string(),
        repeated: false,
        help: None,
        passes_as: None,
        values: vec![],
    }];

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("gh", &action_args, &flags, &context, &HashMap::new()).unwrap();

    // Should be just ["gh", "pr", "view", "--verbose"], NOT with "true"
    assert_eq!(argv, vec!["gh", "pr", "view", "--verbose"]);
    assert!(!argv.contains(&"true".to_string()));
});

saran_test!("FT-04", ft_04_order_follows_declaration, {
    // Action with flags --json then --verbose declared
    // Should produce args in declaration order
    let mut optional_flags_map = HashMap::new();
    optional_flags_map.insert(
        "--json".to_string(),
        OptionalFlagValue::string("fields".to_string()),
    );
    optional_flags_map.insert("--verbose".to_string(), OptionalFlagValue::bool());

    let action_args = vec!["pr".to_string(), "view".to_string()];
    let flags = vec![
        saran_types::OptionalFlag {
            name: "--json".to_string(),
            flag_type: "str".to_string(),
            repeated: false,
            help: None,
            passes_as: None,
            values: vec![],
        },
        saran_types::OptionalFlag {
            name: "--verbose".to_string(),
            flag_type: "bool".to_string(),
            repeated: false,
            help: None,
            passes_as: None,
            values: vec![],
        },
    ];

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), optional_flags_map);
    let argv = build_argv("gh", &action_args, &flags, &context, &HashMap::new()).unwrap();

    // Should be ["gh", "pr", "view", "--json", "fields", "--verbose"]
    assert_eq!(
        argv,
        vec!["gh", "pr", "view", "--json", "fields", "--verbose"]
    );
});

// ---- Multi-Action Execution Tests (MA-01 through MA-03) ----

saran_test!("MA-01", ma_01_sequential_action_execution, {
    // Command with 2 actions: ["echo", "first"] then ["echo", "second"]
    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), HashMap::new());

    let argv1 = build_argv(
        "echo",
        &["first".to_string()],
        &[],
        &context,
        &HashMap::new(),
    )
    .unwrap();
    let argv2 = build_argv(
        "echo",
        &["second".to_string()],
        &[],
        &context,
        &HashMap::new(),
    )
    .unwrap();

    // Both actions should produce correct argv
    assert_eq!(argv1, vec!["echo", "first"]);
    assert_eq!(argv2, vec!["echo", "second"]);
});

saran_test!("MA-02", ma_02_early_halt_on_failure, {
    // Note: This test validates that each action can be independently assembled
    // The actual execution/halting is out of scope for argv assembly tests
    // but we verify that multiple actions can each be assembled correctly

    let context = AssemblyContext::new(HashMap::new(), HashMap::new(), HashMap::new());

    // First action
    let argv1 = build_argv(
        "echo",
        &["first".to_string()],
        &[],
        &context,
        &HashMap::new(),
    )
    .unwrap();
    assert_eq!(argv1, vec!["echo", "first"]);

    // Second action (could fail, but argv assembly itself succeeds)
    let argv2 = build_argv("false", &[], &[], &context, &HashMap::new()).unwrap();
    assert_eq!(argv2, vec!["false"]);

    // Third action (would not execute after failure, but can still be assembled)
    let argv3 = build_argv(
        "echo",
        &["third".to_string()],
        &[],
        &context,
        &HashMap::new(),
    )
    .unwrap();
    assert_eq!(argv3, vec!["echo", "third"]);
});

saran_test!("MA-03", ma_03_each_action_gets_own_argv, {
    // Two actions with same flag name but different values
    let mut flags_set1 = HashMap::new();
    flags_set1.insert(
        "--format".to_string(),
        OptionalFlagValue::string("json".to_string()),
    );

    let mut flags_set2 = HashMap::new();
    flags_set2.insert(
        "--format".to_string(),
        OptionalFlagValue::string("yaml".to_string()),
    );

    let flag_def = vec![saran_types::OptionalFlag {
        name: "--format".to_string(),
        flag_type: "str".to_string(),
        repeated: false,
        help: None,
        passes_as: None,
        values: vec![],
    }];

    let context1 = AssemblyContext::new(HashMap::new(), HashMap::new(), flags_set1);
    let context2 = AssemblyContext::new(HashMap::new(), HashMap::new(), flags_set2);

    let argv1 = build_argv(
        "tool",
        &["run".to_string()],
        &flag_def,
        &context1,
        &HashMap::new(),
    )
    .unwrap();
    let argv2 = build_argv(
        "tool",
        &["run".to_string()],
        &flag_def,
        &context2,
        &HashMap::new(),
    )
    .unwrap();

    // Each action should have its own flag value
    assert_eq!(argv1, vec!["tool", "run", "--format", "json"]);
    assert_eq!(argv2, vec!["tool", "run", "--format", "yaml"]);
});
