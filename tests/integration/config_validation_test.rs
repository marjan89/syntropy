//! Integration tests for config validation
//!
//! Tests the config loading and validation system.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

// ============================================================================
// Mock Config Templates
// ============================================================================

const MINIMAL_CONFIG: &str = "";

const COMPLETE_CONFIG: &str = r#"
default_plugin_icon = "⚒"
default_plugin = "test-plugin"
default_task = "test-task"
status_bar = true
search_bar = true
show_preview_pane = true
exit_on_execute = false

[keybindings]
back = "<esc>"
select_previous = "<up>"
select_next = "<down>"
scroll_preview_up = "["
scroll_preview_down = "]"
toggle_preview = "<C-p>"
select = "<space>"
confirm = "<enter>"

[styles.screen_scaffold]
left_split = 40
right_split = 60

[styles.status]
left_split = 30
right_split = 70

[styles.modal]
vertical_size = 80
horizontal_size = 80
"#;

const CUSTOM_KEYBINDINGS_CONFIG: &str = r#"
[keybindings]
back = "q"
select_previous = "k"
select_next = "j"
scroll_preview_up = "<C-u>"
scroll_preview_down = "<C-d>"
toggle_preview = "<S-p>"
select = "v"
confirm = "<C-S-enter>"
"#;

// ============================================================================
// Category 1: Valid Configs (3 tests)
// ============================================================================

#[test]
fn test_minimal_valid_config() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));
}

#[test]
fn test_complete_valid_config() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", COMPLETE_CONFIG);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));
}

#[test]
fn test_valid_config_with_custom_keybindings() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", CUSTOM_KEYBINDINGS_CONFIG);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config file is valid"));
}

// ============================================================================
// Category 2: Invalid TOML/Structure (3 tests)
// ============================================================================

#[test]
fn test_invalid_toml_syntax() {
    const INVALID_TOML: &str = r#"
[keybindings
back = "
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", INVALID_TOML);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse"));
}

#[test]
fn test_invalid_field_type() {
    const TYPE_MISMATCH: &str = r#"
status_bar = "yes"
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", TYPE_MISMATCH);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid type"));
}

#[test]
fn test_unknown_field_rejected() {
    const UNKNOWN_FIELD: &str = r#"
defualt_plugin = "test"
unknown_option = 42
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", UNKNOWN_FIELD);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure() // DESIRED: Should reject unknown fields
        .stderr(predicate::str::contains("unknown field"));
}

// ============================================================================
// Category 3: Invalid Semantic Rules (5 tests)
// ============================================================================

#[test]
fn test_screen_scaffold_splits_not_sum_to_100() {
    const INVALID_SPLITS: &str = r#"
[styles.screen_scaffold]
left_split = 40
right_split = 50
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", INVALID_SPLITS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Screen scaffold style left and right split must amount to 100",
        ));
}

#[test]
fn test_status_splits_not_sum_to_100() {
    const INVALID_STATUS_SPLITS: &str = r#"
[styles.status]
left_split = 60
right_split = 60
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", INVALID_STATUS_SPLITS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Status style left and right split must amount to 100",
        ));
}

#[test]
fn test_modal_size_equals_100() {
    const MODAL_SIZE_100: &str = r#"
[styles.modal]
vertical_size = 100
horizontal_size = 100
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MODAL_SIZE_100);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Modal style vertical_size and horizontal_size must not exceed 100",
        ));
}

#[test]
fn test_default_plugin_icon_multi_cell() {
    const MULTI_CELL_ICON: &str = r#"
default_plugin_icon = "🔧🎨"
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MULTI_CELL_ICON);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Default plugin icon '🔧🎨' must occupy a single terminal cell",
        ));
}

#[test]
fn test_default_task_without_default_plugin() {
    const TASK_WITHOUT_PLUGIN: &str = r#"
default_task = "export"
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", TASK_WITHOUT_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "default_task requires default_plugin to be set",
        ));
}

// ============================================================================
// Category 4: Invalid Key Bindings (4 tests - ALL WILL FAIL)
// ============================================================================

#[test]
fn test_invalid_keybinding_format() {
    const INVALID_FORMAT: &str = r#"
[keybindings]
back = "ctrl-k"
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", INVALID_FORMAT);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure() // DESIRED: Should fail validation
        .stderr(
            predicate::str::contains("Failed to parse").or(predicate::str::contains("keybinding")),
        );
}

#[test]
fn test_invalid_keybinding_unknown_key() {
    const UNKNOWN_KEY: &str = r#"
[keybindings]
back = "<foobar>"
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", UNKNOWN_KEY);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure() // DESIRED: Should fail validation
        .stderr(predicate::str::contains("Unknown key").or(predicate::str::contains("foobar")));
}

#[test]
fn test_duplicate_key_bindings() {
    const DUPLICATE_BINDINGS: &str = r#"
[keybindings]
back = "<esc>"
confirm = "<esc>"
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", DUPLICATE_BINDINGS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure() // DESIRED: Should detect and reject duplicates
        .stderr(predicate::str::contains("Duplicate").or(predicate::str::contains("conflict")));
}

#[test]
fn test_conflicting_key_bindings() {
    const CONFLICTING_BINDINGS: &str = r#"
[keybindings]
select_next = "k"
scroll_preview_up = "k"
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", CONFLICTING_BINDINGS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure() // DESIRED: Should detect and warn about conflicts
        .stderr(predicate::str::contains("Duplicate").or(predicate::str::contains("conflict")));
}

// ============================================================================
// Category 5: Edge Cases (3 tests)
// ============================================================================

#[test]
fn test_validate_config_file_not_found() {
    let fixture = TestFixture::new();
    let nonexistent = fixture
        .config_path()
        .join("nonexistent")
        .join("syntropy.toml");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--config")
        .arg(&nonexistent)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Config file not found"));
}

#[test]
fn test_validate_config_path_is_directory() {
    let fixture = TestFixture::new();
    let dir = fixture.config_path();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--config")
        .arg(&dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Path must be a file, not a directory",
        ));
}

#[test]
fn test_empty_keybinding_rejected() {
    const EMPTY_BINDING: &str = r#"
[keybindings]
back = ""
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", EMPTY_BINDING);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("validate")
        .arg("--config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty").or(predicate::str::contains("invalid")));
}
