//! Integration tests for graceful plugin loading degradation
//!
//! Tests verify that when one plugin fails validation, other valid plugins
//! continue to load successfully. The system should skip invalid plugins
//! with warnings rather than stopping all plugin loading.
//!
//! **Implementation Status**: ✅ FIXED - Plugin loading now wraps validation in a closure
//! with graceful error handling (loader.rs:186-237), using `match` with `continue` instead
//! of the `?` operator.
//!
//! **Verified Behavior**:
//! - Invalid plugins are skipped with clear warnings
//! - Valid plugins continue to load successfully
//! - Users retain access to working plugins
//! - System remains functional despite partial failures

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

const MINIMAL_CONFIG: &str = r#"
default_plugin_icon = "⚒"

[keybindings]
back = "<esc>"
select_previous = "<up>"
select_next = "<down>"
scroll_preview_up = "["
scroll_preview_down = "]"
toggle_preview = "<C-p>"
select = "<tab>"
confirm = "<enter>"
"#;

const VALID_PLUGIN: &str = r#"
return {
    metadata = {
        name = "valid",
        version = "1.0.0",
        icon = "V",
        description = "Valid plugin",
        platforms = {"macos"}
    },
    tasks = {
        test = {
            description = "Test task",
            execute = function() return "ok", 0 end
        }
    }
}
"#;

#[test]
fn test_lua_syntax_error_skipped_gracefully() {
    // Plugin with Lua syntax error should be skipped with warning
    // Other valid plugins should load successfully
    const SYNTAX_ERROR: &str = r#"
return {
    metadata = {name = "bad" version = "1.0.0"}  -- Missing comma
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("valid-plugin", VALID_PLUGIN);
    fixture.create_plugin("syntax-error", SYNTAX_ERROR);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("valid")
        .arg("--task")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipping plugin"))
        .stderr(predicate::str::contains("syntax-error"));
}

#[test]
fn test_missing_metadata_fields_skipped_gracefully() {
    // Plugin missing required metadata (name, version) should be skipped
    // Other valid plugins should load
    const MISSING_NAME: &str = r#"
return {
    metadata = {
        version = "1.0.0",
        icon = "M",
        description = "Missing name"
    },
    tasks = {
        test = {
            description = "Test",
            execute = function() return "ok", 0 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("valid-plugin", VALID_PLUGIN);
    fixture.create_plugin("missing-name", MISSING_NAME);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("valid")
        .arg("--task")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipping plugin"))
        .stderr(predicate::str::contains("missing-name"));
}

#[test]
fn test_empty_tasks_table_skipped_gracefully() {
    // Plugin with tasks = {} should be skipped with warning
    // Other valid plugins should load
    const EMPTY_TASKS: &str = r#"
return {
    metadata = {
        name = "empty",
        version = "1.0.0",
        icon = "E",
        description = "No tasks"
    },
    tasks = {}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("valid-plugin", VALID_PLUGIN);
    fixture.create_plugin("empty-tasks", EMPTY_TASKS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("valid")
        .arg("--task")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipping plugin"))
        .stderr(predicate::str::contains("empty")); // Plugin name from metadata
}

#[test]
fn test_invalid_task_config_skipped_gracefully() {
    // Plugin with task missing description should be skipped
    // Other valid plugins should load
    const INVALID_TASK: &str = r#"
return {
    metadata = {
        name = "invalid-task",
        version = "1.0.0",
        icon = "I",
        description = "Invalid task config"
    },
    tasks = {
        bad = {
            -- Missing description
            execute = function() return "ok", 0 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("valid-plugin", VALID_PLUGIN);
    fixture.create_plugin("invalid-task", INVALID_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("valid")
        .arg("--task")
        .arg("test")
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipping plugin"))
        .stderr(predicate::str::contains("invalid-task"));
}

#[test]
fn test_multiple_plugins_partial_validation_failure() {
    // Setup: 3 plugins (valid, syntax error, missing field)
    // Expected: 1 valid plugin loads, 2 skipped with warnings
    const PLUGIN_2_SYNTAX_ERROR: &str = r#"
return {
    metadata = {bad syntax here}
}
"#;

    const PLUGIN_3_MISSING_VERSION: &str = r#"
return {
    metadata = {
        name = "no-version",
        icon = "N"
    },
    tasks = {
        test = {
            description = "Test",
            execute = function() return "ok", 0 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("valid-plugin", VALID_PLUGIN);
    fixture.create_plugin("bad-syntax", PLUGIN_2_SYNTAX_ERROR);
    fixture.create_plugin("no-version", PLUGIN_3_MISSING_VERSION);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("valid")
        .arg("--task")
        .arg("test")
        .assert()
        .success();

    // Check that both invalid plugins are skipped
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("Skipping plugin") && stderr.contains("bad-syntax"),
        "Expected warning about bad-syntax plugin"
    );
    assert!(
        stderr.contains("Skipping plugin") && stderr.contains("no-version"),
        "Expected warning about no-version plugin"
    );
}

#[test]
fn test_all_plugins_fail_validation_gracefully() {
    // All plugins invalid - should not crash, but will fail to find plugin
    const ALL_BAD_1: &str = r#"
return {
    metadata = {syntax error}
}
"#;

    const ALL_BAD_2: &str = r#"
return {
    metadata = {name = "bad2"},
    tasks = {}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("bad-1", ALL_BAD_1);
    fixture.create_plugin("bad-2", ALL_BAD_2);

    // Try to execute - should fail gracefully (plugin not found) but not crash
    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("nonexistent")
        .arg("--task")
        .arg("test")
        .assert()
        .failure(); // Fails because no valid plugins exist

    // Check that both invalid plugins were skipped
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("Skipping plugin") && stderr.contains("bad-1"),
        "Expected warning about bad-1 plugin"
    );
    assert!(
        stderr.contains("Skipping plugin") && stderr.contains("bad2") || stderr.contains("bad-2"),
        "Expected warning about bad2 plugin"
    );
}
