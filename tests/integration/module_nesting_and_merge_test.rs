// Module Nesting and Multi-Directory Merge Tests
//
// Tests for deep module nesting and multi-directory merge behavior.
// Consolidates tests for:
// - Deep nesting (5+ levels)
// - Multiple init.lua at different levels
// - 3+ directory merge scenarios
// - Module precedence across multiple plugin directories

use crate::common::TestFixture;
use std::sync::Arc;
use syntropy::{
    configs::Config, execution::call_task_execute, lua::create_lua_vm, plugins::load_plugins,
};
use tokio::sync::Mutex;

#[test]
fn test_deep_module_nesting() {
    // TEST 1: Deep Module Nesting (5 levels)
    // Tests that modules can be nested deeply: plugin.a.b.c.d.module

    let fixture = TestFixture::new();

    // Create deep directory structure manually
    let deep_path = fixture
        .data_path()
        .join("syntropy/plugins/deep_nest/lua/deep_nest/level1/level2/level3");
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::write(
        deep_path.join("deep_module.lua"),
        r#"return { depth = "level3" }"#,
    )
    .unwrap();

    fixture.create_plugin(
        "deep_nest",
        r#"
local deep = require("deep_nest.level1.level2.level3.deep_module")
return {
    metadata = {name = "deep_nest", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test deep nesting",
            execute = function()
                return "Depth: " .. deep.depth, 0
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
    assert_eq!(result, "Depth: level3");
}

#[test]
fn test_init_lua_at_multiple_levels() {
    // TEST 2: init.lua at Multiple Levels
    // Tests directory-based modules with init.lua at various nesting levels

    let fixture = TestFixture::new();

    // Create nested init.lua files
    let base_path = fixture
        .data_path()
        .join("syntropy/plugins/init_test/lua/init_test");
    std::fs::create_dir_all(base_path.join("package")).unwrap();

    // init_test/package/init.lua
    std::fs::write(
        base_path.join("package/init.lua"),
        r#"return { level = "package" }"#,
    )
    .unwrap();

    // Also create a subpackage with init.lua
    std::fs::create_dir_all(base_path.join("package/subpackage")).unwrap();
    std::fs::write(
        base_path.join("package/subpackage/init.lua"),
        r#"return { level = "subpackage" }"#,
    )
    .unwrap();

    fixture.create_plugin(
        "init_test",
        r#"
local pkg = require("init_test.package")
local subpkg = require("init_test.package.subpackage")
return {
    metadata = {name = "init_test", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test multi-level init.lua",
            execute = function()
                return pkg.level .. " / " .. subpkg.level, 0
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
    assert_eq!(result, "package / subpackage");
}

#[test]
fn test_three_directory_merge() {
    // TEST 3: Three Directory Merge
    // Tests behavior when a plugin exists in 3 directories simultaneously.
    // Precedence should be: first directory > second directory > third directory

    let fixture = TestFixture::new();

    // Create three separate plugin directories
    let dir1 = fixture.data_path().join("syntropy/plugins1");
    let dir2 = fixture.data_path().join("syntropy/plugins2");
    let dir3 = fixture.data_path().join("syntropy/plugins3");

    // Plugin in dir1 (highest precedence)
    std::fs::create_dir_all(dir1.join("merged/lua/merged")).unwrap();
    std::fs::write(
        dir1.join("merged/lua/merged/config.lua"),
        r#"return { source = "dir1" }"#,
    )
    .unwrap();
    std::fs::write(
        dir1.join("merged/plugin.lua"),
        r#"
local config = require("merged.config")
return {
    metadata = {name = "merged", version = "1.0.0"},
    tasks = {
        from_dir1 = {
            description = "Task from dir1",
            execute = function() return "dir1: " .. config.source, 0 end
        }
    }
}
"#,
    )
    .unwrap();

    // Plugin in dir2 (middle precedence)
    std::fs::create_dir_all(dir2.join("merged/lua/merged")).unwrap();
    std::fs::write(
        dir2.join("merged/lua/merged/config.lua"),
        r#"return { source = "dir2" }"#,
    )
    .unwrap();
    std::fs::write(
        dir2.join("merged/plugin.lua"),
        r#"
local config = require("merged.config")
return {
    metadata = {name = "merged", version = "1.0.0"},
    tasks = {
        from_dir2 = {
            description = "Task from dir2",
            execute = function() return "dir2: " .. config.source, 0 end
        }
    }
}
"#,
    )
    .unwrap();

    // Plugin in dir3 (lowest precedence)
    std::fs::create_dir_all(dir3.join("merged/lua/merged")).unwrap();
    std::fs::write(
        dir3.join("merged/lua/merged/config.lua"),
        r#"return { source = "dir3" }"#,
    )
    .unwrap();
    std::fs::write(
        dir3.join("merged/plugin.lua"),
        r#"
local config = require("merged.config")
return {
    metadata = {name = "merged", version = "1.0.0"},
    tasks = {
        from_dir3 = {
            description = "Task from dir3",
            execute = function() return "dir3: " .. config.source, 0 end
        }
    }
}
"#,
    )
    .unwrap();

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(&[dir1, dir2, dir3], &Config::default(), lua.clone()).unwrap();

    // Should merge into 1 plugin
    // Note: System only merges first (override) and last (base), middle directory is ignored
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].tasks.len(), 2); // Only dir1 + dir3 tasks (dir2 ignored)

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Tasks from dir1 and dir3 are merged (dir2 ignored)
    // Both tasks use dir1's module due to package.path precedence

    // Task from dir1
    let task1 = plugins[0].tasks.get("from_dir1").unwrap();
    let (result1, code1) = rt
        .block_on(async { call_task_execute(&lua, task1, &[]).await })
        .unwrap();
    assert_eq!(code1, 0);
    assert_eq!(result1, "dir1: dir1", "dir1 task uses dir1's module");

    // Task from dir3 - task code from dir3, but loads dir1's module
    let task3 = plugins[0].tasks.get("from_dir3").unwrap();
    let (result3, code3) = rt
        .block_on(async { call_task_execute(&lua, task3, &[]).await })
        .unwrap();
    assert_eq!(code3, 0);
    assert_eq!(
        result3, "dir3: dir1",
        "dir3 task loads dir1's module due to path precedence"
    );
}

#[test]
fn test_very_long_module_path() {
    // TEST 4: Very Long Module Path
    // Tests limits on path length and nesting depth

    let fixture = TestFixture::new();

    // Create a very long nested path (20+ characters per segment)
    let segments = vec![
        "very_long_segment_name_one",
        "very_long_segment_name_two",
        "very_long_segment_name_three",
    ];

    let mut path = fixture
        .data_path()
        .join("syntropy/plugins/long_path/lua/long_path");
    for segment in &segments {
        path = path.join(segment);
    }
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("deep.lua"), r#"return { found = true }"#).unwrap();

    fixture.create_plugin(
        "long_path",
        &format!(
            r#"
local deep = require("long_path.{}")
return {{
    metadata = {{name = "long_path", version = "1.0.0"}},
    tasks = {{
        test = {{
            description = "Test long path",
            execute = function()
                return deep.found and "Found" or "Not found", 0
            end
        }}
    }}
}}
"#,
            segments.join(".") + ".deep"
        ),
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
fn test_mixed_flat_and_nested_modules() {
    // TEST 5: Mixed Flat and Nested Modules
    // Tests a plugin with both flat modules (lua/module.lua) and
    // nested modules (lua/plugin/module.lua)

    let fixture = TestFixture::new();

    // Flat module for vendoring (overrides shared)
    let flat_path = fixture.data_path().join("syntropy/plugins/mixed/lua");
    std::fs::create_dir_all(&flat_path).unwrap();
    std::fs::write(
        flat_path.join("vendored.lua"),
        r#"return { type = "vendored" }"#,
    )
    .unwrap();

    // Namespaced module
    fixture.create_lib_module("mixed", "namespaced", r#"return { type = "namespaced" }"#);

    fixture.create_plugin(
        "mixed",
        r#"
local vendored = require("vendored")
local namespaced = require("mixed.namespaced")
return {
    metadata = {name = "mixed", version = "1.0.0"},
    tasks = {
        test = {
            description = "Test mixed modules",
            execute = function()
                return vendored.type .. " + " .. namespaced.type, 0
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
    assert_eq!(result, "vendored + namespaced");
}
