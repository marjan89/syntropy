// Plugin Module Isolation Tests
//
// These tests validate that plugins have isolated namespaces for their lua/ modules.
// With the Neovim-style namespaced module system (lua/<pluginname>/), each plugin
// has its own namespace preventing module name collisions.
//
// Architecture:
// - Plugin A: lua/plugin_a/module.lua → require("plugin_a.module")
// - Plugin B: lua/plugin_b/module.lua → require("plugin_b.module")
// - Shared: plugins/shared/module.lua → require("module")
//
// These tests verify that:
// 1. Each plugin loads its own namespaced modules
// 2. Plugins cannot access each other's private modules
// 3. Shared modules remain accessible to all plugins
// 4. Nested module requires work within each plugin's namespace

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_basic_lib_module_isolation() {
    // TEST 1: Basic Isolation
    //
    // Validates that each plugin loads its own version of lua/pluginname/devices.lua
    // using namespaced requires. Even though both plugins have a "devices" module,
    // they load their own versions via require("plugin_name.devices").

    let fixture = TestFixture::new();

    // Plugin A: audio_plugin with lib/devices.lua returning source="audio"
    fixture.create_lib_module(
        "audio_plugin",
        "devices",
        r#"
return {
    source = "audio"
}
"#,
    );

    fixture.create_plugin(
        "audio_plugin",
        r#"
local devices = require("audio_plugin.devices")
return {
    metadata = {
        name = "audio_plugin",
        version = "1.0.0",
        description = "Audio device management",
    },
    tasks = {
        get_source = {
            description = "Get audio device source",
            execute = function()
                return devices.source, 0
            end
        }
    }
}
"#,
    );

    // Plugin B: bluetooth_plugin with lib/devices.lua returning source="bluetooth"
    fixture.create_lib_module(
        "bluetooth_plugin",
        "devices",
        r#"
return {
    source = "bluetooth"
}
"#,
    );

    fixture.create_plugin(
        "bluetooth_plugin",
        r#"
local devices = require("bluetooth_plugin.devices")
return {
    metadata = {
        name = "bluetooth_plugin",
        version = "1.0.0",
        description = "Bluetooth device management",
    },
    tasks = {
        get_source = {
            description = "Get bluetooth device source",
            execute = function()
                return devices.source, 0
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

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Plugin A gets "audio" via require("audio_plugin.devices")
    let audio_plugin = plugins
        .iter()
        .find(|p| p.metadata.name == "audio_plugin")
        .unwrap();
    let task_a = audio_plugin.tasks.get("get_source").unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();

    // Plugin B gets "bluetooth" via require("bluetooth_plugin.devices")
    let bluetooth_plugin = plugins
        .iter()
        .find(|p| p.metadata.name == "bluetooth_plugin")
        .unwrap();
    let task_b = bluetooth_plugin.tasks.get("get_source").unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();

    // Namespaced requires ensure each plugin loads its own module
    assert_eq!(
        result_a, "audio",
        "audio_plugin loads its own lua/audio_plugin/devices.lua"
    );
    assert_eq!(
        result_b, "bluetooth",
        "bluetooth_plugin loads its own lua/bluetooth_plugin/devices.lua"
    );
}

#[test]
fn test_complex_lib_module_isolation_with_functions() {
    // TEST 2: Complex Isolation with Functions
    //
    // Validates that each plugin's namespaced utils module has different function
    // implementations. This ensures isolation works for functions, not just data tables.

    let fixture = TestFixture::new();

    // Plugin A: lib/utils.lua with greet() returning "Hello from A"
    fixture.create_lib_module(
        "plugin_a",
        "utils",
        r#"
local M = {}

function M.greet()
    return "Hello from A"
end

function M.get_name()
    return "Plugin A"
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_a",
        r#"
local utils = require("plugin_a.utils")
return {
    metadata = {
        name = "plugin_a",
        version = "1.0.0",
    },
    tasks = {
        greet = {
            description = "Greet from plugin A",
            execute = function()
                return utils.greet() .. " [" .. utils.get_name() .. "]", 0
            end
        }
    }
}
"#,
    );

    // Plugin B: lib/utils.lua with greet() returning "Hello from B"
    fixture.create_lib_module(
        "plugin_b",
        "utils",
        r#"
local M = {}

function M.greet()
    return "Hello from B"
end

function M.get_name()
    return "Plugin B"
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_b",
        r#"
local utils = require("plugin_b.utils")
return {
    metadata = {
        name = "plugin_b",
        version = "1.0.0",
    },
    tasks = {
        greet = {
            description = "Greet from plugin B",
            execute = function()
                return utils.greet() .. " [" .. utils.get_name() .. "]", 0
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

    // Execute both plugins' tasks
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let task_a = plugin_a.tasks.get("greet").unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();

    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let task_b = plugin_b.tasks.get("greet").unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();

    // Each plugin gets its own function implementations via namespaced requires
    assert_eq!(
        result_a, "Hello from A [Plugin A]",
        "plugin_a loads its own lua/plugin_a/utils.lua with its own functions"
    );
    assert_eq!(
        result_b, "Hello from B [Plugin B]",
        "plugin_b loads its own lua/plugin_b/utils.lua with its own functions"
    );
}

#[test]
fn test_nested_module_isolation() {
    // TEST 3: Nested Module Isolation
    //
    // Validates that when plugin modules require other modules from the same plugin,
    // they load from their own namespace. Plugin A's helper requires plugin_a.config,
    // Plugin B's helper requires plugin_b.config.

    let fixture = TestFixture::new();

    // Plugin A: lib/config.lua
    fixture.create_lib_module(
        "plugin_a",
        "config",
        r#"
return {
    api_key = "key_from_a",
    endpoint = "https://api-a.example.com"
}
"#,
    );

    // Plugin A: lib/helper.lua (requires config)
    fixture.create_lib_module(
        "plugin_a",
        "helper",
        r#"
local config = require("plugin_a.config")

local M = {}

function M.get_endpoint()
    return config.endpoint
end

function M.get_api_key()
    return config.api_key
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_a",
        r#"
local helper = require("plugin_a.helper")
return {
    metadata = {
        name = "plugin_a",
        version = "1.0.0",
    },
    tasks = {
        check = {
            description = "Check config from plugin A",
            execute = function()
                return helper.get_endpoint() .. " [" .. helper.get_api_key() .. "]", 0
            end
        }
    }
}
"#,
    );

    // Plugin B: lib/config.lua (different config)
    fixture.create_lib_module(
        "plugin_b",
        "config",
        r#"
return {
    api_key = "key_from_b",
    endpoint = "https://api-b.example.com"
}
"#,
    );

    // Plugin B: lib/helper.lua (requires config)
    fixture.create_lib_module(
        "plugin_b",
        "helper",
        r#"
local config = require("plugin_b.config")

local M = {}

function M.get_endpoint()
    return config.endpoint
end

function M.get_api_key()
    return config.api_key
end

return M
"#,
    );

    fixture.create_plugin(
        "plugin_b",
        r#"
local helper = require("plugin_b.helper")
return {
    metadata = {
        name = "plugin_b",
        version = "1.0.0",
    },
    tasks = {
        check = {
            description = "Check config from plugin B",
            execute = function()
                return helper.get_endpoint() .. " [" .. helper.get_api_key() .. "]", 0
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

    // Execute both plugins' tasks
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let task_a = plugin_a.tasks.get("check").unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();

    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let task_b = plugin_b.tasks.get("check").unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();

    // Nested requires stay within each plugin's namespace
    assert_eq!(
        result_a, "https://api-a.example.com [key_from_a]",
        "plugin_a's helper correctly loads plugin_a's config via namespaced require"
    );
    assert_eq!(
        result_b, "https://api-b.example.com [key_from_b]",
        "plugin_b's helper correctly loads plugin_b's config via namespaced require"
    );
}

#[test]
fn test_shared_module_works_across_plugins() {
    // TEST 4: Shared Module Still Works
    //
    // Validates that plugins can still access shared modules from plugins/shared/
    // using flat (non-namespaced) requires. The shared/ namespace is global and
    // accessible to all plugins.

    let fixture = TestFixture::new();

    // Create a shared module available to all plugins
    fixture.create_shared_module(
        "common",
        r#"
local M = {}

M.VERSION = "1.0.0"

function M.format_output(message)
    return "[COMMON] " .. message
end

function M.get_platform()
    return "shared_platform"
end

return M
"#,
    );

    // Plugin A uses shared/common.lua
    fixture.create_plugin(
        "plugin_a",
        r#"
local common = require("common")
return {
    metadata = {
        name = "plugin_a",
        version = "1.0.0",
    },
    tasks = {
        use_common = {
            description = "Use shared common module",
            execute = function()
                return common.format_output("from A") .. " v" .. common.VERSION, 0
            end
        }
    }
}
"#,
    );

    // Plugin B uses the same shared/common.lua
    fixture.create_plugin(
        "plugin_b",
        r#"
local common = require("common")
return {
    metadata = {
        name = "plugin_b",
        version = "1.0.0",
    },
    tasks = {
        use_common = {
            description = "Use shared common module",
            execute = function()
                return common.format_output("from B") .. " platform:" .. common.get_platform(), 0
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

    // Both plugins should successfully load the shared module
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let task_a = plugin_a.tasks.get("use_common").unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();

    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let task_b = plugin_b.tasks.get("use_common").unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();

    // Both plugins successfully use the SAME shared module (global namespace)
    assert_eq!(
        result_a, "[COMMON] from A v1.0.0",
        "Plugin A successfully loads shared/common.lua"
    );
    assert_eq!(
        result_b, "[COMMON] from B platform:shared_platform",
        "Plugin B successfully loads shared/common.lua"
    );
}

#[test]
fn test_cannot_access_other_plugins_lib_modules() {
    // TEST 5: Cannot Access Other Plugin's Lib
    //
    // Validates that Plugin B cannot access Plugin A's private module using
    // a flat require("private"). With namespacing, Plugin B would need to use
    // require("plugin_a.private") which is namespaced, so a flat require
    // should fail with module-not-found.

    let fixture = TestFixture::new();

    // Plugin A: has lib/private.lua with sensitive data
    fixture.create_lib_module(
        "plugin_a",
        "private",
        r#"
return {
    secret_key = "super_secret_api_key_from_a",
    internal_data = "confidential"
}
"#,
    );

    fixture.create_plugin(
        "plugin_a",
        r#"
local private = require("plugin_a.private")
return {
    metadata = {
        name = "plugin_a",
        version = "1.0.0",
    },
    tasks = {
        use_private = {
            description = "Use private module",
            execute = function()
                return "Plugin A secret: " .. private.secret_key, 0
            end
        }
    }
}
"#,
    );

    // Plugin B: tries to access Plugin A's private module
    fixture.create_plugin(
        "plugin_b",
        r#"
return {
    metadata = {
        name = "plugin_b",
        version = "1.0.0",
    },
    tasks = {
        steal_private = {
            description = "Try to access plugin A's private module",
            execute = function()
                -- This should FAIL with module-not-found error
                local success, private = pcall(require, "private")

                if success then
                    -- This would only succeed if there was a shared module named "private"
                    return "Plugin B stole secret: " .. private.secret_key, 0
                else
                    -- Expected: flat require("private") fails because module is namespaced
                    return "Cannot access private module (correct!)", 0
                end
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

    // Plugin A should successfully use its own private module
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let task_a = plugin_a.tasks.get("use_private").unwrap();
    let (result_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();

    assert_eq!(
        result_a, "Plugin A secret: super_secret_api_key_from_a",
        "Plugin A should access its own private module"
    );

    // Plugin B should NOT be able to access Plugin A's private module
    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let task_b = plugin_b.tasks.get("steal_private").unwrap();
    let (result_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();

    // Flat require("private") fails because modules are namespaced
    assert_eq!(
        result_b, "Cannot access private module (correct!)",
        "Plugin B cannot load plugin_a's private module using flat require('private')"
    );
}

#[test]
fn test_isolation_with_mixed_lib_presence() {
    // TEST 6: Isolation with Mixed Lib Presence
    //
    // Validates the edge case where some plugins have modules and others don't.
    // Plugin with_helper has lua/with_helper/helper.lua, plugin without_helper
    // does not. A flat require("helper") should fail for without_helper.

    let fixture = TestFixture::new();

    // Plugin A: has lib/helper.lua
    fixture.create_lib_module(
        "with_helper",
        "helper",
        r#"
return {
    value = "from_with_helper"
}
"#,
    );

    fixture.create_plugin(
        "with_helper",
        r#"
local helper = require("with_helper.helper")
return {
    metadata = {
        name = "with_helper",
        version = "1.0.0",
    },
    tasks = {
        use_helper = {
            description = "Use helper module",
            execute = function()
                return helper.value, 0
            end
        }
    }
}
"#,
    );

    // Plugin B: NO lib/ directory at all, but tries to use helper
    fixture.create_plugin(
        "without_helper",
        r#"
return {
    metadata = {
        name = "without_helper",
        version = "1.0.0",
    },
    tasks = {
        try_helper = {
            description = "Try to use helper module",
            execute = function()
                local success, helper = pcall(require, "helper")

                if success then
                    -- Would only succeed if there was a shared module named "helper"
                    return "Got helper: " .. helper.value, 0
                else
                    -- Expected: flat require("helper") fails (module is namespaced)
                    return "No helper available", 0
                end
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

    // Plugin with helper should successfully use it
    let with_helper = plugins
        .iter()
        .find(|p| p.metadata.name == "with_helper")
        .unwrap();
    let task_with = with_helper.tasks.get("use_helper").unwrap();
    let (result_with, _) = rt
        .block_on(async { call_task_execute(&lua, task_with, &[]).await })
        .unwrap();

    assert_eq!(
        result_with, "from_with_helper",
        "Plugin with lib modules should get its own, plugin without lib should not interfere"
    );

    // Plugin without helper should NOT get another plugin's module
    let without_helper = plugins
        .iter()
        .find(|p| p.metadata.name == "without_helper")
        .unwrap();
    let task_without = without_helper.tasks.get("try_helper").unwrap();
    let (result_without, _) = rt
        .block_on(async { call_task_execute(&lua, task_without, &[]).await })
        .unwrap();

    // Flat require("helper") fails because helper module is namespaced
    assert_eq!(
        result_without, "No helper available",
        "Plugin without lua/without_helper/helper.lua cannot load with_helper's module via flat require"
    );
}
