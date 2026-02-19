use serial_test::serial;
use std::env;
use std::fs;
use std::path::PathBuf;
use syntropy::configs::paths::{
    expand_path, find_config_file, get_default_config_dir, get_default_data_dir,
    resolve_plugin_paths,
};

#[test]
fn test_get_default_config_dir() {
    let config_dir = get_default_config_dir().expect("Should get config dir");
    assert!(config_dir.ends_with("syntropy"));
}

#[test]
fn test_get_default_data_dir() {
    let data_dir = get_default_data_dir().expect("Should get data dir");
    assert!(data_dir.ends_with("syntropy"));
}

#[test]
fn test_resolve_plugin_paths_returns_default_directories() {
    let paths = resolve_plugin_paths().expect("Should resolve paths");

    assert_eq!(paths.len(), 2);
    assert!(paths[0].ends_with("syntropy/plugins"));
    assert!(paths[1].ends_with("syntropy/plugins"));
}

#[test]
#[serial]
fn test_xdg_config_home_valid_absolute() {
    unsafe {
        env::set_var("XDG_CONFIG_HOME", "/custom/config");
    }
    let dir = get_default_config_dir().unwrap();
    assert_eq!(dir, PathBuf::from("/custom/config/syntropy"));
    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
#[serial]
fn test_xdg_config_home_empty_string() {
    unsafe {
        env::set_var("XDG_CONFIG_HOME", "");
    }
    let dir = get_default_config_dir().unwrap();
    assert!(dir.to_str().unwrap().contains(".config/syntropy"));
    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
#[serial]
fn test_xdg_config_home_relative_path() {
    unsafe {
        env::set_var("XDG_CONFIG_HOME", "relative/path");
    }
    let dir = get_default_config_dir().unwrap();
    assert!(dir.is_absolute());
    assert!(dir.to_str().unwrap().contains(".config/syntropy"));
    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
#[serial]
fn test_xdg_data_home_valid_absolute() {
    unsafe {
        env::set_var("XDG_DATA_HOME", "/custom/data");
    }
    let dir = get_default_data_dir().unwrap();
    assert_eq!(dir, PathBuf::from("/custom/data/syntropy"));
    unsafe {
        env::remove_var("XDG_DATA_HOME");
    }
}

#[test]
#[serial]
fn test_xdg_data_home_empty_string() {
    unsafe {
        env::set_var("XDG_DATA_HOME", "");
    }
    let dir = get_default_data_dir().unwrap();
    assert!(dir.to_str().unwrap().contains(".local/share/syntropy"));
    unsafe {
        env::remove_var("XDG_DATA_HOME");
    }
}

#[test]
#[serial]
fn test_xdg_data_home_relative_path() {
    unsafe {
        env::set_var("XDG_DATA_HOME", "relative/path");
    }
    let dir = get_default_data_dir().unwrap();
    assert!(dir.is_absolute());
    assert!(dir.to_str().unwrap().contains(".local/share/syntropy"));
    unsafe {
        env::remove_var("XDG_DATA_HOME");
    }
}

// ============================================================================
// find_config_file() Tests - Priority: CLI → XDG → Current Dir
// ============================================================================

#[test]
fn test_find_config_file_with_cli_path_exists() {
    // Create a temporary config file
    let temp_dir = std::env::temp_dir();
    let temp_config = temp_dir.join("test_syntropy_config.toml");
    fs::write(&temp_config, "# test config").expect("Failed to create temp config");

    // Test that CLI path takes priority and is returned
    let result = find_config_file(Some(temp_config.clone()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(temp_config.clone()));

    // Cleanup
    fs::remove_file(temp_config).ok();
}

#[test]
fn test_find_config_file_with_cli_path_missing() {
    let nonexistent = PathBuf::from("/tmp/nonexistent_syntropy_config_12345.toml");

    let result = find_config_file(Some(nonexistent.clone()));
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("does not exist"),
        "Error should mention file doesn't exist: {}",
        err_msg
    );
    assert!(
        err_msg.contains(&nonexistent.to_string_lossy().to_string()),
        "Error should show the path: {}",
        err_msg
    );
}

#[test]
#[serial]
fn test_find_config_file_current_directory() {
    // Save current directory
    let original_dir = env::current_dir().expect("Failed to get current dir");

    // Create temp directory and make it current
    let temp_dir = std::env::temp_dir().join("syntropy_test_current_dir");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    env::set_current_dir(&temp_dir).expect("Failed to change dir");

    // Create config in current directory
    let local_config = PathBuf::from("./syntropy.toml");
    fs::write(&local_config, "# test config").expect("Failed to write config");

    // Test without CLI arg - should find current directory config
    let result = find_config_file(None);
    assert!(
        result.is_ok(),
        "Should find config in current dir: {:?}",
        result
    );
    assert!(
        result.unwrap().is_some(),
        "Should return Some(path) when config exists in current dir"
    );

    // Cleanup
    env::set_current_dir(original_dir).ok();
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
#[serial]
fn test_find_config_file_none_found() {
    // Save current directory
    let original_dir = env::current_dir().expect("Failed to get current dir");

    // Create empty temp directory without config
    let temp_dir = std::env::temp_dir().join("syntropy_test_no_config");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    env::set_current_dir(&temp_dir).expect("Failed to change dir");

    // Clear XDG env vars to avoid finding real config
    let saved_xdg = env::var("XDG_CONFIG_HOME").ok();
    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.join("nonexistent_xdg"));
    }

    let result = find_config_file(None);
    assert!(result.is_ok(), "Should succeed even when no config found");
    assert_eq!(
        result.unwrap(),
        None,
        "Should return None when no config found via auto-discovery"
    );

    // Cleanup
    env::set_current_dir(original_dir).ok();
    if let Some(val) = saved_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", val);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
#[serial]
fn test_find_config_file_cli_priority_over_others() {
    // Create CLI config in temp location
    let temp_dir = std::env::temp_dir();
    let cli_config = temp_dir.join("cli_priority_test.toml");
    fs::write(&cli_config, "# cli config").expect("Failed to create CLI config");

    // Save and change current directory to also have a config
    let original_dir = env::current_dir().expect("Failed to get current dir");
    let current_dir_test = temp_dir.join("syntropy_test_cli_priority");
    fs::create_dir_all(&current_dir_test).expect("Failed to create temp dir");
    env::set_current_dir(&current_dir_test).expect("Failed to change dir");

    // Create config in current directory
    let local_config = PathBuf::from("./syntropy.toml");
    fs::write(&local_config, "# local config").expect("Failed to write local config");

    // When CLI path is provided, it should take priority
    let result = find_config_file(Some(cli_config.clone()));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Some(cli_config.clone()),
        "CLI path should take priority over current directory"
    );

    // Cleanup
    env::set_current_dir(original_dir).ok();
    fs::remove_file(&cli_config).ok();
    fs::remove_dir_all(&current_dir_test).ok();
}

// ============================================================================
// Edge Case Tests - XDG and Path Validation
// ============================================================================

#[test]
#[serial]
fn test_xdg_config_home_whitespace_only() {
    // Test XDG environment variable with only whitespace
    // Per XDG spec, this should be treated as invalid and fallback to default
    unsafe {
        env::set_var("XDG_CONFIG_HOME", "   ");
    }

    let result = get_default_config_dir();

    // Should fallback to default ~/.config/syntropy instead of using "   /syntropy"
    assert!(result.is_ok(), "Should handle whitespace-only XDG value");

    let dir = result.unwrap();
    assert!(
        dir.to_str().unwrap().contains(".config/syntropy"),
        "Should fallback to default config directory, got: {:?}",
        dir
    );

    // Cleanup
    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
#[serial]
fn test_xdg_config_home_trailing_slash() {
    // Test XDG with trailing slash - should not cause double slash
    unsafe {
        env::set_var("XDG_CONFIG_HOME", "/custom/config/");
    }

    let dir = get_default_config_dir().unwrap();

    // PathBuf::join should handle trailing slash correctly
    assert_eq!(dir, PathBuf::from("/custom/config/syntropy"));

    let path_str = dir.to_string_lossy();
    assert!(
        !path_str.contains("//"),
        "Should not have double slash: {}",
        path_str
    );

    // Cleanup
    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
#[serial]
fn test_get_default_config_dir_home_unavailable() {
    // NOTE: This test is challenging because dirs::home_dir() uses system APIs
    // We can't easily mock it without extensive setup
    // This test documents expected behavior:
    //
    // EXPECTED: If HOME is unset/invalid, get_default_config_dir() should return:
    //   Err("Failed to determine home directory")
    //
    // CURRENT: Uses dirs::home_dir() which handles edge cases internally
    //
    // The actual error would occur at line 35 in paths.rs:
    //   .context("Failed to determine home directory")
    //
    // Since we can't reliably test this without mocking, we document it here
    // and trust that dirs crate handles edge cases correctly.

    // This test just validates normal operation
    let result = get_default_config_dir();
    assert!(
        result.is_ok(),
        "Should successfully get config dir in normal conditions"
    );
}

// ============================================================================
// expand_path() Tests - Tilde and Environment Variable Expansion
// ============================================================================

#[test]
fn test_expand_path_tilde() {
    let path = PathBuf::from("~/test.toml");
    let expanded = expand_path(path).expect("Failed to expand tilde path");

    // Should not contain literal tilde
    assert!(!expanded.to_string_lossy().contains('~'));

    // Should be an absolute path
    assert!(expanded.is_absolute());

    // Should end with test.toml
    assert_eq!(
        expanded
            .file_name()
            .expect("expanded path should have a file name"),
        "test.toml"
    );
}

#[test]
fn test_expand_path_tilde_with_subdirs() {
    let path = PathBuf::from("~/.config/syntropy/test.toml");
    let expanded = expand_path(path).expect("Failed to expand tilde path with subdirs");

    // Should not contain literal tilde
    assert!(!expanded.to_string_lossy().contains('~'));

    // Should be an absolute path
    assert!(expanded.is_absolute());

    // Should end with test.toml
    assert_eq!(
        expanded
            .file_name()
            .expect("expanded path should have a file name"),
        "test.toml"
    );

    // Should contain .config/syntropy in the path
    assert!(expanded.to_string_lossy().contains(".config"));
    assert!(expanded.to_string_lossy().contains("syntropy"));
}

#[test]
fn test_expand_path_home_env_var() {
    let path = PathBuf::from("$HOME/.config/syntropy.toml");
    let expanded = expand_path(path).expect("Failed to expand $HOME path");

    // Should not contain literal $HOME
    assert!(!expanded.to_string_lossy().contains("$HOME"));

    // Should be an absolute path
    assert!(expanded.is_absolute());

    // Should end with .config/syntropy.toml
    assert!(
        expanded
            .to_string_lossy()
            .ends_with(".config/syntropy.toml")
    );
}

#[test]
fn test_expand_path_absolute_unchanged() {
    let path = PathBuf::from("/absolute/path/config.toml");
    let expanded = expand_path(path.clone()).expect("Failed to expand absolute path");

    // Should remain unchanged
    assert_eq!(expanded, path);
}

#[test]
fn test_expand_path_relative_unchanged() {
    let path = PathBuf::from("relative/path/config.toml");
    let expanded = expand_path(path.clone()).expect("Failed to expand relative path");

    // Should remain unchanged
    assert_eq!(expanded, path);
}
