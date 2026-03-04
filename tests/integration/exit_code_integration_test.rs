//! Integration tests for exit code clamping in execution pipeline
//!
//! Verifies that exit codes outside POSIX range (0-255) are clamped
//! in ALL execution paths, not just CLI.
//!
//! **Issue**: Exit code clamping only applied in CLI execute path (execute.rs:502),
//! but not in lower-level execution handle (handle.rs:96)
//!
//! **Desired Behavior**:
//! - Negative exit codes → clamped to 1 (with warning)
//! - Exit codes > 255 → clamped to 255 (with warning)
//! - Valid exit codes (0-255) → pass through unchanged
//! - Warning logged when clamping occurs

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

#[test]
fn test_exit_code_negative_clamped_to_one() {
    // Plugin returns -1, should be clamped to 1
    const NEGATIVE_EXIT: &str = r#"
return {
    metadata = {
        name = "test",
        version = "1.0.0",
        icon = "T",
        description = "Test plugin",
        platforms = {"macos", "linux"}
    },
    tasks = {
        negative = {
            description = "Returns negative exit code",
            execute = function() return "output", -1 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", NEGATIVE_EXIT);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("negative")
        .assert()
        .code(1) // Should be clamped to 1
        .stderr(predicate::str::contains(
            "Warning: Exit code -1 clamped to 1",
        ));
}

#[test]
fn test_exit_code_over_255_clamped_to_255() {
    // Plugin returns 300, should be clamped to 255
    const LARGE_EXIT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", description = "Test plugin", platforms = {"macos", "linux"}},
    tasks = {
        large = {
            description = "Returns large exit code",
            execute = function() return "output", 300 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", LARGE_EXIT);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("large")
        .assert()
        .code(255) // Should be clamped to 255
        .stderr(predicate::str::contains(
            "Warning: Exit code 300 clamped to 255",
        ));
}

#[test]
fn test_exit_code_valid_range_unchanged() {
    // Plugin returns 42, should pass through unchanged
    const VALID_EXIT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", description = "Test plugin", platforms = {"macos", "linux"}},
    tasks = {
        valid = {
            description = "Returns valid exit code",
            execute = function() return "output", 42 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", VALID_EXIT);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("valid")
        .assert()
        .code(42) // Should pass through unchanged
        .stderr(predicate::str::contains("Warning").not()); // No warning
}

#[test]
fn test_exit_code_zero_success() {
    // Plugin returns 0, should pass through as success
    const SUCCESS_EXIT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", description = "Test plugin", platforms = {"macos", "linux"}},
    tasks = {
        success = {
            description = "Returns success",
            execute = function() return "output", 0 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", SUCCESS_EXIT);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("success")
        .assert()
        .code(0)
        .success();
}

#[test]
fn test_exit_code_item_source_execute_clamped() {
    // Item source execute() also needs clamping
    const ITEM_SOURCE_EXIT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", description = "Test plugin", platforms = {"macos", "linux"}},
    tasks = {
        items = {
            description = "Item source with invalid exit code",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a"} end,
                    execute = function(items) return "output", -5 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ITEM_SOURCE_EXIT);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("items")
        .arg("--items")
        .arg("a")
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "Warning: Exit code -5 clamped to 1",
        ));
}
