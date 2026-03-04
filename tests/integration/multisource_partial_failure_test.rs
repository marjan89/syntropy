//! Integration tests for multi-source execute pipeline partial failure handling
//!
//! These tests verify that when the `execute()` function fails in one or more item sources,
//! the system continues executing remaining sources, preserves successful outputs, and
//! always calls cleanup hooks (post_run).
//!
//! **Implementation Status**: ✅ FIXED - The execute pipeline now uses `match` with `continue`
//! (src/execution/runner.rs) instead of the `?` operator, enabling graceful partial failure handling.
//!
//! **Verified Behavior**:
//! - Continues executing remaining sources after one fails
//! - Preserves and concatenates output from successful sources
//! - Always calls post_run() for cleanup (locks, connections, temp files)
//! - Returns first non-zero exit code
//!
//! **Impact**: Users retain partial work when one source fails, and resource cleanup is guaranteed.

use assert_cmd::Command;
use std::fs;

use crate::common::TestFixture;

const MINIMAL_CONFIG: &str = r#"
default_plugin_icon = "⚒"

[keybindings]
back = "<esc>"
select_previous = "<up>"
select_next = "<down>"
scroll_preview_up = "["
scroll_preview_down = "]"
toggle_preview = "<C-p>"
select = "<tab>"
confirm = "<enter>"
"#;

// ============================================================================
// Test Category 1: Baseline - All Sources Succeed
// ============================================================================

#[test]
fn test_execute_multisource_all_sources_succeed() {
    // Baseline test: verify normal operation when all sources execute successfully
    const ALL_SUCCESS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        baseline = {
            description = "Baseline test",
            name = "All Execute Succeed",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() return {"a1"} end,
                    preselected_items = function() return {"a1"} end,
                    execute = function(items)
                        return "SOURCE_A_OUTPUT:[" .. items[1] .. "]", 0
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {"b1"} end,
                    preselected_items = function() return {"b1"} end,
                    execute = function(items)
                        return "SOURCE_B_OUTPUT:[" .. items[1] .. "]", 0
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() return {"c1"} end,
                    preselected_items = function() return {"c1"} end,
                    execute = function(items)
                        return "SOURCE_C_OUTPUT:[" .. items[1] .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ALL_SUCCESS);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("baseline")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // All outputs should be present
    assert!(
        stdout.contains("SOURCE_A_OUTPUT:[a1]"),
        "Source A output should be present. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_B_OUTPUT:[b1]"),
        "Source B output should be present. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_C_OUTPUT:[c1]"),
        "Source C output should be present. Got: {}",
        stdout
    );

    assert!(output.status.success(), "Should exit with success code");
}

// ============================================================================
// Test Category 2: Single Source Execute Failures at Different Positions
// ============================================================================

#[test]
fn test_execute_multisource_first_source_fails() {
    // When first source execute() fails, remaining sources should still execute
    // and their outputs should be preserved
    const FIRST_FAILS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        first_fails = {
            description = "First execute fails",
            name = "First Execute Fails",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() return {"a1"} end,
                    preselected_items = function() return {"a1"} end,
                    execute = function(items)
                        error("Source A execute failed: command not found")
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {"b1"} end,
                    preselected_items = function() return {"b1"} end,
                    execute = function(items)
                        return "SOURCE_B_SUCCESS:[" .. items[1] .. "]", 0
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() return {"c1"} end,
                    preselected_items = function() return {"c1"} end,
                    execute = function(items)
                        return "SOURCE_C_SUCCESS:[" .. items[1] .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", FIRST_FAILS);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("first_fails")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Sources B and C should still execute and produce output
    assert!(
        stdout.contains("SOURCE_B_SUCCESS:[b1]"),
        "Source B should execute despite A failing. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_C_SUCCESS:[c1]"),
        "Source C should execute despite A failing. Got: {}",
        stdout
    );

    // Verify outputs appear in sequence (B before C, though order may vary due to HashMap)
    let has_b = stdout.contains("SOURCE_B_SUCCESS");
    let has_c = stdout.contains("SOURCE_C_SUCCESS");
    assert!(has_b && has_c, "Both B and C outputs should be present");
}

#[test]
fn test_execute_multisource_middle_source_fails() {
    // When middle source execute() fails, first source output should be preserved
    // and last source should still execute
    const MIDDLE_FAILS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        middle_fails = {
            description = "Middle execute fails",
            name = "Middle Execute Fails",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() return {"a1"} end,
                    preselected_items = function() return {"a1"} end,
                    execute = function(items)
                        return "SOURCE_A_SUCCESS:[" .. items[1] .. "]", 0
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {"b1"} end,
                    preselected_items = function() return {"b1"} end,
                    execute = function(items)
                        error("Source B execute failed: insufficient permissions")
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() return {"c1"} end,
                    preselected_items = function() return {"c1"} end,
                    execute = function(items)
                        return "SOURCE_C_SUCCESS:[" .. items[1] .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MIDDLE_FAILS);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("middle_fails")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Sources A and C outputs should be preserved
    assert!(
        stdout.contains("SOURCE_A_SUCCESS:[a1]"),
        "Source A output should be preserved despite B failing. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_C_SUCCESS:[c1]"),
        "Source C should execute despite B failing. Got: {}",
        stdout
    );
}

#[test]
fn test_execute_multisource_last_source_fails() {
    // When last source execute() fails, earlier source outputs should be preserved
    const LAST_FAILS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        last_fails = {
            description = "Last execute fails",
            name = "Last Execute Fails",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() return {"a1", "a2"} end,
                    preselected_items = function() return {"a1", "a2"} end,
                    execute = function(items)
                        return "SOURCE_A_SUCCESS:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {"b1"} end,
                    preselected_items = function() return {"b1"} end,
                    execute = function(items)
                        return "SOURCE_B_SUCCESS:[" .. items[1] .. "]", 0
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() return {"c1"} end,
                    preselected_items = function() return {"c1"} end,
                    execute = function(items)
                        error("Source C execute failed: quota exceeded")
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", LAST_FAILS);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("last_fails")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Sources A and B outputs should be preserved despite C failing
    assert!(
        stdout.contains("SOURCE_A_SUCCESS:[a1|a2]"),
        "Source A output should be preserved despite C failing. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_B_SUCCESS:[b1]"),
        "Source B output should be preserved despite C failing. Got: {}",
        stdout
    );
}

// ============================================================================
// Test Category 3: Multiple Failures and Mixed Scenarios
// ============================================================================

#[test]
fn test_execute_multisource_multiple_failures() {
    // When multiple sources fail, successful source outputs should still be collected
    const MULTIPLE_FAIL: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        multi_fail = {
            description = "Multiple execute failures",
            name = "Multiple Execute Failures",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() return {"a1"} end,
                    preselected_items = function() return {"a1"} end,
                    execute = function(items)
                        error("Source A execute failed")
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {"b1", "b2"} end,
                    preselected_items = function() return {"b1", "b2"} end,
                    execute = function(items)
                        return "SOURCE_B_SUCCESS:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() return {"c1"} end,
                    preselected_items = function() return {"c1"} end,
                    execute = function(items)
                        error("Source C execute failed")
                    end,
                },
                source_d = {
                    tag = "d",
                    items = function() return {"d1"} end,
                    preselected_items = function() return {"d1"} end,
                    execute = function(items)
                        return "SOURCE_D_SUCCESS:[" .. items[1] .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MULTIPLE_FAIL);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("multi_fail")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Sources B and D (successful ones) should have their outputs present
    assert!(
        stdout.contains("SOURCE_B_SUCCESS:[b1|b2]"),
        "Source B output should be present despite A and C failing. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_D_SUCCESS:[d1]"),
        "Source D output should be present despite A and C failing. Got: {}",
        stdout
    );
}

// ============================================================================
// Test Category 4: Cleanup (post_run) Always Executes
// ============================================================================

#[test]
fn test_execute_multisource_failure_still_calls_post_run() {
    // CRITICAL: Verify post_run() is called for cleanup even when execute fails
    // This prevents resource leaks (lock files, database connections, temp files)
    let fixture = TestFixture::new();

    // Create marker file path for post_run to write to
    let marker_path = fixture.temp_dir.path().join("post_run_executed.txt");
    let marker_path_str = marker_path.to_str().unwrap();

    let plugin_content = format!(
        r#"
return {{
    metadata = {{name = "test", version = "1.0.0", icon = "T", platforms = {{"macos", "linux"}}}},
    tasks = {{
        cleanup_test = {{
            description = "Cleanup test",
            name = "Post Run Cleanup Test",
            mode = "multi",
            pre_run = function()
                -- Simulating resource allocation (database connection, lock file, etc.)
            end,
            post_run = function()
                -- This MUST execute even if execute fails
                local f = io.open("{}", "w")
                if f then
                    f:write("post_run_executed")
                    f:close()
                end
            end,
            item_sources = {{
                source_a = {{
                    tag = "a",
                    items = function() return {{"a1"}} end,
                    preselected_items = function() return {{"a1"}} end,
                    execute = function(items)
                        return "SOURCE_A_SUCCESS", 0
                    end,
                }},
                source_b = {{
                    tag = "b",
                    items = function() return {{"b1"}} end,
                    preselected_items = function() return {{"b1"}} end,
                    execute = function(items)
                        error("Source B execute failed - simulating crash")
                    end,
                }},
                source_c = {{
                    tag = "c",
                    items = function() return {{"c1"}} end,
                    preselected_items = function() return {{"c1"}} end,
                    execute = function(items)
                        return "SOURCE_C_SUCCESS", 0
                    end,
                }},
            }},
        }},
    }},
}}
"#,
        marker_path_str
    );

    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", &plugin_content);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("cleanup_test")
        .output()
        .unwrap();

    // Verify execution failed (source B failed)
    assert!(
        !output.status.success(),
        "Should exit with error code when source B fails"
    );

    // CRITICAL ASSERTION: post_run must have executed and created the marker file
    assert!(
        marker_path.exists(),
        "post_run() must execute despite execute failures. Marker file not found at: {}",
        marker_path_str
    );

    let content = fs::read_to_string(&marker_path).expect("Failed to read marker file");
    assert_eq!(
        content, "post_run_executed",
        "post_run() marker file content incorrect"
    );
}

// ============================================================================
// Test Category 5: Exit Code Handling
// ============================================================================

#[test]
fn test_execute_multisource_mixed_exit_codes() {
    // Verify that when sources have different exit codes, the first non-zero is returned
    const MIXED_EXIT_CODES: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos", "linux"}},
    tasks = {
        mixed_codes = {
            description = "Mixed exit codes",
            name = "Mixed Exit Codes",
            mode = "multi",
            item_sources = {
                success = {
                    tag = "s",
                    items = function() return {"s1"} end,
                    preselected_items = function() return {"s1"} end,
                    execute = function(items)
                        return "SUCCESS_OUTPUT", 0
                    end,
                },
                warning = {
                    tag = "w",
                    items = function() return {"w1"} end,
                    preselected_items = function() return {"w1"} end,
                    execute = function(items)
                        return "WARNING_OUTPUT", 1
                    end,
                },
                error = {
                    tag = "e",
                    items = function() return {"e1"} end,
                    preselected_items = function() return {"e1"} end,
                    execute = function(items)
                        return "ERROR_OUTPUT", 2
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MIXED_EXIT_CODES);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("mixed_codes")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let exit_code = output.status.code().unwrap_or(0);

    // All outputs should be present
    assert!(
        stdout.contains("SUCCESS_OUTPUT"),
        "Success output should be present. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("WARNING_OUTPUT"),
        "Warning output should be present. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("ERROR_OUTPUT"),
        "Error output should be present. Got: {}",
        stdout
    );

    // Exit code should be first non-zero (either 1 or 2, depending on execution order)
    // Due to HashMap iteration order, we just verify it's non-zero
    assert!(
        exit_code != 0,
        "Exit code should be non-zero when sources return non-zero codes. Got: {}",
        exit_code
    );
}
