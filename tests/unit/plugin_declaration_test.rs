// Unit tests for PluginDeclaration validation and tag comparison utilities.
//
// This file tests low-level validation logic and helper functions:
// - PluginDeclaration::validate() - Validates plugin declarations from TOML
// - compare_tags() - Semantic version comparison for git tags
//
// For CLI plugin management command tests, see:
// tests/integration/plugin_manager_test.rs

use std::cmp::Ordering;
use syntropy::{cli::plugins::compare_tags, configs::PluginDeclaration};

// ============================================================================
// compare_tags() tests
// ============================================================================

#[test]
fn test_compare_tags_semver_basic() {
    assert_eq!(compare_tags("v1.0.0", "v1.0.1"), Ordering::Less);
    assert_eq!(compare_tags("v1.0.1", "v1.0.0"), Ordering::Greater);
    assert_eq!(compare_tags("v1.0.0", "v1.0.0"), Ordering::Equal);
}

#[test]
fn test_compare_tags_semver_without_v_prefix() {
    assert_eq!(compare_tags("1.0.0", "1.0.1"), Ordering::Less);
    assert_eq!(compare_tags("1.2.0", "1.1.0"), Ordering::Greater);
    assert_eq!(compare_tags("2.0.0", "2.0.0"), Ordering::Equal);
}

#[test]
fn test_compare_tags_semver_mixed_v_prefix() {
    assert_eq!(compare_tags("v1.0.0", "1.0.1"), Ordering::Less);
    assert_eq!(compare_tags("1.2.0", "v1.1.0"), Ordering::Greater);
    assert_eq!(compare_tags("v2.0.0", "2.0.0"), Ordering::Equal);
}

#[test]
fn test_compare_tags_semver_major_versions() {
    assert_eq!(compare_tags("v1.0.0", "v2.0.0"), Ordering::Less);
    assert_eq!(compare_tags("v3.0.0", "v2.0.0"), Ordering::Greater);
}

#[test]
fn test_compare_tags_semver_with_prerelease() {
    assert_eq!(compare_tags("v1.0.0-alpha", "v1.0.0"), Ordering::Less);
    assert_eq!(compare_tags("v1.0.0", "v1.0.0-beta"), Ordering::Greater);
    assert_eq!(compare_tags("v1.0.0-alpha", "v1.0.0-beta"), Ordering::Less);
}

#[test]
fn test_compare_tags_non_semver_fallback_to_string() {
    // Non-semver tags fall back to string comparison
    assert_eq!(compare_tags("release-a", "release-b"), Ordering::Less);
    assert_eq!(compare_tags("release-b", "release-a"), Ordering::Greater);
    assert_eq!(compare_tags("release-a", "release-a"), Ordering::Equal);
}

#[test]
fn test_compare_tags_invalid_semver_falls_back() {
    // Invalid semver should fall back to string comparison
    assert_eq!(compare_tags("1.0", "1.0.0"), Ordering::Less); // String: "1.0" < "1.0.0"
    assert_eq!(compare_tags("v1.x", "v1.0.0"), Ordering::Greater); // String: "v1.x" > "v1.0.0"
}

// ============================================================================
// PluginDeclaration::validate() tests
// ============================================================================

#[test]
fn test_plugin_declaration_valid_with_tag() {
    let decl = PluginDeclaration {
        git: "https://github.com/user/repo".to_string(),
        tag: Some("v1.0.0".to_string()),
        commit: None,
    };
    assert!(decl.validate().is_ok());
}

#[test]
fn test_plugin_declaration_valid_with_commit() {
    let decl = PluginDeclaration {
        git: "https://github.com/user/repo".to_string(),
        tag: None,
        commit: Some("abc123def456".to_string()),
    };
    assert!(decl.validate().is_ok());
}

#[test]
fn test_plugin_declaration_valid_git_ssh_format() {
    let decl = PluginDeclaration {
        git: "git@github.com:user/repo".to_string(),
        tag: Some("v1.0.0".to_string()),
        commit: None,
    };
    assert!(decl.validate().is_ok());
}

#[test]
fn test_plugin_declaration_empty_git_url_fails() {
    let decl = PluginDeclaration {
        git: "".to_string(),
        tag: Some("v1.0.0".to_string()),
        commit: None,
    };
    let result = decl.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("git URL cannot be empty")
    );
}

#[test]
fn test_plugin_declaration_invalid_git_url_format() {
    let decl = PluginDeclaration {
        git: "http://github.com/user/repo".to_string(), // http:// not allowed
        tag: Some("v1.0.0".to_string()),
        commit: None,
    };
    let result = decl.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid git URL format")
    );
}

#[test]
fn test_plugin_declaration_both_tag_and_commit_fails() {
    let decl = PluginDeclaration {
        git: "https://github.com/user/repo".to_string(),
        tag: Some("v1.0.0".to_string()),
        commit: Some("abc123".to_string()),
    };
    let result = decl.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("must not declare both tag and commit")
    );
}

#[test]
fn test_plugin_declaration_neither_tag_nor_commit_fails() {
    let decl = PluginDeclaration {
        git: "https://github.com/user/repo".to_string(),
        tag: None,
        commit: None,
    };
    let result = decl.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("must specify either tag or commit")
    );
}

#[test]
fn test_plugin_declaration_file_path_fails() {
    let decl = PluginDeclaration {
        git: "/local/path/to/repo".to_string(),
        tag: Some("v1.0.0".to_string()),
        commit: None,
    };
    let result = decl.validate();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid git URL format")
    );
}
