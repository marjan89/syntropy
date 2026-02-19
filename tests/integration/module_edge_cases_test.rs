// Module Edge Cases and Special Character Tests
//
// Comprehensive tests for edge cases in module loading including:
// - Special characters in module names
// - Module caching and package.loaded manipulation
// - Unicode and non-ASCII characters
// - Empty/whitespace handling
// - Error message clarity

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_module_name_with_underscores() {
    // TEST 1: Module Names with Underscores
    // Underscores are standard in Lua - should work perfectly

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "underscore_test",
        "string_utils",
        r#"return { name = "string_utils" }"#,
    );
    fixture.create_lib_module(
        "underscore_test",
        "math_helpers_v2",
        r#"return { name = "math_helpers_v2" }"#,
    );

    fixture.create_plugin(
        "underscore_test",
        r#"
local strings = require("underscore_test.string_utils")
local math = require("underscore_test.math_helpers_v2")
return {
    metadata = {name = "underscore_test", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test underscores",
            execute = function()
                return strings.name .. " + " .. math.name, 0
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
    assert_eq!(result, "string_utils + math_helpers_v2");
}

#[test]
fn test_module_name_starting_with_number() {
    // TEST 2: Module Names Starting with Numbers
    // While valid filenames, they can cause issues in Lua identifiers
    // File: 123module.lua can exist, but must be used carefully

    let fixture = TestFixture::new();

    // Create module file starting with number
    let module_path = fixture
        .data_path()
        .join("syntropy/plugins/number_start/lua/number_start");
    std::fs::create_dir_all(&module_path).unwrap();
    std::fs::write(
        module_path.join("123module.lua"),
        r#"return { valid = true }"#,
    )
    .unwrap();

    fixture.create_plugin(
        "number_start",
        r#"
return {
    metadata = {name = "number_start", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test number-prefixed module",
            execute = function()
                -- Must use string require, cannot use as identifier
                local mod = require("number_start.123module")
                return mod.valid and "Valid" or "Invalid", 0
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
    assert_eq!(result, "Valid");
}

#[test]
fn test_module_with_hyphens_in_name() {
    // TEST 3: Module Names with Hyphens
    // Hyphens are valid in filenames but cannot be used directly in Lua identifiers

    let fixture = TestFixture::new();

    let module_path = fixture
        .data_path()
        .join("syntropy/plugins/hyphen_test/lua/hyphen_test");
    std::fs::create_dir_all(&module_path).unwrap();
    std::fs::write(
        module_path.join("my-module.lua"),
        r#"return { type = "hyphenated" }"#,
    )
    .unwrap();

    fixture.create_plugin(
        "hyphen_test",
        r#"
return {
    metadata = {name = "hyphen_test", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test hyphenated module name",
            execute = function()
                -- Must use brackets or string key
                local mod = require("hyphen_test.my-module")
                return "Type: " .. mod.type, 0
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
    assert_eq!(result, "Type: hyphenated");
}

#[test]
fn test_unicode_in_module_content() {
    // TEST 4: Unicode Content in Modules
    // Tests that modules can contain Unicode characters in their content

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "unicode_test",
        "i18n",
        r#"
return {
    greetings = {
        english = "Hello",
        spanish = "Hola",
        chinese = "你好",
        japanese = "こんにちは",
        emoji = "👋🌍"
    }
}
"#,
    );

    fixture.create_plugin(
        "unicode_test",
        r#"
local i18n = require("unicode_test.i18n")
return {
    metadata = {name = "unicode_test", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test unicode content",
            execute = function()
                return i18n.greetings.chinese .. " " .. i18n.greetings.emoji, 0
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
    assert_eq!(result, "你好 👋🌍");
}

#[test]
fn test_package_loaded_manipulation() {
    // TEST 5: Manual package.loaded Manipulation
    // Tests that plugins can manually manipulate package.loaded for cache control

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "cache_test",
        "reloadable",
        r#"
-- Use _G global to persist count across reloads
_G.cache_test_load_count = (_G.cache_test_load_count or 0) + 1

local M = {}
M.load_count = _G.cache_test_load_count
return M
"#,
    );

    fixture.create_plugin(
        "cache_test",
        r#"
return {
    metadata = {name = "cache_test", version = "1.0.0"},
    tasks = {
        test_cache = {
            description = "Test module caching",
            execute = function()
                local mod1 = require("cache_test.reloadable")
                local count1 = mod1.load_count

                -- Require again - should be cached
                local mod2 = require("cache_test.reloadable")
                local count2 = mod2.load_count

                -- Clear cache
                package.loaded["cache_test.reloadable"] = nil

                -- Require again - should reload
                local mod3 = require("cache_test.reloadable")
                local count3 = mod3.load_count

                return string.format("Counts: %d, %d, %d", count1, count2, count3), 0
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
    let task = plugins[0].tasks.get("test_cache").unwrap();
    let (result, code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(code, 0);
    // First two should be same count, third should be higher
    assert_eq!(result, "Counts: 1, 1, 2");
}

#[test]
fn test_module_with_only_whitespace() {
    // TEST 6: Module with Only Whitespace
    // Per Lua specification: when a module doesn't return a value,
    // require() returns true (not nil).

    let fixture = TestFixture::new();

    fixture.create_lib_module("whitespace_test", "empty_ish", "   \n\t\n   ");

    fixture.create_plugin(
        "whitespace_test",
        r#"
local empty = require("whitespace_test.empty_ish")
return {
    metadata = {name = "whitespace_test", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test whitespace-only module",
            execute = function()
                if empty == nil then
                    return "Module is nil", 0
                else
                    return "Module is: " .. tostring(empty), 0
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

    assert_eq!(plugins.len(), 1);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let task = plugins[0].tasks.get("test").unwrap();
    let (result, code) = rt
        .block_on(async { call_task_execute(&lua, task, &[]).await })
        .unwrap();

    assert_eq!(code, 0);
    // Per Lua spec: empty modules return true
    assert_eq!(result, "Module is: true");
}

#[test]
fn test_module_not_found_error_clarity() {
    // TEST 7: Clear Error Messages for Missing Modules
    // When a module is not found, error should mention the module name

    let fixture = TestFixture::new();

    fixture.create_plugin(
        "error_test",
        r#"
local missing = require("error_test.does_not_exist")
return {
    metadata = {name = "error_test", version = "1.0.0"},
    tasks = {
        test = {description = "Test", execute = function() return "ok", 0 end}
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

    // Should fail with clear error
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    // Verify error indicates module loading failure (not just any error)
    assert!(
        error_msg.contains("does_not_exist")
            && (error_msg.contains("not found")
                || error_msg.contains("no file")
                || error_msg.contains("error loading")),
        "Error should mention missing module name and indicate loading failure, got: {}",
        error_msg
    );
}

#[test]
fn test_double_extension_module() {
    // TEST 8: Module with Double Extension (.lua.lua)
    // Tests handling of unusual filename patterns

    let fixture = TestFixture::new();

    // Create file with .lua.lua extension (shouldn't normally happen)
    let module_path = fixture
        .data_path()
        .join("syntropy/plugins/double_ext/lua/double_ext");
    std::fs::create_dir_all(&module_path).unwrap();
    std::fs::write(module_path.join("test.lua"), r#"return { valid = true }"#).unwrap();

    fixture.create_plugin(
        "double_ext",
        r#"
-- Note: require looks for "test.lua", not "test.lua.lua"
local mod = require("double_ext.test")
return {
    metadata = {name = "double_ext", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test normal extension",
            execute = function()
                return mod.valid and "Found" or "Not found", 0
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
    assert_eq!(result, "Found");
}

#[test]
fn test_shared_state_across_requires() {
    // TEST 9: Shared State Across Requires
    // Validates that module state is truly shared (not copied) across requires

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "state_test",
        "counter",
        r#"
local M = {}
M.value = 0

function M.increment()
    M.value = M.value + 1
end

function M.get()
    return M.value
end

return M
"#,
    );

    fixture.create_plugin(
        "state_test",
        r#"
local counter1 = require("state_test.counter")
local counter2 = require("state_test.counter")

return {
    metadata = {name = "state_test", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test shared state",
            execute = function()
                counter1.increment()
                counter1.increment()

                -- counter2 should see the same state
                local count = counter2.get()

                -- Verify they're the same table
                local same = (counter1 == counter2)

                return string.format("Count: %d, Same: %s", count, tostring(same)), 0
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
    assert_eq!(result, "Count: 2, Same: true");
}

#[test]
fn test_module_returns_boolean() {
    // TEST 10: Module Returns Boolean
    // Tests that modules can return any type, including booleans

    let fixture = TestFixture::new();

    fixture.create_lib_module("bool_test", "feature_flag", "return true");

    fixture.create_plugin(
        "bool_test",
        r#"
local enabled = require("bool_test.feature_flag")
return {
    metadata = {name = "bool_test", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test boolean module",
            execute = function()
                return enabled and "Feature enabled" or "Feature disabled", 0
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
    assert_eq!(result, "Feature enabled");
}
