//! Integration tests for CLI execute subcommand
//!
//! Tests the non-interactive execution mode used in scripts and CI/CD.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

// ============================================================================
// Mock Plugin Templates and Config
// ============================================================================

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
// Mock Plugin Templates
// ============================================================================

const SIMPLE_PLUGIN: &str = r#"
return {
    metadata = {
        description = "Test task",
        name = "test-plugin",
        version = "1.0.0",
        icon = "T",
        description = "Test",
        platforms = {"macos"},
    },
    tasks = {
        test_task = {
            description = "Test task",
            name = "Test Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "t",
                    items = function() return {"item1", "item2", "item3"} end,
                    execute = function(items) return "Executed " .. #items .. " items", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_PRESELECTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        selective = {
            description = "Test task",
            name = "Selective Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c", "d"} end,
                    preselected_items = function() return {"b", "d"} end,
                    execute = function(items) return "Processed: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_EMPTY_PRESELECTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        all_items = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c", "d"} end,
                    preselected_items = function() return {} end,
                    execute = function(items) return "Processed " .. #items .. " items", 0 end,
                },
            },
        },
    },
}
"#;

const STANDALONE_TASK: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        standalone = {
            description = "Test task",
            name = "Standalone Task",
            execute = function() return "Task completed", 0 end,
        },
    },
}
"#;

const FAILING_TASK: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        failing = {
            description = "Test task",
            execute = function() return "Task failed", 1 end,
        },
    },
}
"#;

// ============================================================================
// Test 1-3: Argument Validation
// ============================================================================

#[test]
fn execute_subcommand_requires_both_plugin_and_task() {
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("execute")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ));
}

#[test]
fn execute_subcommand_requires_plugin_when_only_task_provided() {
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("execute")
        .arg("--task")
        .arg("some_task")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--plugin"));
}

#[test]
fn execute_subcommand_requires_task_when_only_plugin_provided() {
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--task"));
}

// ============================================================================
// Test 4-5: Error Handling
// ============================================================================

#[test]
fn execute_with_nonexistent_plugin() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("nonexistent")
        .arg("--task")
        .arg("foo")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Plugin 'nonexistent' not found"))
        .stderr(predicate::str::contains("Available plugins:"));
}

#[test]
fn execute_with_nonexistent_task() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("nonexistent_task")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Task 'nonexistent_task' not found",
        ))
        .stderr(predicate::str::contains("Available tasks:"));
}

// ============================================================================
// Test 6-9: Success Paths
// ============================================================================

#[test]
fn execute_successful_task_with_items() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 3 items"))
        .stderr(predicate::str::contains("3 item"));
}

#[test]
fn execute_respects_preselected_items() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_WITH_PRESELECTION);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("selective")
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed: b,d"))
        .stderr(predicate::str::contains("2 preselected item"));
}

#[test]
fn execute_with_empty_preselected_uses_all_items() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_EMPTY_PRESELECTION);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("all_items")
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed 4 items"))
        .stderr(predicate::str::contains("all 4 item"));
}

#[test]
fn execute_task_without_item_sources() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", STANDALONE_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("standalone")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task completed"));
}

// ============================================================================
// Test 10-16: Exit Code Propagation and Advanced Features
// ============================================================================

#[test]
fn execute_propagates_exit_code_from_lua() {
    // Verifies that non-zero exit codes from Lua execute() are propagated to process exit
    // Lua execute() returns ("Task failed", 1), syntropy should exit with code 1
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", FAILING_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("failing")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("Task failed"));
}

#[test]
fn execute_propagates_exit_code_2_from_lua() {
    // Test that exit code 2 (common for command line syntax errors) propagates correctly
    const EXIT_CODE_2_TASK: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        exit_2 = {
            description = "Test task",
            name = "Exit Code 2 Task",
            execute = function() return "Command error", 2 end,
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", EXIT_CODE_2_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("exit_2")
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("Command error"));
}

#[test]
fn execute_propagates_exit_code_127_from_lua() {
    // Test that exit code 127 (command not found) propagates correctly
    const EXIT_CODE_127_TASK: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        exit_127 = {
            description = "Test task",
            name = "Exit Code 127 Task",
            execute = function() return "Command not found", 127 end,
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", EXIT_CODE_127_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("exit_127")
        .assert()
        .failure()
        .code(127)
        .stdout(predicate::str::contains("Command not found"));
}

#[test]
fn execute_propagates_first_nonzero_exit_code_from_multisource() {
    // Test that when multiple item sources exist, first non-zero exit code wins
    const MULTISOURCE_MIXED_EXIT_CODES: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        mixed = {
            description = "Test task",
            name = "Mixed Exit Codes Task",
            mode = "multi",
            item_sources = {
                success = {
                    tag = "s",
                    items = function() return {"item1"} end,
                    execute = function(items) return "Success from first source", 0 end,
                },
                failure = {
                    tag = "f",
                    items = function() return {"item2"} end,
                    execute = function(items) return "Failure from second source", 3 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MULTISOURCE_MIXED_EXIT_CODES);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("mixed")
        .assert()
        .failure()
        .code(3)
        .stdout(predicate::str::contains("Success from first source"))
        .stdout(predicate::str::contains("Failure from second source"));
}

#[test]
fn execute_with_negative_exit_code() {
    // Validates that negative exit codes are clamped to 1 with warning
    // POSIX standard: Exit codes are 0-255 (unsigned 8-bit)
    // Negative values are invalid and should be clamped to 1 (generic failure)
    // Fix location: src/cli/execute.rs - exit code validation before return

    const NEGATIVE_EXIT_CODE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        negative = {
            description = "Test task",
            name = "Negative Exit Code Task",
            execute = function() return "Negative exit code", -1 end,
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", NEGATIVE_EXIT_CODE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("negative")
        .output()
        .unwrap();

    // Exit code should be clamped to 1 (not -1, not 255)
    let exit_code = output.status.code().unwrap_or(0);
    assert_eq!(exit_code, 1, "Negative exit code should be clamped to 1");

    // Should warn about clamping in stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning") && stderr.contains("clamped"),
        "Should warn about clamping negative exit code to 1"
    );

    // Should still print the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Negative exit code"));
}

#[test]
fn execute_with_exit_code_greater_than_255() {
    // Validates that exit codes >255 are clamped to 255 with warning
    // POSIX standard: Exit codes are 0-255, values >255 are implementation-defined
    // Without clamping, platforms typically wrap modulo 256 (e.g., 256 → 0)
    // Fix location: src/cli/execute.rs - exit code validation before return

    const LARGE_EXIT_CODE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        large_code = {
            description = "Test task",
            name = "Large Exit Code Task",
            execute = function() return "Exit code 256", 256 end,
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", LARGE_EXIT_CODE);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("large_code")
        .output()
        .unwrap();

    // Exit code should be clamped to 255 (max valid POSIX exit code)
    let exit_code = output.status.code().unwrap_or(0);
    assert_eq!(exit_code, 255, "Exit code >255 should be clamped to 255");

    // Should warn about clamping in stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning") && stderr.contains("clamped"),
        "Should warn about clamping large exit code to 255"
    );

    // Should still print the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Exit code 256"));
}

#[test]
fn execute_stdout_stderr_separation() {
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Stdout: only task output
    assert!(stdout.contains("Executed 3 items"));
    assert!(!stdout.contains("item(s)")); // No informational messages

    // Stderr: only informational messages
    assert!(stderr.contains("3 item"));
    assert!(!stderr.contains("Executed")); // No task output
}

#[test]
fn execute_with_custom_config_path() {
    let fixture = TestFixture::new();
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    let config_content = r#"
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
    fixture.create_config("syntropy.toml", config_content);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--config")
        .arg(fixture.config_path().join("syntropy").join("syntropy.toml"))
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .assert()
        .success();
}

// ============================================================================
// Test 17-21: Additional Edge Cases (5 tests)
// ============================================================================

#[test]
fn execute_with_empty_items_array() {
    // Verifies that when items() returns an empty array, execute is NOT called
    // This is the correct behavior - empty items should skip execution entirely
    const EMPTY_ITEMS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        empty = {
            description = "Test task",
            name = "Empty Items Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {} end,
                    execute = function(items) return "Processed " .. #items .. " items", 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", EMPTY_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("empty")
        .assert()
        .success()
        // When items are empty, execute is NOT called, but we show a message indicating no items were executed
        .stdout(predicate::str::contains("No items were executed"))
        .stderr(predicate::str::contains("all 0 item"));
}

#[test]
fn execute_with_items_function_error() {
    const ITEMS_ERROR: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        broken_items = {
            description = "Test task",
            name = "Broken Items Function",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() error("Failed to fetch items") end,
                    execute = function(items) return "Done", 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ITEMS_ERROR);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("broken_items")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Failed to fetch items").or(predicate::str::contains("error")),
        );
}

#[test]
fn execute_with_preselected_items_error() {
    const PRESELECTED_ERROR: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        broken_preselect = {
            description = "Test task",
            name = "Broken Preselection",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c"} end,
                    preselected_items = function() error("Preselection failed") end,
                    execute = function(items) return "Processed", 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PRESELECTED_ERROR);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("broken_preselect")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Preselection failed").or(predicate::str::contains("error")),
        );
}

#[test]
fn execute_with_lua_runtime_error() {
    const RUNTIME_ERROR: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        runtime_error = {
            description = "Test task",
            name = "Runtime Error Task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item1"} end,
                    execute = function(items)
                        error("Runtime error during execution")
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", RUNTIME_ERROR);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("runtime_error")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Runtime error during execution")
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn execute_propagates_zero_exit_code() {
    const SUCCESS_TASK: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        success = {
            description = "Test task",
            name = "Success Task",
            execute = function() return "Task succeeded", 0 end,
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", SUCCESS_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("success")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("Task succeeded"));
}

#[test]
fn execute_twice_sequential_state_isolation() {
    const STATEFUL_TASK: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        stateful = {
            description = "Test task",
            name = "Stateful Task",
            execute = function()
                -- Set a global variable
                if _G.execution_count then
                    _G.execution_count = _G.execution_count + 1
                else
                    _G.execution_count = 1
                end
                return "Execution count: " .. tostring(_G.execution_count), 0
            end,
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", STATEFUL_TASK);

    // First execution
    let output1 = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("stateful")
        .output()
        .unwrap();

    let stdout1 = String::from_utf8_lossy(&output1.stdout);

    // Second execution (separate process)
    let output2 = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("stateful")
        .output()
        .unwrap();

    let stdout2 = String::from_utf8_lossy(&output2.stdout);

    // Both should show count of 1 (state isolated between executions)
    assert!(
        stdout1.contains("Execution count: 1"),
        "First execution should show count 1"
    );
    assert!(
        stdout2.contains("Execution count: 1"),
        "Second execution should show count 1 (isolated state)"
    );
}

// ============================================================================
// Test 25-40: --items Flag Tests (Core, Tag Handling, Edge Cases)
// ============================================================================

// Mock plugin templates for --items flag testing
const PLUGIN_MODE_NONE_MULTIPLE_ITEMS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        single_select = {
            description = "Test task",
            name = "Single Select Task",
            mode = "none",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item1", "item2", "item3"} end,
                    execute = function(items) return "Selected: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_MODE_NONE_SINGLE_ITEM: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        only_one = {
            description = "Test task",
            name = "Single Item Task",
            mode = "none",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"only_item"} end,
                    execute = function(items) return "Executed: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_MULTISOURCE_WITH_TAGS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        multi_source = {
            description = "Test task",
            name = "Multi Source Task",
            mode = "multi",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"git", "node", "npm"} end,
                    execute = function(items) return "Packages: " .. table.concat(items, ","), 0 end,
                },
                cask = {
                    tag = "cask",
                    items = function() return {"iTerm2", "Docker"} end,
                    execute = function(items) return "Cask: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_CASE_VARIATIONS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        case_test = {
            description = "Test task",
            name = "Case Sensitive Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"PackageManager", "Git", "Node"} end,
                    execute = function(items) return "Matched: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

// ----------------------------------------------------------------------------
// Core Functionality Tests (6 tests)
// ----------------------------------------------------------------------------

#[test]
fn item_flag_with_mode_none_succeeds() {
    // Tests that mode="none" tasks work with --items flag
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_MODE_NONE_MULTIPLE_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("single_select")
        .arg("--items")
        .arg("item2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: item2"));
}

#[test]
fn item_flag_with_mode_multi_overrides_preselection() {
    // Tests that --items overrides preselected_items for mode="multi"
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_WITH_PRESELECTION);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("selective")
        .arg("--items")
        .arg("c")
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed: c"))
        .stderr(predicate::str::contains(
            "Warning: --items flag overrides preselected_items",
        ));
}

#[test]
fn mode_none_without_item_flag_multiple_items_errors() {
    // Tests that mode="none" with multiple items requires --items flag
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_MODE_NONE_MULTIPLE_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("single_select")
        .assert()
        .failure()
        .stderr(predicate::str::contains("mode='none'"))
        .stderr(predicate::str::contains("requires single-item selection"))
        .stderr(predicate::str::contains("Available items:"));
}

#[test]
fn mode_none_without_item_flag_single_item_succeeds() {
    // Tests that mode="none" with single item works without --items
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_MODE_NONE_SINGLE_ITEM);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("only_one")
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed: only_item"));
}

#[test]
fn item_not_found_shows_available_items() {
    // Tests that invalid --items shows helpful error with available items
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_MODE_NONE_MULTIPLE_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("single_select")
        .arg("--items")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Item 'nonexistent' not found"))
        .stderr(predicate::str::contains("Available items:"))
        .stderr(predicate::str::contains("item1"));
}

#[test]
fn item_flag_with_standalone_task_errors() {
    // Tests that --items cannot be used with standalone tasks (no item_sources)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", STANDALONE_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("standalone")
        .arg("--items")
        .arg("something")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no item sources"))
        .stderr(predicate::str::contains("standalone execute-only task"));
}

// ----------------------------------------------------------------------------
// Tag Handling Tests (4 tests)
// ----------------------------------------------------------------------------

#[test]
fn item_flag_exact_match_with_tag() {
    // Tests exact match when user provides full tagged format
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_MULTISOURCE_WITH_TAGS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("multi_source")
        .arg("--items")
        .arg("[pkg] git")
        .assert()
        .success()
        .stdout(predicate::str::contains("Packages: git"));
}

#[test]
fn item_flag_tag_stripped_match_unambiguous() {
    // Tests tag-stripped matching when unambiguous (only one source has the item)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_MULTISOURCE_WITH_TAGS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("multi_source")
        .arg("--items")
        .arg("iTerm2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cask: iTerm2"))
        .stderr(predicate::str::contains(
            "Info: Matched 'iTerm2' to tagged item",
        ));
}

#[test]
fn item_flag_tag_stripped_match_ambiguous_errors() {
    // Tests that ambiguous tag-stripped matches produce clear error
    const AMBIGUOUS_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        ambiguous = {
            description = "Test task",
            name = "Ambiguous Task",
            mode = "multi",
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"git"} end,
                    execute = function(items) return "S1: " .. table.concat(items, ","), 0 end,
                },
                src2 = {
                    tag = "s2",
                    items = function() return {"git"} end,
                    execute = function(items) return "S2: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", AMBIGUOUS_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("ambiguous")
        .arg("--items")
        .arg("git")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Ambiguous item: 'git' matches"))
        .stderr(predicate::str::contains("[s1] git"))
        .stderr(predicate::str::contains("[s2] git"))
        .stderr(predicate::str::contains("Use the full tagged format"));
}

#[test]
fn item_flag_single_source_no_tags() {
    // Tests that single-source tasks (no tags) work with --items
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 1 items"));
}

// ----------------------------------------------------------------------------
// Edge Case Tests (5 tests)
// ----------------------------------------------------------------------------

#[test]
fn item_flag_empty_string_errors() {
    // Tests that empty --items value produces error (Path 1: entire input empty)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--items cannot be empty or whitespace-only",
        )); // ✅ Specific!
}

#[test]
fn item_flag_whitespace_only_errors() {
    // Tests that whitespace-only --items value produces error (Path 1: entire input whitespace)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("   ")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--items cannot be empty or whitespace-only",
        )); // ✅ Specific!
}

#[test]
fn items_flag_comma_with_whitespace_only_item_in_middle() {
    // Tests that whitespace-only item in middle of comma list is handled gracefully
    // Input: "item1,   ,item3" - the middle has only whitespace
    // Implementation: trim() at line 206 converts "   " to "", then filter removes it
    // Result: Behaves same as "item1,,item3" - empty strings are filtered out
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1,   ,item3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 2 items"));
}

#[test]
fn item_flag_case_insensitive_match() {
    // Tests that case-insensitive matching works as fallback
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_WITH_CASE_VARIATIONS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("case_test")
        .arg("--items")
        .arg("packagemanager")
        .assert()
        .success()
        .stdout(predicate::str::contains("Matched: PackageManager"))
        .stderr(predicate::str::contains(
            "Info: Using case-insensitive match 'PackageManager' for 'packagemanager'",
        ));
}

#[test]
fn item_flag_with_unicode() {
    // Tests that Unicode item names work correctly
    const UNICODE_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        unicode = {
            description = "Test task",
            name = "Unicode Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"日本語", "中文", "العربية"} end,
                    execute = function(items) return "Selected: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", UNICODE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("unicode")
        .arg("--items")
        .arg("日本語")
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: 日本語"));
}

#[test]
fn item_flag_with_special_characters() {
    // Tests that special characters in item names are handled safely
    const SPECIAL_CHARS_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        special = {
            description = "Test task",
            name = "Special Chars Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item;with;semicolons", "item'with'quotes", "item\"with\"doublequotes"} end,
                    execute = function(items) return "Selected: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", SPECIAL_CHARS_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("special")
        .arg("--items")
        .arg("item;with;semicolons")
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: item;with;semicolons"));
}

// ============================================================================
// Test 41-44: Comma-separated Items Tests
// ============================================================================

#[test]
fn items_flag_comma_separated_multiple_items() {
    // Tests that --items accepts comma-separated list of items
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1,item2,item3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 3 items"));
}

#[test]
fn items_flag_comma_separated_with_spaces() {
    // Tests that items with spaces in their names work correctly
    const PLUGIN_WITH_SPACES: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        space_test = {
            description = "Test task",
            name = "Space Test Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"windows x", "test 1", "test 2"} end,
                    execute = function(items) return "Selected: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PLUGIN_WITH_SPACES);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("space_test")
        .arg("--items")
        .arg("windows x,test 1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: windows x,test 1"));
}

#[test]
fn items_flag_comma_separated_with_whitespace_trimming() {
    // Tests that whitespace around commas is trimmed correctly
    // Input: "item1, item2 , item3" (with extra spaces)
    // Expected: Items are trimmed to ["item1", "item2", "item3"] and all three execute

    const WHITESPACE_TEST_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        test_task = {
            description = "Test task",
            name = "Test Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "t",
                    items = function() return {"item1", "item2", "item3"} end,
                    execute = function(items)
                        return "Selected: " .. table.concat(items, ","), 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", WHITESPACE_TEST_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1, item2 , item3") // Extra spaces
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: item1,item2,item3")); // ✅ Verifies actual items!
}

#[test]
fn items_flag_comma_separated_with_invalid_item() {
    // Tests that a single invalid item in comma-separated list fails with clear error
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1,invalid,item3")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Item 'invalid' not found"));
}

// ============================================================================
// Test 45-51: Edge Case Tests for Comma-Separated --items Flag
// ============================================================================

#[test]
fn items_flag_comma_separated_with_duplicates() {
    // Tests behavior when the same item is specified multiple times in comma-separated list
    // Expected: The implementation does NOT deduplicate - execute receives duplicates
    // This tests actual behavior: duplicates are passed through to Lua execute function
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1,item2,item1")
        .assert()
        .success()
        // Behavior: Duplicates are passed through, execute receives 3 items (including duplicate)
        .stdout(predicate::str::contains("Executed 3 items"));
}

#[test]
fn items_flag_comma_separated_with_leading_trailing_commas() {
    // Tests that leading and trailing commas are handled correctly
    // Implementation filters empty strings after split, so leading/trailing commas create empty entries
    // Expected: Empty strings are filtered out, only valid items remain
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg(",item1,item2,")
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 2 items"));
}

#[test]
fn items_flag_comma_separated_with_multiple_consecutive_commas() {
    // Tests that multiple consecutive commas are handled correctly
    // Each comma creates a split point; consecutive commas create empty strings between them
    // Implementation filters empty strings, so only valid items remain
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1,,item2,,,item3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 3 items"));
}

#[test]
fn items_flag_case_sensitive_match_takes_precedence() {
    // Tests that exact case-sensitive matches are preferred over case-insensitive fallback
    // When multiple items exist with different cases (git, Git, GIT), exact match should win
    const CASE_SENSITIVE_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        case_precedence = {
            description = "Test task",
            name = "Case Precedence Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"git", "Git", "GIT"} end,
                    execute = function(items) return "Selected: " .. table.concat(items, ","), 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", CASE_SENSITIVE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("case_precedence")
        .arg("--items")
        .arg("git")
        .assert()
        .success()
        // Exact match "git" should be selected (not Git or GIT)
        .stdout(predicate::str::contains("Selected: git"))
        // Should not trigger case-insensitive warning since exact match was found
        .stderr(predicate::str::contains("case-insensitive").not());
}

#[test]
fn items_flag_comma_separated_fails_on_first_invalid() {
    // Tests that validation fails on the first invalid item in a comma-separated list
    // Even if some items are valid, a single invalid item causes the entire command to fail
    // This is correct behavior - fail fast to prevent partial execution
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1,invalid_item,item3")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Item 'invalid_item' not found"))
        .stderr(predicate::str::contains("Available items:"));
}

#[test]
fn items_flag_comma_separated_with_many_items() {
    // Tests that a large number of comma-separated items can be handled efficiently
    // Creates a plugin with 100 items, then selects 50 of them via --items flag
    // Validates that all 50 items are properly validated and executed
    const MANY_ITEMS_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        many = {
            description = "Test task",
            name = "Many Items Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function()
                        local items = {}
                        for i = 1, 100 do
                            table.insert(items, "item" .. i)
                        end
                        return items
                    end,
                    execute = function(items) return "Executed " .. #items .. " items", 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MANY_ITEMS_PLUGIN);

    // Build a comma-separated list of 50 items
    let items_list = (1..=50)
        .map(|i| format!("item{}", i))
        .collect::<Vec<_>>()
        .join(",");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("many")
        .arg("--items")
        .arg(&items_list)
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 50 items"));
}

#[test]
fn items_flag_only_commas_errors() {
    // Tests that --items with only commas (no actual item names) produces clear error
    // After splitting and filtering empty strings, the result is empty
    // Should fail with "cannot be empty" error before any plugin execution
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg(",,,,")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot be empty or whitespace-only",
        ));
}

// ============================================================================
// Test 52-55: Flag Conflict Tests
// ============================================================================

#[test]
fn items_flag_conflicts_with_produce_items() {
    // Tests that --items and --produce-items cannot be used together
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1")
        .arg("--produce-items")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn items_flag_conflicts_with_produce_preselected_items() {
    // Tests that --items and --produce-preselected-items cannot be used together
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1")
        .arg("--produce-preselected-items")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn items_flag_conflicts_with_produce_preselection_matches() {
    // Tests that --items and --produce-preselection-matches cannot be used together
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1")
        .arg("--produce-preselection-matches")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn items_flag_conflicts_with_preview() {
    // Tests that --items and --preview cannot be used together
    // Note: --preview requires a value
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test-plugin", SIMPLE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test-plugin")
        .arg("--task")
        .arg("test_task")
        .arg("--items")
        .arg("item1")
        .arg("--preview")
        .arg("item2") // preview needs a value
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

// ============================================================================
// Test 56-59: TDD Tests for Comma-in-Item-Names Support (EXPECTED TO FAIL)
// ============================================================================
// These tests specify the expected behavior for item names containing commas.
// They currently FAIL because comma escaping/quoting is not yet implemented.
// When implementing comma support, these tests should guide the implementation.

#[test]
fn items_flag_selects_item_with_comma_in_name() {
    // TDD Test: This specifies EXPECTED behavior, not current behavior
    // Expected: Item names can contain commas and be selectable
    // Current: FAILS because comma-splitting happens before validation

    const COMMA_ITEM_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        comma_test = {
            description = "Test task",
            name = "Comma Item Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"normal-item", "backup-2024,full", "test-item"} end,
                    execute = function(items) return "Selected: " .. table.concat(items, "|"), 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", COMMA_ITEM_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("comma_test")
        .arg("--items")
        .arg("backup-2024\\,full") // Escape the comma with backslash
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: backup-2024,full"));
}

#[test]
fn items_flag_comma_separated_list_with_item_containing_comma() {
    // TDD Test: Specifies expected behavior for mixed list
    // Expected: Can specify multiple items where one contains a comma
    // Plugin has 3 items, one contains a comma
    // User wants to select all 3
    // Solution: Use backslash escaping (\,) for commas in item names

    const MIXED_COMMA_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        mixed = {
            description = "Test task",
            name = "Mixed Comma Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item1", "backup-2024,full", "item3"} end,
                    execute = function(items) return "Executed " .. #items .. " items: " .. table.concat(items, "|"), 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MIXED_COMMA_PLUGIN);

    // Expected: Should parse as 3 items: ["item1", "backup-2024,full", "item3"]
    // With backslash escaping: item1,backup-2024\,full,item3
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("mixed")
        .arg("--items")
        .arg("item1,backup-2024\\,full,item3") // Escape the comma in second item
        .assert()
        .success()
        .stdout(predicate::str::contains("Executed 3 items"))
        .stdout(predicate::str::contains("item1|backup-2024,full|item3"));
}

#[test]
fn items_flag_selects_item_with_multiple_commas() {
    // TDD Test: Items can have multiple commas
    // Expected: Single item with 3 commas should be selectable
    // Solution: Use backslash escaping (\,) for all commas in item name

    const MULTI_COMMA_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        multi = {
            description = "Test task",
            name = "Multi Comma Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"normal", "data,2024,backup,full", "other"} end,
                    execute = function(items) return "Selected: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MULTI_COMMA_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("multi")
        .arg("--items")
        .arg("data\\,2024\\,backup\\,full") // Escape all commas in the item name
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: data,2024,backup,full"));
}

#[test]
fn items_flag_item_with_comma_and_other_special_chars() {
    // TDD Test: Items can have commas along with other special characters
    // Expected: Complex item names should work (colons, spaces, commas, periods)
    // Solution: Use backslash escaping (\,) for commas in complex item names

    const COMPLEX_ITEM_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        complex = {
            description = "Test task",
            name = "Complex Item Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"simple", "file: backup,2024.tar.gz", "another"} end,
                    execute = function(items) return "Got: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", COMPLEX_ITEM_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("complex")
        .arg("--items")
        .arg("file: backup\\,2024.tar.gz") // Escape the comma
        .assert()
        .success()
        .stdout(predicate::str::contains("Got: file: backup,2024.tar.gz"));
}

// ============================================================================
// Test 60-64: Escape Mechanism Tests
// ============================================================================

#[test]
fn items_flag_escaped_comma_preserved_in_item_name() {
    // Tests that \, in input becomes , in the actual item name
    const ESCAPE_TEST_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        escape = {
            description = "Test task",
            name = "Escape Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item,with,commas"} end,
                    execute = function(items) return "Selected: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ESCAPE_TEST_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("escape")
        .arg("--items")
        .arg("item\\,with\\,commas")
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: item,with,commas"));
}

#[test]
fn items_flag_escaped_backslash_preserved() {
    // Tests that \\ in input becomes \ in the actual item name
    const BACKSLASH_TEST_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        backslash = {
            description = "Test task",
            name = "Backslash Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"path\\to\\file"} end,
                    execute = function(items) return "Path: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", BACKSLASH_TEST_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("backslash")
        .arg("--items")
        .arg("path\\\\to\\\\file")
        .assert()
        .success()
        .stdout(predicate::str::contains("Path: path\\to\\file"));
}

#[test]
fn items_flag_mixed_escaped_and_unescaped_commas() {
    // Tests mixed list with both separating commas and escaped commas
    const MIXED_ESCAPE_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        mixed = {
            description = "Test task",
            name = "Mixed Escape Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"simple", "item,with,comma", "another"} end,
                    execute = function(items) return "Count: " .. #items .. " Items: " .. table.concat(items, "|"), 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MIXED_ESCAPE_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("mixed")
        .arg("--items")
        .arg("simple,item\\,with\\,comma,another")
        .assert()
        .success()
        .stdout(predicate::str::contains("Count: 3"))
        .stdout(predicate::str::contains(
            "Items: simple|item,with,comma|another",
        ));
}

#[test]
fn items_flag_unescaped_backslash_not_followed_by_comma() {
    // Tests that backslash not followed by comma or backslash is preserved
    const UNESCAPED_BACKSLASH_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        unescaped = {
            description = "Test task",
            name = "Unescaped Backslash Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item\\n", "other"} end,
                    execute = function(items) return "Got: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", UNESCAPED_BACKSLASH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("unescaped")
        .arg("--items")
        .arg("item\\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Got: item\\n"));
}

#[test]
fn items_flag_trailing_backslash() {
    // Tests that trailing backslash is preserved
    const TRAILING_BACKSLASH_PLUGIN: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        trailing = {
            description = "Test task",
            name = "Trailing Backslash Test",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item\\"} end,
                    execute = function(items) return "Selected: " .. items[1], 0 end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", TRAILING_BACKSLASH_PLUGIN);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("trailing")
        .arg("--items")
        .arg("item\\")
        .assert()
        .success()
        .stdout(predicate::str::contains("Selected: item\\"));
}

// ============================================================================
// Test 65-82: --preview and --produce-* Flags
// ============================================================================

// Mock plugin templates for preview and produce flag tests
const PLUGIN_WITH_PREVIEW: &str = r#"
return {
    metadata = {name = "preview-test", version = "1.0.0", icon = "P", platforms = {"macos"}},
    tasks = {
        with_task_preview = {
            description = "Test task",
            name = "Task with Preview",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "t",
                    items = function() return {"safari", "chrome", "firefox"} end,
                    execute = function(items) return "Executed", 0 end,
                },
            },
            preview = function(item) return "Task preview for: " .. item end,
        },
        with_item_source_preview = {
            description = "Test task",
            name = "Item Source Preview",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"doc1", "doc2", "doc3"} end,
                    preview = function(item) return "Item source preview: " .. item end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
        no_preview = {
            description = "Test task",
            name = "No Preview",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "n",
                    items = function() return {"item1", "item2"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const MULTISOURCE_PLUGIN_WITH_PREVIEW: &str = r#"
return {
    metadata = {name = "multi-preview", version = "1.0.0", icon = "M", platforms = {"macos"}},
    tasks = {
        browsers = {
            description = "Test task",
            name = "Browsers Multi-Source",
            mode = "multi",
            item_sources = {
                windows = {
                    tag = "w",
                    items = function() return {"Safari", "Chrome"} end,
                    preview = function(item) return "Window: " .. item end,
                    execute = function(items) return "OK", 0 end,
                },
                apps = {
                    tag = "a",
                    items = function() return {"Safari", "Firefox"} end,
                    preview = function(item) return "App: " .. item end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_PRESELECTION_TESTS: &str = r#"
return {
    metadata = {name = "preselect-test", version = "1.0.0", icon = "S", platforms = {"macos"}},
    tasks = {
        selective = {
            description = "Test task",
            name = "Selective Task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"alpha", "beta", "gamma", "delta", "epsilon"} end,
                    preselected_items = function() return {"beta", "delta"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
        all_selected = {
            description = "Test task",
            name = "All Selected",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "a",
                    items = function() return {"one", "two", "three"} end,
                    preselected_items = function() return {"one", "two", "three"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
        none_selected = {
            description = "Test task",
            name = "None Selected",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "n",
                    items = function() return {"item1", "item2", "item3"} end,
                    preselected_items = function() return {} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
        partial_match = {
            description = "Test task",
            name = "Partial Match",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "p",
                    items = function() return {"valid1", "valid2", "valid3"} end,
                    preselected_items = function() return {"valid1", "invalid_item", "valid3"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

// --preview tests

#[test]
fn preview_basic_task_level() {
    // Tests basic --preview with task-level preview function
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preview-test", PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preview-test")
        .arg("--task")
        .arg("with_task_preview")
        .arg("--preview")
        .arg("safari")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task preview for: safari"));
}

#[test]
fn preview_item_source_level() {
    // Tests --preview with item_source-level preview function (takes precedence)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preview-test", PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preview-test")
        .arg("--task")
        .arg("with_item_source_preview")
        .arg("--preview")
        .arg("doc2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Item source preview: doc2"));
}

#[test]
fn preview_no_preview_function() {
    // Tests --preview when no preview function exists (should output "No preview")
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preview-test", PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preview-test")
        .arg("--task")
        .arg("no_preview")
        .arg("--preview")
        .arg("item1")
        .assert()
        .success()
        .stdout(predicate::str::contains("No preview"));
}

#[test]
fn preview_nonexistent_item() {
    // Tests --preview with an item that doesn't exist
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preview-test", PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preview-test")
        .arg("--task")
        .arg("with_task_preview")
        .arg("--preview")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn preview_multisource_with_tag() {
    // Tests --preview with multi-source task using full tagged item
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multi-preview", MULTISOURCE_PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("multi-preview")
        .arg("--task")
        .arg("browsers")
        .arg("--preview")
        .arg("[w] Safari")
        .assert()
        .success()
        .stdout(predicate::str::contains("Window: Safari"));
}

#[test]
fn preview_multisource_tag_stripped_unambiguous() {
    // Tests --preview with tag-stripped item that matches only one source
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multi-preview", MULTISOURCE_PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("multi-preview")
        .arg("--task")
        .arg("browsers")
        .arg("--preview")
        .arg("Firefox")
        .assert()
        .success()
        .stdout(predicate::str::contains("App: Firefox"))
        .stderr(predicate::str::contains(
            "Matched 'Firefox' to tagged item '[a] Firefox'",
        ));
}

#[test]
fn preview_multisource_ambiguous() {
    // Tests --preview with ambiguous item (exists in multiple sources)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multi-preview", MULTISOURCE_PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("multi-preview")
        .arg("--task")
        .arg("browsers")
        .arg("--preview")
        .arg("Safari")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Ambiguous item"))
        .stderr(predicate::str::contains("matches 2 items"));
}

#[test]
fn preview_case_insensitive_fallback() {
    // Tests --preview with case-insensitive matching
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preview-test", PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preview-test")
        .arg("--task")
        .arg("with_task_preview")
        .arg("--preview")
        .arg("SAFARI")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task preview for: safari"))
        .stderr(predicate::str::contains("case-insensitive match"));
}

#[test]
fn preview_standalone_task_error() {
    // Tests that --preview errors appropriately on tasks without item_sources
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", STANDALONE_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("standalone")
        .arg("--preview")
        .arg("anything")
        .assert()
        .failure()
        .stderr(predicate::str::contains("has no item sources"));
}

#[test]
fn preview_empty_string() {
    // Tests --preview with empty string
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preview-test", PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preview-test")
        .arg("--task")
        .arg("with_task_preview")
        .arg("--preview")
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot contain empty"));
}

// --produce-items tests

#[test]
fn produce_items_basic() {
    // Tests --produce-items outputs all items
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preview-test", PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preview-test")
        .arg("--task")
        .arg("with_task_preview")
        .arg("--produce-items")
        .assert()
        .success()
        .stdout(predicate::str::contains("safari"))
        .stdout(predicate::str::contains("chrome"))
        .stdout(predicate::str::contains("firefox"));
}

#[test]
fn produce_items_multisource_shows_tags() {
    // Tests --produce-items with multi-source task shows tags
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multi-preview", MULTISOURCE_PLUGIN_WITH_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("multi-preview")
        .arg("--task")
        .arg("browsers")
        .arg("--produce-items")
        .assert()
        .success()
        .stdout(predicate::str::contains("[w] Safari"))
        .stdout(predicate::str::contains("[w] Chrome"))
        .stdout(predicate::str::contains("[a] Safari"))
        .stdout(predicate::str::contains("[a] Firefox"));
}

#[test]
fn produce_items_standalone_task_error() {
    // Tests that --produce-items errors on tasks without item_sources
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", STANDALONE_TASK);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("standalone")
        .arg("--produce-items")
        .assert()
        .failure()
        .stderr(predicate::str::contains("has no item sources"));
}

// --produce-preselected-items tests

#[test]
fn produce_preselected_items_basic() {
    // Tests --produce-preselected-items outputs only preselected items
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preselect-test", PLUGIN_WITH_PRESELECTION_TESTS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preselect-test")
        .arg("--task")
        .arg("selective")
        .arg("--produce-preselected-items")
        .assert()
        .success()
        .stdout(predicate::str::contains("beta"))
        .stdout(predicate::str::contains("delta"))
        .stdout(predicate::str::contains("alpha").not())
        .stdout(predicate::str::contains("gamma").not());
}

#[test]
fn produce_preselected_items_empty() {
    // Tests --produce-preselected-items with no preselection (empty output)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preselect-test", PLUGIN_WITH_PRESELECTION_TESTS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preselect-test")
        .arg("--task")
        .arg("none_selected")
        .arg("--produce-preselected-items")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// --produce-preselection-matches tests

#[test]
fn produce_preselection_matches_partial() {
    // Tests --produce-preselection-matches with partial overlap
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preselect-test", PLUGIN_WITH_PRESELECTION_TESTS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preselect-test")
        .arg("--task")
        .arg("partial_match")
        .arg("--produce-preselection-matches")
        .assert()
        .success()
        .stdout(predicate::str::contains("valid1"))
        .stdout(predicate::str::contains("valid3"))
        .stdout(predicate::str::contains("valid2").not())
        .stdout(predicate::str::contains("invalid_item").not());
}

#[test]
fn produce_preselection_matches_all() {
    // Tests --produce-preselection-matches when all items are preselected
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preselect-test", PLUGIN_WITH_PRESELECTION_TESTS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preselect-test")
        .arg("--task")
        .arg("all_selected")
        .arg("--produce-preselection-matches")
        .assert()
        .success()
        .stdout(predicate::str::contains("one"))
        .stdout(predicate::str::contains("two"))
        .stdout(predicate::str::contains("three"));
}

#[test]
fn produce_preselection_matches_none() {
    // Tests --produce-preselection-matches with no overlap (empty output)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("preselect-test", PLUGIN_WITH_PRESELECTION_TESTS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("preselect-test")
        .arg("--task")
        .arg("none_selected")
        .arg("--produce-preselection-matches")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ============================================================================
// Test 83-100: Error Handling and Edge Cases for Execute Flags
// ============================================================================

// Mock plugins for error and edge case testing
const PLUGIN_WITH_ERROR_PREVIEW: &str = r#"
return {
    metadata = {name = "error-preview", version = "1.0.0", icon = "E", platforms = {"macos"}},
    tasks = {
        failing_preview = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item1", "item2"} end,
                    preview = function(item) error("Preview failed") end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_ERROR_ITEMS: &str = r#"
return {
    metadata = {name = "error-items", version = "1.0.0", icon = "E", platforms = {"macos"}},
    tasks = {
        failing_items = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() error("Items loading failed") end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_ERROR_PRESELECTED: &str = r#"
return {
    metadata = {name = "error-preselect", version = "1.0.0", icon = "E", platforms = {"macos"}},
    tasks = {
        failing_preselect = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item1", "item2"} end,
                    preselected_items = function() error("Preselection failed") end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_NUMBER_PREVIEW: &str = r#"
return {
    metadata = {name = "number-preview", version = "1.0.0", icon = "N", platforms = {"macos"}},
    tasks = {
        number_type = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item1", "item2"} end,
                    preview = function(item) return 42 end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_EMPTY_ITEMS: &str = r#"
return {
    metadata = {name = "empty", version = "1.0.0", icon = "E", platforms = {"macos"}},
    tasks = {
        no_items = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_SINGLE_ITEM: &str = r#"
return {
    metadata = {name = "single", version = "1.0.0", icon = "S", platforms = {"macos"}},
    tasks = {
        one_item = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"only-one"} end,
                    preselected_items = function() return {"only-one"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_MULTISOURCE_PRESELECTION: &str = r#"
return {
    metadata = {name = "multi-preselect", version = "1.0.0", icon = "M", platforms = {"macos"}},
    tasks = {
        tagged_preselect = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                files = {
                    tag = "f",
                    items = function() return {"doc1.txt", "doc2.txt", "doc3.txt"} end,
                    preselected_items = function() return {"doc1.txt", "doc3.txt"} end,
                    execute = function(items) return "OK", 0 end,
                },
                folders = {
                    tag = "d",
                    items = function() return {"Documents", "Downloads"} end,
                    preselected_items = function() return {"Documents"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

const PLUGIN_WITH_SPECIAL_ITEMS: &str = r#"
return {
    metadata = {name = "special", version = "1.0.0", icon = "S", platforms = {"macos"}},
    tasks = {
        with_newlines = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"line1\nline2", "normal", "line3\nline4\nline5"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
        with_unicode = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"🚀 Rocket", "日本語", "Regular"} end,
                    preview = function(item) return "Preview: " .. item end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
        with_special_chars = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"item!@#$%", "item<>?", "item|&"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
        order_test = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"zebra", "middle", "alpha"} end,
                    execute = function(items) return "OK", 0 end,
                },
            },
        },
    },
}
"#;

// Lua error handling tests

#[test]
fn preview_lua_error_in_preview_function() {
    // Tests that Lua errors in preview function are handled gracefully
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("error-preview", PLUGIN_WITH_ERROR_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("error-preview")
        .arg("--task")
        .arg("failing_preview")
        .arg("--preview")
        .arg("item1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn preview_lua_error_in_items_function() {
    // Tests that errors in items() function propagate correctly
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("error-items", PLUGIN_WITH_ERROR_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("error-items")
        .arg("--task")
        .arg("failing_items")
        .arg("--preview")
        .arg("anything")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to fetch items"));
}

#[test]
fn produce_items_lua_error() {
    // Tests that --produce-items handles items() errors
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("error-items", PLUGIN_WITH_ERROR_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("error-items")
        .arg("--task")
        .arg("failing_items")
        .arg("--produce-items")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to fetch items"));
}

#[test]
fn produce_preselected_items_lua_error() {
    // Tests that --produce-preselected-items handles preselected_items() errors
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("error-preselect", PLUGIN_WITH_ERROR_PRESELECTED);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("error-preselect")
        .arg("--task")
        .arg("failing_preselect")
        .arg("--produce-preselected-items")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to fetch items"));
}

#[test]
fn preview_returns_number_type() {
    // Tests that preview returning a number is handled (Lua coerces to string)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("number-preview", PLUGIN_WITH_NUMBER_PREVIEW);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("number-preview")
        .arg("--task")
        .arg("number_type")
        .arg("--preview")
        .arg("item1")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// Empty/boundary tests

#[test]
fn produce_items_with_empty_array() {
    // Tests --produce-items with empty items list (valid edge case - succeeds with empty output)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("empty", PLUGIN_WITH_EMPTY_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("empty")
        .arg("--task")
        .arg("no_items")
        .arg("--produce-items")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn produce_items_with_single_item() {
    // Tests --produce-items with exactly one item (boundary condition)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("single", PLUGIN_WITH_SINGLE_ITEM);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("single")
        .arg("--task")
        .arg("one_item")
        .arg("--produce-items")
        .assert()
        .success()
        .stdout(predicate::eq("only-one\n"));
}

#[test]
fn preview_with_empty_items_array() {
    // Tests that --preview fails gracefully when items list is empty
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("empty", PLUGIN_WITH_EMPTY_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("empty")
        .arg("--task")
        .arg("no_items")
        .arg("--preview")
        .arg("anything")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn produce_preselection_matches_single_item() {
    // Tests --produce-preselection-matches with single item boundary
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("single", PLUGIN_WITH_SINGLE_ITEM);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("single")
        .arg("--task")
        .arg("one_item")
        .arg("--produce-preselection-matches")
        .assert()
        .success()
        .stdout(predicate::eq("only-one\n"));
}

// Multi-source edge case tests

#[test]
fn produce_preselected_items_multisource_with_tags() {
    // Tests that --produce-preselected-items includes tags in multi-source tasks
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multi-preselect", PLUGIN_MULTISOURCE_PRESELECTION);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("multi-preselect")
        .arg("--task")
        .arg("tagged_preselect")
        .arg("--produce-preselected-items")
        .assert()
        .success()
        .stdout(predicate::str::contains("[f] doc1.txt"))
        .stdout(predicate::str::contains("[f] doc3.txt"))
        .stdout(predicate::str::contains("[d] Documents"));
}

#[test]
fn produce_preselection_matches_multisource() {
    // Tests --produce-preselection-matches preserves tags in multi-source tasks
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multi-preselect", PLUGIN_MULTISOURCE_PRESELECTION);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("multi-preselect")
        .arg("--task")
        .arg("tagged_preselect")
        .arg("--produce-preselection-matches")
        .assert()
        .success()
        .stdout(predicate::str::contains("[f] doc1.txt"))
        .stdout(predicate::str::contains("[f] doc3.txt"))
        .stdout(predicate::str::contains("[d] Documents"))
        .stdout(predicate::str::contains("[f] doc2.txt").not())
        .stdout(predicate::str::contains("[d] Downloads").not());
}

#[test]
fn preview_multisource_all_sources_have_preview() {
    // Tests preview with multi-source where each source has its own preview
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("multi-preselect", PLUGIN_MULTISOURCE_PRESELECTION);

    // Note: This plugin doesn't have preview functions, so should fall back to "No preview"
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("multi-preselect")
        .arg("--task")
        .arg("tagged_preselect")
        .arg("--preview")
        .arg("[f] doc1.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("No preview"));
}

// Output format validation tests

#[test]
fn produce_items_order_preservation() {
    // Tests that --produce-items preserves the exact order from items()
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("special", PLUGIN_WITH_SPECIAL_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("special")
        .arg("--task")
        .arg("order_test")
        .arg("--produce-items")
        .assert()
        .success()
        .stdout(predicate::eq("zebra\nmiddle\nalpha\n"));
}

#[test]
fn produce_items_with_newlines_in_items() {
    // Tests that items containing newlines are preserved
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("special", PLUGIN_WITH_SPECIAL_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("special")
        .arg("--task")
        .arg("with_newlines")
        .arg("--produce-items")
        .assert()
        .success()
        .stdout(predicate::str::contains("line1\nline2"))
        .stdout(predicate::str::contains("normal"))
        .stdout(predicate::str::contains("line3\nline4\nline5"));
}

#[test]
fn produce_items_exact_format() {
    // Tests exact output format: one item per line, newline-terminated
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("single", PLUGIN_WITH_SINGLE_ITEM);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("single")
        .arg("--task")
        .arg("one_item")
        .arg("--produce-items")
        .output()
        .expect("Failed to execute command");

    assert_eq!(String::from_utf8_lossy(&output.stdout), "only-one\n");
}

// Special character tests

#[test]
fn preview_with_unicode_emoji_item() {
    // Tests that unicode emoji items work correctly with preview
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("special", PLUGIN_WITH_SPECIAL_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("special")
        .arg("--task")
        .arg("with_unicode")
        .arg("--preview")
        .arg("🚀 Rocket")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview: 🚀 Rocket"));
}

#[test]
fn produce_items_with_special_chars() {
    // Tests that special characters are preserved in output
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("special", PLUGIN_WITH_SPECIAL_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("special")
        .arg("--task")
        .arg("with_special_chars")
        .arg("--produce-items")
        .assert()
        .success()
        .stdout(predicate::str::contains("item!@#$%"))
        .stdout(predicate::str::contains("item<>?"))
        .stdout(predicate::str::contains("item|&"));
}
