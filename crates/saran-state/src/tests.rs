//! Unit tests for `saran-state` crate.
//!
//! Tests follow the specifications in `spec/tests/unit/06-state-management.md`.

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::*;
use saran_test::saran_test;

// ============================================================================
// Test Utilities
// ============================================================================

/// Creates a state instance with a custom data directory for testing.
fn state_with_temp_dir() -> (SaranState, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let dir = temp.path().to_path_buf();

    // Use the test-only constructor to create state with temp directory
    let state = SaranState::with_data_dir(dir);

    (state, temp)
}

// ============================================================================
// env.yaml Write Operations (W-01 to W-05)
// ============================================================================

saran_test!("W-01", w_01_set_global_variable, {
    let (state, _temp) = state_with_temp_dir();

    state.set_global_var("GH_REPO", "myorg/myrepo").unwrap();

    let content = fs::read_to_string(state.data_dir().join("env.yaml")).unwrap();
    let env_yaml: SaranEnvYaml = serde_yaml::from_str(&content).unwrap();

    assert_eq!(
        env_yaml.global.get("GH_REPO"),
        Some(&"myorg/myrepo".to_string())
    );
});

saran_test!("W-02", w_02_set_wrapper_variable, {
    let (state, _temp) = state_with_temp_dir();

    state
        .set_wrapper_var("gh-pr.ro", "GH_REPO", "myorg/myrepo")
        .unwrap();

    let content = fs::read_to_string(state.data_dir().join("env.yaml")).unwrap();
    let env_yaml: SaranEnvYaml = serde_yaml::from_str(&content).unwrap();

    let wrapper_vars = env_yaml.wrappers.get("gh-pr.ro");
    assert!(wrapper_vars.is_some());
    assert_eq!(
        wrapper_vars.unwrap().get("GH_REPO"),
        Some(&"myorg/myrepo".to_string())
    );
});

saran_test!("W-03", w_03_set_multiple_vars, {
    let (state, _temp) = state_with_temp_dir();

    state
        .set_wrapper_var("gh-pr.ro", "GH_REPO", "myorg/myrepo")
        .unwrap();
    state
        .set_wrapper_var("gh-pr.ro", "GH_TOKEN", "gho_xxx")
        .unwrap();

    let content = fs::read_to_string(state.data_dir().join("env.yaml")).unwrap();
    let env_yaml: SaranEnvYaml = serde_yaml::from_str(&content).unwrap();

    let wrapper_vars = env_yaml.wrappers.get("gh-pr.ro").unwrap();
    assert_eq!(
        wrapper_vars.get("GH_REPO"),
        Some(&"myorg/myrepo".to_string())
    );
    assert_eq!(wrapper_vars.get("GH_TOKEN"), Some(&"gho_xxx".to_string()));
});

saran_test!("W-04", w_04_overwrite_existing, {
    let (state, _temp) = state_with_temp_dir();

    state
        .set_global_var("GH_REPO", "firstorg/firstrepo")
        .unwrap();
    state
        .set_global_var("GH_REPO", "secondorg/secondrepo")
        .unwrap();

    let content = fs::read_to_string(state.data_dir().join("env.yaml")).unwrap();
    let env_yaml: SaranEnvYaml = serde_yaml::from_str(&content).unwrap();

    assert_eq!(
        env_yaml.global.get("GH_REPO"),
        Some(&"secondorg/secondrepo".to_string())
    );
});

saran_test!("W-05", w_05_file_created_if_missing, {
    let (state, _temp) = state_with_temp_dir();

    assert!(!state.data_dir().join("env.yaml").exists());
    state.set_global_var("GH_REPO", "myorg/myrepo").unwrap();
    assert!(state.data_dir().join("env.yaml").exists());
});

// ============================================================================
// env.yaml Unset Operations (U-01 to U-04)
// ============================================================================

saran_test!("U-01", u_01_unset_global_variable, {
    let (state, _temp) = state_with_temp_dir();

    state.set_global_var("GH_REPO", "myorg/myrepo").unwrap();
    state.unset_global_var("GH_REPO").unwrap();

    let env = state.get_env().unwrap();
    assert!(!env.global.contains_key("GH_REPO"));
});

saran_test!("U-02", u_02_unset_wrapper_variable, {
    let (state, _temp) = state_with_temp_dir();

    state
        .set_wrapper_var("gh-pr.ro", "GH_REPO", "myorg/myrepo")
        .unwrap();
    state.unset_wrapper_var("gh-pr.ro", "GH_REPO").unwrap();

    let env = state.get_env().unwrap();
    let wrapper_vars = env.wrappers.get("gh-pr.ro");
    assert!(wrapper_vars
        .map(|m| !m.contains_key("GH_REPO"))
        .unwrap_or(true));
});

saran_test!("U-03", u_03_unset_nonexistent_var, {
    let (state, _temp) = state_with_temp_dir();

    state.unset_global_var("NONEXISTENT").unwrap();
});

saran_test!("U-04", u_04_unset_cascades_to_fallback, {
    let (state, _temp) = state_with_temp_dir();

    state
        .set_wrapper_var("gh-pr.ro", "GH_REPO", "override")
        .unwrap();
    state.unset_wrapper_var("gh-pr.ro", "GH_REPO").unwrap();

    let env = state.get_env().unwrap();
    let wrapper_vars = env.wrappers.get("gh-pr.ro");
    assert!(wrapper_vars.map(|m| m.is_empty()).unwrap_or(true));
});

// ============================================================================
// env.yaml Read Operations (R-01 to R-04)
// ============================================================================

saran_test!("R-01", r_01_empty_env_yaml, {
    let (state, _temp) = state_with_temp_dir();

    let env = state.get_env().unwrap();
    assert!(env.global.is_empty());
    assert!(env.wrappers.is_empty());
});

saran_test!("R-02", r_02_only_global_section, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = "global:\n  GH_REPO: myorg/myrepo\n";
    fs::write(state.data_dir().join("env.yaml"), yaml_content).unwrap();

    let env = state.get_env().unwrap();

    assert!(env.wrappers.is_empty());
});

saran_test!("R-03", r_03_only_wrappers_section, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = "wrappers:\n  gh-pr.ro:\n    GH_REPO: myorg/myrepo\n";
    fs::write(state.data_dir().join("env.yaml"), yaml_content).unwrap();

    let env = state.get_env().unwrap();

    assert!(env.global.is_empty());
    assert!(env.wrappers.contains_key("gh-pr.ro"));
});

saran_test!("R-04", r_04_both_sections_populated, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = r#"
global:
  GH_DEBUG: "1"
wrappers:
  gh-pr.ro:
    GH_REPO: myorg/myrepo
"#;
    fs::write(state.data_dir().join("env.yaml"), yaml_content).unwrap();

    let env = state.get_env().unwrap();

    assert_eq!(env.global.get("GH_DEBUG"), Some(&"1".to_string()));
    assert_eq!(
        env.wrappers.get("gh-pr.ro").unwrap().get("GH_REPO"),
        Some(&"myorg/myrepo".to_string())
    );
});

// ============================================================================
// quotas.yaml Read Operations (Q-01 to Q-04)
// ============================================================================

saran_test!("Q-01", q_01_read_wrapper_quotas, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = r#"
gh-pr.ro:
  comment:
    remaining: 3
    limit: 5
  list:
    remaining: 10
    limit: 10
"#;
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    let quotas = state.get_quotas().unwrap();

    let wrapper_quotas = quotas.get("gh-pr.ro").unwrap();
    assert_eq!(wrapper_quotas.get("comment").unwrap().remaining, 3);
    assert_eq!(wrapper_quotas.get("list").unwrap().remaining, 10);
});

saran_test!("Q-02", q_02_read_single_action, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = "gh-pr.ro:\n  comment:\n    remaining: 2\n    limit: 5\n";
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    let wrapper_quotas = state.get_wrapper_quotas("gh-pr.ro").unwrap();

    let inner = wrapper_quotas.unwrap();
    let comment_quota = inner.get("comment").unwrap();
    assert_eq!(comment_quota.remaining, 2);
    assert_eq!(comment_quota.limit, 5);
});

saran_test!("Q-03", q_03_read_nonexistent_wrapper, {
    let (state, _temp) = state_with_temp_dir();

    let wrapper_quotas = state.get_wrapper_quotas("nonexistent").unwrap();
    assert!(wrapper_quotas.is_none());
});

saran_test!("Q-04", q_04_empty_quotas_yaml, {
    let (state, _temp) = state_with_temp_dir();

    let quotas = state.get_quotas().unwrap();
    assert!(quotas.is_empty());
});

// ============================================================================
// quotas.yaml Decrement Operations (D-01 to D-04)
// ============================================================================

saran_test!("D-01", d_01_decrement_from_positive, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = "gh-pr.ro:\n  comment:\n    remaining: 3\n    limit: 5\n";
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    state.decrement_quota("gh-pr.ro", "comment").unwrap();

    let wrapper_quotas = state.get_wrapper_quotas("gh-pr.ro").unwrap().unwrap();
    assert_eq!(wrapper_quotas.get("comment").unwrap().remaining, 2);
});

saran_test!("D-02", d_02_decrement_to_zero, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = "gh-pr.ro:\n  comment:\n    remaining: 1\n    limit: 5\n";
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    state.decrement_quota("gh-pr.ro", "comment").unwrap();

    let wrapper_quotas = state.get_wrapper_quotas("gh-pr.ro").unwrap().unwrap();
    assert_eq!(wrapper_quotas.get("comment").unwrap().remaining, 0);
});

saran_test!("D-03", d_03_decrement_at_zero_fails, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = "gh-pr.ro:\n  comment:\n    remaining: 0\n    limit: 5\n";
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    let result = state.decrement_quota("gh-pr.ro", "comment");

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("quota exhausted"));
});

saran_test!("D-04", d_04_decrement_nonexistent_action, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = "gh-pr.ro:\n  comment:\n    remaining: 3\n    limit: 5\n";
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    let result = state.decrement_quota("gh-pr.ro", "nonexistent");

    assert!(result.is_err());
});

// ============================================================================
// quotas.yaml Reset Operations (RS-01 to RS-03)
// ============================================================================

saran_test!("RS-01", rs_01_reset_single_wrapper, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = r#"
gh-pr.ro:
  comment:
    remaining: 1
    limit: 5
  list:
    remaining: 2
    limit: 10
"#;
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    state.reset_wrapper_quotas("gh-pr.ro").unwrap();

    let wrapper_quotas = state.get_wrapper_quotas("gh-pr.ro").unwrap().unwrap();
    assert_eq!(wrapper_quotas.get("comment").unwrap().remaining, 5);
    assert_eq!(wrapper_quotas.get("list").unwrap().remaining, 10);
});

saran_test!("RS-02", rs_02_reset_all_wrappers, {
    let (state, _temp) = state_with_temp_dir();
    let yaml_content = r#"
gh-pr.ro:
  comment:
    remaining: 1
    limit: 5
glab-mr.ro:
  list:
    remaining: 2
    limit: 10
"#;
    fs::write(state.data_dir().join("quotas.yaml"), yaml_content).unwrap();

    state.reset_all_quotas().unwrap();

    let quotas = state.get_quotas().unwrap();
    assert_eq!(
        quotas
            .get("gh-pr.ro")
            .unwrap()
            .get("comment")
            .unwrap()
            .remaining,
        5
    );
    assert_eq!(
        quotas
            .get("glab-mr.ro")
            .unwrap()
            .get("list")
            .unwrap()
            .remaining,
        10
    );
});

saran_test!("RS-03", rs_03_reset_nonexistent_wrapper, {
    let (state, _temp) = state_with_temp_dir();

    state.reset_wrapper_quotas("nonexistent").unwrap();
});

// ============================================================================
// Data Directory Resolution (SD-01 to SD-03)
// ============================================================================

saran_test!("SD-01", sd_01_default_path_from_home, {
    let home = env::var("HOME").expect("HOME must be set for this test");

    let state = SaranState::new().expect("SaranState::new() failed");
    let expected = PathBuf::from(&home).join(".local/share/saran");

    // Verify data_dir() accessor returns the expected path
    assert_eq!(state.data_dir(), expected.as_path());
});

saran_test!("SD-02", sd_02_home_not_set_fails, {
    let saved_home = env::var("HOME").ok();
    env::remove_var("HOME");

    let result = SaranState::new();

    // Restore HOME before assertions so other tests are not broken
    if let Some(h) = saved_home {
        env::set_var("HOME", h);
    }

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("HOME"));
});

saran_test!("SD-03", sd_03_ensure_data_dir_creates_missing_dir, {
    let temp = tempfile::tempdir().unwrap();
    let nested = temp.path().join("deeply/nested/new-dir");

    // Create a state with the nested directory using the test-only constructor
    let state = SaranState::with_data_dir(nested.clone());

    assert!(!nested.exists(), "directory should not exist yet");

    state.ensure_data_dir().unwrap();
    assert!(nested.exists(), "directory should have been created");
});
