//! Integration tests for Lua runtime error propagation
//!
//! These tests define correct behaviour for plugin functions that throw Lua errors at runtime.
//! They cover items(), preview(), require() failures, and sandbox enforcement.
//!
//! **Test 1 will fail until runner.rs is fixed**: single-source items() errors currently
//! print a spurious "Warning: item_source" line before the actual error, double-printing
//! the failure. Desired behaviour is one clean error — no warning prefix for total failures.

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

// === items() errors ===

/// Single-source task where items() throws a Lua error.
///
/// Desired: one clean error in stderr, no spurious "Warning: item_source" prefix.
/// The warning prefix is intended for multi-source partial failures where other sources
/// continue working. For a total failure it is misleading noise.
///
/// This test fails until runner.rs suppresses the eprintln! for single-source tasks.
#[test]
fn test_single_source_items_error_no_double_print() {
    const PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        items_fail = {
            name = "Items Fail",
            description = "items() throws a Lua error",
            mode = "none",
            item_sources = {
                source = {
                    tag = "s",
                    items = function()
                        error("deliberate items() error")
                    end,
                    execute = function(items)
                        return "SHOULD_NOT_EXECUTE", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "execute",
            "--plugin",
            "test",
            "--task",
            "items_fail",
            "--produce-items",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "items() error must cause non-zero exit; got:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "stdout must be empty when items() fails; got: {stdout}"
    );
    assert!(
        stderr.contains("deliberate items() error"),
        "stderr must contain the Lua error message; got: {stderr}"
    );
    assert!(
        !stderr.contains("SHOULD_NOT_EXECUTE"),
        "execute() must not be called when items() fails; got: {stderr}"
    );
    // Single-source total failure: no "Warning:" prefix — just a clean error.
    assert!(
        !stderr.contains("Warning: item_source"),
        "single-source total failure must not print a warning prefix — it is not a partial \
         failure; got:\n{stderr}"
    );
}

// === preview() errors ===

/// preview() throws a Lua error when called via --preview flag.
///
/// Desired: non-zero exit, error message in stderr, stdout empty.
/// The error must surface clearly without panicking or freezing.
#[test]
fn test_preview_error_surfaces_in_stderr() {
    const PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        preview_fail = {
            name = "Preview Fail",
            description = "preview() throws a Lua error",
            mode = "none",
            item_sources = {
                source = {
                    tag = "s",
                    items = function()
                        return {"item one", "item two"}
                    end,
                    preview = function(item)
                        error("deliberate preview() error for: " .. item)
                    end,
                    execute = function(items)
                        return "executed", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "execute",
            "--plugin",
            "test",
            "--task",
            "preview_fail",
            "--preview",
            "item one",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "preview() error must cause non-zero exit; got:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "stdout must be empty when preview() fails; got: {stdout}"
    );
    assert!(
        stderr.contains("deliberate preview() error"),
        "stderr must contain the Lua error message; got: {stderr}"
    );
}

// === require() failures ===

/// items() calls require() for a Lua module that does not exist.
///
/// Desired: non-zero exit, error message identifies the missing module by name
/// so the user knows what to fix.
#[test]
fn test_require_missing_module_surfaces_module_name() {
    const PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        missing_module = {
            name = "Missing Module",
            description = "require() for a module that does not exist",
            mode = "none",
            item_sources = {
                source = {
                    tag = "s",
                    items = function()
                        local m = require("nonexistent_lua_module_xyz")
                        return {m.get()}
                    end,
                    execute = function(items)
                        return "executed", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "execute",
            "--plugin",
            "test",
            "--task",
            "missing_module",
            "--produce-items",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "missing require() must cause non-zero exit; got:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "stdout must be empty when require() fails; got: {stdout}"
    );
    assert!(
        stderr.contains("nonexistent_lua_module_xyz"),
        "stderr must name the missing module so the user can identify it; got: {stderr}"
    );
}

// === sandbox enforcement ===

/// items() calls os.execute(), which is blocked by the Lua VM sandbox.
///
/// Desired: non-zero exit with a clear error. The sandbox must hold — plugins
/// cannot bypass it via os.execute(). The error must not panic the application.
#[test]
fn test_os_execute_removed_from_sandbox() {
    const PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        os_exec = {
            name = "OS Execute",
            description = "attempts to call os.execute which is removed from the sandbox",
            mode = "none",
            item_sources = {
                source = {
                    tag = "s",
                    items = function()
                        os.execute("ls")
                        return {"item"}
                    end,
                    execute = function(items)
                        return "SHOULD_NOT_EXECUTE", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "execute",
            "--plugin",
            "test",
            "--task",
            "os_exec",
            "--produce-items",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "os.execute() call must cause non-zero exit; got:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "stdout must be empty when the sandbox rejects os.execute(); got: {stdout}"
    );
    assert!(
        !stderr.contains("SHOULD_NOT_EXECUTE"),
        "execute() must not run when items() hits a sandbox violation; got: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must contain an error message explaining the sandbox violation; got nothing"
    );
}

/// os.execute() error message names the blocked function.
///
/// Desired: stderr contains "os.execute" so the user knows exactly which call was
/// blocked. "attempt to call a nil value (field 'execute')" is ambiguous — it could
/// refer to any field named 'execute'. The sandbox stub must name the function explicitly.
#[test]
fn test_os_execute_sandbox_error_is_descriptive() {
    const PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        os_exec = {
            name = "OS Execute",
            description = "attempts to call os.execute which is blocked by the sandbox",
            mode = "none",
            item_sources = {
                source = {
                    tag = "s",
                    items = function()
                        os.execute("ls")
                        return {"item"}
                    end,
                    execute = function(items)
                        return "SHOULD_NOT_EXECUTE", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args([
            "execute",
            "--plugin",
            "test",
            "--task",
            "os_exec",
            "--produce-items",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("os.execute"));
}

// === malformed output ===

/// execute() returns a Lua string containing bytes that are not valid UTF-8.
///
/// Desired: non-zero exit, error reported in stderr, stdout empty.
/// The malformed bytes must not be written to stdout — that would corrupt the
/// terminal or corrupt output piped to other tools.
/// The application must not panic — mlua's string conversion failure must
/// surface as a clean error.
#[test]
fn test_execute_invalid_utf8_output_surfaces_error() {
    const PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        bad_utf8 = {
            name = "Bad UTF-8",
            description = "execute() returns a string with invalid UTF-8 bytes",
            mode = "none",
            item_sources = {
                source = {
                    tag = "s",
                    items = function()
                        return {"item"}
                    end,
                    execute = function(items)
                        -- 0xFF and 0xFE are never valid in UTF-8
                        return string.char(0xFF, 0xFE), 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .args(["execute", "--plugin", "test", "--task", "bad_utf8"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("bad_utf8"));
}
