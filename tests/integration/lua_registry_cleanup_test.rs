//! Integration tests for Lua registry cleanup and state hygiene
//!
//! This test suite verifies that `__syntropy_current_plugin__` registry value
//! is properly cleaned up after each Lua function call, preventing context
//! leakage between plugin executions.
//!
//! **Behavior Contract**: Registry cleanup must happen after EVERY execution path,
//! including task abort. This is enforced via RAII guards that clean up on drop.
//!
//! **Required Behavior**: Registry MUST be cleaned after EVERY execution path:
//! - Normal completion
//! - Error/exception
//! - Abort/cancellation
//! - Hook failures (pre_run, post_run)
//!
//! **Impact**: Stale context causes `expand_path()` to resolve relative paths
//! using wrong plugin directory, leading to file access errors or incorrect
//! file operations.

use assert_cmd::Command;
use std::fs;
use std::path::Path;

use crate::common::TestFixture;

const MINIMAL_CONFIG: &str = r#"
default_plugin_icon = "⚒"

[keybindings]
back = "<esc>"
select_previous = "<up>"
select_next = "<down>"
confirm = "<enter>"
"#;

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates a plugin that writes expand_path() result to a marker file.
/// This acts as a "probe" to verify the registry state without directly inspecting it.
fn create_probe_plugin(
    fixture: &TestFixture,
    plugin_name: &str,
    task_name: &str,
    marker_path: &str,
    relative_path: &str,
) {
    let plugin_content = format!(
        r#"
return {{
    metadata = {{name = "{}", version = "1.0.0", icon = "P", platforms = {{"macos", "linux"}}}},
    tasks = {{
        {} = {{
            description = "Probe task for registry state",
            name = "Registry Probe",
            execute = function()
                local path = syntropy.expand_path("{}")
                local f = io.open("{}", "w")
                if f then
                    f:write(path)
                    f:close()
                end
                return "probe completed", 0
            end
        }}
    }}
}}
"#,
        plugin_name, task_name, relative_path, marker_path
    );

    fixture.create_plugin(plugin_name, &plugin_content);
}

/// Creates a plugin with items() that throws an error
fn create_items_error_plugin(fixture: &TestFixture, plugin_name: &str, task_name: &str) {
    let plugin_content = format!(
        r#"
return {{
    metadata = {{name = "{}", version = "1.0.0", icon = "E", platforms = {{"macos", "linux"}}}},
    tasks = {{
        {} = {{
            description = "Task with failing items()",
            name = "Items Error Test",
            items = function()
                error("items() failed intentionally")
            end,
            execute = function()
                return "should not run", 0
            end
        }}
    }}
}}
"#,
        plugin_name, task_name
    );

    fixture.create_plugin(plugin_name, &plugin_content);
}

/// Creates a plugin with execute() that throws an error
fn create_execute_error_plugin(fixture: &TestFixture, plugin_name: &str, task_name: &str) {
    let plugin_content = format!(
        r#"
return {{
    metadata = {{name = "{}", version = "1.0.0", icon = "E", platforms = {{"macos", "linux"}}}},
    tasks = {{
        {} = {{
            description = "Task with failing execute()",
            name = "Execute Error Test",
            mode = "multi",
            item_sources = {{
                source = {{
                    tag = "s",
                    items = function() return {{"item1"}} end,
                    preselected_items = function() return {{"item1"}} end,
                    execute = function(items)
                        error("execute() failed intentionally")
                    end
                }}
            }}
        }}
    }}
}}
"#,
        plugin_name, task_name
    );

    fixture.create_plugin(plugin_name, &plugin_content);
}

/// Creates a plugin with preview() that throws an error
fn create_preview_error_plugin(fixture: &TestFixture, plugin_name: &str, task_name: &str) {
    let plugin_content = format!(
        r#"
return {{
    metadata = {{name = "{}", version = "1.0.0", icon = "E", platforms = {{"macos", "linux"}}}},
    tasks = {{
        {} = {{
            description = "Task with failing preview()",
            name = "Preview Error Test",
            items = function()
                return {{"item1"}}
            end,
            preview = function(item)
                error("preview() failed intentionally")
            end,
            execute = function()
                return "done", 0
            end
        }}
    }}
}}
"#,
        plugin_name, task_name
    );

    fixture.create_plugin(plugin_name, &plugin_content);
}

/// Reads marker file and verifies it contains the expected plugin directory name
fn assert_marker_contains_plugin_dir(marker_path: &Path, expected_plugin: &str) {
    let content = fs::read_to_string(marker_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read marker file at {}: {}",
            marker_path.display(),
            e
        )
    });

    assert!(
        content.contains(expected_plugin),
        "Expected marker to contain plugin '{}', but got: '{}'",
        expected_plugin,
        content
    );
}

/// Runs syntropy execute command and returns the output
fn run_execute(fixture: &TestFixture, plugin: &str, task: &str) -> std::process::Output {
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg(plugin)
        .arg("--task")
        .arg(task)
        .output()
        .expect("Failed to execute syntropy command")
}

// ============================================================================
// Category 1: Normal Error Cleanup (Baseline Tests)
// ============================================================================

#[test]
fn test_registry_cleanup_after_items_error() {
    // Verify that when items() throws an error, the registry is properly cleaned
    // so the next plugin execution sees correct context
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: has items() that fails
    create_items_error_plugin(&fixture, "plugin_a", "failing_items");

    // Plugin B: probe that uses expand_path()
    let marker_path = fixture.temp_dir.path().join("probe_marker.txt");
    let marker_path_str = marker_path.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "probe", marker_path_str, "./data.txt");

    // Execute Plugin A (items() fails - may or may not cause CLI failure)
    let _output_a = run_execute(&fixture, "plugin_a", "failing_items");
    // Note: items() errors may be handled gracefully at CLI level

    // Execute Plugin B (should succeed with correct context)
    let output_b = run_execute(&fixture, "plugin_b", "probe");
    assert!(
        output_b.status.success(),
        "Plugin B should succeed. stderr: {}",
        String::from_utf8_lossy(&output_b.stderr)
    );

    // Verify Plugin B saw its own directory, not Plugin A's
    assert_marker_contains_plugin_dir(&marker_path, "plugin_b");
}

#[test]
fn test_registry_cleanup_after_execute_error() {
    // Verify that when execute() throws an error, the registry is properly cleaned
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: has execute() that fails
    create_execute_error_plugin(&fixture, "plugin_a", "failing_execute");

    // Plugin B: probe
    let marker_path = fixture.temp_dir.path().join("probe_marker.txt");
    let marker_path_str = marker_path.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "probe", marker_path_str, "./data.txt");

    // Execute Plugin A (should fail)
    let output_a = run_execute(&fixture, "plugin_a", "failing_execute");
    assert!(!output_a.status.success(), "Plugin A should fail");

    // Execute Plugin B (should succeed with correct context)
    let output_b = run_execute(&fixture, "plugin_b", "probe");
    assert!(
        output_b.status.success(),
        "Plugin B should succeed. stderr: {}",
        String::from_utf8_lossy(&output_b.stderr)
    );

    // Verify Plugin B saw its own directory
    assert_marker_contains_plugin_dir(&marker_path, "plugin_b");
}

#[test]
fn test_registry_cleanup_after_preview_error() {
    // Verify that when preview() throws an error, the registry is properly cleaned
    // Note: This test runs execute after preview error, as preview errors don't block execution
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: has preview() that fails (but execute still works)
    create_preview_error_plugin(&fixture, "plugin_a", "failing_preview");

    // Plugin B: probe
    let marker_path = fixture.temp_dir.path().join("probe_marker.txt");
    let marker_path_str = marker_path.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "probe", marker_path_str, "./data.txt");

    // Execute Plugin A (preview fails but we test execute path)
    let _output_a = run_execute(&fixture, "plugin_a", "failing_preview");
    // Note: Plugin A may succeed or fail depending on when preview is called
    // The important part is that Plugin B sees clean state

    // Execute Plugin B (should succeed with correct context)
    let output_b = run_execute(&fixture, "plugin_b", "probe");
    assert!(
        output_b.status.success(),
        "Plugin B should succeed. stderr: {}",
        String::from_utf8_lossy(&output_b.stderr)
    );

    // Verify Plugin B saw its own directory
    assert_marker_contains_plugin_dir(&marker_path, "plugin_b");
}

// ============================================================================
// Category 2: Sequential Execution Isolation
// ============================================================================

#[test]
fn test_registry_isolation_sequential_tasks_different_plugins() {
    // Verify that executing tasks from different plugins sequentially
    // maintains proper context isolation
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: writes expand_path result
    let marker_a = fixture.temp_dir.path().join("plugin_a_marker.txt");
    let marker_a_str = marker_a.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_a", "task", marker_a_str, "./data.txt");

    // Plugin B: writes expand_path result
    let marker_b = fixture.temp_dir.path().join("plugin_b_marker.txt");
    let marker_b_str = marker_b.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "task", marker_b_str, "./data.txt");

    // Execute both tasks
    let output_a = run_execute(&fixture, "plugin_a", "task");
    assert!(output_a.status.success(), "Plugin A should succeed");

    let output_b = run_execute(&fixture, "plugin_b", "task");
    assert!(output_b.status.success(), "Plugin B should succeed");

    // Verify each saw its own directory
    assert_marker_contains_plugin_dir(&marker_a, "plugin_a");
    assert_marker_contains_plugin_dir(&marker_b, "plugin_b");
}

#[test]
fn test_registry_isolation_with_intermediate_failure() {
    // Verify that a failing task in the middle doesn't affect subsequent tasks
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: succeeds
    let marker_a = fixture.temp_dir.path().join("plugin_a_marker.txt");
    let marker_a_str = marker_a.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_a", "task", marker_a_str, "./data.txt");

    // Plugin B: fails
    create_execute_error_plugin(&fixture, "plugin_b", "task");

    // Plugin C: succeeds
    let marker_c = fixture.temp_dir.path().join("plugin_c_marker.txt");
    let marker_c_str = marker_c.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_c", "task", marker_c_str, "./data.txt");

    // Execute all three
    let output_a = run_execute(&fixture, "plugin_a", "task");
    assert!(output_a.status.success(), "Plugin A should succeed");

    let output_b = run_execute(&fixture, "plugin_b", "task");
    assert!(!output_b.status.success(), "Plugin B should fail");

    let output_c = run_execute(&fixture, "plugin_c", "task");
    assert!(output_c.status.success(), "Plugin C should succeed");

    // Verify Plugin A and C saw their own directories
    assert_marker_contains_plugin_dir(&marker_a, "plugin_a");
    assert_marker_contains_plugin_dir(&marker_c, "plugin_c");
}

// ============================================================================
// Category 3: expand_path() Isolation Tests
// ============================================================================

#[test]
fn test_expand_path_sees_correct_plugin_after_error() {
    // Verify expand_path() behavior when called after another plugin errors
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: calls expand_path then errors
    let plugin_a_content = r#"
return {
    metadata = {name = "plugin_a", version = "1.0.0", icon = "A", platforms = {"macos", "linux"}},
    tasks = {
        fail_after_expand = {
            description = "Calls expand_path then fails",
            name = "Fail After Expand",
            items = function()
                local path = syntropy.expand_path("./data.txt")
                error("intentional failure after expand_path")
            end,
            execute = function()
                return "done", 0
            end
        }
    }
}
"#;
    fixture.create_plugin("plugin_a", plugin_a_content);

    // Plugin B: calls expand_path and returns it
    let marker_b = fixture.temp_dir.path().join("plugin_b_marker.txt");
    let marker_b_str = marker_b.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "probe", marker_b_str, "./data.txt");

    // Execute Plugin A (fails after expand_path - may or may not cause CLI failure)
    let _output_a = run_execute(&fixture, "plugin_a", "fail_after_expand");
    // Note: items() errors may be handled gracefully at CLI level

    // Execute Plugin B
    let output_b = run_execute(&fixture, "plugin_b", "probe");
    assert!(output_b.status.success(), "Plugin B should succeed");

    // Verify Plugin B's expand_path saw plugin_b directory, not plugin_a
    assert_marker_contains_plugin_dir(&marker_b, "plugin_b");
}

#[test]
fn test_expand_path_in_pre_run_execute_post_run() {
    // Verify expand_path works correctly in all hooks of the same task
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    let marker_pre = fixture.temp_dir.path().join("pre_run_path.txt");
    let marker_exec = fixture.temp_dir.path().join("execute_path.txt");
    let marker_post = fixture.temp_dir.path().join("post_run_path.txt");

    let marker_pre_str = marker_pre.to_str().unwrap();
    let marker_exec_str = marker_exec.to_str().unwrap();
    let marker_post_str = marker_post.to_str().unwrap();

    let plugin_content = format!(
        r#"
return {{
    metadata = {{name = "plugin_a", version = "1.0.0", icon = "A", platforms = {{"macos", "linux"}}}},
    tasks = {{
        hooks_test = {{
            description = "Test expand_path in all hooks",
            name = "Hooks Test",
            pre_run = function()
                local path = syntropy.expand_path("./pre_run.txt")
                local f = io.open("{}", "w")
                if f then
                    f:write(path)
                    f:close()
                end
            end,
            execute = function()
                local path = syntropy.expand_path("./execute.txt")
                local f = io.open("{}", "w")
                if f then
                    f:write(path)
                    f:close()
                end
                return "done", 0
            end,
            post_run = function()
                local path = syntropy.expand_path("./post_run.txt")
                local f = io.open("{}", "w")
                if f then
                    f:write(path)
                    f:close()
                end
            end
        }}
    }}
}}
"#,
        marker_pre_str, marker_exec_str, marker_post_str
    );

    fixture.create_plugin("plugin_a", &plugin_content);

    // Execute the task
    let output = run_execute(&fixture, "plugin_a", "hooks_test");
    assert!(output.status.success(), "Task should succeed");

    // Verify all three markers contain plugin_a directory
    assert_marker_contains_plugin_dir(&marker_pre, "plugin_a");
    assert_marker_contains_plugin_dir(&marker_exec, "plugin_a");
    assert_marker_contains_plugin_dir(&marker_post, "plugin_a");
}

#[test]
fn test_expand_path_in_item_sources_execute_context() {
    // CRITICAL: Verify expand_path() isolation when called from item_sources.execute()
    // This is a common production pattern that wasn't covered by other tests.
    // The item_sources.execute() path goes through call_item_source_execute() in lua.rs:148-178
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: Multi-source with expand_path in item_sources.execute()
    let plugin_a_content = r#"
return {
    metadata = {name = "plugin_a", version = "1.0.0", icon = "A", platforms = {"macos", "linux"}},
    tasks = {
        expand_in_source_execute = {
            description = "Calls expand_path in item_sources.execute",
            name = "Multi Source Expand Test",
            mode = "multi",
            item_sources = {
                source = {
                    tag = "s",
                    items = function() return {"item1"} end,
                    preselected_items = function() return {"item1"} end,
                    execute = function(items)
                        -- This is the key: expand_path is called inside item_sources.execute()
                        local path = syntropy.expand_path("./config.txt")
                        -- Then fail to test cleanup after error
                        error("execute() failed after expand_path call")
                    end
                }
            }
        }
    }
}
"#;
    fixture.create_plugin("plugin_a", plugin_a_content);

    // Plugin B: Probe
    let marker_b = fixture.temp_dir.path().join("plugin_b_marker.txt");
    let marker_b_str = marker_b.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "probe", marker_b_str, "./data.txt");

    // Execute Plugin A (fails after calling expand_path in item_sources context)
    let _output_a = run_execute(&fixture, "plugin_a", "expand_in_source_execute");
    // Note: Should fail due to error() in execute

    // Execute Plugin B
    let output_b = run_execute(&fixture, "plugin_b", "probe");
    assert!(
        output_b.status.success(),
        "Plugin B should succeed. stderr: {}",
        String::from_utf8_lossy(&output_b.stderr)
    );

    // CRITICAL: Verify Plugin B saw its own directory, not plugin_a's
    // This confirms registry was cleaned after item_sources.execute() error
    assert_marker_contains_plugin_dir(&marker_b, "plugin_b");
}

// ============================================================================
// Category 4: Hook Failure Edge Cases
// ============================================================================

#[test]
fn test_registry_cleanup_when_post_run_fails() {
    // Verify registry is cleaned even when post_run fails
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: execute succeeds, post_run fails
    let plugin_a_content = r#"
return {
    metadata = {name = "plugin_a", version = "1.0.0", icon = "A", platforms = {"macos", "linux"}},
    tasks = {
        post_run_fails = {
            description = "post_run fails",
            name = "Post Run Fails",
            execute = function()
                return "success", 0
            end,
            post_run = function()
                error("post_run failed intentionally")
            end
        }
    }
}
"#;
    fixture.create_plugin("plugin_a", plugin_a_content);

    // Plugin B: probe
    let marker_b = fixture.temp_dir.path().join("plugin_b_marker.txt");
    let marker_b_str = marker_b.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "probe", marker_b_str, "./data.txt");

    // Execute Plugin A (post_run fails)
    let _output_a = run_execute(&fixture, "plugin_a", "post_run_fails");
    // May succeed or fail depending on how post_run errors are handled

    // Execute Plugin B
    let output_b = run_execute(&fixture, "plugin_b", "probe");
    assert!(output_b.status.success(), "Plugin B should succeed");

    // Verify registry was cleaned despite post_run failure
    assert_marker_contains_plugin_dir(&marker_b, "plugin_b");
}

#[test]
fn test_registry_state_after_pre_run_failure() {
    // Verify registry is cleaned when pre_run fails
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);

    // Plugin A: pre_run fails
    let plugin_a_content = r#"
return {
    metadata = {name = "plugin_a", version = "1.0.0", icon = "A", platforms = {"macos", "linux"}},
    tasks = {
        pre_run_fails = {
            description = "pre_run fails",
            name = "Pre Run Fails",
            pre_run = function()
                error("pre_run failed intentionally")
            end,
            execute = function()
                return "should not run", 0
            end
        }
    }
}
"#;
    fixture.create_plugin("plugin_a", plugin_a_content);

    // Plugin B: probe
    let marker_b = fixture.temp_dir.path().join("plugin_b_marker.txt");
    let marker_b_str = marker_b.to_str().unwrap();
    create_probe_plugin(&fixture, "plugin_b", "probe", marker_b_str, "./data.txt");

    // Execute Plugin A (pre_run fails)
    let output_a = run_execute(&fixture, "plugin_a", "pre_run_fails");
    assert!(!output_a.status.success(), "Plugin A should fail");

    // Execute Plugin B
    let output_b = run_execute(&fixture, "plugin_b", "probe");
    assert!(output_b.status.success(), "Plugin B should succeed");

    // Verify registry was cleaned after pre_run failure
    assert_marker_contains_plugin_dir(&marker_b, "plugin_b");
}
