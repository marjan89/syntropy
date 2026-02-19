//! Integration tests for multi-source execute pipeline routing
//!
//! These tests verify the critical fix where execute functions are only called
//! for item sources that have matching items, not ALL item sources unconditionally.
//!
//! **Bug Fixed**: Previously, ALL item source execute functions were called even when
//! no items matched that source's tag. This caused incorrect behavior in mode="none"
//! multi-source tasks where only ONE execute should be called.
//!
//! **Fix Location**: src/execution/runner.rs:237-239 - Added empty check to skip execution

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
// Category 1: Mode=None Multi-Source Routing (Core Fix)
// ============================================================================

#[test]
fn mode_none_multisource_only_calls_matching_execute_tag_a() {
    // CRITICAL: When selecting item with tag "a", ONLY source "a" execute should be called
    // This is the PRIMARY test for the bug fix
    const TWO_SOURCE_MODE_NONE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        dual = {
            description = "Test task",
            name = "Dual Source Mode None",
            mode = "none",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"git", "node"} end,
                    execute = function(items)
                        return "PKG_EXECUTE_CALLED:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                cask = {
                    tag = "cask",
                    items = function() return {"Chrome", "iTerm"} end,
                    execute = function(items)
                        return "CASK_EXECUTE_CALLED:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", TWO_SOURCE_MODE_NONE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("dual")
        .arg("--items")
        .arg("git")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // CRITICAL ASSERTIONS: Only PKG execute called, NOT cask
    assert!(
        stdout.contains("PKG_EXECUTE_CALLED:[git]"),
        "Package execute should be called for git item. Got: {}",
        stdout
    );
    assert!(
        !stdout.contains("CASK_EXECUTE_CALLED"),
        "Cask execute should NOT be called for git item. Got: {}",
        stdout
    );
}

#[test]
fn mode_none_multisource_only_calls_matching_execute_tag_b() {
    // Verify the opposite direction - selecting tag "cask" item only calls cask execute
    const TWO_SOURCE_MODE_NONE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        dual = {
            description = "Test task",
            name = "Dual Source Mode None",
            mode = "none",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"git", "node"} end,
                    execute = function(items)
                        return "PKG_EXECUTE_CALLED:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                cask = {
                    tag = "cask",
                    items = function() return {"Chrome", "iTerm"} end,
                    execute = function(items)
                        return "CASK_EXECUTE_CALLED:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", TWO_SOURCE_MODE_NONE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("dual")
        .arg("--items")
        .arg("Chrome")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // CRITICAL ASSERTIONS: Only CASK execute called, NOT pkg
    assert!(
        stdout.contains("CASK_EXECUTE_CALLED:[Chrome]"),
        "Cask execute should be called for Chrome item. Got: {}",
        stdout
    );
    assert!(
        !stdout.contains("PKG_EXECUTE_CALLED"),
        "Package execute should NOT be called for Chrome item. Got: {}",
        stdout
    );
}

#[test]
fn mode_none_multisource_three_sources_only_one_called() {
    // Verify with 3 sources that only the matching one is called
    const THREE_SOURCE_MODE_NONE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        triple = {
            description = "Test task",
            name = "Triple Source Mode None",
            mode = "none",
            item_sources = {
                src_a = {
                    tag = "a",
                    items = function() return {"a1", "a2"} end,
                    execute = function(items) return "A_CALLED", 0 end,
                },
                src_b = {
                    tag = "b",
                    items = function() return {"b1", "b2"} end,
                    execute = function(items) return "B_CALLED", 0 end,
                },
                src_c = {
                    tag = "c",
                    items = function() return {"c1", "c2"} end,
                    execute = function(items) return "C_CALLED", 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", THREE_SOURCE_MODE_NONE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("triple")
        .arg("--items")
        .arg("b1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Only B should be called
    assert!(
        stdout.contains("B_CALLED"),
        "Source B execute should be called. Got: {}",
        stdout
    );
    assert!(
        !stdout.contains("A_CALLED"),
        "Source A execute should NOT be called. Got: {}",
        stdout
    );
    assert!(
        !stdout.contains("C_CALLED"),
        "Source C execute should NOT be called. Got: {}",
        stdout
    );
}

// ============================================================================
// Category 2: Mode=Multi Correct Routing
// ============================================================================

#[test]
fn mode_multi_only_calls_sources_with_selected_items() {
    // Verify mode=multi also respects the fix - only sources with selected items execute
    const MULTI_MODE_SELECTIVE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        selective = {
            description = "Test task",
            name = "Selective Multi",
            mode = "multi",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"git", "node", "vim"} end,
                    preselected_items = function() return {} end,
                    execute = function(items)
                        return "PKG:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                cask = {
                    tag = "cask",
                    items = function() return {"Chrome", "iTerm"} end,
                    preselected_items = function() return {} end,
                    execute = function(items)
                        return "CASK:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MULTI_MODE_SELECTIVE);

    // Select only package items via --items flag
    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("selective")
        .arg("--items")
        .arg("git")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Only PKG execute should be called (no cask items selected)
    assert!(
        stdout.contains("PKG:[git]"),
        "Package execute should be called. Got: {}",
        stdout
    );
    assert!(
        !stdout.contains("CASK:"),
        "Cask execute should NOT be called (no items selected). Got: {}",
        stdout
    );
}

#[test]
fn mode_multi_calls_both_sources_when_both_have_items() {
    // Verify that when BOTH sources have selected items, BOTH executes are called
    const MULTI_MODE_BOTH: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        both = {
            description = "Test task",
            name = "Both Sources",
            mode = "multi",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"git"} end,
                    preselected_items = function() return {"git"} end,
                    execute = function(items)
                        return "PKG:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                cask = {
                    tag = "cask",
                    items = function() return {"Chrome"} end,
                    preselected_items = function() return {"Chrome"} end,
                    execute = function(items)
                        return "CASK:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MULTI_MODE_BOTH);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("both")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Both should be called (both have preselected items)
    assert!(
        stdout.contains("PKG:[git]"),
        "Package execute should be called. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("CASK:[Chrome]"),
        "Cask execute should be called. Got: {}",
        stdout
    );
}

// ============================================================================
// Category 3: Empty Items Edge Cases
// ============================================================================

#[test]
fn empty_source_execute_not_called() {
    // Verify execute is NOT called when a source has no items
    const EMPTY_SOURCE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        empty = {
            description = "Test task",
            name = "Empty Source",
            mode = "multi",
            item_sources = {
                empty = {
                    tag = "e",
                    items = function() return {} end,
                    execute = function(items)
                        error("EMPTY_EXECUTE_SHOULD_NOT_BE_CALLED")
                    end,
                },
                nonempty = {
                    tag = "n",
                    items = function() return {"item1"} end,
                    execute = function(items)
                        return "NONEMPTY_CALLED", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", EMPTY_SOURCE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("empty")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Empty execute should NOT be called
    assert!(
        !stdout.contains("EMPTY_EXECUTE_SHOULD_NOT_BE_CALLED"),
        "Empty source execute should not be called. Got stdout: {}",
        stdout
    );
    assert!(
        !stderr.contains("EMPTY_EXECUTE_SHOULD_NOT_BE_CALLED"),
        "Empty source execute should not be called. Got stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("NONEMPTY_CALLED"),
        "Non-empty source execute should be called. Got: {}",
        stdout
    );
}

#[test]
fn all_sources_empty_no_executes_called() {
    // Verify when ALL sources are empty, NO executes are called
    const ALL_EMPTY: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        all_empty = {
            description = "Test task",
            name = "All Empty",
            mode = "multi",
            item_sources = {
                src_a = {
                    tag = "a",
                    items = function() return {} end,
                    execute = function(items) error("A_SHOULD_NOT_BE_CALLED") end,
                },
                src_b = {
                    tag = "b",
                    items = function() return {} end,
                    execute = function(items) error("B_SHOULD_NOT_BE_CALLED") end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ALL_EMPTY);

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

    // No executes should be called
    assert!(
        !stdout.contains("SHOULD_NOT_BE_CALLED") && !stderr.contains("SHOULD_NOT_BE_CALLED"),
        "No execute functions should be called when all sources are empty. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

// ============================================================================
// Category 4: Multiple Items Same Source
// ============================================================================

#[test]
fn multiple_items_same_source_single_execute_call() {
    // Verify that selecting multiple items from the SAME source calls execute ONCE with all items
    const MULTI_ITEMS_SINGLE_SOURCE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        multi = {
            description = "Test task",
            name = "Multiple Items",
            mode = "multi",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"git", "node", "vim"} end,
                    preselected_items = function() return {"git", "node", "vim"} end,
                    execute = function(items)
                        -- Verify all 3 items received in single call
                        if #items ~= 3 then
                            error("Expected 3 items, got " .. #items)
                        end
                        return "EXECUTED_COUNT:" .. #items .. "|ITEMS:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MULTI_ITEMS_SINGLE_SOURCE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("multi")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify execute called once with all 3 items
    assert!(
        stdout.contains("EXECUTED_COUNT:3"),
        "Execute should receive all 3 items in single call. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("ITEMS:[git|node|vim]"),
        "Execute should receive all items. Got: {}",
        stdout
    );
}

// ============================================================================
// Category 5: Single Source Tasks (No Tag Filtering)
// ============================================================================

#[test]
fn single_source_task_execute_called_normally() {
    // Verify single-source tasks continue to work correctly (no tag filtering)
    const SINGLE_SOURCE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        single = {
            description = "Test task",
            name = "Single Source",
            mode = "multi",
            item_sources = {
                only = {
                    tag = "o",
                    items = function() return {"item1", "item2", "item3"} end,
                    execute = function(items)
                        return "SINGLE_CALLED:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", SINGLE_SOURCE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("single")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Execute should be called with all items
    assert!(
        stdout.contains("SINGLE_CALLED:[item1|item2|item3]"),
        "Single-source execute should be called with all items. Got: {}",
        stdout
    );
}

// ============================================================================
// Category 6: Execution Order and Exit Code Propagation
// ============================================================================

#[test]
fn all_sources_with_items_execute() {
    // Verify all sources with items have their execute functions called
    // NOTE: Execution order is not guaranteed (HashMap iteration order varies)
    const ORDERED_SOURCES: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        ordered = {
            description = "Test task",
            name = "Ordered Sources",
            mode = "multi",
            item_sources = {
                first = {
                    tag = "1st",
                    items = function() return {"a"} end,
                    execute = function(items) return "FIRST", 0 end,
                },
                second = {
                    tag = "2nd",
                    items = function() return {"b"} end,
                    execute = function(items) return "SECOND", 0 end,
                },
                third = {
                    tag = "3rd",
                    items = function() return {"c"} end,
                    execute = function(items) return "THIRD", 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ORDERED_SOURCES);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("ordered")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify all three sources executed (order not guaranteed due to HashMap iteration)
    assert!(
        stdout.contains("FIRST"),
        "First source should execute. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("SECOND"),
        "Second source should execute. Got: {}",
        stdout
    );
    assert!(
        stdout.contains("THIRD"),
        "Third source should execute. Got: {}",
        stdout
    );
}

#[test]
fn first_nonzero_exit_code_propagated_with_selective_execution() {
    // Verify exit code propagation works correctly when only some sources execute
    const MIXED_EXIT_CODES: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        mixed = {
            description = "Test task",
            name = "Mixed Exit Codes",
            mode = "multi",
            item_sources = {
                success = {
                    tag = "s",
                    items = function() return {"ok"} end,
                    execute = function(items) return "SUCCESS", 0 end,
                },
                failure = {
                    tag = "f",
                    items = function() return {"fail"} end,
                    execute = function(items) return "FAILURE", 42 end,
                },
                skipped = {
                    tag = "skip",
                    items = function() return {} end,
                    execute = function(items) error("SHOULD_NOT_BE_CALLED") end,
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
        .arg("mixed")
        .output()
        .unwrap();

    let exit_code = output.status.code().unwrap_or(0);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify exit code 42 propagated
    assert_eq!(
        exit_code, 42,
        "Should propagate exit code 42 from failure source"
    );

    // Verify only success and failure executed (not skipped)
    assert!(stdout.contains("SUCCESS"), "Success source should execute");
    assert!(stdout.contains("FAILURE"), "Failure source should execute");
    assert!(
        !stdout.contains("SHOULD_NOT_BE_CALLED"),
        "Skipped source should not execute"
    );
}
