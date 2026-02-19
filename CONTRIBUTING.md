# Contributing to Syntropy

Welcome! Syntropy is a TUI tool with a Lua plugin framework. This guide will help you navigate the codebase and contribute effectively.

## Architecture Overview

Syntropy has two execution modes (CLI and TUI) sharing a common core engine and plugin system.

```
                         ┌─────────────┐
                         │syntropy bin │
                         │   (main.rs) │
                         └──────┬──────┘
                                │
                         ┌──────▼───────┐
                         │  app::run()  │
                         │              │
                         │ - Parse args │
                         │ - Load config│
                         │ - Init Lua VM│
                         │ - Load plugins│
                         └──────┬───────┘
                                │
                 ┌──────────────┼──────────────┐
                 │                             │
           ┌─────▼─────┐                ┌─────▼─────┐
           │    CLI    │                │    TUI    │
           │  (clap)   │                │ (ratatui) │
           │           │                │           │
           │  execute  │                │ Navigator │
           │ --plugin  │                │  Screens  │
           │ --task    │                │  Events   │
           └─────┬─────┘                │  Fuzzy    │
                 │                      └─────┬─────┘
                 └──────────┬─────────────────┘
                            │
                     ┌──────▼───────┐
                     │ Core Engine  │
                     │              │
                     │ TaskRunner   │
                     │ Config       │
                     └──────┬───────┘
                            │
                     ┌──────▼────────┐
                     │ PluginManager │
                     │               │
                     │ - Discovery   │
                     │ - Loading     │
                     │ - Merge       │
                     │ - Validation  │
                     └──────┬────────┘
                            │
                     ┌──────▼────────┐
                     │  Lua Bridge   │
                     │    (mlua)     │
                     │               │
                     │ - VM Runtime  │
                     │ - API binding │
                     │ - Async calls │
                     └──────┬────────┘
                            │
                 ┌──────────┴──────────┐
                 │                     │
         ┌───────▼────────┐   ┌────────▼────────┐
         │ System Plugins │   │  User Plugins   │
         │   (builtin)    │   │ ~/.config/syntropy/ │
         └────────────────┘   └─────────────────┘

    XDG Directories:
    CONFIG: ~/.config/syntropy/     (config.toml, user plugins)
    DATA:   ~/.local/share/syntropy (managed plugins, state)
```

**Key Data Flow (TUI Mode):**
```
Keyboard Event → Screen.handle_event() → Intent
    ↓
Navigator.resolve(Intent) → New Route
    ↓
ScreenDispatcher.on_enter(Route) → Load Data (call Lua)
    ↓
Screen.render() → Ratatui Widgets
```

## Codebase Navigation

### Directory Structure

| Path | Purpose | When to Touch |
|------|---------|---------------|
| `src/main.rs` | Entry point | Never (just calls `app::run()`) |
| `src/app/` | App orchestration, CLI/TUI routing | Adding new subcommands or modes |
| `src/cli/` | CLI subcommands (execute, init, validate, plugins, completions) | Adding new CLI functionality |
| `src/configs/` | Config parsing, XDG paths, keybindings, styles | Adding new config options |
| `src/plugins/` | Plugin discovery, loading, merging, validation | Modifying plugin system behavior |
| `src/lua/` | Lua VM setup, stdlib APIs, Rust↔Lua bridge | Adding new Lua APIs for plugins |
| `src/execution/` | Task execution pipeline (pre-run → items → execute → post-run) | Changing task execution logic |
| `src/tui/` | TUI screens, navigation, events, fuzzy search | Adding UI features or screens |
| `tests/unit/` | Pure logic tests (no I/O) | Testing parsers, validators, algorithms |
| `tests/integration/` | End-to-end tests with temp dirs and Lua | Testing plugin loading, execution, CLI |
| `tests/common/` | Shared test fixtures and helpers | Reusable test setup code |

### Key Modules Explained

**`src/app/run.rs`** - The heart of syntropy
- Parses CLI args with clap
- Loads `config.toml` from XDG config dir
- Initializes Lua VM with stdlib
- Discovers and loads plugins (with merge support)
- Routes to CLI or TUI mode based on execute subcommand

**`src/plugins/loader.rs`** - Plugin loading and merging
- Two-pass loading: peek names, then merge duplicates
- Merge logic: config plugins override data plugins
- Uses Lua's `merge()` function for deep merging tables
- Validates plugin structure (metadata, tasks, semver)

**`src/lua/runtime.rs`** - Lua VM initialization
- Creates sandboxed Lua environment (no `os.execute`, `os.exit`)
- Registers `syntropy.*` stdlib functions
- Provides global `merge(base, override)` function

**`src/execution/runner.rs`** - Task execution pipeline
- Calls task's `pre_run()` hook
- Fetches items from all item sources via `items()` function
- Calls `preview(item)` when cursor moves
- Executes selected items via `execute(items)` function
- Calls task's `post_run()` hook
- Returns output string and exit code

**`src/tui/navigation/navigator.rs`** - Route stack management
- Stack-based navigation: `Plugin → Task → Item`
- Resolves intents (Select, Back, Quit) into route changes
- Maintains history for back button behavior

**`src/tui/dispatcher.rs`** - Screen lifecycle manager
- Owns all screen instances (PluginList, TaskList, ItemList)
- Routes events to current screen based on route
- Calls `on_enter()` when route changes (loads data)
- Calls `on_update()` for background updates
- Calls `on_exit()` for cleanup

### "Where Do I Make Changes?" Guide

| I want to... | Look here | Key files |
|--------------|-----------|-----------|
| Add a new Lua API | `src/lua/stdlib.rs` → `register_syntropy_stdlib()` | `stdlib.rs`, `scaffold_templates/syntropy.lua`, `scaffold_templates/plugin.lua` |
| Change plugin validation rules | `src/plugins/loader.rs` → `validate_plugin()` | `loader.rs` |
| Add a new CLI command | `src/cli/args.rs` + `src/cli/<cmd>.rs` + `src/app/run.rs` | `args.rs`, `run.rs` |
| Add a new keybinding | `src/configs/key_bindings.rs` + `default_config.toml` | `key_bindings.rs` |
| Add a new TUI screen | Implement `Screen` trait in `src/tui/screens/` | `screens/<name>.rs`, `dispatcher.rs`, `routes.rs` |
| Change task execution flow | `src/execution/runner.rs` | `runner.rs`, `lua.rs` |
| Modify config schema | `src/configs/config.rs` struct + `default_config.toml` | `config.rs` |
| Modify plugin structure | `src/plugins/plugin.rs` → `Plugin`/`Task`/`ItemSource` structs | `plugin.rs`, `loader.rs`, `scaffold_templates/plugin.lua` |

## Development Setup

### Building

```bash
# Debug build (fast compilation, slower runtime)
cargo build

# Release build (slow compilation, fast runtime)
cargo build --release

# Run without installing
cargo run -- [args]

# Example: Run with specific plugin/task
XDG_CONFIG_HOME=/tmp/test cargo run -- execute --plugin test --task demo
```

### Useful Commands

```bash
# Check for errors without building
cargo check

# Run linter (enforces code style)
cargo clippy

# Format code
cargo fmt

# Generate docs and open in browser
cargo doc --open

# Test with specific temp config dir
XDG_CONFIG_HOME=/tmp/syntropy_test XDG_DATA_HOME=/tmp/syntropy_data cargo run
```

### Plugin Development Cycle

```bash
# 1. Initialize plugin scaffold
cargo run -- init

# 2. Create your plugin
mkdir -p ~/.config/syntropy/plugins/my-plugin
vim ~/.config/syntropy/plugins/my-plugin/plugin.lua

# 3. Test it
cargo run  # Auto-discovers plugins

# 4. Validate plugin structure
cargo run -- validate --plugin ~/.config/syntropy/plugins/my-plugin/plugin.lua
```

## Testing Philosophy

Syntropy follows **Test-Driven Development (TDD)** principles:

1. **Write test first** - Define expected behavior before implementation
2. **Red → Green → Refactor** - Fail, pass, improve
3. **Regression tests** - When fixing bugs, add test that reproduces the bug first
4. **Integration over mocking** - Use real Lua VM in tests when possible
5. **Test behavior, not bugs** - Tests validate correct behavior, not current broken state

### Test Organization

**Unit Tests** (`tests/unit/`) - Fast, isolated, pure logic
- No file I/O, no Lua VM, no network
- Test pure functions: parsers, validators, algorithms
- Examples: tag parsing, keybind parsing, semver validation

**Integration Tests** (`tests/integration/`) - Real components, temp directories
- Real Lua VM, actual file system (in temp dirs)
- Test full workflows: plugin loading, merging, execution
- Use `TestFixture` for setup/teardown

### Running Tests

```bash
# All tests (unit + integration)
cargo test

# Only unit tests (fast)
cargo test --lib

# Only integration tests
cargo test --test '*'

# Specific test
cargo test test_plugin_merge

# Show println! output
cargo test -- --nocapture

# Run tests in series (not parallel)
cargo test -- --test-threads=1
```

### Writing Tests

**Unit Test Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() {
        let result = parse_tag("[pkg] package-name");
        assert_eq!(result, Some(("pkg", "package-name")));
    }
}
```

**Integration Test Example:**
```rust
use syntropy::test_common::TestFixture;

#[test]
fn test_plugin_loading() {
    let fixture = TestFixture::new();

    // Create plugin in temp directory
    fixture.create_plugin("test", r#"
        return {
            metadata = {name = "test", version = "1.0.0"},
            tasks = {demo = {execute = function() return "ok", 0 end}}
        }
    "#);

    // Load and validate
    let plugins = fixture.load_plugins().unwrap();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].metadata.name, "test");
}
```

### When to Write Tests

| Scenario | Test Type | Why |
|----------|-----------|-----|
| Adding new Lua API | Integration | Ensure Rust↔Lua bridge works correctly |
| Fixing a bug | Regression test first | Prevents bug from returning |
| Adding config field | Unit test for parsing | Fast feedback on serde attributes |
| Changing plugin validation | Integration | Validate real plugin loading |
| Adding navigation intent | Unit test | Pure state machine logic |
| Changing keybind parsing | Unit test | Pure string parsing |

### Test Patterns

**TestFixture Pattern** - Automatic cleanup
```rust
let fixture = TestFixture::new(); // Creates temp dirs
fixture.create_plugin("name", lua_code);
// Temp dirs automatically deleted on drop
```

**Async Runtime in Tests**
```rust
let rt = tokio::runtime::Runtime::new().unwrap();
rt.block_on(async {
    let result = call_task_execute(&lua, task, &items).await;
    assert_eq!(result.unwrap().1, 0); // exit code 0
});
```

**Testing Plugin Merge**
```rust
fixture.create_plugin("pkg", base_plugin_code);
fixture.create_plugin_override("pkg", override_code);
let plugins = fixture.load_plugins().unwrap();
// Verify merged behavior
```

## Code Patterns

### Error Handling

**Use `anyhow::Result` everywhere:**
```rust
use anyhow::{Context, Result, bail, ensure};

pub fn load_config(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .context("Failed to read config file")?;

    let config: Config = toml::from_str(&contents)
        .context("Failed to parse TOML")?;

    ensure!(!config.plugins.is_empty(), "No plugins configured");

    Ok(config)
}
```

**Chain context for debugging:**
```rust
load_plugin(path)
    .with_context(|| format!("Failed to load plugin from {}", path.display()))?;
```

**Bail early on fatal errors:**
```rust
if !metadata.name.is_empty() {
    bail!("Plugin metadata.name cannot be empty");
}
```

### Async Patterns

**All I/O and Lua calls are async:**
```rust
use tokio;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn call_task_execute(
    lua: &Arc<Mutex<Lua>>,
    task: &Task,
    items: &[String],
) -> Result<(String, i32)> {
    let lua_guard = lua.lock().await; // Async lock

    let func = get_lua_function(&lua_guard, &[
        &task.plugin_name,
        "tasks",
        &task.task_key,
        "execute",
    ])?;

    let result: (String, i32) = func.call_async(items).await
        .context("Error calling execute function")?;

    Ok(result)
}
```

**Spawning background tasks:**
```rust
tokio::spawn(async move {
    // Background work
});
```

### Lua Bridge Patterns

**Registering Lua functions:**
```rust
pub fn register_syntropy_stdlib(lua: &Lua) -> Result<()> {
    let syntropy_table = lua.create_table()?;

    syntropy_table.set("shell", lua.create_async_function(lua_shell)?)?;
    syntropy_table.set("expand_tilde", lua.create_function(lua_expand_tilde)?)?;

    lua.globals().set("syntropy", syntropy_table)?;
    Ok(())
}

fn lua_shell<'lua>(
    lua: &'lua Lua,
    cmd: String,
) -> impl Future<Output = LuaResult<(String, i32)>> + 'lua {
    async move {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .await
            .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

        Ok((String::from_utf8_lossy(&output.stdout).to_string(), output.status.code().unwrap_or(-1)))
    }
}
```

**Calling Lua from Rust:**
```rust
pub fn get_lua_function<'lua>(
    lua: &'lua Lua,
    path: &[&str],
) -> Result<LuaFunction<'lua>> {
    let mut table: LuaValue = lua.globals().into();

    for segment in &path[..path.len() - 1] {
        table = table.get::<_, LuaTable>(segment)?.into();
    }

    let func = table.get::<_, LuaFunction>(path.last().unwrap())?;
    Ok(func)
}
```

**Important: Update scaffold templates when adding stdlib functions:**

When adding a new `syntropy.*` function, you must also update:
1. `scaffold_templates/syntropy.lua` - Add type annotations for LSP/autocomplete
2. `scaffold_templates/plugin.lua` - Update example usage if relevant

This ensures plugin developers get proper IDE support for new APIs.

**Important: Update scaffold templates when modifying plugin structure:**

When modifying `src/plugins/plugin.rs` (adding fields to `Plugin`, `Task`, `ItemSource`, or `Metadata`), you must also update:
1. `scaffold_templates/plugin.lua` - Update type annotations to match new structure
2. Update documentation in `docs/plugin-api-reference.md`

This ensures type definitions stay in sync and plugin developers have accurate LSP support.

**External TUI Request Architecture:**

The `syntropy.invoke_tui()` and `syntropy.invoke_editor()` functions use a channel-based architecture to coordinate between Lua execution (on Tokio runtime) and the TUI main loop (on the main thread):

```rust
// 1. Create channel in TUI initialization (src/tui/mod.rs)
let (tui_tx, tui_rx) = create_tui_channel();
set_tui_sender(tui_tx.clone())?;

// 2. Lua stdlib sends request via channel (src/lua/stdlib.rs)
pub async fn lua_invoke_tui<'lua>(
    lua: &'lua Lua,
    (command, args): (String, LuaTable<'lua>),
) -> LuaResult<i32> {
    let (response_tx, response_rx) = oneshot::channel();

    let request = ExternalTuiRequest {
        command,
        args: convert_lua_table_to_vec(args)?,
        response: response_tx,
    };

    // Send to TUI main loop
    get_tui_sender()
        .ok_or_else(|| LuaError::RuntimeError("TUI sender not initialized".into()))?
        .send(request)
        .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

    // Wait for response (blocks this Tokio task)
    let exit_code = response_rx.await
        .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

    Ok(exit_code)
}

// 3. TUI main loop receives request (src/tui/tui_app.rs)
pub fn run(&mut self, terminal: &mut Terminal<Backend>) -> Result<()> {
    loop {
        // Check for external TUI requests on every render cycle
        if let Ok(request) = self.tui_rx.try_recv() {
            self.suspend_and_run_tui(request, terminal)?;
            continue;
        }

        // Normal event polling and rendering...
    }
}

// 4. Suspend TUI and run external command (src/tui/tui_app.rs)
fn suspend_and_run_tui(&mut self, request: ExternalTuiRequest, terminal: &mut Terminal) -> Result<()> {
    // Suspend TUI
    disable_raw_mode()?;
    terminal.leave_alternate_screen()?;

    // Run external command (blocks main thread)
    let exit_code = run_tui_command_blocking(&request.command, &request.args)?;

    // Restore TUI
    terminal.enter_alternate_screen()?;
    enable_raw_mode()?;
    terminal.clear()?;

    // Send response back to Lua
    request.response.send(exit_code).ok();

    Ok(())
}
```

**Key architectural points:**
- **Channel-based coordination:** `tokio::sync::mpsc::unbounded_channel` for requests, `tokio::sync::oneshot::channel` for responses
- **Main thread ownership:** Only the main thread has terminal access (Ratatui requirement), so external TUI commands must run there
- **Intentional blocking:** The main thread blocks on `std::process::Command::status()` while the external app runs
- **State preservation:** Lua execution state is held in the awaiting Tokio task; TUI state is held in memory during suspension
- **Global sender:** `OnceLock<TuiRequestSender>` provides global access for Lua stdlib functions

**Testing external TUI features:**
1. Use CLI mode for automated testing (no TUI suspension needed)
2. In TUI mode, test with simple commands like `vim --version` to verify suspend/restore works
3. Test error handling with non-existent commands
4. Verify exit code propagation and clamping (POSIX range 0-255)

### Config Handling

**XDG Base Directory Spec:**
```rust
use dirs;

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syntropy")
}

pub fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syntropy")
}
```

**Config priority (CLI > File > Defaults):**
```rust
let mut config = Config::load(config_path)?;

// CLI args override config
if let Some(plugin) = &cli_args.plugin {
    config.default_plugin = Some(plugin.clone());
}
```

**Serde patterns:**
```rust
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde_inline_default(String::from("default"))]
    pub theme: String,

    #[serde(default)]
    pub plugins: Vec<String>,
}
```

### State Management (TUI)

**Navigator pattern (stack-based routing):**
```rust
pub struct Navigator {
    route_stack: Vec<Route>,
}

impl Navigator {
    pub fn push(&mut self, route: Route) {
        self.route_stack.push(route);
    }

    pub fn pop(&mut self) -> Option<Route> {
        if self.route_stack.len() > 1 {
            self.route_stack.pop()
        } else {
            None
        }
    }

    pub fn current(&self) -> &Route {
        self.route_stack.last().unwrap()
    }
}
```

**Screen lifecycle:**
```rust
pub trait Screen {
    fn on_enter(&mut self, app: &App, payload: &Payload);
    fn on_update(&mut self, app: &App, payload: &Payload);
    fn handle_event(&mut self, event: InputEvent) -> Intent;
    fn render(&mut self, frame: &mut Frame, rect: Rect, styles: &Styles);
    fn on_exit(&mut self);
}
```

## Pull Request Process

1. **Fork and branch** - Create feature branch from `master`
2. **Write tests** - TDD approach, tests before implementation
3. **Run checks** - `cargo test && cargo clippy && cargo fmt`
4. **Commit** - Clear, concise commit messages
5. **Push** - Push to your fork
6. **PR** - Open PR against `master` with description of changes

**Commit message style:**
```
<type>: <description>

Examples:
feat: add syntropy.read_file() Lua API
fix: prevent crash on invalid plugin metadata
test: add regression test for merge bug
docs: update plugin API reference
refactor: simplify navigator state machine
```

**PR template:**
- What: Brief description of changes
- Why: Reason for changes (bug, feature, refactor)
- Testing: How you tested it
- Breaking: Any breaking changes

---

Questions? Open an issue or start a discussion. Happy coding!
