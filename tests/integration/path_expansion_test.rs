//! Integration tests for path expansion (tilde and environment variables)
//!
//! Tests that --config and validate subcommand paths properly expand
//! tilde (~) and environment variables ($VAR).

use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs;

use crate::common::TestFixture;

const MINIMAL_CONFIG: &str = "";

// ============================================================================
// Tilde Expansion Tests
// ============================================================================

#[test]
fn test_config_path_with_tilde_expansion() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Create a config file in a known location
    let home = dirs::home_dir().expect("Failed to get home dir");
    let config_subdir = home.join(".test-syntropy-config-tilde");
    fs::create_dir_all(&config_subdir).expect("Failed to create test dir");
    let config_file = config_subdir.join("test-config.toml");
    fs::write(&config_file, MINIMAL_CONFIG).expect("Failed to write config");

    // Use tilde in path
    let tilde_path = "~/.test-syntropy-config-tilde/test-config.toml";

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--config")
        .arg(tilde_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));

    // Clean up
    fs::remove_dir_all(&config_subdir).ok();
}

#[test]
fn test_validate_config_with_tilde() {
    let home = dirs::home_dir().expect("Failed to get home dir");
    let config_subdir = home.join(".test-syntropy-validate-tilde");
    fs::create_dir_all(&config_subdir).expect("Failed to create test dir");
    let config_file = config_subdir.join("validate-test.toml");
    fs::write(&config_file, MINIMAL_CONFIG).expect("Failed to write config");

    // Use tilde in path
    let tilde_path = "~/.test-syntropy-validate-tilde/validate-test.toml";

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--config")
        .arg(tilde_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));

    // Clean up
    fs::remove_dir_all(&config_subdir).ok();
}

// ============================================================================
// Environment Variable Expansion Tests
// ============================================================================

#[test]
fn test_config_path_with_home_env_var() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Create a config file in a known location
    let home = env::var("HOME").expect("HOME not set");
    let config_subdir = format!("{}/.test-syntropy-config-home", home);
    fs::create_dir_all(&config_subdir).expect("Failed to create test dir");
    let config_file = format!("{}/test-config.toml", config_subdir);
    fs::write(&config_file, MINIMAL_CONFIG).expect("Failed to write config");

    // Use $HOME in path
    let env_path = "$HOME/.test-syntropy-config-home/test-config.toml";

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--config")
        .arg(env_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));

    // Clean up
    fs::remove_dir_all(&config_subdir).ok();
}

#[test]
fn test_config_path_with_custom_env_var() {
    let fixture = TestFixture::new();
    let custom_dir = fixture.config_path().join("custom");
    fs::create_dir_all(&custom_dir).expect("Failed to create custom dir");
    let config_file = custom_dir.join("syntropy.toml");
    fs::write(&config_file, MINIMAL_CONFIG).expect("Failed to write config");

    // Use custom environment variable
    let custom_dir_str = custom_dir.to_str().expect("Invalid path");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("TEST_SYNTROPY_CONFIG_DIR", custom_dir_str)
        .arg("validate")
        .arg("--config")
        .arg("$TEST_SYNTROPY_CONFIG_DIR/syntropy.toml")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));
}

#[test]
fn test_config_path_with_braced_env_var() {
    let fixture = TestFixture::new();
    let custom_dir = fixture.config_path().join("braced");
    fs::create_dir_all(&custom_dir).expect("Failed to create custom dir");
    let config_file = custom_dir.join("syntropy.toml");
    fs::write(&config_file, MINIMAL_CONFIG).expect("Failed to write config");

    // Use braced syntax ${VAR}
    let custom_dir_str = custom_dir.to_str().expect("Invalid path");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("TEST_SYNTROPY_BRACED_DIR", custom_dir_str)
        .arg("validate")
        .arg("--config")
        .arg("${TEST_SYNTROPY_BRACED_DIR}/syntropy.toml")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));
}

// ============================================================================
// Plugin Path Expansion Tests
// ============================================================================

#[test]
fn test_validate_plugin_with_tilde() {
    let home = dirs::home_dir().expect("Failed to get home dir");
    let plugin_subdir = home.join(".test-syntropy-plugin-tilde");
    fs::create_dir_all(&plugin_subdir).expect("Failed to create test dir");
    let plugin_file = plugin_subdir.join("plugin.lua");

    // Use a simple single-cell icon instead of the emoji
    let simple_plugin = r#"
return {
    metadata = {
        name = "test-plugin",
        version = "1.0.0",
        icon = "T",
        description = "Test plugin for path expansion",
        platforms = {"macos", "linux"},
    },
    tasks = {
        test_task = {
            description = "Test task",
            name = "Test Task",
            mode = "none",
            item_sources = {
                test_source = {
                    tag = "t",
                    items = function()
                        return {"item1"}
                    end,
                    execute = function(items)
                        return "OK", 0
                    end,
                },
            },
        },
    },
}
"#;

    fs::write(&plugin_file, simple_plugin).expect("Failed to write plugin");

    // Use tilde in path
    let tilde_path = "~/.test-syntropy-plugin-tilde";

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(tilde_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));

    // Clean up
    fs::remove_dir_all(&plugin_subdir).ok();
}

#[test]
fn test_validate_plugin_with_env_var() {
    let fixture = TestFixture::new();
    let plugin_dir = fixture.data_path().join("test-plugin");
    fs::create_dir_all(&plugin_dir).expect("Failed to create plugin dir");
    let plugin_file = plugin_dir.join("plugin.lua");

    // Use a simple single-cell icon
    let simple_plugin = r#"
return {
    metadata = {
        name = "test-plugin",
        version = "1.0.0",
        icon = "P",
        description = "Test plugin for path expansion",
        platforms = {"macos", "linux"},
    },
    tasks = {
        test_task = {
            description = "Test task",
            name = "Test Task",
            mode = "none",
            item_sources = {
                test_source = {
                    tag = "t",
                    items = function()
                        return {"item1"}
                    end,
                    execute = function(items)
                        return "OK", 0
                    end,
                },
            },
        },
    },
}
"#;

    fs::write(&plugin_file, simple_plugin).expect("Failed to write plugin");

    let plugin_dir_str = plugin_dir.to_str().expect("Invalid path");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("TEST_PLUGIN_DIR", plugin_dir_str)
        .arg("validate")
        .arg("--plugin")
        .arg("$TEST_PLUGIN_DIR")
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_undefined_env_var_fails() {
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--config")
        .arg("$UNDEFINED_SYNTROPY_VAR_12345/config.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to expand"));
}

#[test]
fn test_tilde_expanded_file_not_found() {
    // Use a tilde path that will expand but won't exist
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--config")
        .arg("~/nonexistent-syntropy-dir-12345/config.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Config file not found"));
}
