// Integration tests for shared module loading
//
// Context: Shared modules in plugins/shared/ should be accessible to all plugins
// via require(). This tests the bug fix where env::current_dir() was incorrectly
// used instead of deriving shared paths from plugin directories.
//
// These tests ensure:
// 1. Shared modules can be required from any plugin
// 2. Multiple plugins can require the same shared module
// 3. Plugin lib/ modules take precedence over shared modules (vendoring)
// 4. Shared modules work from both config and data directories
// 5. Clear error messages when shared modules are not found
// 6. Shared modules work in runtime execution (not just load time)

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_basic_shared_module_require() {
    // BASIC TEST: A single plugin can require a shared module
    let fixture = TestFixture::new();

    // Create shared module
    fixture.create_shared_module(
        "string_utils",
        r#"
local utils = {}

function utils.trim(s)
    return s:match("^%s*(.-)%s*$")
end

function utils.upper(s)
    return string.upper(s)
end

return utils
"#,
    );

    // Create plugin that uses shared module
    fixture.create_plugin(
        "plugin_a",
        r#"
local string_utils = require("string_utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        trim_text = {
            description = "Test shared module trim function",
            execute = function()
                local result = string_utils.trim("  hello  ")
                return result, 0
            end
        }
    }
}
"#,
    );

    // Load plugin
    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);

    // Execute task that uses shared module
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("trim_text").unwrap();

    let (output, exit_code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(output, "hello");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_shared_module_multiple_plugins() {
    // TEST: Multiple plugins can require and use the same shared module
    let fixture = TestFixture::new();

    // Create shared module
    fixture.create_shared_module(
        "math_utils",
        r#"
local utils = {}

function utils.add(a, b)
    return a + b
end

function utils.multiply(a, b)
    return a * b
end

return utils
"#,
    );

    // Create first plugin using shared module
    fixture.create_plugin(
        "plugin_a",
        r#"
local math_utils = require("math_utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        calc = {
            description = "Test task",
            execute = function()
                return tostring(math_utils.add(2, 3)), 0
            end
        }
    }
}
"#,
    );

    // Create second plugin using same shared module
    fixture.create_plugin(
        "plugin_b",
        r#"
local math_utils = require("math_utils")
return {
    metadata = {name = "plugin_b", version = "1.0.0"},
    tasks = {
        calc = {
            description = "Test task",
            execute = function()
                return tostring(math_utils.multiply(4, 5)), 0
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

    // Both plugins should successfully execute using the shared module
    let plugin_a = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_a")
        .unwrap();
    let task_a = plugin_a.tasks.get("calc").unwrap();
    let (output_a, _) = rt
        .block_on(async { call_task_execute(&lua, task_a, &[]).await })
        .unwrap();
    assert_eq!(output_a, "5");

    let plugin_b = plugins
        .iter()
        .find(|p| p.metadata.name == "plugin_b")
        .unwrap();
    let task_b = plugin_b.tasks.get("calc").unwrap();
    let (output_b, _) = rt
        .block_on(async { call_task_execute(&lua, task_b, &[]).await })
        .unwrap();
    assert_eq!(output_b, "20");
}

#[test]
fn test_shared_module_precedence_plugin_lib_wins() {
    // TEST: Plugin lib/ modules take precedence over shared modules (vendoring)
    let fixture = TestFixture::new();

    // Create shared module
    fixture.create_shared_module(
        "utils",
        r#"
return {
    version = "shared"
}
"#,
    );

    // Create plugin with its own lua/utils.lua (flat vendored module) that shadows the shared module
    // Vendored modules go in lua/ directly, not lua/plugin_name/
    let vendored_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("plugin_a")
        .join("lua");
    std::fs::create_dir_all(&vendored_path).unwrap();
    std::fs::write(
        vendored_path.join("utils.lua"),
        r#"
return {
    version = "vendored"
}
"#,
    )
    .unwrap();

    fixture.create_plugin(
        "plugin_a",
        r#"
local utils = require("utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        check = {
            description = "Test task",
            execute = function()
                return utils.version, 0
            end
        }
    }
}
"#,
    );

    // Load plugin
    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);

    // Execute task - should use vendored version, not shared
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("check").unwrap();

    let (output, _) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(output, "vendored");
}

#[test]
fn test_plugin_lib_overrides_config_shared() {
    // TEST: Plugin lib/ modules take precedence over config shared/ modules
    // This verifies that the vendoring mechanism (plugin lib/ directory) correctly
    // overrides shared modules from the config directory, not just data directory.
    let fixture = TestFixture::new();

    // Create shared module in config dir
    fixture.create_shared_module_override(
        "utils",
        r#"
return {
    version = "config_shared"
}
"#,
    );

    // Create plugin with its own lua/utils.lua (flat vendored module) that shadows the config shared module
    // Vendored modules go in lua/ directly, not lua/plugin_name/
    let vendored_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("plugin_a")
        .join("lua");
    std::fs::create_dir_all(&vendored_path).unwrap();
    std::fs::write(
        vendored_path.join("utils.lua"),
        r#"
return {
    version = "vendored"
}
"#,
    )
    .unwrap();

    fixture.create_plugin(
        "plugin_a",
        r#"
local utils = require("utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        check = {
            description = "Test task",
            execute = function()
                return utils.version, 0
            end
        }
    }
}
"#,
    );

    // Load plugin with both config and data directories
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

    assert_eq!(plugins.len(), 1);

    // Execute task - should use vendored version, not config shared
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("check").unwrap();

    let (output, exit_code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(output, "vendored");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_plugin_lib_overrides_both_shared_dirs() {
    // TEST: Plugin lib/ modules take precedence over BOTH shared/ directories
    // This verifies the complete precedence hierarchy:
    // 1. Plugin lib/ (highest precedence - vendored modules)
    // 2. Config shared/ (middle precedence)
    // 3. Data shared/ (lowest precedence)
    let fixture = TestFixture::new();

    // Create shared module in data dir
    fixture.create_shared_module(
        "utils",
        r#"
return {
    version = "data_shared"
}
"#,
    );

    // Create shared module in config dir (should override data, but not plugin lib/)
    fixture.create_shared_module_override(
        "utils",
        r#"
return {
    version = "config_shared"
}
"#,
    );

    // Create plugin with its own lua/utils.lua (flat vendored module) that shadows both shared modules
    // Vendored modules go in lua/ directly, not lua/plugin_name/
    let vendored_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("plugin_a")
        .join("lua");
    std::fs::create_dir_all(&vendored_path).unwrap();
    std::fs::write(
        vendored_path.join("utils.lua"),
        r#"
return {
    version = "vendored"
}
"#,
    )
    .unwrap();

    fixture.create_plugin(
        "plugin_a",
        r#"
local utils = require("utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        check = {
            description = "Test task",
            execute = function()
                return utils.version, 0
            end
        }
    }
}
"#,
    );

    // Load plugin with both config and data directories
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

    assert_eq!(plugins.len(), 1);

    // Execute task - should use vendored version, not either shared version
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("check").unwrap();

    let (output, exit_code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(output, "vendored");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_shared_module_from_both_dirs() {
    // TEST: Shared modules work from both config and data directories
    let fixture = TestFixture::new();

    // Create shared module in data dir
    fixture.create_shared_module(
        "data_utils",
        r#"
return {
    source = "data"
}
"#,
    );

    // Create shared module in config dir (override)
    fixture.create_shared_module_override(
        "config_utils",
        r#"
return {
    source = "config"
}
"#,
    );

    // Create plugin that uses both
    fixture.create_plugin(
        "plugin_a",
        r#"
local data_utils = require("data_utils")
local config_utils = require("config_utils")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        check = {
            description = "Test task",
            execute = function()
                return data_utils.source .. "," .. config_utils.source, 0
            end
        }
    }
}
"#,
    );

    // Load plugin with both directories
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

    assert_eq!(plugins.len(), 1);

    // Execute task
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("check").unwrap();

    let (output, _) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(output, "data,config");
}

#[test]
fn test_shared_module_not_found_error() {
    // TEST: Clear error when shared module doesn't exist
    let fixture = TestFixture::new();

    // Create plugin that tries to require non-existent shared module
    fixture.create_plugin(
        "plugin_a",
        r#"
local missing = require("nonexistent_module")
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test task",
            execute = function() return "ok", 0 end
        }
    }
}
"#,
    );

    // Load plugin - should fail with clear error
    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let result = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    );

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = format!("{:?}", error); // Use Debug format to see the full error chain
    // The error should mention either the failed peek or the missing module
    assert!(
        error_msg.contains("nonexistent_module") || error_msg.contains("Failed to peek plugin"),
        "Expected error about nonexistent module or failed peek, got: {}",
        error_msg
    );
}

#[test]
fn test_shared_module_runtime_require() {
    // TEST: Shared modules can be required at runtime (in execute function)
    let fixture = TestFixture::new();

    // Create shared module
    fixture.create_shared_module(
        "runtime_utils",
        r#"
return {
    greet = function(name) return "Hello, " .. name end
}
"#,
    );

    // Create plugin that requires module at runtime, not load time
    fixture.create_plugin(
        "plugin_a",
        r#"
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {
        greet = {
            description = "Test task",
            execute = function()
                -- Require at runtime, not during plugin load
                local runtime_utils = require("runtime_utils")
                return runtime_utils.greet("World"), 0
            end
        }
    }
}
"#,
    );

    // Load plugin
    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);

    // Execute task that requires module at runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = &plugins[0];
    let task = plugin.tasks.get("greet").unwrap();

    let (output, exit_code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(output, "Hello, World");
    assert_eq!(exit_code, 0);
}
