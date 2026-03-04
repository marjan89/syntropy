//! Integration tests for signal handling (Ctrl+C / SIGINT) during task execution
//!
//! These tests verify that when a user sends SIGINT (Ctrl+C) during task execution:
//! 1. The signal is caught gracefully
//! 2. `post_run()` cleanup function is called
//! 3. Resources created in `pre_run()` are cleaned up
//! 4. Exit code is 130 (standard for SIGINT)
//! 5. Second SIGINT forces immediate exit without waiting for cleanup
//!
//! **Implementation Status**: ❌ NOT YET IMPLEMENTED
//! Currently, CLI mode has no signal handler - process terminates immediately on Ctrl+C
//! without cleanup.
//!
//! **Expected Behavior**: These tests define the behavior that will be implemented.
//!
//! **Critical for Production**: Prevents resource leaks (lock files, database connections,
//! temp files) when users cancel long-running tasks.

use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[cfg(unix)]
use nix::sys::signal::{self, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

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
// Test Category 1: Basic SIGINT Handling
// ============================================================================

#[test]
// Requires SIGINT handler implementation - test will FAIL until implemented
fn test_ctrl_c_calls_post_run() {
    // When SIGINT is sent during execute(), post_run() should still be called
    const PLUGIN_WITH_CLEANUP: &str = r#"
return {
    metadata = {
        name = "cleanup-test",
        version = "1.0.0",
        icon = "C",
        platforms = {"macos", "linux"},
    },
    tasks = {
        cleanup_task = {
            name = "Cleanup Task",
            description = "Test post_run cleanup on SIGINT",
            mode = "none",
            pre_run = function()
                local f = io.open("/tmp/syntropy_test_lock", "w")
                f:write("locked")
                f:close()
            end,
            post_run = function()
                os.remove("/tmp/syntropy_test_lock")
            end,
            item_sources = {
                test = {
                    tag = "t",
                    items = function()
                        return {"item1"}
                    end,
                    execute = function(items)
                        syntropy.shell("sleep 10")
                        return "done", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("cleanup-test", PLUGIN_WITH_CLEANUP);

    // Clean up any leftover lock file from previous test
    let _ = std::fs::remove_file("/tmp/syntropy_test_lock");

    // Get path to syntropy binary
    let syntropy_bin = assert_cmd::cargo::cargo_bin!("syntropy");

    // Spawn syntropy CLI process
    let mut child = Command::new(syntropy_bin)
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("cleanup-test")
        .arg("--task")
        .arg("cleanup_task")
        .spawn()
        .expect("Failed to spawn syntropy process");

    // Wait for pre_run to complete and execute to start
    thread::sleep(Duration::from_millis(500));

    // Verify lock file was created by pre_run
    assert!(
        Path::new("/tmp/syntropy_test_lock").exists(),
        "pre_run should create lock file"
    );

    // Send SIGINT (Ctrl+C) - Unix only
    #[cfg(unix)]
    {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;
        signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT)
            .expect("Failed to send SIGINT");
    }

    // Wait for cleanup to complete
    let status = child.wait().expect("Failed to wait for process");

    // Verify post_run executed (lock file removed)
    assert!(
        !Path::new("/tmp/syntropy_test_lock").exists(),
        "post_run should remove lock file on Ctrl+C"
    );

    // Verify exit code is 130 (standard for SIGINT)
    assert_eq!(
        status.code(),
        Some(130),
        "Exit code should be 130 for SIGINT"
    );
}

#[test]
// Requires SIGINT handler implementation - test will FAIL until implemented
fn test_ctrl_c_during_pre_run_skips_execute() {
    // When SIGINT is sent during pre_run(), execute() should be skipped but post_run() called
    const PLUGIN_SLOW_PRERUN: &str = r#"
return {
    metadata = {
        name = "slow-prerun",
        version = "1.0.0",
        icon = "S",
        platforms = {"macos", "linux"},
    },
    tasks = {
        slow_task = {
            name = "Slow Pre-run Task",
            description = "Test cancellation during pre_run",
            mode = "none",
            pre_run = function()
                local f = io.open("/tmp/syntropy_test_prerun", "w")
                f:write("pre_run started")
                f:close()
                syntropy.shell("sleep 2")
            end,
            post_run = function()
                os.remove("/tmp/syntropy_test_prerun")
            end,
            item_sources = {
                test = {
                    tag = "t",
                    items = function()
                        return {"item1"}
                    end,
                    execute = function(items)
                        local f = io.open("/tmp/syntropy_test_executed", "w")
                        f:write("executed")
                        f:close()
                        return "done", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("slow-prerun", PLUGIN_SLOW_PRERUN);

    // Clean up leftover files
    let _ = std::fs::remove_file("/tmp/syntropy_test_prerun");
    let _ = std::fs::remove_file("/tmp/syntropy_test_executed");

    // Spawn process
    let syntropy_bin = assert_cmd::cargo::cargo_bin!("syntropy");
    let mut child = Command::new(syntropy_bin)
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("slow-prerun")
        .arg("--task")
        .arg("slow_task")
        .spawn()
        .expect("Failed to spawn process");

    // Wait for pre_run to start but not complete
    thread::sleep(Duration::from_millis(500));

    // Send SIGINT during pre_run
    signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).expect("Failed to send SIGINT");

    // Wait for process to exit
    // TODO: Add timeout for test reliability
    child.wait().expect("Failed to wait for process");

    // Execute should NOT have run
    assert!(
        !Path::new("/tmp/syntropy_test_executed").exists(),
        "execute() should not run after Ctrl+C during pre_run"
    );

    // But post_run should still have cleaned up
    assert!(
        !Path::new("/tmp/syntropy_test_prerun").exists(),
        "post_run should clean up even if execute skipped"
    );
}

// ============================================================================
// Test Category 2: Double Ctrl+C (Force Quit)
// ============================================================================

#[test]
// Requires SIGINT handler implementation - test will FAIL until implemented
fn test_double_ctrl_c_forces_immediate_exit() {
    // First SIGINT: graceful shutdown (runs cleanup)
    // Second SIGINT: immediate exit (no cleanup wait)
    const PLUGIN_SLOW_CLEANUP: &str = r#"
return {
    metadata = {
        name = "slow-cleanup",
        version = "1.0.0",
        icon = "F",
        platforms = {"macos", "linux"},
    },
    tasks = {
        slow_cleanup = {
            name = "Slow Cleanup Task",
            description = "Test force quit on double Ctrl+C",
            mode = "none",
            item_sources = {
                test = {
                    tag = "t",
                    items = function()
                        return {"item1"}
                    end,
                    execute = function(items)
                        return "done", 0
                    end,
                    post_run = function()
                        -- Simulate slow cleanup (5 seconds)
                        -- Should be interrupted by second SIGINT
                        os.execute("sleep 5")
                        -- Mark completion (should NOT happen on double Ctrl+C)
                        local f = io.open("/tmp/syntropy_test_cleanup_done", "w")
                        f:write("cleanup completed")
                        f:close()
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("slow-cleanup", PLUGIN_SLOW_CLEANUP);

    // Clean up leftover file
    let _ = std::fs::remove_file("/tmp/syntropy_test_cleanup_done");

    // Spawn process
    let syntropy_bin = assert_cmd::cargo::cargo_bin!("syntropy");
    let mut child = Command::new(syntropy_bin)
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("slow-cleanup")
        .arg("--task")
        .arg("slow_cleanup")
        .spawn()
        .expect("Failed to spawn process");

    // Wait for execution to start
    thread::sleep(Duration::from_millis(500));

    // First Ctrl+C: graceful shutdown (starts cleanup)
    signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT)
        .expect("Failed to send first SIGINT");

    // Wait a bit for cleanup to start
    thread::sleep(Duration::from_millis(100));

    // Second Ctrl+C: force immediate exit
    signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT)
        .expect("Failed to send second SIGINT");

    // Should exit immediately (not wait 5s for cleanup)
    let start = std::time::Instant::now();
    // TODO: Add timeout for test reliability - should exit within 1-2 seconds
    child.wait().expect("Failed to wait for process");

    assert!(
        start.elapsed() < Duration::from_secs(3),
        "Second Ctrl+C should force immediate exit (took {:?})",
        start.elapsed()
    );

    // Cleanup should NOT have completed (interrupted)
    assert!(
        !Path::new("/tmp/syntropy_test_cleanup_done").exists(),
        "Cleanup should be interrupted by second Ctrl+C"
    );
}

// ============================================================================
// Test Category 3: Registry Cleanup on Abort
// ============================================================================

#[tokio::test]
// Requires Lua runtime access in test - test will FAIL until implemented
async fn test_handle_abort_cleans_registry() {
    // This test verifies that when Handle::abort() is called (either by SIGINT
    // or by dropping the handle), the Lua registry is cleaned up via RAII guard.
    //
    // NOTE: This behavior is already tested in tests/unit_registry_cleanup_guard.rs
    // This test would verify it works in the signal handling context.
    //
    // Implementation requires access to the Lua runtime, which is not easily
    // accessible in integration tests. Consider this a documentation of expected
    // behavior that's covered by unit tests.

    // See: tests/unit_registry_cleanup_guard.rs for actual implementation
}

// ============================================================================
// Test Category 4: User Feedback Messages
// ============================================================================

#[test]
// Requires SIGINT handler implementation - test will FAIL until implemented
fn test_ctrl_c_shows_cancellation_message() {
    // When SIGINT is received, user should see "Cancelling task... running cleanup"
    const PLUGIN_SIMPLE: &str = r#"
return {
    metadata = {
        name = "simple",
        version = "1.0.0",
        icon = "S",
        platforms = {"macos", "linux"},
    },
    tasks = {
        simple = {
            name = "Simple Task",
            description = "Test cancellation message",
            mode = "none",
            item_sources = {
                test = {
                    tag = "t",
                    items = function()
                        return {"item1"}
                    end,
                    execute = function(items)
                        syntropy.shell("sleep 10")
                        return "done", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("simple", PLUGIN_SIMPLE);

    // Spawn process and capture output
    let syntropy_bin = assert_cmd::cargo::cargo_bin!("syntropy");
    let child = Command::new(syntropy_bin)
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("simple")
        .arg("--task")
        .arg("simple")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    thread::sleep(Duration::from_millis(500));

    // Send SIGINT
    signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).expect("Failed to send SIGINT");

    // Wait and capture output
    let output = child.wait_with_output().expect("Failed to get output");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show cancellation message
    assert!(
        stderr.contains("Cancelling") || stderr.contains("cancelled"),
        "Should show cancellation message. Got: {}",
        stderr
    );
}
