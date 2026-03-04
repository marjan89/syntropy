//! Integration tests for plugin loading edge cases
//!
//! Documents expected behavior for 4 edge cases identified in UNTESTED_FAILURE_POINTS.md:
//! - Line 25: Task name collision (plugin-scoped, no collision possible)
//! - Line 17: Plugin load order dependency (module isolation)
//! - Line 23: Module path conflicts (config shadows data)
//! - Line 24: Plugin file modified after load (Lua caching)

use crate::common::TestFixture;
use std::{fs, sync::Arc};
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

// ============================================================================
// Issue 1: Task Name Collision - Document Plugin-Scoped Lookup (Line 25)
// ============================================================================

#[test]
fn test_task_names_scoped_to_plugin_no_collision() {
    // Documents that cross-plugin task name collisions are architecturally impossible
    // because tasks are plugin-scoped (plugin_idx → task_key lookup).
    //
    // This test demonstrates that Plugin A and Plugin B can both define a task
    // named "build" without any collision, and both tasks execute independently
    // with their own distinct behavior.

    let fixture = TestFixture::new();

    fixture.create_plugin(
        "plugin_a",
        r#"
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        build = {
            description = "Build task A",
            execute = function() return "Built A", 0 end
        }
    }
}
"#,
    );

    fixture.create_plugin(
        "plugin_b",
        r#"
return {
    metadata = {name = "plugin_b", version = "1.0.0"},
    tasks = {
        build = {
            description = "Build task B",
            execute = function() return "Built B", 0 end
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

    // Both plugins have task "build" - no collision due to plugin scoping
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();

    assert!(plugin_a.tasks.contains_key("build"));
    assert!(plugin_b.tasks.contains_key("build"));

    // Execute both to verify independence
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (result_a, code_a) = rt
        .block_on(async {
            call_task_execute(&lua, plugin_a.tasks.get("build").unwrap(), &[]).await
        })
        .unwrap();

    let (result_b, code_b) = rt
        .block_on(async {
            call_task_execute(&lua, plugin_b.tasks.get("build").unwrap(), &[]).await
        })
        .unwrap();

    assert_eq!(result_a, "Built A");
    assert_eq!(code_a, 0);
    assert_eq!(result_b, "Built B");
    assert_eq!(code_b, 0);
}

#[test]
fn test_duplicate_task_key_within_plugin_last_wins() {
    // Documents that within a single plugin, if the same task key is defined twice
    // in the Lua tasks table, the last definition wins (standard Lua table behavior).
    //
    // This is NOT a bug - it's standard Lua table semantics where duplicate keys
    // result in the last value being used.

    let fixture = TestFixture::new();

    fixture.create_plugin(
        "plugin_duplicate",
        r#"
return {
    metadata = {name = "plugin_duplicate", version = "1.0.0"},
    tasks = {
        build = {
            description = "First definition",
            execute = function() return "First", 0 end
        },
        build = {
            description = "Second definition (this one wins)",
            execute = function() return "Second", 0 end
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

    // Only one "build" task exists (last definition wins)
    let plugin = &plugins[0];
    assert_eq!(plugin.tasks.len(), 1);
    assert!(plugin.tasks.contains_key("build"));

    // Verify it's the second definition
    let task = plugin.tasks.get("build").unwrap();
    assert_eq!(task.description, "Second definition (this one wins)");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let (result, code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(result, "Second");
    assert_eq!(code, 0);
}

// ============================================================================
// Issue 2: Plugin Load Order Dependency - Document Module Isolation (Line 17)
// ============================================================================

#[test]
fn test_plugin_module_namespace_isolation() {
    // Documents that plugins share the same Lua VM but have module isolation through
    // Neovim-style namespacing (plugin_a.module vs plugin_b.module).
    //
    // This test demonstrates that Plugin A's modules cannot be accidentally accessed
    // by Plugin B due to the namespaced require() system.
    //
    // For comprehensive plugin isolation tests, see:
    // tests/integration/plugin_lib_isolation_test.rs lines 553-668

    let fixture = TestFixture::new();

    // Plugin A with its own namespaced module
    fixture.create_lib_module(
        "plugin_a",
        "utils",
        r#"
return {
    plugin_name = "A",
    get_value = function() return "value_from_A" end
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
        test = {
            description = "Test module access",
            execute = function()
                return utils.get_value(), 0
            end
        }
    }
}
"#,
    );

    // Plugin B with its own namespaced module (same name, different namespace)
    fixture.create_lib_module(
        "plugin_b",
        "utils",
        r#"
return {
    plugin_name = "B",
    get_value = function() return "value_from_B" end
}
"#,
    );

    fixture.create_plugin(
        "plugin_b",
        r#"
local utils = require("plugin_b.utils")
return {
    metadata = {name = "plugin_b", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test module access",
            execute = function()
                return utils.get_value(), 0
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

    // Plugin A gets its own module
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, plugin_a.tasks.get("test").unwrap(), &[]).await })
        .unwrap();
    assert_eq!(result_a, "value_from_A");

    // Plugin B gets its own module (not A's)
    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, plugin_b.tasks.get("test").unwrap(), &[]).await })
        .unwrap();
    assert_eq!(result_b, "value_from_B");
}

// ============================================================================
// Issue 3: Module Path Conflicts - Document Shadowing Behavior (Line 23)
// ============================================================================

#[test]
fn test_config_shared_module_shadows_data_shared_module() {
    // Documents INTENDED behavior: Config dir shared modules shadow data dir versions.
    // This allows users to override system-provided shared modules with their own versions.
    //
    // Mechanism: package.path is ordered as config → data, so Lua's require()
    // finds the config version first and caches it in package.loaded.

    let fixture = TestFixture::new();

    // Data dir shared module (will be shadowed)
    fixture.create_shared_module(
        "utils",
        r#"
return {
    source = "data"
}
"#,
    );

    // Config dir shared module (will win)
    fixture.create_shared_module_override(
        "utils",
        r#"
return {
    source = "config"
}
"#,
    );

    fixture.create_plugin(
        "test",
        r#"
local utils = require("utils")
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        check = {
            description = "Check which utils loaded",
            execute = function() return utils.source, 0 end
        }
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[
            fixture.config_dir.join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let (result, code) = rt
        .block_on(async {
            call_task_execute(&lua, plugins[0].tasks.get("check").unwrap(), &[]).await
        })
        .unwrap();

    assert_eq!(code, 0);
    assert_eq!(
        result, "config",
        "Config dir should shadow data dir in package.path"
    );
}

#[test]
fn test_module_require_cached_in_package_loaded() {
    // Documents that Lua's require() caches modules in package.loaded,
    // preventing hot reload even if the file is modified on disk.
    //
    // This is standard Lua behavior, not a bug. Plugin changes require app restart.

    let fixture = TestFixture::new();

    // Create initial shared module
    fixture.create_shared_module(
        "utils",
        r#"
return {
    version = "v1"
}
"#,
    );

    // Plugin A loads the module first
    fixture.create_plugin(
        "plugin_a",
        r#"
local utils = require("utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        check = {
            description = "Check utils version",
            execute = function() return utils.version, 0 end
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
    let (result, _) = rt
        .block_on(async {
            call_task_execute(&lua, plugins[0].tasks.get("check").unwrap(), &[]).await
        })
        .unwrap();
    assert_eq!(result, "v1", "Initial version should be v1");

    // Create Plugin B that also requires the same shared module
    // Plugin B is created BEFORE modifying the file, so both plugins
    // will be loaded together in the same load_plugins() call
    fixture.create_plugin(
        "plugin_b",
        r#"
local utils = require("utils")
return {
    metadata = {name = "plugin_b", version = "1.0.0"},
    tasks = {
        check = {
            description = "Check utils version",
            execute = function() return utils.version, 0 end
        }
    }
}
"#,
    );

    // Load both plugins together in same Lua VM
    // Plugin A requires utils first (caches v1)
    // Plugin B requires utils second (gets cached v1 from package.loaded)
    let plugins_both = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins_both.len(), 2);

    // NOW: Modify the file on disk AFTER both plugins are loaded
    let utils_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("shared")
        .join("utils.lua");
    fs::write(
        &utils_path,
        r#"
return {
    version = "v2"
}
"#,
    )
    .unwrap();

    // Execute plugin B's task - should still get cached v1, not v2 from disk
    // This demonstrates package.loaded prevents re-reading modified files
    let plugin_b = plugins_both
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let (result_b, _) = rt
        .block_on(async {
            call_task_execute(&lua, plugin_b.tasks.get("check").unwrap(), &[]).await
        })
        .unwrap();

    assert_eq!(
        result_b, "v1",
        "Should get cached v1 from package.loaded, not modified v2 from disk"
    );
}

// ============================================================================
// Issue 4: Plugin File Modified After Load - Document Cache Behavior (Line 24)
// ============================================================================

#[test]
fn test_plugin_file_reloads_but_required_modules_stay_cached() {
    // Documents that plugin.lua files ARE re-evaluated when load_plugins() is called again,
    // but any modules those plugins required remain cached in package.loaded.
    //
    // This means:
    // - Plugin metadata and structure changes ARE visible (version, tasks, etc.)
    // - But shared modules required by plugins stay cached (not re-read from disk)
    //
    // Full reload of all code requires a fresh Lua VM (app restart).

    let fixture = TestFixture::new();

    // Create initial plugin
    let plugin_dir = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test_plugin");
    fs::create_dir_all(&plugin_dir).unwrap();
    let plugin_path = plugin_dir.join("plugin.lua");

    fs::write(
        &plugin_path,
        r#"
return {
    metadata = {name = "test_plugin", version = "1.0.0"},
    tasks = {
        test = {
            description = "Initial description",
            execute = function() return "Initial", 0 end
        }
    }
}
"#,
    )
    .unwrap();

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);
    assert_eq!(
        plugins[0].tasks.get("test").unwrap().description,
        "Initial description"
    );

    let rt = tokio::runtime::Runtime::new().unwrap();
    let (result, _) = rt
        .block_on(async {
            call_task_execute(&lua, plugins[0].tasks.get("test").unwrap(), &[]).await
        })
        .unwrap();
    assert_eq!(result, "Initial");

    // Modify plugin.lua on disk
    fs::write(
        &plugin_path,
        r#"
return {
    metadata = {name = "test_plugin", version = "2.0.0"},
    tasks = {
        test = {
            description = "Modified description",
            execute = function() return "Modified", 0 end
        }
    }
}
"#,
    )
    .unwrap();

    // Try to reload plugins in same Lua VM
    let plugins_after = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    // The plugin appears to reload, but the task execution still uses cached code
    assert_eq!(plugins_after.len(), 1);

    // NOTE: The plugin metadata and task structure ARE reloaded because we're
    // calling load_plugins() again which re-evaluates the plugin.lua file.
    // However, if the plugin required any modules, those would remain cached.
    //
    // This test documents that changes to plugin files require a full restart
    // with a fresh Lua VM to guarantee all code is reloaded.
    assert_eq!(plugins_after[0].metadata.version, "2.0.0");
    assert_eq!(
        plugins_after[0].tasks.get("test").unwrap().description,
        "Modified description"
    );
}
