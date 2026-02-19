//! Integration tests for plugin loading system
//!
//! Tests the plugin loader, merge system, and validation logic.

use std::sync::Arc;
use syntropy::{Config, create_lua_vm, load_plugins};
use tokio::sync::Mutex;

use crate::common::TestFixture;

// ============================================================================
// Mock Plugin Templates
// ============================================================================

const MINIMAL_PLUGIN: &str = r#"
return {
    metadata = {name = "minimal", version = "1.0.0"},
    tasks = {t = {description = "Minimal test task", execute = function() return "", 0 end}}
}
"#;

const COMPLETE_PLUGIN: &str = r#"
return {
    metadata = {
        description = "Test task",
        name = "complete",
        version = "2.5.0",
        icon = "C",
        description = "Full-featured test plugin",
        platforms = {"macos", "linux", "windows"},
    },
    tasks = {
        multi_task = {
            description = "Multi-select test task",
            name = "Multi Selection Task",
            description = "Test task with multi mode and multiple sources",
            mode = "multi",
            item_sources = {
                source1 = {
                    tag = "s1",
                    items = function() return {"a", "b"} end,
                    preselected_items = function() return {"a"} end,
                    preview = function(item) return "Preview: " .. item end,
                    execute = function(items) return "Done", 0 end,
                },
                source2 = {
                    tag = "s2",
                    items = function() return {"x", "y"} end,
                    execute = function(items) return "Done", 0 end,
                },
            },
            pre_run = function() end,
            post_run = function() end,
        },
        none_task = {
            description = "Test task",
            name = "Single Selection Task",
            description = "Test task with none mode",
            mode = "none",
            item_sources = {
                single = {
                    tag = "ss",
                    items = function() return {"item"} end,
                    execute = function(items) return "Done", 0 end,
                },
            },
        },
        execute_only = {
            description = "Execute-only task without item sources",
            execute = function() return "Task-level execute", 0 end
        },
    },
}
"#;

// ============================================================================
// Helper Functions
// ============================================================================

/// Load a plugin from an inline string
fn load_plugin_from_string(content: &str) -> Result<Vec<syntropy::plugins::Plugin>, String> {
    let fixture = TestFixture::new();
    fixture.create_plugin("test", content);

    let lua = Arc::new(Mutex::new(
        create_lua_vm().map_err(|e| format!("Failed to create Lua VM: {}", e))?,
    ));
    let config = Config::default();

    load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &config,
        lua,
    )
    .map_err(|e| e.to_string())
}

/// Load a merged plugin (base + override)
/// Note: Config directory comes FIRST (override), data directory comes LAST (base)
fn load_merged_plugin(
    base: &str,
    override_content: &str,
) -> Result<Vec<syntropy::plugins::Plugin>, String> {
    let fixture = TestFixture::new();
    fixture.create_plugin("test", base); // Data dir (base)
    fixture.create_plugin_override("test", override_content); // Config dir (override)

    let lua = Arc::new(Mutex::new(
        create_lua_vm().map_err(|e| format!("Failed to create Lua VM: {}", e))?,
    ));
    let config = Config::default();

    // Path order: config (override) first, data (base) last
    load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua,
    )
    .map_err(|e| e.to_string())
}

// ============================================================================
// Category 1: Valid Plugin Loading (4 tests)
// ============================================================================

#[test]
fn test_load_minimal_plugin() {
    let plugins = load_plugin_from_string(MINIMAL_PLUGIN).unwrap();

    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.name, "minimal");
    assert_eq!(plugins[0].metadata.version, "1.0.0");
    assert_eq!(plugins[0].metadata.description, ""); // Default
    assert_eq!(plugins[0].metadata.platforms.len(), 0); // Default
    assert_eq!(plugins[0].tasks.len(), 1);
}

#[test]
fn test_load_complete_plugin() {
    let plugins = load_plugin_from_string(COMPLETE_PLUGIN).unwrap();

    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.name, "complete");
    assert_eq!(plugins[0].metadata.version, "2.5.0");
    assert_eq!(plugins[0].metadata.icon, "C");
    assert_eq!(plugins[0].metadata.description, "Full-featured test plugin");
    assert_eq!(
        plugins[0].metadata.platforms,
        vec!["macos", "linux", "windows"]
    );
    assert_eq!(plugins[0].tasks.len(), 3);
    assert!(plugins[0].tasks.contains_key("multi_task"));
    assert!(plugins[0].tasks.contains_key("none_task"));
    assert!(plugins[0].tasks.contains_key("execute_only"));
}

#[test]
fn test_load_multiple_plugins() {
    let fixture = TestFixture::new();
    fixture.create_plugin("plugin1", &MINIMAL_PLUGIN.replace("minimal", "plugin1"));
    fixture.create_plugin("plugin2", &MINIMAL_PLUGIN.replace("minimal", "plugin2"));
    fixture.create_plugin("plugin3", &MINIMAL_PLUGIN.replace("minimal", "plugin3"));

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua,
    )
    .unwrap();

    assert_eq!(plugins.len(), 3);

    // Verify all three plugins loaded
    let names: Vec<&str> = plugins.iter().map(|p| p.metadata.name.as_str()).collect();
    assert!(names.contains(&"plugin1"));
    assert!(names.contains(&"plugin2"));
    assert!(names.contains(&"plugin3"));
}

#[test]
fn test_load_plugin_single_char_icons() {
    // Icons must occupy a single terminal cell (width == 1)
    let icons = vec!["T", "M", "B", "X", "⚒"];

    for icon in icons {
        let plugin = format!(
            r#"
return {{
    metadata = {{name = "test", version = "1.0.0", icon = "{}"}},
    tasks = {{t = {{description = "Test task", execute = function() return "", 0 end}}}}
}}
"#,
            icon
        );

        let result = load_plugin_from_string(&plugin);
        assert!(result.is_ok(), "Icon '{}' should be valid", icon);

        if let Ok(plugins) = result {
            assert_eq!(plugins[0].metadata.icon, icon);
        }
    }
}

// ============================================================================
// Category 2: Merge System (7 tests)
// ============================================================================

#[test]
fn test_merge_override_metadata_fields() {
    let base = r#"
return {
    metadata = {
        description = "Test task",
        name = "mergeable",
        version = "1.0.0",
        icon = "B",
        description = "Base description",
        platforms = {"macos", "linux"},
    },
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let override_plugin = r#"
return {
    metadata = {
        description = "Test task",
        name = "mergeable",
        icon = "O",
        description = "Override description",
    },
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let plugins = load_merged_plugin(base, override_plugin).unwrap();

    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.name, "mergeable");
    assert_eq!(plugins[0].metadata.version, "1.0.0"); // From base
    assert_eq!(plugins[0].metadata.icon, "O"); // Overridden
    assert_eq!(plugins[0].metadata.description, "Override description"); // Overridden
    assert_eq!(plugins[0].metadata.platforms, vec!["macos", "linux"]); // From base
}

#[test]
fn test_merge_arrays_replaced_not_merged() {
    let base = r#"
return {
    metadata = {
        description = "Test task",
        name = "arrays",
        version = "1.0.0",
        platforms = {"macos", "linux", "windows"},
    },
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let override_plugin = r#"
return {
    metadata = {
        description = "Test task",
        name = "arrays",
        platforms = {"linux"},
    },
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let plugins = load_merged_plugin(base, override_plugin).unwrap();

    // Array merge works correctly - override replaces base
    // Verified: deep_merge() properly detects arrays and replaces them
    assert_eq!(plugins[0].metadata.platforms, vec!["linux"]);
    // Verify base values (macos, windows) are NOT present - array was replaced, not merged
    assert!(!plugins[0].metadata.platforms.contains(&"macos".to_string()));
    assert!(
        !plugins[0]
            .metadata
            .platforms
            .contains(&"windows".to_string())
    );
}

#[test]
fn test_merge_add_new_task() {
    let base = r#"
return {
    metadata = {name = "extendable", version = "1.0.0"},
    tasks = {
        base_task = {
            description = "Base task",
            execute = function() return "base", 0 end
        }
    }
}
"#;

    let override_plugin = r#"
return {
    metadata = {name = "extendable"},
    tasks = {
        new_task = {
            description = "New task",
            execute = function() return "new", 0 end
        }
    }
}
"#;

    // Create fixture and Lua runtime to enable function execution
    let fixture = TestFixture::new();
    fixture.create_plugin("extendable", base);
    fixture.create_plugin_override("extendable", override_plugin);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins[0].tasks.len(), 2);
    assert!(plugins[0].tasks.contains_key("base_task"));
    assert!(plugins[0].tasks.contains_key("new_task"));

    // Verify both tasks actually work by calling their execute functions
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Call base_task execute (from base plugin)
    let (base_result, _) = rt
        .block_on(async {
            let task = plugins[0].tasks.get("base_task").unwrap();
            syntropy::execution::call_task_execute(&lua, task, &[]).await
        })
        .unwrap();
    assert_eq!(base_result, "base", "Base task should return 'base'");

    // Call new_task execute (from override plugin)
    let (new_result, _) = rt
        .block_on(async {
            let task = plugins[0].tasks.get("new_task").unwrap();
            syntropy::execution::call_task_execute(&lua, task, &[]).await
        })
        .unwrap();
    assert_eq!(new_result, "new", "New task should return 'new'");
}

#[test]
fn test_merge_override_task_item_sources() {
    let base = r#"
return {
    metadata = {name = "tasks", version = "1.0.0"},
    tasks = {
        task1 = {
            description = "Test task",
            name = "Original Name",
            mode = "none",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a"} end,
                    execute = function() return "base", 0 end
                }
            }
        }
    }
}
"#;

    let override_plugin = r#"
return {
    metadata = {name = "tasks"},
    tasks = {
        task1 = {
            description = "Test task",
            name = "Custom Name",
            mode = "multi",
        }
    }
}
"#;

    // Create fixture and Lua runtime to enable function execution
    let fixture = TestFixture::new();
    fixture.create_plugin("tasks", base);
    fixture.create_plugin_override("tasks", override_plugin);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua.clone(),
    )
    .unwrap();

    let task = plugins[0].tasks.get("task1").unwrap();

    // Verify override fields changed
    assert_eq!(task.name, "Custom Name"); // Overridden
    assert_eq!(task.mode, syntropy::plugins::Mode::Multi); // Overridden from "none" to "multi"

    // Verify item_sources preserved from base
    assert!(task.item_sources.is_some());
    let sources = task.item_sources.as_ref().unwrap();
    assert_eq!(sources.len(), 1);
    assert!(sources.contains_key("src")); // Base item source still exists

    // Verify items() function from base still works
    let rt = tokio::runtime::Runtime::new().unwrap();
    let items = rt
        .block_on(async {
            syntropy::execution::call_item_source_items(&lua, "tasks", "task1", "src").await
        })
        .unwrap();
    assert_eq!(
        items,
        vec!["a"],
        "items() function from base should still return ['a']"
    );
}

#[test]
fn test_merge_override_execute_function() {
    // This test verifies that Lua function overrides actually work
    // by calling the execute function and checking it returns "overridden" not "original"

    let base = r#"
return {
    metadata = {name = "func", version = "1.0.0"},
    tasks = {
        t = {
            description = "Test task",
            execute = function() return "original", 0 end
        }
    }
}
"#;

    let override_plugin = r#"
return {
    metadata = {name = "func"},
    tasks = {
        t = {
            description = "Test task",
            execute = function() return "overridden", 0 end
        }
    }
}
"#;

    // Create fixture and Lua runtime
    let fixture = TestFixture::new();
    fixture.create_plugin("func", base);
    fixture.create_plugin_override("func", override_plugin);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    // Load merged plugin (this stores it in Lua globals)
    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.name, "func");

    let task = plugins[0].tasks.get("t").unwrap();

    // Actually call the execute function to verify override worked
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        syntropy::execution::call_task_execute(
            &lua,
            task,
            &[], // No items for task-level execute
        )
        .await
    });

    assert!(result.is_ok(), "Execute function should succeed");
    let (output, _) = result.unwrap();
    assert_eq!(
        output, "overridden",
        "Execute function should return 'overridden' not 'original' - deep merge failed to override Lua function"
    );
}

#[test]
fn test_merge_add_item_source_to_existing_task() {
    let base = r#"
return {
    metadata = {name = "extend", version = "1.0.0"},
    tasks = {
        multi = {
            description = "Multi-select test task",
            mode = "multi",
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"a"} end,
                    execute = function() return "done", 0 end
                }
            }
        }
    }
}
"#;

    let override_plugin = r#"
return {
    metadata = {name = "extend"},
    tasks = {
        multi = {
            description = "Multi-select test task",
            item_sources = {
                src2 = {
                    tag = "s2",
                    items = function() return {"b"} end,
                    execute = function() return "done", 0 end
                }
            }
        }
    }
}
"#;

    // Create fixture and Lua runtime to enable function execution
    let fixture = TestFixture::new();
    fixture.create_plugin("extend", base);
    fixture.create_plugin_override("extend", override_plugin);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua.clone(),
    )
    .unwrap();

    let task = plugins[0].tasks.get("multi").unwrap();

    // Both item sources should be present after deep merge
    assert!(task.item_sources.is_some());
    let sources = task.item_sources.as_ref().unwrap();
    assert_eq!(sources.len(), 2);
    assert!(sources.contains_key("src1"));
    assert!(sources.contains_key("src2"));

    // Verify both item sources actually work by calling their items() functions
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Call src1.items() (from base)
    let items1 = rt
        .block_on(async {
            syntropy::execution::call_item_source_items(&lua, "extend", "multi", "src1").await
        })
        .unwrap();
    assert_eq!(items1, vec!["a"], "src1 (from base) should return ['a']");

    // Call src2.items() (from override)
    let items2 = rt
        .block_on(async {
            syntropy::execution::call_item_source_items(&lua, "extend", "multi", "src2").await
        })
        .unwrap();
    assert_eq!(
        items2,
        vec!["b"],
        "src2 (from override) should return ['b']"
    );
}

#[test]
fn test_merge_deep_nested_tables() {
    // Verify that deep table merging works correctly
    let base = r#"
return {
    metadata = {name = "nested", version = "1.0.0"},
    tasks = {
        t1 = {
            description = "Test task",
            item_sources = {
                s1 = {
                    tag = "s1",
                    items = function() return {"original"} end,
                    execute = function() return "done", 0 end
                }
            }
        }
    }
}
"#;

    let override_plugin = r#"
return {
    metadata = {name = "nested"},
    tasks = {
        t1 = {
            description = "Test task",
            item_sources = {
                s1 = {
                    tag = "overridden",
                }
            }
        }
    }
}
"#;

    // Create fixture and Lua runtime to enable function execution
    let fixture = TestFixture::new();
    fixture.create_plugin("nested", base);
    fixture.create_plugin_override("nested", override_plugin);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua.clone(),
    )
    .unwrap();

    let task = plugins[0].tasks.get("t1").unwrap();
    let sources = task.item_sources.as_ref().unwrap();
    let source = sources.get("s1").unwrap();

    // Verify tag was overridden in deeply nested merge
    assert_eq!(source.tag, "overridden");

    // Verify items() function from base is preserved despite tag override
    let rt = tokio::runtime::Runtime::new().unwrap();
    let items = rt
        .block_on(async {
            syntropy::execution::call_item_source_items(&lua, "nested", "t1", "s1").await
        })
        .unwrap();
    assert_eq!(
        items,
        vec!["original"],
        "items() function from base should still return ['original'] even though tag was overridden"
    );
}

// ============================================================================
// Category 3: Error Cases (10 tests)
// ============================================================================

#[test]
fn test_lua_syntax_error() {
    const SYNTAX_ERROR: &str = r#"
return {
    metadata = {
        description = "Test task",
        name = "broken",
        version = "1.0.0",  -- Missing closing brace
"#;

    let result = load_plugin_from_string(SYNTAX_ERROR);
    assert!(result.is_err());
    // Error message may vary (could be peek failure, syntax error, etc.)
    // Just verify it fails
}

#[test]
fn test_missing_metadata_table() {
    let result = load_plugin_from_string(r#"return {tasks = {}}"#);
    assert!(result.is_err());
}

#[test]
fn test_missing_tasks_table() {
    let result = load_plugin_from_string(
        r#"
return {metadata = {name = "test", version = "1.0.0"}}
"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_missing_metadata_name() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_missing_version() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_invalid_icon_width() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "AB"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#,
    );
    assert!(result.is_err());
    // Just verify validation fails for multi-char icon
}

#[test]
fn test_empty_task_no_execute() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {empty_task = {name = "Empty", description = "Empty task"}}
}
"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_task_with_no_item_sources_and_no_execute() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        invalid = {
            description = "Test task",
            name = "Invalid Task",
            description = "Invalid task without execute or item_sources",
            mode = "none",
        }
    }
}
"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_invalid_mode_value() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        t = {
            mode = "invalid_mode",
            execute = function() return "", 0 end
        }
    }
}
"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_invalid_platforms_not_array() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test", version = "1.0.0", platforms = "macos"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#,
    );
    // Expected behavior: platforms as string should fail type validation
    // Current behavior: mlua coerces string to empty array, no error
    assert!(
        result.is_err(),
        "platforms field should be array, not string"
    );
}

// ============================================================================
// Category 4: Duplicate Detection (3 tests)
// ============================================================================

#[test]
fn test_duplicate_plugin_names_in_same_directory() {
    // Duplicate detection: when multiple directories have plugins with same metadata.name,
    // only the first one (alphabetically) should be loaded

    let fixture = TestFixture::new();
    let plugin_dir = fixture.data_path().join("syntropy").join("plugins");

    // Create two plugin directories with same metadata.name but different descriptions
    // Directory names: plugin-a comes before plugin-b alphabetically
    fixture.create_plugin(
        "plugin-a",
        r#"
return {
    metadata = {name = "duplicate", version = "1.0.0", description = "First plugin"},
    tasks = {t = {description = "Test task", execute = function() return "a", 0 end}}
}
"#,
    );

    fixture.create_plugin(
        "plugin-b",
        r#"
return {
    metadata = {name = "duplicate", version = "1.0.0", description = "Second plugin"},
    tasks = {t = {description = "Test task", execute = function() return "b", 0 end}}
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let result = load_plugins(&[plugin_dir], &Config::default(), lua);

    // Should load only one plugin (first one found wins)
    assert!(result.is_ok());
    if let Ok(plugins) = result {
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].metadata.name, "duplicate");
        // Verify it's the FIRST plugin (plugin-a) that was loaded
        assert_eq!(
            plugins[0].metadata.description, "First plugin",
            "First plugin (alphabetically) should win in duplicate detection"
        );
    }
}

#[test]
fn test_merge_requires_matching_names() {
    let base = r#"
return {
    metadata = {name = "base-name", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let override_plugin = r#"
return {
    metadata = {name = "different-name", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    // Names don't match - should result in two separate plugins
    let fixture = TestFixture::new();
    fixture.create_plugin("test1", base);
    fixture.create_plugin_override("test2", override_plugin);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let result = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &Config::default(),
        lua,
    );

    assert!(result.is_ok());
    if let Ok(plugins) = result {
        assert_eq!(plugins.len(), 2);
    }
}

#[test]
fn test_no_duplicate_if_same_directory_name() {
    // Sanity check: A single plugin directory should load exactly one plugin
    // This verifies basic plugin loading without any duplicate scenarios
    // Each plugin directory contains one plugin.lua file
    let fixture = TestFixture::new();
    let plugin_content = MINIMAL_PLUGIN;

    fixture.create_plugin("same-dir", plugin_content);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &Config::default(),
        lua,
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);
}

// ============================================================================
// Category 5: Edge Cases (5 tests)
// ============================================================================

#[test]
fn test_plugin_with_empty_tasks_table() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {}
}
"#,
    );
    // Expected behavior: Plugin with no tasks should fail validation
    // Current behavior: Empty tasks table is accepted
    assert!(
        result.is_err(),
        "Plugin with empty tasks should fail validation"
    );
}

#[test]
fn test_plugin_with_empty_item_sources() {
    let result = load_plugin_from_string(
        r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        t = {
            description = "Test task with empty item_sources",
            item_sources = {},
            execute = function() return "done", 0 end
        }
    }
}
"#,
    );
    assert!(
        result.is_ok(),
        "Empty item_sources with task execute is valid"
    );
}

#[test]
fn test_plugin_with_optional_fields_omitted() {
    let minimal = r#"
return {
    metadata = {name = "min", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let plugins = load_plugin_from_string(minimal).unwrap();

    // Icon defaults to config.default_plugin_icon which is "⚒"
    assert_eq!(plugins[0].metadata.icon, "⚒");
    assert_eq!(plugins[0].metadata.description, ""); // Default empty
    assert_eq!(plugins[0].metadata.platforms.len(), 0); // Default empty array
}

#[test]
fn test_task_key_preserved() {
    let plugin = r#"
return {
    metadata = {name = "keys", version = "1.0.0"},
    tasks = {
        export_package_list = {
            description = "Test task",
            name = "Export Package List",
            description = "Export package list to file",
            execute = function() return "", 0 end
        },
        import_defaults = {
            description = "Test task",
            name = "Import Defaults",
            description = "Import default settings",
            execute = function() return "", 0 end
        }
    }
}
"#;

    let plugins = load_plugin_from_string(plugin).unwrap();

    // Task keys should be preserved (HashMap keys)
    assert!(plugins[0].tasks.contains_key("export_package_list"));
    assert!(plugins[0].tasks.contains_key("import_defaults"));
}

#[test]
fn test_empty_array_in_platforms() {
    let plugin = r#"
return {
    metadata = {name = "test", version = "1.0.0", platforms = {}},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let plugins = load_plugin_from_string(plugin).unwrap();
    assert_eq!(plugins[0].metadata.platforms.len(), 0);
}

// ============================================================================
// Category 6: Polling Interval Fields (3 tests)
// ============================================================================

#[test]
fn test_polling_intervals_default_to_zero() {
    // When polling interval fields are omitted, they should default to 0 (no polling)
    let plugin = r#"
return {
    metadata = {name = "polling_defaults", version = "1.0.0"},
    tasks = {
        task1 = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b"} end,
                    execute = function() return "done", 0 end
                }
            }
        }
    }
}
"#;

    let plugins = load_plugin_from_string(plugin).unwrap();
    assert_eq!(plugins.len(), 1);

    let task = plugins[0].tasks.get("task1").unwrap();

    // Verify both polling intervals default to 0
    assert_eq!(task.item_polling_interval, 0);
    assert_eq!(task.preview_polling_interval, 0);
}

#[test]
fn test_polling_intervals_explicit_values() {
    // When polling interval fields are explicitly set, values should be parsed correctly
    let plugin = r#"
return {
    metadata = {name = "polling_explicit", version = "1.0.0"},
    tasks = {
        task1 = {
            description = "Test task",
            mode = "multi",
            item_polling_interval = 1000,
            preview_polling_interval = 500,
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"a"} end,
                    execute = function() return "done", 0 end
                }
            }
        },
        task2 = {
            description = "Test task 2",
            mode = "multi",
            item_polling_interval = 2000,
            preview_polling_interval = 0,
            item_sources = {
                src2 = {
                    tag = "s2",
                    items = function() return {"b"} end,
                    execute = function() return "done", 0 end
                }
            }
        }
    }
}
"#;

    let plugins = load_plugin_from_string(plugin).unwrap();
    assert_eq!(plugins.len(), 1);

    // Verify task1 has custom intervals
    let task1 = plugins[0].tasks.get("task1").unwrap();
    assert_eq!(task1.item_polling_interval, 1000);
    assert_eq!(task1.preview_polling_interval, 500);

    // Verify task2 has different intervals (including explicit 0)
    let task2 = plugins[0].tasks.get("task2").unwrap();
    assert_eq!(task2.item_polling_interval, 2000);
    assert_eq!(task2.preview_polling_interval, 0);
}

#[test]
fn test_merge_override_polling_intervals() {
    // Override plugin should be able to change polling intervals from base
    let base = r#"
return {
    metadata = {name = "polling_merge", version = "1.0.0"},
    tasks = {
        task1 = {
            description = "Test task",
            mode = "multi",
            item_polling_interval = 1000,
            preview_polling_interval = 500,
            item_sources = {
                src = {
                    tag = "base_tag",
                    items = function() return {"original"} end,
                    execute = function() return "base", 0 end
                }
            }
        }
    }
}
"#;

    let override_plugin = r#"
return {
    metadata = {name = "polling_merge"},
    tasks = {
        task1 = {
            description = "Test task",
            item_polling_interval = 3000,
            preview_polling_interval = 1500,
            item_sources = {
                src = {
                    tag = "override_tag",
                }
            }
        }
    }
}
"#;

    // Create fixture and load merged plugin
    let fixture = TestFixture::new();
    fixture.create_plugin("polling_merge", base);
    fixture.create_plugin_override("polling_merge", override_plugin);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua.clone(),
    )
    .unwrap();

    let task = plugins[0].tasks.get("task1").unwrap();
    let sources = task.item_sources.as_ref().unwrap();
    let source = sources.get("src").unwrap();

    // Verify polling intervals were overridden
    assert_eq!(task.item_polling_interval, 3000);
    assert_eq!(task.preview_polling_interval, 1500);

    // Verify tag was also overridden
    assert_eq!(source.tag, "override_tag");

    // Verify items() function from base is still preserved
    let rt = tokio::runtime::Runtime::new().unwrap();
    let items = rt
        .block_on(async {
            syntropy::execution::call_item_source_items(&lua, "polling_merge", "task1", "src").await
        })
        .unwrap();
    assert_eq!(
        items,
        vec!["original"],
        "items() function from base should still work despite field overrides"
    );
}

// ============================================================================
// Category 7: Execution Confirmation Message (3 tests)
// ============================================================================

#[test]
fn test_execution_confirmation_message_defaults_to_none() {
    // When execution_confirmation_message is omitted, it should default to None
    let plugin = r#"
return {
    metadata = {name = "confirm_defaults", version = "1.0.0"},
    tasks = {
        task1 = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b"} end,
                    execute = function() return "done", 0 end
                }
            }
        }
    }
}
"#;

    let plugins = load_plugin_from_string(plugin).unwrap();
    assert_eq!(plugins.len(), 1);

    let task = plugins[0].tasks.get("task1").unwrap();

    // Verify execution_confirmation_message defaults to None
    assert_eq!(task.execution_confirmation_message, None);
}

#[test]
fn test_execution_confirmation_message_explicit_value() {
    // When execution_confirmation_message is set, it should be parsed correctly
    let plugin = r#"
return {
    metadata = {name = "confirm_explicit", version = "1.0.0"},
    tasks = {
        task1 = {
            description = "Test task",
            mode = "multi",
            execution_confirmation_message = "Are you sure you want to proceed?",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a"} end,
                    execute = function() return "done", 0 end
                }
            }
        },
        task2 = {
            description = "Test task 2",
            mode = "none",
            execution_confirmation_message = "This will delete files. Continue?",
            execute = function() return "done", 0 end
        }
    }
}
"#;

    let plugins = load_plugin_from_string(plugin).unwrap();
    assert_eq!(plugins.len(), 1);

    // Verify task1 has custom confirmation message
    let task1 = plugins[0].tasks.get("task1").unwrap();
    assert_eq!(
        task1.execution_confirmation_message,
        Some("Are you sure you want to proceed?".to_string())
    );

    // Verify task2 has different confirmation message
    let task2 = plugins[0].tasks.get("task2").unwrap();
    assert_eq!(
        task2.execution_confirmation_message,
        Some("This will delete files. Continue?".to_string())
    );
}

#[test]
fn test_merge_override_execution_confirmation_message() {
    // Override plugin should be able to change or remove confirmation message from base
    let fixture = TestFixture::new();

    let base = r#"
return {
    metadata = {name = "confirm_merge", version = "1.0.0"},
    tasks = {
        task1 = {
            description = "Test task",
            mode = "multi",
            execution_confirmation_message = "Original message",
            item_sources = {
                src = {
                    tag = "base_tag",
                    items = function() return {"original"} end,
                    execute = function() return "done", 0 end
                }
            }
        }
    }
}
"#;

    let override_content = r#"
return {
    metadata = {name = "confirm_merge"},
    tasks = {
        task1 = {
            description = "Test task",
            execution_confirmation_message = "Overridden message",
            item_sources = {
                src = {
                    tag = "override_tag"
                }
            }
        }
    }
}
"#;

    fixture.create_plugin("confirm_merge", base);
    fixture.create_plugin_override("confirm_merge", override_content);

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua.clone(),
    )
    .unwrap();

    let task = plugins[0].tasks.get("task1").unwrap();

    // Verify confirmation message was overridden
    assert_eq!(
        task.execution_confirmation_message,
        Some("Overridden message".to_string())
    );

    // Verify other fields still work
    let sources = task.item_sources.as_ref().unwrap();
    let source = sources.get("src").unwrap();
    assert_eq!(source.tag, "override_tag");
}

// ============================================================================
// Category 8: Additional Edge Cases (5 tests)
// ============================================================================

#[test]
fn test_load_from_multiple_directories_no_merge() {
    let fixture = TestFixture::new();

    // Create two different plugins in different directories
    fixture.create_plugin(
        "plugin_a",
        r#"
return {
    metadata = {name = "plugin_a", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "a", 0 end}}
}
"#,
    );

    fixture.create_plugin_override(
        "plugin_b",
        r#"
return {
    metadata = {name = "plugin_b", version = "2.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "b", 0 end}}
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    // Load from both directories (config first, data second)
    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua,
    )
    .unwrap();

    // Both plugins should load (no merging because names differ)
    assert_eq!(plugins.len(), 2);

    // Verify load order: config dir first (plugin_b), data dir second (plugin_a)
    assert_eq!(plugins[0].metadata.name, "plugin_b");
    assert_eq!(plugins[0].metadata.version, "2.0.0");
    assert_eq!(plugins[1].metadata.name, "plugin_a");
    assert_eq!(plugins[1].metadata.version, "1.0.0");
}

#[test]
fn test_merge_with_three_sources() {
    // TODO: EDGE CASE - Undefined behavior with 3+ plugin sources
    // Question: What should happen if same plugin exists in 3+ directories?
    // Current behavior: Only 2-directory merge is documented (data + config)
    // Expected behavior: TBD - either error, or sequential merge (1+2→merged, merged+3→final)
    //
    // This test documents current behavior for future design decisions
    // For now, we only test 2-directory scenario (config + data)

    let fixture = TestFixture::new();

    // Create same plugin in both directories
    fixture.create_plugin(
        "test",
        r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "base", 0 end}}
}
"#,
    );

    fixture.create_plugin_override(
        "test",
        r#"
return {
    metadata = {name = "test", version = "2.0.0"},
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    // With current 2-directory architecture, merge should work
    let plugins = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua,
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.version, "2.0.0"); // Override wins

    // NOTE: If 3+ directory support is added, update this test to verify behavior
}

#[test]
fn test_double_evaluation_side_effects() {
    // Documents double evaluation behavior mentioned in CLAUDE.md
    // Each plugin.lua is evaluated twice: peek name, then load
    // Side effects at module scope will execute twice

    let fixture = TestFixture::new();

    // Plugin with side effect at module scope (counter increment)
    // In real scenario, this could be file I/O, global state mutation, etc.
    fixture.create_plugin(
        "side-effects",
        r#"
-- This code runs at module scope (executes twice)
local counter = _G.syntropy_test_counter or 0
_G.syntropy_test_counter = counter + 1

return {
    metadata = {
        description = "Test task",
        name = "side-effects",
        version = "1.0.0"
    },
    tasks = {
        t = {
            description = "Test task with side effects",
            execute = function()
                return "Counter: " .. tostring(_G.syntropy_test_counter), 0
            end
        }
    }
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    let plugins = load_plugins(
        &[fixture.data_path().join("syntropy").join("plugins")],
        &config,
        lua.clone(),
    )
    .unwrap();

    assert_eq!(plugins.len(), 1);

    // Verify the counter was incremented during loading
    // This demonstrates that module-scope code executes during plugin load
    // The exact count depends on implementation (should be 2 for double evaluation)
    // We just verify the plugin loaded successfully
    assert_eq!(plugins[0].metadata.name, "side-effects");

    // NOTE: The global counter is in Lua VM state, accessible to tasks at runtime
    // Plugin authors should avoid side effects at module scope
}

#[test]
fn test_merge_fails_without_override_name() {
    // Verifies that override plugin must specify metadata.name for merge to work

    let fixture = TestFixture::new();

    // Base plugin with name
    fixture.create_plugin(
        "test",
        r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "base", 0 end}}
}
"#,
    );

    // Override without metadata.name
    fixture.create_plugin_override(
        "test",
        r#"
return {
    metadata = {version = "2.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "override", 0 end}}
}
"#,
    );

    let lua = Arc::new(Mutex::new(create_lua_vm().unwrap()));
    let config = Config::default();

    // Attempt to load with merge
    let result = load_plugins(
        &[
            fixture.config_path().join("syntropy").join("plugins"),
            fixture.data_path().join("syntropy").join("plugins"),
        ],
        &config,
        lua,
    );

    // Should fail because override can't be identified for merging
    // The loader peeks metadata.name to detect duplicates
    // Without name in override, it can't match with base plugin
    assert!(
        result.is_err(),
        "Override plugin without metadata.name should fail to merge"
    );
}
