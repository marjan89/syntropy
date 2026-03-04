//! Integration tests for multi-source items pipeline partial failure handling
//!
//! These tests verify that when the `items()` function fails in one or more item sources,
//! the system continues collecting items from remaining sources and returns partial results
//! rather than failing the entire operation.
//!
//! **Implementation Status**: ✅ FIXED - The items pipeline now uses `match` with `continue`
//! (src/execution/runner.rs) instead of the `?` operator, enabling graceful partial failure handling.
//!
//! **Verified Behavior**: Collects items from all successful sources, logs/skips failed sources,
//! and returns partial results. Only fails if ALL sources fail.
//!
//! **Impact**: Users with multi-source tasks retain access to items from working sources even when one source has issues.

use assert_cmd::Command;

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
fn test_all_sources_succeed() {
    // Baseline test: verify normal operation when all sources succeed
    const ALL_SUCCESS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        baseline = {
            description = "Baseline test",
            name = "All Sources Succeed",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() return {"a1", "a2"} end,
                    preselected_items = function() return {"a1", "a2"} end,
                    execute = function(items)
                        return "SOURCE_A:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {"b1", "b2"} end,
                    preselected_items = function() return {"b1", "b2"} end,
                    execute = function(items)
                        return "SOURCE_B:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() return {"c1"} end,
                    preselected_items = function() return {"c1"} end,
                    execute = function(items)
                        return "SOURCE_C:[" .. table.concat(items, "|") .. "]", 0
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

    // All sources should execute successfully
    assert!(
        stdout.contains("SOURCE_A:[a1|a2]"),
        "Source A should execute. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_B:[b1|b2]"),
        "Source B should execute. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_C:[c1]"),
        "Source C should execute. Got: {}",
        stdout
    );
    assert!(output.status.success(), "Should exit with success code");
}

// ============================================================================
// Test Category 2: Single Source Failures at Different Positions
// ============================================================================

#[test]
fn test_first_source_fails() {
    // When first source items() fails, remaining sources should still provide items
    const FIRST_FAILS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        first_fails = {
            description = "First source fails",
            name = "First Fails",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() error("Source A failed: network timeout") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_A")
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
                        return "SOURCE_C_SUCCESS:[" .. table.concat(items, "|") .. "]", 0
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
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Sources B and C should still execute despite A failing
    assert!(
        stdout.contains("SOURCE_B_SUCCESS:[b1|b2]"),
        "Source B should execute despite A failing. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_C_SUCCESS:[c1]"),
        "Source C should execute despite A failing. Got: {}",
        stdout
    );

    // Source A should not execute (no items collected)
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE_A") && !stderr.contains("SHOULD_NOT_EXECUTE_A"),
        "Source A execute should not be called. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_middle_source_fails() {
    // When middle source items() fails, first and last sources should still work
    const MIDDLE_FAILS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        middle_fails = {
            description = "Middle source fails",
            name = "Middle Fails",
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
                    items = function() error("Source B failed: permission denied") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_B")
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
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Sources A and C should execute successfully
    assert!(
        stdout.contains("SOURCE_A_SUCCESS:[a1]"),
        "Source A should execute despite B failing. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_C_SUCCESS:[c1]"),
        "Source C should execute despite B failing. Got: {}",
        stdout
    );

    // Source B should not execute (no items collected)
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE_B") && !stderr.contains("SHOULD_NOT_EXECUTE_B"),
        "Source B execute should not be called. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_last_source_fails() {
    // When last source items() fails, earlier sources should still work
    const LAST_FAILS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        last_fails = {
            description = "Last source fails",
            name = "Last Fails",
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
                    items = function() error("Source C failed: API rate limit") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_C")
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
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Sources A and B should execute successfully
    assert!(
        stdout.contains("SOURCE_A_SUCCESS:[a1|a2]"),
        "Source A should execute despite C failing. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SOURCE_B_SUCCESS:[b1]"),
        "Source B should execute despite C failing. Got: {}",
        stdout
    );

    // Source C should not execute (no items collected)
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE_C") && !stderr.contains("SHOULD_NOT_EXECUTE_C"),
        "Source C execute should not be called. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

// ============================================================================
// Test Category 3: Multiple Failures
// ============================================================================

#[test]
fn test_multiple_failures_one_success() {
    // When multiple sources fail, the one successful source should still work
    const MULTIPLE_FAIL: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        multi_fail = {
            description = "Multiple failures",
            name = "Multiple Failures",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() error("Source A failed") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_A")
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {"b1", "b2", "b3"} end,
                    preselected_items = function() return {"b1", "b2", "b3"} end,
                    execute = function(items)
                        return "SOURCE_B_SUCCESS:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() error("Source C failed") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_C")
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
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Source B (the only successful one) should execute
    assert!(
        stdout.contains("SOURCE_B_SUCCESS:[b1|b2|b3]"),
        "Source B should execute despite A and C failing. Got: {}",
        stdout
    );

    // Sources A and C should not execute (no items collected)
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE_A") && !stderr.contains("SHOULD_NOT_EXECUTE_A"),
        "Source A execute should not be called. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE_C") && !stderr.contains("SHOULD_NOT_EXECUTE_C"),
        "Source C execute should not be called. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_all_sources_fail() {
    // When ALL sources fail, the operation should fail (no partial results to return)
    const ALL_FAIL: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        all_fail = {
            description = "All sources fail",
            name = "All Fail",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() error("Source A failed") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_A")
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() error("Source B failed") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_B")
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() error("Source C failed") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_C")
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ALL_FAIL);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("all_fail")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // No execute functions should be called
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE") && !stderr.contains("SHOULD_NOT_EXECUTE"),
        "No execute functions should be called when all sources fail. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Operation should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Should exit with error code when all sources fail"
    );
}

// ============================================================================
// Test Category 4: Mixed Empty/Fail/Success
// ============================================================================

#[test]
fn test_mixed_empty_fail_success() {
    // Test combination of empty arrays (valid), failures (errors), and successes
    // Empty arrays should be treated as valid (just no items), not failures
    const MIXED: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        mixed = {
            description = "Mixed scenarios",
            name = "Mixed Empty/Fail/Success",
            mode = "multi",
            item_sources = {
                empty = {
                    tag = "e",
                    items = function() return {} end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_EMPTY")
                    end,
                },
                failing = {
                    tag = "f",
                    items = function() error("Source failing failed") end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_FAILING")
                    end,
                },
                success = {
                    tag = "s",
                    items = function() return {"s1", "s2"} end,
                    preselected_items = function() return {"s1", "s2"} end,
                    execute = function(items)
                        return "SOURCE_SUCCESS:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MIXED);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("mixed")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Only success source should execute
    assert!(
        stdout.contains("SOURCE_SUCCESS:[s1|s2]"),
        "Success source should execute. Got: {}",
        stdout
    );

    // Empty and failing sources should not execute
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE") && !stderr.contains("SHOULD_NOT_EXECUTE"),
        "Empty and failing sources should not execute. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Should succeed overall (at least one source worked)
    assert!(
        output.status.success(),
        "Should exit with success when at least one source succeeds"
    );
}

// ============================================================================
// Test Category 5: All Sources Empty Items (Valid Scenario)
// ============================================================================

#[test]
fn test_all_sources_empty_items() {
    // All sources return empty item arrays (valid, not error)
    // This is different from test_all_sources_fail which returns errors
    const ALL_EMPTY_ITEMS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        all_empty = {
            description = "All sources empty",
            name = "All Sources Empty Items",
            mode = "multi",
            item_sources = {
                source_a = {
                    tag = "a",
                    items = function() return {} end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_A")
                    end,
                },
                source_b = {
                    tag = "b",
                    items = function() return {} end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_B")
                    end,
                },
                source_c = {
                    tag = "c",
                    items = function() return {} end,
                    execute = function(items)
                        error("SHOULD_NOT_EXECUTE_C")
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ALL_EMPTY_ITEMS);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("all_empty")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Empty items is valid (not error) - no execute functions should be called
    assert!(
        !stdout.contains("SHOULD_NOT_EXECUTE") && !stderr.contains("SHOULD_NOT_EXECUTE"),
        "No execute functions should be called when all sources have empty items. Got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Should succeed (empty items is valid, different from all sources erroring)
    assert!(
        output.status.success(),
        "Should exit with success when all sources have empty items (valid scenario)"
    );

    // Verify output indicates no items executed
    assert!(
        stdout.contains("No items were executed") || stdout.is_empty(),
        "Should indicate no items executed. Got: {}",
        stdout
    );
}
