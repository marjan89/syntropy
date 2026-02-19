// Case Sensitivity Tests for Module Loading
//
// These tests document and validate the case sensitivity behavior of Lua module
// loading on different platforms. Note that behavior varies by filesystem:
// - macOS (APFS/HFS+): Case-insensitive by default (but case-preserving)
// - Linux (ext4/etc): Case-sensitive
// - Windows (NTFS): Case-insensitive
//
// Lua's require() is case-sensitive in the module name, but file resolution
// depends on the underlying filesystem.

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_require_exact_case_match() {
    // TEST 1: Exact Case Match
    //
    // When module name and filename match exactly, require() should work on
    // all platforms regardless of filesystem case sensitivity.

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "case_plugin",
        "MyModule",
        r#"
return { name = "MyModule" }
"#,
    );

    fixture.create_plugin(
        "case_plugin",
        r#"
local my_module = require("case_plugin.MyModule")
return {
    metadata = {name = "case_plugin", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test exact case",
            execute = function()
                return my_module.name, 0
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
    assert_eq!(result, "MyModule");
}

#[test]
fn test_require_case_mismatch_behavior() {
    // TEST 2: Case Mismatch Behavior (Platform-dependent)
    //
    // File is named "Utils.lua" but require uses "utils" (lowercase).
    // Behavior depends on filesystem:
    // - Case-insensitive FS (macOS/Windows): Works
    // - Case-sensitive FS (Linux): Fails with module not found
    //
    // This test documents the behavior without enforcing it.

    let fixture = TestFixture::new();

    // Create module file with specific case: Utils.lua
    fixture.create_lib_module(
        "case_mismatch",
        "Utils",
        r#"
return { name = "Utils" }
"#,
    );

    fixture.create_plugin(
        "case_mismatch",
        r#"
return {
    metadata = {name = "case_mismatch", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test case mismatch",
            execute = function()
                -- Try to require with different case
                local success, result = pcall(require, "case_mismatch.utils")

                if success then
                    return "Found (case-insensitive FS): " .. result.name, 0
                else
                    -- On case-sensitive filesystems, this will fail
                    return "Not found (case-sensitive FS)", 0
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

    // On macOS/Windows (case-insensitive): should find the module
    // On Linux (case-sensitive): should not find the module
    // Validate each outcome properly instead of just accepting anything
    if result.contains("Found") {
        // If found, must include the actual module name to prove it loaded correctly
        assert!(
            result.contains("Utils"),
            "When module is found, result should contain module name 'Utils': {}",
            result
        );
    } else if result.contains("Not found") {
        // If not found, verify it's the expected platform-dependent message
        assert_eq!(result, "Not found (case-sensitive FS)");
    } else {
        panic!(
            "Result must be either successful module load or platform-dependent not found, got: {}",
            result
        );
    }
}

#[test]
fn test_multiple_case_variations_conflict() {
    // TEST 3: Multiple Files with Same Name, Different Cases
    //
    // On case-sensitive filesystems, you could theoretically have both
    // utils.lua and Utils.lua. On case-insensitive filesystems, this
    // is not possible (second file would overwrite first).
    //
    // This test documents that best practice is to always use consistent
    // casing to avoid platform-specific issues.

    let fixture = TestFixture::new();

    // Create module with lowercase name
    fixture.create_lib_module(
        "multi_case",
        "helper",
        r#"
return { version = "lowercase" }
"#,
    );

    // Note: We cannot create both "helper.lua" and "Helper.lua" via the
    // test fixture on case-insensitive filesystems, as the second would
    // overwrite the first. This test documents the limitation.

    fixture.create_plugin(
        "multi_case",
        r#"
local helper = require("multi_case.helper")
return {
    metadata = {name = "multi_case", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test case consistency",
            execute = function()
                return "Version: " .. helper.version, 0
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
    assert_eq!(result, "Version: lowercase");
}

#[test]
fn test_consistent_casing_best_practice() {
    // TEST 4: Consistent Casing Best Practice
    //
    // Demonstrates the recommended approach: use consistent, predictable
    // casing (typically snake_case for Lua modules) to avoid any
    // platform-specific issues.

    let fixture = TestFixture::new();

    // Use consistent snake_case naming
    fixture.create_lib_module(
        "best_practice",
        "string_utils",
        r#"
return {
    trim = function(s) return s:match("^%s*(.-)%s*$") end
}
"#,
    );

    fixture.create_lib_module(
        "best_practice",
        "math_utils",
        r#"
return {
    add = function(a, b) return a + b end
}
"#,
    );

    fixture.create_plugin(
        "best_practice",
        r#"
local string_utils = require("best_practice.string_utils")
local math_utils = require("best_practice.math_utils")

return {
    metadata = {name = "best_practice", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test consistent naming",
            execute = function()
                local trimmed = string_utils.trim("  hello  ")
                local sum = math_utils.add(2, 3)
                return trimmed .. " " .. tostring(sum), 0
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
    assert_eq!(result, "hello 5");
}

#[test]
fn test_namespace_case_sensitivity() {
    // TEST 5: Namespace Case Sensitivity
    //
    // Lua's require() is always case-sensitive for the module name/namespace,
    // even on case-insensitive filesystems. This means:
    // - require("Plugin.utils") != require("plugin.utils")
    // - Only one will match the actual file structure

    let fixture = TestFixture::new();

    fixture.create_lib_module(
        "namespace_case",
        "data",
        r#"
return { value = "correct" }
"#,
    );

    fixture.create_plugin(
        "namespace_case",
        r#"
return {
    metadata = {name = "namespace_case", version = "1.0.0"},
    tasks = {
        test_correct = {
            description = "Test correct namespace case",
            execute = function()
                -- Correct: matches plugin name exactly
                local data = require("namespace_case.data")
                return "Correct case: " .. data.value, 0
            end
        },
        test_wrong = {
            description = "Test wrong namespace case",
            execute = function()
                -- Wrong: namespace case doesn't match plugin name
                local success, result = pcall(require, "Namespace_Case.data")

                if success then
                    return "Found with wrong case (filesystem is case-insensitive)", 0
                else
                    return "Not found (correct - namespace is case-sensitive)", 0
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

    // Test correct case
    let task_correct = plugins[0].tasks.get("test_correct").unwrap();
    let (result_correct, code) = rt
        .block_on(async { call_task_execute(&lua, task_correct, &[]).await })
        .unwrap();

    assert_eq!(code, 0);
    assert_eq!(result_correct, "Correct case: correct");

    // Test wrong case - behavior may vary by platform
    let task_wrong = plugins[0].tasks.get("test_wrong").unwrap();
    let (result_wrong, code) = rt
        .block_on(async { call_task_execute(&lua, task_wrong, &[]).await })
        .unwrap();

    assert_eq!(code, 0);
    // Validate each outcome properly - behavior depends on filesystem
    if result_wrong.contains("Found") {
        // If found on case-insensitive FS, verify it's the expected message
        assert_eq!(
            result_wrong, "Found with wrong case (filesystem is case-insensitive)",
            "When found with wrong case, must return expected message"
        );
    } else if result_wrong.contains("Not found") {
        // If not found on case-sensitive FS, verify it's the expected message
        assert_eq!(
            result_wrong, "Not found (correct - namespace is case-sensitive)",
            "When not found, must return expected message"
        );
    } else {
        panic!(
            "Result must indicate either found or not found with specific message, got: {}",
            result_wrong
        );
    }
}
