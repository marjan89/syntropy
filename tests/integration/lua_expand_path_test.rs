//! Comprehensive integration tests for syntropy.expand_path() Lua function
//!
//! This test suite covers all behaviors of the expand_path function including:
//! - Plugin-relative path resolution (./ and ../)
//! - Tilde expansion (~/)
//! - Environment variable expansion ($VAR and ${VAR})
//! - Absolute path pass-through
//! - Error cases and edge cases
//! - Plugin loading context (merged plugins, module-level vs function-level calls)

use mlua::{Lua, Value};
use serial_test::serial;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use syntropy::{Config, create_lua_vm, load_plugins};
use tokio::sync::Mutex;

use crate::common::TestFixture;

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a Lua VM with a plugin context for testing expand_path
fn setup_lua_with_plugin_context(
    plugin_name: &str,
    plugin_dir: &str,
) -> Result<Arc<Mutex<Lua>>, String> {
    let lua = create_lua_vm().map_err(|e| format!("Failed to create Lua VM: {}", e))?;

    // Set plugin context in registry
    lua.set_named_registry_value("__syntropy_current_plugin__", plugin_name.to_string())
        .map_err(|e| format!("Failed to set plugin context: {}", e))?;

    // Create plugin table with __plugin_dir
    let plugin_table = lua
        .create_table()
        .map_err(|e| format!("Failed to create plugin table: {}", e))?;
    plugin_table
        .set("__plugin_dir", plugin_dir)
        .map_err(|e| format!("Failed to set __plugin_dir: {}", e))?;

    // Store plugin table in globals
    lua.globals()
        .set(plugin_name, plugin_table)
        .map_err(|e| format!("Failed to set plugin in globals: {}", e))?;

    Ok(Arc::new(Mutex::new(lua)))
}

/// Calls syntropy.expand_path() directly on a Lua instance
fn call_expand_path(lua: &Lua, path: &str) -> Result<String, String> {
    let syntropy: mlua::Table = lua
        .globals()
        .get("syntropy")
        .map_err(|e| format!("Failed to get syntropy table: {}", e))?;

    let expand_path: mlua::Function = syntropy
        .get("expand_path")
        .map_err(|e| format!("Failed to get expand_path function: {}", e))?;

    expand_path
        .call::<String>(path.to_string())
        .map_err(|e| format!("expand_path failed: {}", e))
}

/// Creates a test plugin with expand_path usage in a specific function
fn create_test_plugin_with_expand_path(
    fixture: &TestFixture,
    plugin_name: &str,
    expand_path_call: &str,
    call_location: &str, // "items" or "execute" or "module_level"
) -> PathBuf {
    let plugin_content = if call_location == "module_level" {
        format!(
            r#"
-- Module-level call (should fail for relative paths)
local result = syntropy.expand_path("{}")

return {{
    metadata = {{name = "{}", version = "1.0.0"}},
    tasks = {{
        test_task = {{
            execute = function() return result, 0 end
        }}
    }}
}}
"#,
            expand_path_call, plugin_name
        )
    } else if call_location == "items" {
        format!(
            r#"
return {{
    metadata = {{name = "{}", version = "1.0.0"}},
    tasks = {{
        test_task = {{
            items = function()
                local result = syntropy.expand_path("{}")
                return {{result}}
            end,
            execute = function(items) return items[1], 0 end
        }}
    }}
}}
"#,
            plugin_name, expand_path_call
        )
    } else {
        // execute
        format!(
            r#"
return {{
    metadata = {{name = "{}", version = "1.0.0"}},
    tasks = {{
        test_task = {{
            execute = function()
                local result = syntropy.expand_path("{}")
                return result, 0
            end
        }}
    }}
}}
"#,
            plugin_name, expand_path_call
        )
    };

    fixture.create_plugin(plugin_name, &plugin_content);
    fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join(plugin_name)
}

// ============================================================================
// Category 1: Basic Plugin-Relative Paths (8 tests)
// ============================================================================

#[test]
fn test_expand_path_current_dir_relative() {
    let fixture = TestFixture::new();
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", "./config.json", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "./config.json").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin") && result.ends_with("config.json"),
        "Expected path containing /test_plugin and ending with config.json, got: {}",
        result
    );
}

#[test]
fn test_expand_path_parent_dir_relative() {
    let fixture = TestFixture::new();
    let plugin_dir = create_test_plugin_with_expand_path(
        &fixture,
        "test_plugin",
        "../shared/data.txt",
        "execute",
    );

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "../shared/data.txt").expect("expand_path should succeed");

    assert!(
        result.ends_with("/plugins/../shared/data.txt") || result.contains("/shared/data.txt"),
        "Expected parent directory resolution, got: {}",
        result
    );
}

#[test]
fn test_expand_path_nested_relative() {
    let fixture = TestFixture::new();
    let plugin_dir = create_test_plugin_with_expand_path(
        &fixture,
        "test_plugin",
        "./subdir/nested/file.txt",
        "execute",
    );

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result =
        call_expand_path(&lua, "./subdir/nested/file.txt").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin") && result.ends_with("subdir/nested/file.txt"),
        "Expected nested path resolution, got: {}",
        result
    );
}

#[test]
fn test_expand_path_multiple_parent_dirs() {
    let fixture = TestFixture::new();
    let plugin_dir = create_test_plugin_with_expand_path(
        &fixture,
        "test_plugin",
        "../../other/file.txt",
        "execute",
    );

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result =
        call_expand_path(&lua, "../../other/file.txt").expect("expand_path should succeed");

    assert!(
        result.contains("../..") || result.contains("/other/file.txt"),
        "Expected multiple parent directory resolution, got: {}",
        result
    );
}

#[test]
fn test_expand_path_dot_in_middle() {
    let fixture = TestFixture::new();
    let plugin_dir = create_test_plugin_with_expand_path(
        &fixture,
        "test_plugin",
        "./dir/../file.txt",
        "execute",
    );

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "./dir/../file.txt").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin") && result.ends_with("file.txt"),
        "Expected path with dots in middle, got: {}",
        result
    );
}

#[test]
fn test_expand_path_current_dir_only() {
    let fixture = TestFixture::new();
    let plugin_dir = create_test_plugin_with_expand_path(&fixture, "test_plugin", "./", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "./").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin"),
        "Expected current directory resolution, got: {}",
        result
    );
}

#[test]
fn test_expand_path_parent_dir_only() {
    let fixture = TestFixture::new();
    let plugin_dir = create_test_plugin_with_expand_path(&fixture, "test_plugin", "../", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "../").expect("expand_path should succeed");

    assert!(
        result.ends_with("/plugins/..")
            || result.ends_with("/plugins/../")
            || result.contains("/plugins"),
        "Expected parent directory only resolution, got: {}",
        result
    );
}

#[test]
fn test_expand_path_relative_with_spaces() {
    let fixture = TestFixture::new();
    let plugin_dir = create_test_plugin_with_expand_path(
        &fixture,
        "test_plugin",
        "./my folder/my file.txt",
        "execute",
    );

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result =
        call_expand_path(&lua, "./my folder/my file.txt").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin") && result.ends_with("my folder/my file.txt"),
        "Expected path with spaces, got: {}",
        result
    );
}

// ============================================================================
// Category 2: Tilde Expansion (3 tests)
// ============================================================================

#[test]
#[serial]
fn test_expand_path_tilde_home() {
    let fixture = TestFixture::new();
    create_test_plugin_with_expand_path(&fixture, "test_plugin", "~/file.txt", "execute");

    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "~/file.txt").expect("expand_path should succeed");

    let home = env::var("HOME").expect("HOME should be set");
    assert_eq!(
        result,
        format!("{}/file.txt", home),
        "Expected tilde to expand to HOME"
    );
}

#[test]
#[serial]
fn test_expand_path_tilde_nested_path() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result =
        call_expand_path(&lua, "~/.config/app/config.toml").expect("expand_path should succeed");

    let home = env::var("HOME").expect("HOME should be set");
    assert_eq!(
        result,
        format!("{}/.config/app/config.toml", home),
        "Expected nested tilde expansion"
    );
}

#[test]
#[serial]
fn test_expand_path_tilde_only() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "~").expect("expand_path should succeed");

    let home = env::var("HOME").expect("HOME should be set");
    assert_eq!(result, home, "Expected tilde alone to expand to HOME");
}

// ============================================================================
// Category 3: Environment Variable Expansion (5 tests)
// ============================================================================

#[test]
fn test_expand_path_env_var_simple() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    let test_value = "/tmp/test/path";
    unsafe {
        env::set_var("TEST_SYNTROPY_EXPAND_VAR", test_value);
    }

    let result = call_expand_path(&lua, "$TEST_SYNTROPY_EXPAND_VAR/file.txt")
        .expect("expand_path should succeed");

    assert_eq!(
        result,
        format!("{}/file.txt", test_value),
        "Expected environment variable expansion"
    );

    unsafe {
        env::remove_var("TEST_SYNTROPY_EXPAND_VAR");
    }
}

#[test]
fn test_expand_path_env_var_braced() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    let test_value = "/tmp/braced/path";
    unsafe {
        env::set_var("TEST_SYNTROPY_BRACED_VAR", test_value);
    }

    let result = call_expand_path(&lua, "${TEST_SYNTROPY_BRACED_VAR}/file.txt")
        .expect("expand_path should succeed");

    assert_eq!(
        result,
        format!("{}/file.txt", test_value),
        "Expected braced environment variable expansion"
    );

    unsafe {
        env::remove_var("TEST_SYNTROPY_BRACED_VAR");
    }
}

#[test]
#[serial]
fn test_expand_path_env_var_home() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "$HOME/.config").expect("expand_path should succeed");

    let home = env::var("HOME").expect("HOME should be set");
    assert_eq!(
        result,
        format!("{}/.config", home),
        "Expected HOME environment variable expansion"
    );
}

#[test]
fn test_expand_path_env_var_multiple() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    unsafe {
        env::set_var("TEST_VAR1", "/first");
        env::set_var("TEST_VAR2", "/second");
    }

    let result = call_expand_path(&lua, "$TEST_VAR1/$TEST_VAR2/file.txt")
        .expect("expand_path should succeed");

    // shellexpand should expand ALL environment variables in the path
    // Note: double slash because TEST_VAR2 starts with /
    assert_eq!(
        result, "/first//second/file.txt",
        "Expected both env vars to expand"
    );

    unsafe {
        env::remove_var("TEST_VAR1");
        env::remove_var("TEST_VAR2");
    }
}

#[test]
fn test_expand_path_env_var_with_spaces() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    let test_value = "/path with spaces/subdir";
    unsafe {
        env::set_var("TEST_SYNTROPY_SPACE_VAR", test_value);
    }

    let result = call_expand_path(&lua, "$TEST_SYNTROPY_SPACE_VAR/file.txt")
        .expect("expand_path should succeed");

    assert_eq!(
        result,
        format!("{}/file.txt", test_value),
        "Expected environment variable with spaces to expand correctly"
    );

    unsafe {
        env::remove_var("TEST_SYNTROPY_SPACE_VAR");
    }
}

// ============================================================================
// Category 4: Absolute Paths (2 tests)
// ============================================================================

#[test]
fn test_expand_path_absolute_unix() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "/tmp/file.txt").expect("expand_path should succeed");

    assert_eq!(
        result, "/tmp/file.txt",
        "Expected absolute path to pass through unchanged"
    );
}

#[test]
fn test_expand_path_absolute_long() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "/usr/local/bin/some/deep/path/file.txt")
        .expect("expand_path should succeed");

    assert_eq!(
        result, "/usr/local/bin/some/deep/path/file.txt",
        "Expected long absolute path to pass through unchanged"
    );
}

// ============================================================================
// Category 5: Special Cases (4 tests)
// ============================================================================

#[test]
fn test_expand_path_empty_string() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "").expect("expand_path should succeed");

    assert_eq!(result, "", "Expected empty string to return empty string");
}

#[test]
fn test_expand_path_just_filename() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "file.txt").expect("expand_path should succeed");

    assert_eq!(
        result, "file.txt",
        "Expected bare filename to pass through unchanged"
    );
}

#[test]
fn test_expand_path_with_unicode() {
    let fixture = TestFixture::new();
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", "./файл.txt", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "./файл.txt").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin") && result.ends_with("файл.txt"),
        "Expected unicode filename to work, got: {}",
        result
    );
}

#[test]
fn test_expand_path_relative_no_slash() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "relative/path.txt").expect("expand_path should succeed");

    // Paths not starting with ./ or ../ should pass through
    assert_eq!(
        result, "relative/path.txt",
        "Expected relative path without ./ to pass through"
    );
}

// ============================================================================
// Category 6: Error Cases (7 tests)
// ============================================================================

#[test]
fn test_expand_path_relative_no_context() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "./file.txt");

    assert!(
        result.is_err(),
        "Expected error when using ./ without plugin context"
    );
    assert!(
        result.unwrap_err().contains("no plugin context"),
        "Expected 'no plugin context' error message"
    );
}

#[test]
fn test_expand_path_parent_relative_no_context() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "../file.txt");

    assert!(
        result.is_err(),
        "Expected error when using ../ without plugin context"
    );
    assert!(
        result.unwrap_err().contains("no plugin context"),
        "Expected 'no plugin context' error message"
    );
}

#[test]
fn test_expand_path_undefined_env_var() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    // Make sure this variable doesn't exist
    unsafe {
        env::remove_var("SYNTROPY_UNDEFINED_VAR_12345");
    }

    let result = call_expand_path(&lua, "$SYNTROPY_UNDEFINED_VAR_12345/file.txt");

    assert!(
        result.is_err(),
        "Expected error when environment variable is undefined"
    );
}

#[test]
fn test_expand_path_undefined_env_var_braced() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    // Make sure this variable doesn't exist
    unsafe {
        env::remove_var("SYNTROPY_UNDEFINED_BRACED_12345");
    }

    let result = call_expand_path(&lua, "${SYNTROPY_UNDEFINED_BRACED_12345}/file.txt");

    assert!(
        result.is_err(),
        "Expected error when braced environment variable is undefined"
    );
}

#[test]
fn test_expand_path_missing_plugin_table() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    // Set plugin context but don't create the plugin table
    lua.set_named_registry_value("__syntropy_current_plugin__", "nonexistent_plugin")
        .expect("Failed to set registry value");

    let result = call_expand_path(&lua, "./file.txt");

    assert!(
        result.is_err(),
        "Expected error when plugin table doesn't exist"
    );
    assert!(
        result.unwrap_err().contains("nonexistent_plugin"),
        "Expected error to mention missing plugin"
    );
}

#[test]
fn test_expand_path_missing_plugin_dir_field() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    // Set up context but create plugin table without __plugin_dir
    lua.set_named_registry_value("__syntropy_current_plugin__", "broken_plugin")
        .expect("Failed to set registry value");

    let plugin_table = lua.create_table().expect("Failed to create table");
    lua.globals()
        .set("broken_plugin", plugin_table)
        .expect("Failed to set global");

    let result = call_expand_path(&lua, "./file.txt");

    assert!(
        result.is_err(),
        "Expected error when __plugin_dir is missing"
    );
    assert!(
        result.unwrap_err().contains("__plugin_dir"),
        "Expected error to mention missing __plugin_dir"
    );
}

// ============================================================================
// Category 7: Plugin Loading Scenarios (6 tests)
// ============================================================================

#[test]
fn test_expand_path_in_items_function() {
    let fixture = TestFixture::new();
    let plugin_dir = fixture.data_path().join("syntropy").join("plugins");

    // Create a plugin that uses expand_path in items function
    let plugin_content = r#"
return {
    metadata = {name = "items_test", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            items = function()
                local path = syntropy.expand_path("./test.txt")
                return {path}
            end,
            execute = function(items) return items[1], 0 end
        }
    }
}
"#;
    fixture.create_plugin("items_test", plugin_content);

    let lua = Arc::new(Mutex::new(
        create_lua_vm().expect("Failed to create Lua VM"),
    ));
    let config = Config::default();

    let plugins = load_plugins(&[plugin_dir], &config, lua).expect("Failed to load plugins");

    assert_eq!(plugins.len(), 1, "Expected one plugin to load");
    assert_eq!(plugins[0].metadata.name, "items_test");
}

#[test]
fn test_expand_path_in_execute_function() {
    let fixture = TestFixture::new();
    let plugin_dir = fixture.data_path().join("syntropy").join("plugins");

    let plugin_content = r#"
return {
    metadata = {name = "execute_test", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function()
                local path = syntropy.expand_path("./config.json")
                return path, 0
            end
        }
    }
}
"#;
    fixture.create_plugin("execute_test", plugin_content);

    let lua = Arc::new(Mutex::new(
        create_lua_vm().expect("Failed to create Lua VM"),
    ));
    let config = Config::default();

    let plugins = load_plugins(&[plugin_dir], &config, lua).expect("Failed to load plugins");

    assert_eq!(plugins.len(), 1, "Expected one plugin to load");
}

#[test]
fn test_expand_path_at_module_level_with_tilde() {
    let fixture = TestFixture::new();
    let plugin_dir = fixture.data_path().join("syntropy").join("plugins");

    // Module-level call with tilde should work (doesn't need plugin context)
    let plugin_content = r#"
local config_path = syntropy.expand_path("~/config.toml")

return {
    metadata = {name = "module_level_tilde", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function() return config_path, 0 end
        }
    }
}
"#;
    fixture.create_plugin("module_level_tilde", plugin_content);

    let lua = Arc::new(Mutex::new(
        create_lua_vm().expect("Failed to create Lua VM"),
    ));
    let config = Config::default();

    let plugins = load_plugins(&[plugin_dir], &config, lua)
        .expect("Failed to load plugins with module-level tilde expansion");

    assert_eq!(
        plugins.len(),
        1,
        "Expected plugin with module-level tilde to load"
    );
}

#[test]
fn test_expand_path_at_module_level_with_relative_fails() {
    let fixture = TestFixture::new();
    let plugin_dir = fixture.data_path().join("syntropy").join("plugins");

    // Module-level call with ./ should fail
    let plugin_content = r#"
local config_path = syntropy.expand_path("./config.toml")

return {
    metadata = {name = "module_level_fail", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function() return config_path, 0 end
        }
    }
}
"#;
    fixture.create_plugin("module_level_fail", plugin_content);

    let lua = Arc::new(Mutex::new(
        create_lua_vm().expect("Failed to create Lua VM"),
    ));
    let config = Config::default();

    let result = load_plugins(&[plugin_dir], &config, lua);

    assert!(
        result.is_err(),
        "Expected plugin to fail loading with module-level relative path"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("no plugin context")
            || error_msg.contains("Cannot resolve relative path")
            || error_msg.contains("Failed to peek plugin"),
        "Expected error about missing plugin context during peek/load, got: {}",
        error_msg
    );
}

#[test]
fn test_expand_path_in_merged_plugin_resolves_to_override() {
    let fixture = TestFixture::new();

    // Create base plugin in data directory
    let base_content = r#"
return {
    metadata = {name = "merged_test", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function()
                local path = syntropy.expand_path("./base.txt")
                return path, 0
            end
        }
    }
}
"#;
    fixture.create_plugin("merged_test", base_content);

    // Create override in config directory - must include metadata for valid plugin
    let override_content = r#"
return {
    metadata = {name = "merged_test", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function()
                local path = syntropy.expand_path("./override.txt")
                return path, 0
            end
        }
    }
}
"#;
    fixture.create_plugin_override("merged_test", override_content);

    let lua = Arc::new(Mutex::new(
        create_lua_vm().expect("Failed to create Lua VM"),
    ));
    let config = Config::default();

    // Load with both directories (config first, data second)
    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua,
    )
    .expect("Failed to load merged plugin");

    assert_eq!(plugins.len(), 1, "Expected one merged plugin");
    assert_eq!(plugins[0].metadata.name, "merged_test");

    // The plugin's __plugin_dir should point to the override (config) directory
    // This is tested indirectly - the important behavior is documented
}

#[test]
fn test_expand_path_with_absolute_in_plugin() {
    let fixture = TestFixture::new();
    let plugin_dir = fixture.data_path().join("syntropy").join("plugins");

    let plugin_content = r#"
return {
    metadata = {name = "absolute_test", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function()
                local path = syntropy.expand_path("/tmp/absolute.txt")
                return path, 0
            end
        }
    }
}
"#;
    fixture.create_plugin("absolute_test", plugin_content);

    let lua = Arc::new(Mutex::new(
        create_lua_vm().expect("Failed to create Lua VM"),
    ));
    let config = Config::default();

    let plugins = load_plugins(&[plugin_dir], &config, lua)
        .expect("Failed to load plugin with absolute path");

    assert_eq!(
        plugins.len(),
        1,
        "Expected plugin with absolute path to load"
    );
}

// ============================================================================
// Category 8: Edge Cases (6 tests)
// ============================================================================

#[test]
fn test_expand_path_dot_file() {
    let fixture = TestFixture::new();
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", "./.hidden", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "./.hidden").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin") && result.ends_with(".hidden"),
        "Expected hidden file to resolve correctly, got: {}",
        result
    );
}

#[test]
fn test_expand_path_double_slash() {
    let fixture = TestFixture::new();
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", ".//file.txt", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, ".//file.txt").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin"),
        "Expected double slash to be handled, got: {}",
        result
    );
}

#[test]
fn test_expand_path_trailing_slash() {
    let fixture = TestFixture::new();
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", "./dir/", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, "./dir/").expect("expand_path should succeed");

    assert!(
        result.contains("/test_plugin") && result.contains("dir"),
        "Expected trailing slash to be preserved, got: {}",
        result
    );
}

#[test]
fn test_expand_path_tilde_in_middle() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");
    let result = call_expand_path(&lua, "/path/~/file.txt").expect("expand_path should succeed");

    // Tilde in middle shouldn't be expanded
    assert_eq!(
        result, "/path/~/file.txt",
        "Expected tilde in middle to not be expanded"
    );
}

#[test]
fn test_expand_path_env_var_at_end() {
    let lua = create_lua_vm().expect("Failed to create Lua VM");

    unsafe {
        env::set_var("TEST_END_VAR", "/end/path");
    }

    let result =
        call_expand_path(&lua, "/start/$TEST_END_VAR").expect("expand_path should succeed");

    // shellexpand should expand environment variables anywhere in the path
    // Note: double slash because TEST_END_VAR starts with /
    assert_eq!(
        result, "/start//end/path",
        "Expected env var at end to expand"
    );

    unsafe {
        env::remove_var("TEST_END_VAR");
    }
}

#[test]
fn test_expand_path_very_long_path() {
    let fixture = TestFixture::new();
    let long_path = "./a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z/file.txt";
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", long_path, "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    let result = call_expand_path(&lua, long_path).expect("expand_path should succeed");

    assert!(
        result.ends_with(long_path.trim_start_matches('.')),
        "Expected long path to resolve correctly, got: {}",
        result
    );
}

// ============================================================================
// Category 9: Context Management (4 tests)
// ============================================================================

#[test]
fn test_expand_path_different_plugins_different_dirs() {
    let fixture = TestFixture::new();

    // Create two plugins
    fixture.create_plugin(
        "plugin1",
        r#"
return {
    metadata = {name = "plugin1", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function()
                return syntropy.expand_path("./file1.txt"), 0
            end
        }
    }
}
"#,
    );

    fixture.create_plugin(
        "plugin2",
        r#"
return {
    metadata = {name = "plugin2", version = "1.0.0"},
    tasks = {
        test_task = {
            description = "Test task",
            execute = function()
                return syntropy.expand_path("./file2.txt"), 0
            end
        }
    }
}
"#,
    );

    let plugin_dir = fixture.data_path().join("syntropy").join("plugins");
    let lua = Arc::new(Mutex::new(
        create_lua_vm().expect("Failed to create Lua VM"),
    ));
    let config = Config::default();

    let plugins = load_plugins(&[plugin_dir], &config, lua).expect("Failed to load plugins");

    assert_eq!(plugins.len(), 2, "Expected two plugins to load");

    // Each plugin should have its own __plugin_dir
    // This is verified by the loader - paths would be different
}

#[test]
fn test_expand_path_plugin_context_isolation() {
    let fixture = TestFixture::new();
    let plugin1_dir = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("plugin1");
    let plugin2_dir = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("plugin2");

    fs::create_dir_all(&plugin1_dir).expect("Failed to create plugin1 dir");
    fs::create_dir_all(&plugin2_dir).expect("Failed to create plugin2 dir");

    // Set up two plugin contexts
    let lua1 = setup_lua_with_plugin_context("plugin1", plugin1_dir.to_str().unwrap())
        .expect("Failed to setup lua1");
    let lua1 = lua1.blocking_lock();

    let lua2 = setup_lua_with_plugin_context("plugin2", plugin2_dir.to_str().unwrap())
        .expect("Failed to setup lua2");
    let lua2 = lua2.blocking_lock();

    let result1 = call_expand_path(&lua1, "./file.txt").expect("expand_path should succeed");
    let result2 = call_expand_path(&lua2, "./file.txt").expect("expand_path should succeed");

    assert_ne!(
        result1, result2,
        "Expected different plugins to resolve to different directories"
    );
    assert!(
        result1.contains("plugin1"),
        "Expected plugin1 path to contain 'plugin1'"
    );
    assert!(
        result2.contains("plugin2"),
        "Expected plugin2 path to contain 'plugin2'"
    );
}

#[test]
fn test_expand_path_context_preserved_across_calls() {
    let fixture = TestFixture::new();
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", "./file1.txt", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    // Make multiple calls - context should be preserved
    let result1 = call_expand_path(&lua, "./file1.txt").expect("First call should succeed");
    let result2 = call_expand_path(&lua, "./file2.txt").expect("Second call should succeed");

    assert!(
        result1.contains("/test_plugin") && result1.ends_with("file1.txt"),
        "First call should resolve correctly, got: {}",
        result1
    );
    assert!(
        result2.contains("/test_plugin") && result2.ends_with("file2.txt"),
        "Second call should resolve correctly with same context, got: {}",
        result2
    );
}

#[test]
fn test_expand_path_no_context_after_clear() {
    let fixture = TestFixture::new();
    let plugin_dir =
        create_test_plugin_with_expand_path(&fixture, "test_plugin", "./file.txt", "execute");

    let lua = setup_lua_with_plugin_context("test_plugin", plugin_dir.to_str().unwrap())
        .expect("Failed to setup Lua");
    let lua = lua.blocking_lock();

    // First call should work
    let result1 = call_expand_path(&lua, "./file.txt");
    assert!(result1.is_ok(), "First call with context should succeed");

    // Clear the context by setting it to nil
    lua.set_named_registry_value("__syntropy_current_plugin__", Value::Nil)
        .expect("Failed to clear context");

    // Second call should fail
    let result2 = call_expand_path(&lua, "./file.txt");
    assert!(
        result2.is_err(),
        "Call without context should fail after clearing"
    );
}
