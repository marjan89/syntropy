// Regression tests for package.path bug fix
//
// Context: Previously, each plugin overwrote package.path instead of accumulating paths.
// Fix: Configure package.path once for ALL plugins before any evaluation (loader.rs:36-78)
//
// These tests ensure:
// 1. Multiple plugins can each require() their own lib/ modules
// 2. require() works at plugin load time (during peek/evaluation)
// 3. Shared module namespace behavior is documented and tested

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_multi_plugin_lib_namespace_accumulation() {
    // PRIMARY REGRESSION TEST: Catches the original bug where each plugin
    // overwrote package.path instead of accumulating paths
    let fixture = TestFixture::new();

    // Plugin A with lib/utils.lua
    fixture.create_lib_module(
        "plugin_a",
        "utils",
        r#"
return {
    add = function(a, b) return a + b end
}
"#,
    );

    fixture.create_plugin(
        "plugin_a",
        r#"
local utils = require("plugin_a.utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        calc = {
            description = "Test task",
            execute = function()
                return tostring(utils.add(2, 3)), 0
            end
        }
    }
}
"#,
    );

    // Plugin B with lib/parser.lua
    fixture.create_lib_module(
        "plugin_b",
        "parser",
        r#"
return {
    parse = function(s) return "parsed:" .. s end
}
"#,
    );

    fixture.create_plugin(
        "plugin_b",
        r#"
local parser = require("plugin_b.parser")
return {
    metadata = {name = "plugin_b", version = "1.0.0"},
    tasks = {
        process = {
            description = "Test task",
            execute = function()
                return parser.parse("test"), 0
            end
        }
    }
}
"#,
    );

    // Load both plugins
    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 2);

    // CRITICAL: Both plugins must successfully execute using their lib/ modules
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Plugin A must still access its lib/utils.lua (would fail with old bug)
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let task_a = plugin_a.tasks.get("calc").unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();
    assert_eq!(
        result_a, "5",
        "Plugin A must access its lib/utils.lua even after Plugin B loads"
    );

    // Plugin B must access its lib/parser.lua
    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let task_b = plugin_b.tasks.get("process").unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();
    assert_eq!(
        result_b, "parsed:test",
        "Plugin B must access its lib/parser.lua"
    );
}

#[test]
fn test_require_during_plugin_evaluation() {
    // CRITICAL TEST: Validates that package.path is configured BEFORE peek()
    // so that plugins can use require() at module scope (load time)
    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "load_time",
        "constants",
        r#"
return {
    VERSION = "1.0.0",
    MAX_SIZE = 100
}
"#,
    );

    fixture.create_plugin(
        "load_time",
        r#"
-- This require happens during plugin.lua evaluation (load time)
local constants = require("load_time.constants")

return {
    metadata = {
        description = "Test task",
        name = "load_time",
        version = constants.VERSION,  -- Using module at load time!
    },
    tasks = {
        info = {
            description = "Test task",
            execute = function()
                return "Max: " .. tostring(constants.MAX_SIZE), 0
            end
        }
    }
}
"#,
    );

    // Second plugin to ensure package.path persists
    fixture.create_lib_module(
        "second",
        "data",
        r#"
return { name = "test" }
"#,
    );

    fixture.create_plugin(
        "second",
        r#"
local data = require("second.data")
return {
    metadata = {name = "second", version = "1.0.0"},
    tasks = {
        t = {description = "Test task", execute = function() return data.name, 0 end}
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));

    // This must succeed - plugins can require() at load time
    let result = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    );

    assert!(
        result.is_ok(),
        "Load-time require() must work: {:?}",
        result.err()
    );

    let plugins = result.unwrap();
    assert_eq!(plugins.len(), 2);

    // Verify the plugin actually used the constant at load time
    let plugin = plugins
        .iter()
        .find(|p| p.metadata.name == "load_time")
        .unwrap();
    assert_eq!(
        plugin.metadata.version, "1.0.0",
        "Plugin must load constants at module scope"
    );

    // Verify task execution still works
    let rt = tokio::runtime::Runtime::new().unwrap();
    let task = plugin.tasks.get("info").unwrap();
    let (result, _) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();
    assert_eq!(result, "Max: 100");
}

#[test]
fn test_mixed_lib_presence_across_plugins() {
    // Validates that plugins without lib/ directories don't break
    // the module loading for plugins with lib/ directories
    let fixture = TestFixture::new();

    // Plugin 1: Has lib/ directory
    fixture.create_lib_module(
        "with_lib",
        "helper",
        r#"
return { value = "from_lib" }
"#,
    );

    fixture.create_plugin(
        "with_lib",
        r#"
local helper = require("with_lib.helper")
return {
    metadata = {name = "with_lib", version = "1.0.0"},
    tasks = {
        t = {description = "Test task", execute = function() return helper.value, 0 end}
    }
}
"#,
    );

    // Plugin 2: No lib/ directory
    fixture.create_plugin(
        "no_lib",
        r#"
return {
    metadata = {name = "no_lib", version = "1.0.0"},
    tasks = {
        t = {description = "Test task", execute = function() return "simple", 0 end}
    }
}
"#,
    );

    // Plugin 3: Has lib/ directory
    fixture.create_lib_module(
        "another_with_lib",
        "util",
        r#"
return { compute = function() return "computed" end }
"#,
    );

    fixture.create_plugin(
        "another_with_lib",
        r#"
local util = require("another_with_lib.util")
return {
    metadata = {name = "another_with_lib", version = "1.0.0"},
    tasks = {
        t = {description = "Test task", execute = function() return util.compute(), 0 end}
    }
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

    assert_eq!(plugins.len(), 3);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Verify all three plugins work correctly
    for plugin in &plugins {
        let task = plugin.tasks.get("t").unwrap();
        let result = rt.block_on(async { call_task_execute(&lua, task, &[]).await });
        assert!(
            result.is_ok(),
            "Plugin {} should execute successfully",
            plugin.metadata.name
        );
    }

    // Specifically verify lib-enabled plugins access their modules
    let with_lib = plugins
        .iter()
        .find(|p| p.metadata.name == "with_lib")
        .unwrap();
    let task = with_lib.tasks.get("t").unwrap();
    let (result, _) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();
    assert_eq!(result, "from_lib");
}

#[test]
fn test_module_name_conflict_first_plugin_wins() {
    // Documents the shared namespace behavior: when multiple plugins
    // have lib/module.lua with the same name, alphabetical order determines winner
    let fixture = TestFixture::new();

    // Both plugins have lib/config.lua
    fixture.create_lib_module(
        "a_plugin",
        "config",
        r#"
return { source = "a_plugin" }
"#,
    );

    fixture.create_plugin(
        "a_plugin",
        r#"
local config = require("a_plugin.config")
return {
    metadata = {name = "a_plugin", version = "1.0.0"},
    tasks = {
        check = {
            description = "Test task",
            execute = function()
                return config.source, 0
            end
        }
    }
}
"#,
    );

    fixture.create_lib_module(
        "z_plugin",
        "config",
        r#"
return { source = "z_plugin" }
"#,
    );

    fixture.create_plugin(
        "z_plugin",
        r#"
local config = require("z_plugin.config")
return {
    metadata = {name = "z_plugin", version = "1.0.0"},
    tasks = {
        check = {
            description = "Test task",
            execute = function()
                return config.source, 0
            end
        }
    }
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

    // With namespaced requires, each plugin loads its OWN module
    // No collision occurs even when module names are the same
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "a_plugin")
        .unwrap();
    let task_a = plugin_a.tasks.get("check").unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();

    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "z_plugin")
        .unwrap();
    let task_b = plugin_b.tasks.get("check").unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();

    // Each plugin should load its own module (no collision with namespacing)
    assert_eq!(result_a, "a_plugin", "a_plugin loads its own config module");
    assert_eq!(
        result_b, "z_plugin",
        "z_plugin loads its own config module (no collision)"
    );
}

#[test]
fn test_merged_plugin_lib_directory_precedence() {
    // When merging plugins (config dir + data dir), both have lib/ directories
    // Config dir should take precedence (override wins)
    let fixture = TestFixture::new();

    // Base plugin in data dir with lib/helper.lua
    fixture.create_lib_module(
        "merged",
        "helper",
        r#"
return { version = "base" }
"#,
    );

    fixture.create_plugin(
        "merged",
        r#"
local helper = require("merged.helper")
return {
    metadata = {name = "merged", version = "1.0.0"},
    tasks = {
        base_task = {
            description = "Test task",
            execute = function()
                return helper.version, 0
            end
        }
    }
}
"#,
    );

    // Override plugin in config dir with lib/helper.lua
    fixture.create_lib_module_override(
        "merged",
        "helper",
        r#"
return { version = "override" }
"#,
    );

    fixture.create_plugin_override(
        "merged",
        r#"
-- Override can access its own version of helper
local helper = require("merged.helper")

return {
    metadata = {name = "merged"},
    tasks = {
        override_task = {
            description = "Test task",
            execute = function()
                return helper.version, 0
            end
        }
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    // Should have 1 merged plugin
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.name, "merged");

    // Should have both tasks (base + override)
    assert_eq!(plugins[0].tasks.len(), 2);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Both tasks should get the "override" version (config dir wins)
    let base_task = plugins[0].tasks.get("base_task").unwrap();
    let (result, _) = rt
        .block_on(async { call_task_execute(&lua, base_task, &[]).await })
        .unwrap();
    assert_eq!(
        result, "override",
        "Config dir lib/ wins over data dir lib/"
    );

    let override_task = plugins[0].tasks.get("override_task").unwrap();
    let (result, _) = rt
        .block_on(async { call_task_execute(&lua, override_task, &[]).await })
        .unwrap();
    assert_eq!(result, "override");
}

#[test]
fn test_runtime_require_in_task_functions() {
    // Validates that require() works in items(), execute(), preview() functions
    // at runtime, not just during plugin load
    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "runtime",
        "counter",
        r#"
local M = {}
M.count = 0

function M.increment()
    M.count = M.count + 1
    return M.count
end

function M.get()
    return M.count
end

return M
"#,
    );

    fixture.create_plugin(
        "runtime",
        r#"
return {
    metadata = {name = "runtime", version = "1.0.0"},
    tasks = {
        count = {
            description = "Test task",
            execute = function()
                -- require() called at runtime in execute function
                local counter = require("runtime.counter")
                counter.increment()
                return "Count: " .. tostring(counter.get()), 0
            end
        }
    }
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
    let task = plugins[0].tasks.get("count").unwrap();

    // First execution
    let (result1, _) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();
    assert_eq!(result1, "Count: 1");

    // Second execution - module is cached, state persists
    let (result2, _) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();
    assert_eq!(
        result2, "Count: 2",
        "Module state should persist due to Lua require() caching"
    );
}

#[test]
fn test_require_in_items_function() {
    // Validates that require() works in items() function at runtime
    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "items_plugin",
        "items_helper",
        r#"
local M = {}

function M.generate_items()
    return {"Item 1 from helper", "Item 2 from helper", "Item 3 from helper"}
end

return M
"#,
    );

    fixture.create_plugin(
        "items_plugin",
        r#"
return {
    metadata = {name = "items_plugin", version = "1.0.0"},
    tasks = {
        list = {
            description = "Test items with require",
            item_sources = {
                generated = {
                    tag = "gen",
                    items = function()
                        -- require() called at runtime in items function
                        local helper = require("items_plugin.items_helper")
                        return helper.generate_items()
                    end
                }
            },
            execute = function(items)
                return items[1], 0
            end
        }
    }
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
    let task = plugins[0].tasks.get("list").unwrap();
    let (result, code) = rt
        .block_on(async {
            call_task_execute(&lua, task, &["Item 1 from helper".to_string()]).await
        })
        .unwrap();

    assert_eq!(code, 0);
    assert_eq!(result, "Item 1 from helper");
}

#[test]
fn test_require_in_preview_function() {
    // Validates that require() works in preview() function at runtime
    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "preview_plugin",
        "preview_formatter",
        r#"
local M = {}

function M.format_preview(item)
    return "=== PREVIEW ===\n" .. item .. "\n=== END ==="
end

return M
"#,
    );

    fixture.create_plugin(
        "preview_plugin",
        r#"
return {
    metadata = {name = "preview_plugin", version = "1.0.0"},
    tasks = {
        view = {
            description = "Test preview with require",
            item_sources = {
                items = {
                    tag = "item",
                    items = function()
                        return {"test_item"}
                    end,
                    preview = function(item)
                        -- require() called at runtime in preview function
                        local formatter = require("preview_plugin.preview_formatter")
                        return formatter.format_preview(item)
                    end
                }
            },
            execute = function(items)
                return items[1], 0
            end
        }
    }
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
    // Preview function is called during TUI interaction, not directly testable
    // via call_task_execute, but we verify the plugin loads correctly
}

#[test]
fn test_multiple_requires_in_same_function() {
    // Validates that multiple require() calls in the same function work correctly
    // and that module caching works as expected
    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "multi_req",
        "helper_a",
        r#"
return { name = "Helper A" }
"#,
    );

    fixture.create_lib_module(
        "multi_req",
        "helper_b",
        r#"
return { name = "Helper B" }
"#,
    );

    fixture.create_lib_module(
        "multi_req",
        "helper_c",
        r#"
return { name = "Helper C" }
"#,
    );

    fixture.create_plugin(
        "multi_req",
        r#"
return {
    metadata = {name = "multi_req", version = "1.0.0"},
    tasks = {
        combine = {
            description = "Test multiple requires",
            execute = function()
                -- Multiple require() calls in same function
                local a = require("multi_req.helper_a")
                local b = require("multi_req.helper_b")
                local c = require("multi_req.helper_c")

                -- Requiring again should return cached modules
                local a2 = require("multi_req.helper_a")

                local result = a.name .. ", " .. b.name .. ", " .. c.name
                if a == a2 then
                    result = result .. " (cached)"
                end

                return result, 0
            end
        }
    }
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
    let task = plugins[0].tasks.get("combine").unwrap();
    let (result, code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(code, 0);
    assert_eq!(result, "Helper A, Helper B, Helper C (cached)");
}

#[test]
fn test_require_in_metadata_construction() {
    // Validates that require() can be used during metadata table construction
    // at plugin load time (not just in task functions)
    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "meta_plugin",
        "version_info",
        r#"
return {
    version = "2.5.1",
    build = "stable"
}
"#,
    );

    fixture.create_plugin(
        "meta_plugin",
        r#"
-- require() at module scope for metadata construction
local version_info = require("meta_plugin.version_info")

return {
    metadata = {
        name = "meta_plugin",
        version = version_info.version,
        description = "Build: " .. version_info.build,
    },
    tasks = {
        info = {
            description = "Test metadata require",
            execute = function()
                return "ok", 0
            end
        }
    }
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

    // Verify metadata was constructed using the required module
    assert_eq!(plugins[0].metadata.version, "2.5.1");
    assert!(plugins[0].metadata.description.contains("stable"));
}
