// Integration tests for plugin isolation and side effects
//
// These tests verify that plugins cannot affect each other or the host process
// through shared state, stdout pollution, environment mutation, or background
// processes.
//
// Tests 1, 2, and 5 define desired behavior not yet implemented (expected to FAIL).
// Tests 3 and 4 are regression guards for existing correct behavior (expected to PASS).

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

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

// ============================================================================
// Test 1: Shared module state not leaked between plugins (Line 15)
// ============================================================================

#[test]
fn test_shared_module_state_not_leaked_between_plugins() {
    // Plugin A sets cache["key"] = "from_plugin_a".
    // Plugin B reads cache["key"] — must see "nil", not Plugin A's value.
    // Currently FAILS because both closures captured the same table reference
    // from package.loaded in the shared Lua VM.
    let fixture = TestFixture::new();

    // Shared module returns a fresh empty table
    fixture.create_shared_module(
        "cache",
        r#"
local t = {}
return t
"#,
    );

    // Plugin A: requires cache at load time; task mutates the table
    fixture.create_plugin(
        "plugin_a",
        r#"
local cache = require("cache")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        set_cache = {
            description = "Set a value in the shared cache",
            execute = function()
                cache["key"] = "from_plugin_a"
                return "set", 0
            end,
        },
    },
}
"#,
    );

    // Plugin B: requires cache at load time; task reads from the table
    fixture.create_plugin(
        "plugin_b",
        r#"
local cache = require("cache")
return {
    metadata = {name = "plugin_b", version = "1.0.0"},
    tasks = {
        read_cache = {
            description = "Read a value from the shared cache",
            execute = function()
                return tostring(cache["key"]), 0
            end,
        },
    },
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 2);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Run Plugin A's task — should succeed and set the cache entry
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let task_a = plugin_a.tasks.get("set_cache").unwrap();
    let (output_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();
    assert_eq!(output_a, "set");

    // Run Plugin B's task — must NOT see Plugin A's mutation
    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let task_b = plugin_b.tasks.get("read_cache").unwrap();
    let (output_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();

    // Each plugin should get an isolated copy of the module table.
    // Currently this is "from_plugin_a" (leaking) — the test documents desired "nil".
    assert_eq!(
        output_b, "nil",
        "Plugin B should not see state set by Plugin A in the shared module"
    );
}

// ============================================================================
// Test 2: Plugin print() does not pollute stdout (Line 36)
// ============================================================================

#[test]
fn test_plugin_print_does_not_pollute_stdout() {
    // A plugin that calls print() during execute() must not write to process
    // stdout. Currently FAILS because create_lua_vm() does not override print().
    use assert_cmd::Command;
    use predicates::prelude::*;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    fixture.create_plugin(
        "test_print_plugin",
        r#"
return {
    metadata = {
        name = "test_print_plugin",
        version = "1.0.0",
        platforms = {"macos", "linux"},
    },
    tasks = {
        task = {
            description = "Task that calls print()",
            execute = function()
                print("unexpected_debug_output")
                return "done", 0
            end,
        },
    },
}
"#,
    );

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test_print_plugin")
        .arg("--task")
        .arg("task")
        .assert()
        .success()
        .stdout(predicate::str::contains("done"))
        .stdout(predicate::str::contains("unexpected_debug_output").not());
}

// ============================================================================
// Test 3: os.setenv is not available (Line 37)
// ============================================================================

#[test]
fn test_plugin_setenv_function_not_available() {
    // os.setenv does not exist in Lua 5.4's stdlib.
    // Calling it must produce a runtime error, not modify the environment.
    // This test is a regression guard — we must never accidentally expose this.
    let fixture = TestFixture::new();

    // Ensure the variable is unset before the test
    // SAFETY: single-threaded test setup; no other threads reading this var
    unsafe { std::env::remove_var("SYNTROPY_ISOLATION_TEST") };
    assert!(
        std::env::var("SYNTROPY_ISOLATION_TEST").is_err(),
        "env var must be unset before test"
    );

    fixture.create_plugin(
        "setenv_plugin",
        r#"
return {
    metadata = {name = "setenv_plugin", version = "1.0.0"},
    tasks = {
        try_setenv = {
            description = "Attempt to call os.setenv",
            execute = function()
                os.setenv("SYNTROPY_ISOLATION_TEST", "leaked")
                return "ok", 0
            end,
        },
    },
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("try_setenv").unwrap();

    // Must return an error — os.setenv is nil in Lua 5.4
    let result = rt.block_on(async { call_task_execute(&lua, task, &[]).await });
    assert!(
        result.is_err(),
        "Calling os.setenv should return a Lua runtime error"
    );

    // Environment must be unchanged
    assert!(
        std::env::var("SYNTROPY_ISOLATION_TEST").is_err(),
        "os.setenv must not modify the process environment"
    );
}

// ============================================================================
// Test 4: os.chdir is not available (Line 38)
// ============================================================================

#[test]
fn test_plugin_chdir_function_not_available() {
    // os.chdir does not exist in Lua 5.4's stdlib.
    // Calling it must produce a runtime error, not change the working directory.
    // This test is a regression guard.
    let fixture = TestFixture::new();

    let original_dir = std::env::current_dir().expect("Failed to get current dir");

    fixture.create_plugin(
        "chdir_plugin",
        r#"
return {
    metadata = {name = "chdir_plugin", version = "1.0.0"},
    tasks = {
        try_chdir = {
            description = "Attempt to call os.chdir",
            execute = function()
                os.chdir("/tmp")
                return "ok", 0
            end,
        },
    },
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("try_chdir").unwrap();

    // Must return an error — os.chdir is nil in Lua 5.4
    let result = rt.block_on(async { call_task_execute(&lua, task, &[]).await });
    assert!(
        result.is_err(),
        "Calling os.chdir should return a Lua runtime error"
    );

    // Working directory must be unchanged
    let current_dir = std::env::current_dir().expect("Failed to get current dir");
    assert_eq!(
        current_dir, original_dir,
        "os.chdir must not change the process working directory"
    );
}

// ============================================================================
// Test 5: Background process does not block exit (Line 39)
// ============================================================================

#[test]
fn test_plugin_background_process_does_not_block_exit() {
    // syntropy.shell("sleep 30 &") should spawn a background process and return.
    // The parent process must exit promptly — it must not block waiting for
    // the orphaned child. Currently FAILS: syntropy.shell() blocks for the
    // full sleep duration rather than returning immediately after shell fork.
    use std::process::Command;
    use std::time::{Duration, Instant};

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    fixture.create_plugin(
        "bg_plugin",
        r#"
return {
    metadata = {
        name = "bg_plugin",
        version = "1.0.0",
        platforms = {"macos", "linux"},
    },
    tasks = {
        spawn_bg = {
            description = "Spawn a background process and return immediately",
            execute = function()
                syntropy.shell("sleep 30 &")
                return "spawned", 0
            end,
        },
    },
}
"#,
    );

    let syntropy_bin = assert_cmd::cargo::cargo_bin!("syntropy");

    let start = Instant::now();
    let mut child = Command::new(syntropy_bin)
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("bg_plugin")
        .arg("--task")
        .arg("spawn_bg")
        .spawn()
        .expect("Failed to spawn syntropy process");

    let status = child.wait().expect("Failed to wait for process");
    let elapsed = start.elapsed();

    assert!(
        status.success(),
        "syntropy should exit with code 0 after spawning background process"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "syntropy should exit within 5 seconds, but took {:?}",
        elapsed
    );
}
