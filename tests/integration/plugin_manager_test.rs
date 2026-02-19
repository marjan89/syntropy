// Integration tests for CLI plugin management commands.
//
// This file tests the `syntropy plugins` subcommand and its operations:
// - --list: List user/managed/orphaned plugins
// - --install: Install missing plugins from TOML declarations
// - --upgrade: Upgrade plugins to latest tags
// - --remove: Remove orphaned plugins
//
// For unit tests of PluginDeclaration validation and compare_tags(), see:
// tests/unit/plugin_declaration_test.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

use crate::common::{TestFixture, sample_plugin};

#[test]
fn test_list_plugins_empty() {
    let fixture = TestFixture::new();

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No plugins found."));
}

#[test]
fn test_list_plugins_shows_user_plugins() {
    let fixture = TestFixture::new();

    fixture.create_plugin_override("user-plugin", sample_plugin());

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("User plugins:"))
        .stdout(predicate::str::contains("user-plugin"));
}

#[test]
fn test_list_plugins_shows_managed_plugins() {
    let fixture = TestFixture::new();

    fixture.create_plugin("managed-plugin", sample_plugin());

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"

        [plugins.managed-plugin]
        git = "https://github.com/example/repo"
        tag = "v1.0.0"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Managed plugins:"))
        .stdout(predicate::str::contains("managed-plugin"))
        .stdout(predicate::str::contains("tag=v1.0.0"));
}

#[test]
fn test_list_plugins_shows_orphaned_plugins() {
    let fixture = TestFixture::new();

    fixture.create_plugin("orphaned-plugin", sample_plugin());

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Orphaned plugins:"))
        .stdout(predicate::str::contains("orphaned-plugin"))
        .stdout(predicate::str::contains("candidate for removal"));
}

#[test]
fn test_list_plugins_warns_about_user_override() {
    let fixture = TestFixture::new();

    fixture.create_plugin_override("same-plugin", sample_plugin());

    fixture.create_plugin("same-plugin", sample_plugin());

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"

        [plugins.same-plugin]
        git = "https://github.com/example/repo"
        tag = "v1.0.0"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("⚠ overrides managed plugin"))
        .stdout(predicate::str::contains("(overridden by user plugin)"));
}

#[test]
fn test_list_plugins_filters_non_plugin_directories() {
    let fixture = TestFixture::new();

    // Create valid plugin
    fixture.create_plugin("valid-plugin", sample_plugin());

    // Create non-plugin directories
    let data_plugins = fixture.data_path().join("syntropy").join("plugins");
    fs::create_dir_all(&data_plugins).unwrap();
    fs::create_dir(data_plugins.join("shared")).unwrap();
    fs::create_dir(data_plugins.join(".claude")).unwrap();

    fixture.create_config("syntropy.toml", r#"default_plugin_icon = "⚒""#);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid-plugin"))
        .stdout(predicate::str::contains("shared").not())
        .stdout(predicate::str::contains(".claude").not());
}

#[test]
fn test_list_plugins_empty_with_non_plugin_directories() {
    let fixture = TestFixture::new();

    // Create only non-plugin directories
    let data_plugins = fixture.data_path().join("syntropy").join("plugins");
    fs::create_dir_all(&data_plugins).unwrap();
    fs::create_dir(data_plugins.join("not-a-plugin")).unwrap();

    fixture.create_config("syntropy.toml", r#"default_plugin_icon = "⚒""#);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No plugins found."));
}

#[test]
fn test_list_plugins_mixed_with_non_plugin_directories() {
    let fixture = TestFixture::new();

    // Create user and managed plugins
    fixture.create_plugin_override("user-plugin", sample_plugin());
    fixture.create_plugin("managed-plugin", sample_plugin());

    // Create non-plugin directories in both locations
    let config_plugins = fixture.config_path().join("syntropy").join("plugins");
    fs::create_dir_all(&config_plugins).unwrap();
    fs::create_dir(config_plugins.join("not-user-plugin")).unwrap();

    let data_plugins = fixture.data_path().join("syntropy").join("plugins");
    fs::create_dir_all(&data_plugins).unwrap();
    fs::create_dir(data_plugins.join("not-managed-plugin")).unwrap();

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"

        [plugins.managed-plugin]
        git = "https://github.com/example/repo"
        tag = "v1.0.0"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("user-plugin"))
        .stdout(predicate::str::contains("managed-plugin"))
        .stdout(predicate::str::contains("not-user-plugin").not())
        .stdout(predicate::str::contains("not-managed-plugin").not());
}

#[test]
fn test_plugins_command_requires_exactly_one_flag() {
    let fixture = TestFixture::new();

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["plugins", "--list", "--install"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Exactly one operation flag must be specified",
        ));
}

#[test]
fn test_plugins_command_plugin_flag_requires_upgrade() {
    let fixture = TestFixture::new();

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["plugins", "--list", "--plugin", "test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--plugin can only be used with --upgrade",
        ));
}

#[test]
fn test_install_shows_message_when_all_installed() {
    let fixture = TestFixture::new();

    fixture.create_plugin("already-installed", sample_plugin());

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"

        [plugins.already-installed]
        git = "https://github.com/example/repo"
        tag = "v1.0.0"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--install"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "All declared plugins already installed",
        ));
}

#[test]
fn test_remove_shows_message_when_no_orphans() {
    let fixture = TestFixture::new();

    fixture.create_config(
        "syntropy.toml",
        r#"
        default_plugin_icon = "⚒"
        "#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .args(["plugins", "--remove"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No orphaned plugins to remove"));
}
