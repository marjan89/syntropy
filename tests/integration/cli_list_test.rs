//! Integration tests for CLI list subcommand
//!
//! Tests plugin and task discovery without launching the TUI.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

// ============================================================================
// Config and Plugin Constants
// ============================================================================

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

/// Plugin with two tasks: one multi-mode with item_sources, one execute-only.
const RICH_PLUGIN: &str = r#"
return {
    metadata = {
        name = "list-test-plugin",
        version = "1.2.3",
        icon = "L",
        description = "A plugin for list testing",
        platforms = {"macos", "linux"},
    },
    tasks = {
        multi_task = {
            name = "Multi Task",
            description = "A task with item sources and multi mode",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b"} end,
                    execute = function(items) return "ok", 0 end,
                },
            },
        },
        standalone = {
            name = "Standalone Task",
            description = "An execute-only task with no item sources",
            execute = function() return "done", 0 end,
        },
    },
}
"#;

const SECOND_PLUGIN: &str = r#"
return {
    metadata = {
        name = "second-plugin",
        version = "0.5.0",
        icon = "S",
        description = "The second plugin",
        platforms = {"macos", "linux"},
    },
    tasks = {
        only_task = {
            name = "Only Task",
            description = "The only task in this plugin",
            execute = function() return "ok", 0 end,
        },
    },
}
"#;

/// Plugins whose names sort differently from filesystem insertion order (W1, W2).
const ZEBRA_PLUGIN: &str = r#"
return {
    metadata = {
        name = "zebra-plugin",
        version = "1.0.0",
        icon = "Z",
        description = "The zebra plugin",
        platforms = {"macos", "linux"},
    },
    tasks = {
        zebra_task = {
            description = "The zebra task",
            execute = function() return "ok", 0 end,
        },
    },
}
"#;

const ALPHA_PLUGIN: &str = r#"
return {
    metadata = {
        name = "alpha-plugin",
        version = "1.0.0",
        icon = "A",
        description = "The alpha plugin",
        platforms = {"macos", "linux"},
    },
    tasks = {
        alpha_task = {
            description = "The alpha task",
            execute = function() return "ok", 0 end,
        },
    },
}
"#;

/// Plugin with a task whose `name` field is explicitly set to empty string (W3).
const PLUGIN_WITH_EMPTY_TASK_NAME: &str = r#"
return {
    metadata = {
        name = "empty-name-plugin",
        version = "1.0.0",
        icon = "E",
        description = "Plugin with a task that has an empty name field",
        platforms = {"macos", "linux"},
    },
    tasks = {
        my_task = {
            name = "",
            description = "A task with an empty name field",
            execute = function() return "ok", 0 end,
        },
    },
}
"#;

/// Plugin with a task that has two distinct item sources (W4).
const PLUGIN_WITH_TWO_SOURCES: &str = r#"
return {
    metadata = {
        name = "multisource-plugin",
        version = "1.0.0",
        icon = "M",
        description = "Plugin with a task that has two item sources",
        platforms = {"macos", "linux"},
    },
    tasks = {
        dual_task = {
            name = "Dual Source Task",
            description = "A task with two item sources",
            mode = "multi",
            item_sources = {
                source_one = {
                    tag = "one",
                    items = function() return {"a", "b"} end,
                    execute = function(items) return "ok", 0 end,
                },
                source_two = {
                    tag = "two",
                    items = function() return {"c", "d"} end,
                    execute = function(items) return "ok", 0 end,
                },
            },
        },
    },
}
"#;

// ============================================================================
// syntropy list — Plugin Discovery
// ============================================================================

#[test]
fn test_list_no_plugins_exits_successfully() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_list_shows_plugin_name() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("list-test-plugin"));
}

#[test]
fn test_list_shows_plugin_version() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("1.2.3"));
}

#[test]
fn test_list_shows_plugin_description() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("A plugin for list testing"));
}

#[test]
fn test_list_shows_all_plugins() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);
    fixture.create_plugin("second-plugin", SECOND_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("list-test-plugin"))
        .stdout(predicate::str::contains("second-plugin"));
}

// ============================================================================
// syntropy list --plugin NAME — Task Discovery
// ============================================================================

#[test]
fn test_list_plugin_shows_task_keys() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "list-test-plugin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("multi_task"))
        .stdout(predicate::str::contains("standalone"));
}

#[test]
fn test_list_plugin_shows_task_descriptions() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "list-test-plugin"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A task with item sources and multi mode",
        ))
        .stdout(predicate::str::contains(
            "An execute-only task with no item sources",
        ));
}

#[test]
fn test_list_plugin_does_not_show_other_plugin_tasks() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);
    fixture.create_plugin("second-plugin", SECOND_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "list-test-plugin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("only_task").not());
}

#[test]
fn test_list_plugin_not_found_fails() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

#[test]
fn test_list_plugin_not_found_shows_available_plugins() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("list-test-plugin"));
}

// ============================================================================
// syntropy list --plugin NAME --task KEY — Task Detail
// ============================================================================

#[test]
fn test_list_task_shows_name() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "multi_task",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi Task"));
}

#[test]
fn test_list_task_shows_description() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "multi_task",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A task with item sources and multi mode",
        ));
}

#[test]
fn test_list_task_shows_mode_multi() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "multi_task",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("multi"));
}

#[test]
fn test_list_task_shows_mode_none_for_standalone() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "standalone",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("none"));
}

#[test]
fn test_list_task_with_item_sources_indicates_yes() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "multi_task",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("item_sources: 1"));
}

#[test]
fn test_list_task_without_item_sources_indicates_no() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "standalone",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("item_sources: 0"));
}

#[test]
fn test_list_task_not_found_fails() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "nonexistent",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

#[test]
fn test_list_task_not_found_shows_available_tasks() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "nonexistent",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("multi_task"))
        .stderr(predicate::str::contains("standalone"));
}

// ============================================================================
// S2: Task detail output includes the task key
// ============================================================================

#[test]
fn test_list_task_detail_includes_key() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "multi_task",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("key: multi_task"));
}

// ============================================================================
// S4: Mode field uses exact Display strings "multi" and "none"
// ============================================================================

#[test]
fn test_list_task_mode_multi_exact_label() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "multi_task",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode: multi"));
}

#[test]
fn test_list_task_mode_none_exact_label() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "list-test-plugin",
            "--task",
            "standalone",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode: none"));
}

// ============================================================================
// B1: Empty plugin list should produce a non-empty message
// ============================================================================

#[test]
fn test_list_no_plugins_shows_message() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ============================================================================
// B2: Plugin-not-found error wording is identical via --plugin and --plugin --task
// ============================================================================

#[test]
fn test_list_plugin_not_found_wording_consistent_across_paths() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("list-test-plugin", RICH_PLUGIN);

    let out_tasks = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "ghost-plugin"])
        .output()
        .unwrap();

    let out_detail = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "ghost-plugin", "--task", "some_task"])
        .output()
        .unwrap();

    let stderr_tasks = String::from_utf8_lossy(&out_tasks.stderr);
    let stderr_detail = String::from_utf8_lossy(&out_detail.stderr);

    assert!(
        stderr_tasks.contains("Plugin 'ghost-plugin' not found"),
        "list --plugin error should say: Plugin 'ghost-plugin' not found, got: {stderr_tasks}"
    );
    assert!(
        stderr_detail.contains("Plugin 'ghost-plugin' not found"),
        "list --plugin --task error should say: Plugin 'ghost-plugin' not found, got: {stderr_detail}"
    );
}

// ============================================================================
// W1: Plugins are listed in alphabetical order
// ============================================================================

#[test]
fn test_list_plugins_sorted_alphabetically() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    // Create zebra first to verify sort is not insertion-order dependent.
    fixture.create_plugin("zebra-plugin", ZEBRA_PLUGIN);
    fixture.create_plugin("alpha-plugin", ALPHA_PLUGIN);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pos_alpha = stdout
        .find("alpha-plugin")
        .expect("alpha-plugin not in output");
    let pos_zebra = stdout
        .find("zebra-plugin")
        .expect("zebra-plugin not in output");
    assert!(
        pos_alpha < pos_zebra,
        "alpha-plugin should appear before zebra-plugin in sorted output"
    );
}

// ============================================================================
// W2: Available plugins in not-found error are listed alphabetically
// ============================================================================

#[test]
fn test_list_plugin_not_found_available_plugins_sorted() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    // Create zebra first to verify sort is not insertion-order dependent.
    fixture.create_plugin("zebra-plugin", ZEBRA_PLUGIN);
    fixture.create_plugin("alpha-plugin", ALPHA_PLUGIN);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "nonexistent"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let pos_alpha = stderr
        .find("alpha-plugin")
        .expect("alpha-plugin not in error output");
    let pos_zebra = stderr
        .find("zebra-plugin")
        .expect("zebra-plugin not in error output");
    assert!(
        pos_alpha < pos_zebra,
        "alpha-plugin should appear before zebra-plugin in available plugins list"
    );
}

// ============================================================================
// W3: Task with empty name field falls back to task key
// ============================================================================

#[test]
fn test_list_task_empty_name_falls_back_to_task_key() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("empty-name-plugin", PLUGIN_WITH_EMPTY_TASK_NAME);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--plugin", "empty-name-plugin", "--task", "my_task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: my_task"));
}

// ============================================================================
// W4: Task with multiple item sources shows source count
// ============================================================================

#[test]
fn test_list_task_with_two_sources_shows_count() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multisource-plugin", PLUGIN_WITH_TWO_SOURCES);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "list",
            "--plugin",
            "multisource-plugin",
            "--task",
            "dual_task",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("item_sources: 2"));
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_list_task_flag_without_plugin_flag_fails() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["list", "--task", "some_task"])
        .assert()
        .failure();
}
