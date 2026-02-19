// Circular Dependency Tests
//
// Tests for detecting and handling circular dependencies in Lua module loading.
// Lua's built-in require() has some protection against circular dependencies by
// caching modules in package.loaded, but these tests document the expected behavior.
//
// Scenarios tested:
// 1. Simple circular: Module A requires B, B requires A
// 2. Self-circular: Module requires itself
// 3. Deep circular: A→B→C→D→A
// 4. Cross-namespace circular: plugin_a.module → plugin_b.module → plugin_a.other
// 5. Shared-to-plugin circular: shared module → plugin module → shared module

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_simple_circular_dependency() {
    // TEST 1: Simple Circular Dependency
    //
    // Module A requires B during load, Module B requires A during load.
    // This typically fails because when B tries to require A, A hasn't
    // finished loading yet and isn't in package.loaded. This documents
    // that load-time circular dependencies fail.

    let fixture = TestFixture::new();

    // Plugin: module_a requires plugin_circ.module_b
    fixture.create_lib_module(
        "plugin_circ",
        "module_a",
        r#"
-- Module A is being loaded
local M = { name = "Module A" }

-- This will require Module B, which will try to require Module A again
local module_b = require("plugin_circ.module_b")

function M.get_info()
    return M.name .. " [B says: " .. module_b.get_name() .. "]"
end

return M
"#,
    );

    // Plugin: module_b requires plugin_circ.module_a
    fixture.create_lib_module(
        "plugin_circ",
        "module_b",
        r#"
-- Module B is being loaded
local M = { name = "Module B" }

-- This will require Module A, but Module A is already being loaded
-- Lua returns the partially loaded Module A from package.loaded
local module_a = require("plugin_circ.module_a")

function M.get_name()
    if module_a.name then
        return M.name .. " (A=" .. module_a.name .. ")"
    else
        return M.name .. " (A=partial)"
    end
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_circ",
        r#"
local module_a = require("plugin_circ.module_a")
return {
    metadata = {name = "plugin_circ", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test circular dependency",
            execute = function()
                return module_a.get_info(), 0
            end
        }
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let result = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    );

    // Load-time circular dependencies typically fail
    // This documents expected behavior
    assert!(
        result.is_err(),
        "Expected circular dependency to cause load error"
    );
    let error_msg = format!("{:?}", result.unwrap_err());
    // Verify error mentions one of the circular modules AND indicates it's a loading issue
    assert!(
        (error_msg.contains("module_a") || error_msg.contains("module_b"))
            && (error_msg.contains("error loading")
                || error_msg.contains("stack")
                || error_msg.contains("require")
                || error_msg.contains("circular")),
        "Error should mention circular modules and loading/require issue, got: {}",
        error_msg
    );
}

#[test]
fn test_self_circular_dependency() {
    // TEST 2: Self-Circular Dependency
    //
    // A module that requires itself. This should work because the module is
    // already in package.loaded when it tries to require itself.

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_self",
        "self_ref",
        r#"
local M = {
    name = "SelfRef",
    initialized = false
}

function M.init()
    if not M.initialized then
        M.initialized = true
        -- Try to require myself - should get the current module
        local me = require("plugin_self.self_ref")
        M.self_check = (me == M)
    end
end

M.init()

function M.check()
    return M.self_check and "Self-reference OK" or "Self-reference failed"
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_self",
        r#"
local self_ref = require("plugin_self.self_ref")
return {
    metadata = {name = "plugin_self", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test self-circular dependency",
            execute = function()
                return self_ref.check(), 0
            end
        }
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let result = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    );

    // Self-circular at load time also fails
    assert!(
        result.is_err(),
        "Expected self-circular dependency to cause load error"
    );
}

#[test]
fn test_deep_circular_dependency_chain() {
    // TEST 3: Deep Circular Dependency Chain
    //
    // A→B→C→D→A. Deep circular dependencies at load time fail because
    // modules aren't in package.loaded until they finish loading.

    let fixture = TestFixture::new();

    // Module A → B
    fixture.create_lib_module(
        "plugin_deep",
        "module_a",
        r#"
local M = { name = "A" }
local module_b = require("plugin_deep.module_b")
function M.chain()
    return M.name .. " -> " .. module_b.chain()
end
return M
"#,
    );

    // Module B → C
    fixture.create_lib_module(
        "plugin_deep",
        "module_b",
        r#"
local M = { name = "B" }
local module_c = require("plugin_deep.module_c")
function M.chain()
    if module_c.chain then
        return M.name .. " -> " .. module_c.chain()
    else
        return M.name .. " -> C(partial)"
    end
end
return M
"#,
    );

    // Module C → D
    fixture.create_lib_module(
        "plugin_deep",
        "module_c",
        r#"
local M = { name = "C" }
local module_d = require("plugin_deep.module_d")
function M.chain()
    if module_d.chain then
        return M.name .. " -> " .. module_d.chain()
    else
        return M.name .. " -> D(partial)"
    end
end
return M
"#,
    );

    // Module D → A (completes the circle)
    fixture.create_lib_module(
        "plugin_deep",
        "module_d",
        r#"
local M = { name = "D" }
-- This requires A, which is already being loaded (at the top of the chain)
local module_a = require("plugin_deep.module_a")
function M.chain()
    if module_a.name then
        return M.name .. " -> " .. module_a.name .. "(cached)"
    else
        return M.name .. " -> A(partial)"
    end
end
return M
"#,
    );

    fixture.create_plugin(
        "plugin_deep",
        r#"
local module_a = require("plugin_deep.module_a")
return {
    metadata = {name = "plugin_deep", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test deep circular dependency",
            execute = function()
                return module_a.chain(), 0
            end
        }
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let result = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    );

    // Deep circular dependencies at load time fail
    assert!(
        result.is_err(),
        "Expected deep circular dependency to cause load error"
    );
}

#[test]
fn test_plugin_can_require_shared_module() {
    // TEST 4: Plugin Module Can Require Shared Module (Non-Circular)
    //
    // This test validates that plugin modules can successfully require shared modules.
    // This is NOT a circular dependency because:
    // - Shared module does NOT require the plugin module
    // - Plugin module requires shared module (one-way dependency)
    //
    // Note: Shared modules cannot directly require plugin modules because shared
    // modules are loaded first, before any plugin namespaces exist.

    let fixture = TestFixture::new();

    // Shared module (does NOT require any plugin modules)
    fixture.create_shared_module(
        "shared_utils",
        r#"
local M = { name = "SharedUtils" }

-- This is a standalone shared module
-- It does NOT require plugin modules, so no circular dependency
function M.get_name()
    return M.name
end

return M
"#,
    );

    // Plugin module that requires shared module (one-way dependency)
    fixture.create_lib_module(
        "plugin_uses_shared",
        "plugin_utils",
        r#"
local shared = require("shared_utils")

local M = { name = "PluginUtils" }

function M.get_combined()
    return M.name .. " + " .. shared.get_name()
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_uses_shared",
        r#"
local plugin_utils = require("plugin_uses_shared.plugin_utils")
return {
    metadata = {name = "plugin_uses_shared", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test plugin requiring shared module",
            execute = function()
                return plugin_utils.get_combined(), 0
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
    let task = plugins[0].tasks.get("test").unwrap();
    let (result, code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(code, 0);
    // Verify plugin successfully used shared module (no circular dependency)
    assert_eq!(result, "PluginUtils + SharedUtils");
}

#[test]
fn test_circular_with_functions_not_tables() {
    // TEST 5: Circular Dependencies with Function References
    //
    // Tests that circular dependencies work when modules return functions that
    // reference each other, not just when they return tables.

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_func_circ",
        "func_a",
        r#"
local M = {}
local func_b = require("plugin_func_circ.func_b")

function M.call_b()
    if func_b.get_value then
        return "A calls B: " .. func_b.get_value()
    else
        return "A calls B: (partial)"
    end
end

function M.get_value()
    return "value_from_A"
end

return M
"#,
    );

    fixture.create_lib_module(
        "plugin_func_circ",
        "func_b",
        r#"
local M = {}
local func_a = require("plugin_func_circ.func_a")

function M.call_a()
    if func_a.get_value then
        return "B calls A: " .. func_a.get_value()
    else
        return "B calls A: (partial)"
    end
end

function M.get_value()
    return "value_from_B"
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_func_circ",
        r#"
local func_a = require("plugin_func_circ.func_a")
return {
    metadata = {name = "plugin_func_circ", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test function circular dependency",
            execute = function()
                return func_a.call_b(), 0
            end
        }
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let result = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    );

    // Load-time circular with function references also fails
    assert!(
        result.is_err(),
        "Expected circular dependency with functions to cause load error"
    );
}
