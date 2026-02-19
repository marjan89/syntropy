//! Integration tests for plugin validation (validate --plugin)
//!
//! Tests the plugin loading and validation system using the CLI.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

// ============================================================================
// Mock Plugin Templates
// ============================================================================

const MINIMAL_VALID_PLUGIN: &str = r#"
return {
    metadata = {name = "minimal", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

const COMPLETE_VALID_PLUGIN: &str = r#"
return {
    metadata = {
        name = "complete",
        version = "2.5.0",
        icon = "C",
        description = "Complete test plugin",
        platforms = {"macos", "linux"},
    },
    tasks = {
        multi_task = {
            description = "Multi-select test task",
            name = "Multi Selection",
            description = "Test task with multi mode",
            mode = "multi",
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"a", "b"} end,
                    preselected_items = function() return {"a"} end,
                    preview = function(item) return "Preview: " .. item end,
                    execute = function(items) return "Done", 0 end,
                },
                src2 = {
                    tag = "s2",
                    items = function() return {"x", "y"} end,
                    execute = function(items) return "Done", 0 end,
                },
            },
            pre_run = function() end,
            post_run = function() end,
        },
        none_task = {
            name = "Single Selection",
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
            description = "Execute-only task",
            execute = function() return "Task-level execute", 0 end
        },
    },
}
"#;

// ============================================================================
// Category 1: Valid Plugins (3 tests)
// ============================================================================

#[test]
fn test_minimal_valid_plugin() {
    let fixture = TestFixture::new();
    fixture.create_plugin("test", MINIMAL_VALID_PLUGIN);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_complete_valid_plugin() {
    let fixture = TestFixture::new();
    fixture.create_plugin("complete", COMPLETE_VALID_PLUGIN);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("complete")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_validate_plugin_directory_path() {
    let fixture = TestFixture::new();
    fixture.create_plugin("dirtest", MINIMAL_VALID_PLUGIN);

    let plugin_dir = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("dirtest");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

// ============================================================================
// Category 2: Invalid Lua/Structure (7 tests - 4 WILL FAIL)
// ============================================================================

#[test]
fn test_lua_syntax_error() {
    const SYNTAX_ERROR: &str = r#"
return {
    metadata = {
        name = "broken",
        version = "1.0.0",
"#; // Missing closing braces

    let fixture = TestFixture::new();
    fixture.create_plugin("broken", SYNTAX_ERROR);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("broken")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Failed to parse")
                .or(predicate::str::contains("syntax error")),
        );
}

#[test]
fn test_missing_metadata_table() {
    const NO_METADATA: &str = r#"
return {
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("no-metadata", NO_METADATA);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("no-metadata")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("metadata"));
}

#[test]
fn test_missing_tasks_table() {
    const NO_TASKS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("no-tasks", NO_TASKS);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("no-tasks")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("tasks"));
}

#[test]
fn test_plugin_returns_nil() {
    const RETURNS_NIL: &str = "return nil";

    let fixture = TestFixture::new();
    fixture.create_plugin("nil-return", RETURNS_NIL);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("nil-return")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should fail with clear message
        .stderr(
            predicate::str::contains("error converting Lua nil to table")
                .or(predicate::str::contains("must return a table"))
                .or(predicate::str::contains("expected table")),
        );
}

#[test]
fn test_plugin_returns_string() {
    const RETURNS_STRING: &str = r#"return "not a plugin""#;

    let fixture = TestFixture::new();
    fixture.create_plugin("string-return", RETURNS_STRING);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("string-return")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should fail with clear message
        .stderr(
            predicate::str::contains("error converting Lua string to table")
                .or(predicate::str::contains("must return a table"))
                .or(predicate::str::contains("expected table")),
        );
}

#[test]
fn test_metadata_not_table() {
    const METADATA_WRONG_TYPE: &str = r#"
return {
    metadata = "invalid",
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("bad-metadata", METADATA_WRONG_TYPE);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("bad-metadata")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should fail with type error
        .stderr(
            predicate::str::contains("metadata")
                .and(predicate::str::contains("table").or(predicate::str::contains("type"))),
        );
}

#[test]
fn test_tasks_not_table() {
    const TASKS_WRONG_TYPE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = "invalid"
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("bad-tasks", TASKS_WRONG_TYPE);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("bad-tasks")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should fail with type error
        .stderr(
            predicate::str::contains("tasks")
                .and(predicate::str::contains("table").or(predicate::str::contains("type"))),
        );
}

// ============================================================================
// Category 3: Invalid Metadata (6 tests - 2 WILL FAIL)
// ============================================================================

#[test]
fn test_missing_metadata_name() {
    //          Then validate_plugin() checks !name.is_empty() but it's called AFTER parsing succeeds
    const NO_NAME: &str = r#"
return {
    metadata = {version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("no-name", NO_NAME);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("no-name")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must have a name"));
}

#[test]
fn test_missing_metadata_version() {
    const NO_VERSION: &str = r#"
return {
    metadata = {name = "test"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("no-version", NO_VERSION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("no-version")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must have a specified version"));
}

#[test]
fn test_empty_name_string() {
    const EMPTY_NAME: &str = r#"
return {
    metadata = {name = "", version = "1.0.0"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("empty-name", EMPTY_NAME);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("empty-name")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must have a name"));
}

#[test]
fn test_multi_character_icon() {
    const MULTI_CHAR_ICON: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "ABC"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("multi-icon", MULTI_CHAR_ICON);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("multi-icon")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("single terminal cell"));
}

#[test]
fn test_platforms_wrong_type() {
    const PLATFORMS_STRING: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", platforms = "macos"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("bad-platforms", PLATFORMS_STRING);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("bad-platforms")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should reject type mismatch
        .stderr(
            predicate::str::contains("platforms")
                .and(predicate::str::contains("array").or(predicate::str::contains("table"))),
        );
}

#[test]
fn test_version_format_not_validated() {
    const INVALID_VERSION: &str = r#"
return {
    metadata = {name = "test", version = "not-a-version"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("bad-version", INVALID_VERSION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("bad-version")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should validate version format
        .stderr(
            predicate::str::contains("version")
                .and(predicate::str::contains("format").or(predicate::str::contains("invalid"))),
        );
}

// ============================================================================
// Category 4: Invalid Tasks (4 tests)
// ============================================================================

#[test]
fn test_empty_tasks_table() {
    //   ensure!(!plugin.tasks.is_empty(), "Plugin must define at least one task");
    const EMPTY_TASKS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("empty-tasks", EMPTY_TASKS);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("empty-tasks")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should fail
        .stderr(predicate::str::contains("at least one task"));
}

#[test]
fn test_task_no_item_sources_no_execute() {
    const NO_EXECUTE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {empty = {name = "Empty Task"}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("no-execute", NO_EXECUTE);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("no-execute")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "must have either 'item_sources' or 'execute'",
        ));
}

#[test]
fn test_invalid_mode_value() {
    const INVALID_MODE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {t = {mode = "batch", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("invalid-mode", INVALID_MODE);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("invalid-mode")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid mode"));
}

#[test]
fn test_mode_wrong_type() {
    const MODE_NUMBER: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {t = {mode = 123, execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("mode-number", MODE_NUMBER);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("mode-number")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("mode").or(predicate::str::contains("type")));
}

// ============================================================================
// Category 5: Invalid Item Sources (4 tests - 1 WILL FAIL)
// ============================================================================

#[test]
fn test_item_source_missing_tag() {
    const NO_TAG: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        t = {
            item_sources = {
                src = {items = function() return {"a"} end}
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("no-tag", NO_TAG);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("no-tag")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("tag"));
}

#[test]
fn test_empty_item_sources_with_execute() {
    // This is semantically questionable but may be acceptable if task has execute
    const EMPTY_SOURCES: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        t = {
            description = "Test task",
            item_sources = {},
            execute = function() return "", 0 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("empty-sources", EMPTY_SOURCES);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("empty-sources")
        .join("plugin.lua");

    // This may actually pass - having execute with empty item_sources is valid
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .success(); // Acceptable edge case
}

#[test]
fn test_item_source_missing_items_function() {
    //   ensure!(source_table.get::<mlua::Function>("items").is_ok(), "Missing 'items' function");
    const NO_ITEMS_FUNCTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        t = {
            item_sources = {
                src = {
                    tag = "s",
                    execute = function(items) return "", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("no-items-fn", NO_ITEMS_FUNCTION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("no-items-fn")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure() // DESIRED: Should fail validation
        .stderr(
            predicate::str::contains("items")
                .and(predicate::str::contains("function").or(predicate::str::contains("required"))),
        );
}

#[test]
fn test_multi_source_empty_tag() {
    const EMPTY_TAG: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        t = {
            description = "Test task",
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"item1", "item2"} end,
                    execute = function(items)
                        local output = "src1 received: " .. table.concat(items, ", ")
                        return output, 0
                    end
                },
                src2 = {
                    tag = "",
                    items = function() return {"item3", "item4"} end,
                    execute = function(items)
                        local output = "src2 received: " .. table.concat(items, ", ")
                        return output, 0
                    end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("empty-tag", EMPTY_TAG);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("empty-tag")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("tag"));
}

// ============================================================================
// Category 7: Edge Cases (5 tests)
// ============================================================================

#[test]
fn test_plugin_file_not_found() {
    let fixture = TestFixture::new();
    let nonexistent = fixture.data_path().join("nonexistent").join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&nonexistent)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_path_not_plugin_lua() {
    let fixture = TestFixture::new();

    // Create a Lua file not named plugin.lua
    let script_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("script.lua");
    std::fs::create_dir_all(script_path.parent().unwrap()).unwrap();
    std::fs::write(&script_path, MINIMAL_VALID_PLUGIN).unwrap();

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&script_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("plugin.lua"));
}

#[test]
fn test_empty_plugin_file() {
    let fixture = TestFixture::new();
    fixture.create_plugin("empty", ""); // Empty file

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("empty")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse").or(predicate::str::contains("error")));
}

#[test]
fn test_plugin_returns_empty_table() {
    const EMPTY_TABLE: &str = "return {}";

    let fixture = TestFixture::new();
    fixture.create_plugin("empty-table", EMPTY_TABLE);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("empty-table")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("metadata"));
}

#[test]
fn test_unicode_icon_single_cell() {
    const UNICODE_ICON: &str = r#"
return {
    metadata = {name = "unicode", version = "1.0.0", icon = "★"},
    tasks = {t = {description = "Test task", execute = function() return "", 0 end}}
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("unicode", UNICODE_ICON);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("unicode")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .success() // Unicode emoji should be accepted
        .stdout(predicate::str::contains("is valid"));
}
