//! Integration tests for CLI init subcommand
//!
//! Tests scaffold creation: directories and template files.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

// ============================================================================
// Exit behaviour
// ============================================================================

#[test]
fn test_init_exits_successfully() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();
}

// ============================================================================
// Directory creation
// ============================================================================

#[test]
fn test_init_creates_plugins_dir() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture
            .config_path()
            .join("syntropy")
            .join("plugins")
            .is_dir(),
        "plugins/ directory should be created"
    );
}

#[test]
fn test_init_creates_docs_dir() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture.config_path().join("syntropy").join("docs").is_dir(),
        "docs/ directory should be created"
    );
}

// ============================================================================
// Template file creation
// ============================================================================

#[test]
fn test_init_creates_syntropy_lua() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture
            .config_path()
            .join("syntropy")
            .join("plugins")
            .join("syntropy.lua")
            .is_file(),
        "syntropy.lua should be created in plugins/"
    );
}

#[test]
fn test_init_creates_luarc_json() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture
            .config_path()
            .join("syntropy")
            .join("plugins")
            .join(".luarc.json")
            .is_file(),
        ".luarc.json should be created in plugins/"
    );
}

#[test]
fn test_init_creates_plugin_lua() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture
            .config_path()
            .join("syntropy")
            .join("plugins")
            .join("plugin.lua")
            .is_file(),
        "plugin.lua should be created in plugins/"
    );
}

// ============================================================================
// Doc file creation
// ============================================================================

#[test]
fn test_init_creates_plugins_md() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture
            .config_path()
            .join("syntropy")
            .join("docs")
            .join("plugins.md")
            .is_file(),
        "plugins.md should be created in docs/"
    );
}

#[test]
fn test_init_creates_config_reference() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture
            .config_path()
            .join("syntropy")
            .join("docs")
            .join("config-reference.md")
            .is_file(),
        "config-reference.md should be created in docs/"
    );
}

#[test]
fn test_init_creates_api_reference() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    assert!(
        fixture
            .config_path()
            .join("syntropy")
            .join("docs")
            .join("plugin-api-reference.md")
            .is_file(),
        "plugin-api-reference.md should be created in docs/"
    );
}

#[test]
fn test_init_creates_all_api_reference_sections() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    let docs = fixture.config_path().join("syntropy").join("docs");
    let sections = [
        "plugin-api-reference-section-advanced.md",
        "plugin-api-reference-section-api-functions.md",
        "plugin-api-reference-section-data-structures.md",
        "plugin-api-reference-section-examples.md",
        "plugin-api-reference-section-item-sources.md",
        "plugin-api-reference-section-tasks.md",
        "available-plugins.md",
        "recipes.md",
    ];
    for name in &sections {
        assert!(
            docs.join(name).is_file(),
            "{} should be created in docs/",
            name
        );
    }
}

// ============================================================================
// Output
// ============================================================================

#[test]
fn test_init_output_mentions_plugins_path() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("plugins"));
}

#[test]
fn test_init_output_mentions_docs_path() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("docs"));
}

// ============================================================================
// Idempotency
// ============================================================================

#[test]
fn test_init_twice_exits_successfully() {
    let fixture = TestFixture::new();

    for _ in 0..2 {
        Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
            .env("XDG_CONFIG_HOME", fixture.config_path())
            .arg("init")
            .assert()
            .success();
    }
}

#[test]
fn test_init_twice_warns_about_overwritten_files() {
    let fixture = TestFixture::new();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("init")
        .assert()
        .success()
        .stderr(predicate::str::contains("Warning"));
}
