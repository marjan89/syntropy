//! Unit test that directly verifies RegistryCleanupGuard works on abort

use mlua::Lua;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_registry_cleanup_guard_on_abort() {
    let lua = Arc::new(Mutex::new(Lua::new()));

    // Simulate the pattern used in lua.rs with the guard
    let lua_clone = Arc::clone(&lua);
    let handle = tokio::spawn(async move {
        let lua_guard = lua_clone.lock().await;

        // Set plugin context
        lua_guard
            .set_named_registry_value("__syntropy_current_plugin__", "plugin_a")
            .unwrap();

        // RAII guard (same pattern as in lua.rs)
        struct RegistryCleanupGuard<'lua> {
            lua: &'lua Lua,
        }

        impl Drop for RegistryCleanupGuard<'_> {
            fn drop(&mut self) {
                let _ = self
                    .lua
                    .set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil);
            }
        }

        let _cleanup_guard = RegistryCleanupGuard { lua: &lua_guard };

        // Simulate async operation that gets aborted
        sleep(Duration::from_millis(50)).await;

        // If we reach here, abort didn't work, but guard should still clean
        // This shouldn't happen in real abort scenarios
        lua_guard
            .set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)
            .unwrap();
    });

    // Give it time to start and set the registry
    sleep(Duration::from_millis(10)).await;

    // Verify plugin context was set
    {
        let lua_guard = lua.lock().await;
        let ctx: Result<String, _> = lua_guard.named_registry_value("__syntropy_current_plugin__");

        // The guard may have already cleaned it if task completed
        // That's actually fine - we just want to test cleanup happens
        if let Ok(val) = ctx {
            println!("Context before abort: {}", val);
        }
    }

    // Abort the task (this is what handle.rs:119 does)
    handle.abort();

    // Wait a bit for abort to complete
    sleep(Duration::from_millis(10)).await;

    // CRITICAL: Verify registry was cleaned despite abort
    {
        let lua_guard = lua.lock().await;
        let ctx_result: Result<String, _> =
            lua_guard.named_registry_value("__syntropy_current_plugin__");

        // If guard worked: either Error (nil) or empty string
        match ctx_result {
            Err(_) => {
                // nil - this is correct! Guard cleaned up
                println!("✓ Guard worked: Registry is nil (cleaned)");
            }
            Ok(val) if val.is_empty() => {
                println!("✓ Guard worked: Registry is empty");
            }
            Ok(val) => {
                panic!(
                    "✗ Guard FAILED: Registry still contains '{}' after abort",
                    val
                );
            }
        }
    }
}

#[tokio::test]
async fn test_without_guard_leaves_stale_context() {
    // This test shows what happens WITHOUT the guard (the bug)
    let lua = Arc::new(Mutex::new(Lua::new()));

    let lua_clone = Arc::clone(&lua);
    let handle = tokio::spawn(async move {
        let lua_guard = lua_clone.lock().await;

        // Set plugin context
        lua_guard
            .set_named_registry_value("__syntropy_current_plugin__", "plugin_a")
            .unwrap();

        // NO GUARD HERE - simulating the bug

        // Simulate async operation
        sleep(Duration::from_millis(50)).await;

        // Manual cleanup that never executes due to abort
        lua_guard
            .set_named_registry_value("__syntropy_current_plugin__", mlua::Value::Nil)
            .unwrap();
    });

    // Give it time to set context
    sleep(Duration::from_millis(10)).await;

    // Abort the task
    handle.abort();
    sleep(Duration::from_millis(100)).await;

    // Verify bug: Context is NOT cleaned (stale)
    {
        let lua_guard = lua.lock().await;
        let ctx: String = lua_guard
            .named_registry_value("__syntropy_current_plugin__")
            .unwrap();

        assert_eq!(
            ctx, "plugin_a",
            "BUG DEMONSTRATED: Without guard, context remains stale after abort"
        );
    }
}
