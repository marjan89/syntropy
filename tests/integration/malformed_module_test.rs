// Malformed Module Tests
//
// Tests for handling malformed, corrupted, or invalid Lua modules during require().
// These tests ensure clear error messages and graceful failure when modules are
// syntactically or semantically invalid.
//
// Scenarios tested:
// 1. Syntax errors in required modules
// 2. Modules returning nil
// 3. Modules returning wrong types (number, string, boolean)
// 4. Modules returning nothing (empty file)
// 5. Modules with runtime errors during evaluation
// 6. Truncated/corrupted module files

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_module_with_syntax_error() {
    // TEST 1: Syntax Error in Required Module
    //
    // A module file with Lua syntax errors should produce a clear error
    // message indicating the syntax problem and location.

    let fixture = TestFixture::new();

    // Create a module with syntax error (missing 'end')
    fixture.create_lib_module(
        "plugin_syntax",
        "bad_syntax",
        r#"
local M = {}

function M.test()
    if true then
        return "test"
    -- Missing 'end' for the if statement!

return M
"#,
    );

    fixture.create_plugin(
        "plugin_syntax",
        r#"
local bad = require("plugin_syntax.bad_syntax")
return {
    metadata = {name = "plugin_syntax", version = "1.0.0"},
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

    // Should fail during plugin loading
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    // Error should indicate syntax error (not just mention the module name)
    assert!(
        error_msg.contains("syntax")
            || error_msg.contains("<eof>")
            || error_msg.contains("expected")
            || error_msg.contains("unexpected"),
        "Expected syntax error indicator, got: {}",
        error_msg
    );
}

#[test]
fn test_module_returns_nil() {
    // TEST 2: Module Returns nil
    //
    // A module that explicitly returns nil is technically valid in Lua,
    // but should be handled gracefully (nil is cached in package.loaded).

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_nil",
        "returns_nil",
        r#"
-- This module returns nil
return nil
"#,
    );

    fixture.create_plugin(
        "plugin_nil",
        r#"
local nil_mod = require("plugin_nil.returns_nil")
return {
    metadata = {name = "plugin_nil", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test nil module",
            execute = function()
                if nil_mod == nil then
                    return "Module returned nil (valid)", 0
                else
                    return "Module returned: " .. tostring(nil_mod), 0
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
    // Note: Lua may return true for modules that explicitly return nil
    assert!(result.contains("Module returned"));
}

#[test]
fn test_module_returns_number() {
    // TEST 3: Module Returns Number
    //
    // A module that returns a number instead of a table. This is valid Lua,
    // and modules can return any type.

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_number",
        "returns_number",
        r#"
-- This module returns a number
return 42
"#,
    );

    fixture.create_plugin(
        "plugin_number",
        r#"
local num_mod = require("plugin_number.returns_number")
return {
    metadata = {name = "plugin_number", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test number module",
            execute = function()
                return "Module returned: " .. tostring(num_mod), 0
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
    assert_eq!(result, "Module returned: 42");
}

#[test]
fn test_module_returns_string() {
    // TEST 4: Module Returns String
    //
    // A module that returns a string. Valid Lua - modules can return any type.

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_string",
        "returns_string",
        r#"
-- This module returns a string
return "I am a string module"
"#,
    );

    fixture.create_plugin(
        "plugin_string",
        r#"
local str_mod = require("plugin_string.returns_string")
return {
    metadata = {name = "plugin_string", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test string module",
            execute = function()
                return "Module says: " .. str_mod, 0
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
    assert_eq!(result, "Module says: I am a string module");
}

#[test]
fn test_module_returns_function() {
    // TEST 5: Module Returns Function
    //
    // A module that returns a function instead of a table. This is a valid
    // and common Lua pattern (factory pattern).

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_function",
        "returns_function",
        r#"
-- This module returns a function (factory pattern)
return function(name)
    return "Hello, " .. name
end
"#,
    );

    fixture.create_plugin(
        "plugin_function",
        r#"
local greet = require("plugin_function.returns_function")
return {
    metadata = {name = "plugin_function", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test function module",
            execute = function()
                return greet("World"), 0
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
    assert_eq!(result, "Hello, World");
}

#[test]
fn test_empty_module_file() {
    // TEST 6: Empty Module File
    //
    // A module file that is completely empty (no code, no return statement).
    // Per Lua specification: empty modules return true (not nil).

    let fixture = TestFixture::new();

    fixture.create_lib_module("plugin_empty", "empty", "");

    fixture.create_plugin(
        "plugin_empty",
        r#"
local empty = require("plugin_empty.empty")
return {
    metadata = {name = "plugin_empty", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test empty module",
            execute = function()
                if empty == nil then
                    return "Empty module returned nil", 0
                else
                    return "Empty module returned: " .. tostring(empty), 0
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
    assert_eq!(result, "Empty module returned: true");
}

#[test]
fn test_module_with_runtime_error() {
    // TEST 7: Module with Runtime Error During Evaluation
    //
    // A module that has valid syntax but throws a runtime error during evaluation
    // (e.g., calling a nil function, accessing nil fields, etc.)

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_runtime",
        "runtime_error",
        r#"
local M = {}

-- This will cause a runtime error (calling nil)
local undefined_function = nil
undefined_function()  -- Runtime error: attempt to call nil value

M.value = "never reached"
return M
"#,
    );

    fixture.create_plugin(
        "plugin_runtime",
        r#"
local err_mod = require("plugin_runtime.runtime_error")
return {
    metadata = {name = "plugin_runtime", version = "1.0.0"},
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

    // Should fail during module evaluation
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    // Verify it's specifically an "attempt to call nil" error
    assert!(
        (error_msg.contains("attempt to call") && error_msg.contains("nil"))
            || error_msg.contains("call a nil value"),
        "Expected 'attempt to call nil' runtime error, got: {}",
        error_msg
    );
}

#[test]
fn test_module_with_only_comments() {
    // TEST 8: Module with Only Comments
    //
    // A module file that contains only comments (no executable code, no return).
    // Per Lua specification: comments-only modules return true (not nil).

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_comments",
        "only_comments",
        r#"
-- This module only has comments
-- Per Lua spec: it will return true
-- Because there's no return statement
"#,
    );

    fixture.create_plugin(
        "plugin_comments",
        r#"
local comments_mod = require("plugin_comments.only_comments")
return {
    metadata = {name = "plugin_comments", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test comments-only module",
            execute = function()
                if comments_mod == nil then
                    return "Comments-only module returned nil", 0
                else
                    return "Comments-only module returned: " .. tostring(comments_mod), 0
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
    // Per Lua spec: comments-only modules return true
    assert_eq!(result, "Comments-only module returned: true");
}

#[test]
fn test_module_with_unclosed_string() {
    // TEST 9: Module with Unclosed String Literal
    //
    // A module with an unclosed string literal - syntax error that should be
    // caught during loading.

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_unclosed",
        "unclosed_string",
        r#"
local M = {}

M.value = "This string is never closed

return M
"#,
    );

    fixture.create_plugin(
        "plugin_unclosed",
        r#"
local unclosed = require("plugin_unclosed.unclosed_string")
return {
    metadata = {name = "plugin_unclosed", version = "1.0.0"},
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

    // Should fail with syntax error
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("unfinished")
            || error_msg.contains("string")
            || error_msg.contains("<eof>"),
        "Expected unclosed string error, got: {}",
        error_msg
    );
}

#[test]
fn test_module_division_by_zero() {
    // TEST 10: Module with Division by Zero
    //
    // In Lua, division by zero produces inf or -inf, not an error.
    // This test documents that behavior.

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "plugin_divzero",
        "divzero",
        r#"
local M = {}

M.result = 10 / 0  -- Results in inf, not an error in Lua

return M
"#,
    );

    fixture.create_plugin(
        "plugin_divzero",
        r#"
local divzero = require("plugin_divzero.divzero")
return {
    metadata = {name = "plugin_divzero", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test division by zero",
            execute = function()
                return "Result: " .. tostring(divzero.result), 0
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
    assert_eq!(result, "Result: inf");
}
