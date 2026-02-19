# Plugin API Reference - API Functions & Utilities

> Part of the [Plugin API Reference](plugin-api-reference.md)  
> **See also:** [Tasks](plugin-api-reference-section-tasks.md) | [Examples](plugin-api-reference-section-examples.md)

## Syntropy Functions

These functions are in the `syntropy` namespace.

### syntropy.shell

Execute shell command asynchronously.

**Function signature:**
```lua
syntropy.shell(command: string) -> string, integer
```

**Parameters:**
- `command` (string) - Shell command to execute

**Returns:**
- `output` (string) - Combined stdout and stderr
- `exit_code` (integer) - Process exit code (0 = success, -1 = spawn failed)

**Behavior:**
- Runs via `sh -c`, supports pipes, redirects, etc.
- Async execution (doesn't block TUI)
- Captures both stdout and stderr (combined)
- Returns when command completes

**Examples:**

```lua
-- Simple command
local out, code = syntropy.shell("ls -la")
if code == 0 then
    print("Success: " .. out)
end

-- With error handling
local out, code = syntropy.shell("git status")
if code ~= 0 then
    return "Git failed: " .. out, code
end

-- Multi-line script
local script = [[
cd ~/project
git pull origin main
cargo build --release
]]
local output, code = syntropy.shell(script)

-- Command with arguments
local file = "test.txt"
local cmd = string.format("cat %s | wc -l", file)
local lines, code = syntropy.shell(cmd)
```

**Security Note:**
- Be careful with user input in shell commands
- Escape special characters or use Lua string manipulation
- Avoid command injection vulnerabilities

### syntropy.expand_path

Expands paths with support for plugin-relative paths, tilde expansion, and environment variables.

**Function signature:**
```lua
syntropy.expand_path(path: string) -> string
```

**Supported path types:**

1. **Plugin-relative paths** - Paths starting with `./` or `../` resolve relative to the plugin's directory:
   - `./config.json` → `{plugin_dir}/config.json`
   - `../data/file.txt` → `{plugin_parent}/data/file.txt`

2. **Tilde expansion** - Expands `~` to the user's home directory:
   - `~/Documents/file.txt` → `/Users/username/Documents/file.txt`

3. **Environment variables** - Expands environment variables:
   - `$HOME/.config/app` → `/Users/username/.config/app`
   - `${XDG_CONFIG_HOME}/app` → `/Users/username/.config/app`

4. **Absolute paths** - Pass through unchanged:
   - `/tmp/file.txt` → `/tmp/file.txt`

**Important limitations:**
- ⚠️ Plugin-relative paths (`./`, `../`) only work when called inside plugin functions (items, execute, preview, pre_run, post_run)
- ⚠️ Calling `syntropy.expand_path("./file")` at module level (top of plugin.lua) will fail with error: "Cannot resolve relative path: no plugin context"
- 💡 **Solution:** Store unexpanded paths in a `config` table and expand them at runtime - see [Plugin Configuration](#plugin-configuration) for the recommended pattern

**Example - Correct usage:**
```lua
return {
    tasks = {
        my_task = {
            items = function()
                -- ✅ Works: called inside a function
                local path = syntropy.expand_path("./exclusions.txt")
                local file = io.open(path, "r")
                -- ...
            end,

            execute = function(items)
                -- ✅ Works: called inside a function
                local config = syntropy.expand_path("./config.json")
                -- ...
            end
        }
    }
}
```

**Example - Incorrect usage:**
```lua
-- ❌ FAILS: called at module level
local EXCLUSIONS = syntropy.expand_path("./exclusions.txt")
-- Error: Cannot resolve relative path: no plugin context

return {
    -- ...
}
```

**Technical details:**
- For merged plugins (config override + data base), plugin-relative paths resolve to the override directory (in config), not the base directory (in data)
- Plugin-relative resolution uses the `__plugin_dir` field injected into the plugin table during loading
- Paths are resolved using standard path joining, so `.` and `..` components work as expected

### syntropy.invoke_tui

Launches an external TUI (Text User Interface) application with full terminal control.

**Function signature:**
```lua
syntropy.invoke_tui(command: string, args: table) -> integer
```

**Parameters:**
- `command` (string) - The command/program to execute
- `args` (table) - Array-style table of string arguments

**Returns:**
- `exit_code` (integer) - Exit code from the external application (clamped to POSIX range 0-255)

**Behavior:**
- **TUI mode:** Suspends syntropy's TUI, gives full terminal control to the external app, then restores syntropy's TUI when it exits
  - Disables raw mode
  - Leaves alternate screen
  - Runs external command with inherited stdin/stdout/stderr
  - Restores alternate screen and raw mode after exit
- **CLI mode:** Runs command directly with inherited stdio
- **Blocking:** The plugin execution pauses until the external application exits
- **State preservation:** All Lua variables and execution state are preserved; execution resumes exactly where it left off
- Exit code is properly clamped to valid POSIX range (0-255)

**Use cases:**
- Launch interactive file managers (ranger, lf, nnn)
- Run process monitors (htop, btop, top)
- Use fuzzy finders (fzf, skim)
- Open Git TUIs (tig, lazygit, gitui)
- Launch any full-screen terminal application

**Examples:**

```lua
-- Launch fzf for file selection
local code = syntropy.invoke_tui("fzf", {
    "--multi",
    "--preview", "cat {}",
    "--preview-window", "right:50%"
})
if code == 0 then
    print("Selection made")
else
    print("Selection cancelled")
end

-- Launch htop for process monitoring
local code = syntropy.invoke_tui("htop", {})

-- Launch ranger file manager in a specific directory
local code = syntropy.invoke_tui("ranger", {"/home/user/documents"})

-- Launch tig for Git history browsing
local code = syntropy.invoke_tui("tig", {"--all"})

-- Error handling
local code = syntropy.invoke_tui("nonexistent", {})
if code ~= 0 then
    return "Command failed or was cancelled", code
end
```

**Important notes:**
- The external application receives **complete terminal control** - this is intentional and necessary
- Blocking behavior is **correct by design** - external TUI apps cannot share terminal control
- All plugin state (variables, call stack) is preserved during the pause
- The TUI state is restored exactly as it was before suspension
- Security: Validate commands before invoking to prevent command injection

**Comparison with syntropy.shell:**
- `syntropy.shell()` captures output and runs in the background
- `syntropy.invoke_tui()` gives full terminal access and blocks the TUI
- Use `invoke_tui()` for interactive full-screen apps
- Use `shell()` for background commands where you need the output

### syntropy.invoke_editor

Opens a file in the user's configured editor.

**Function signature:**
```lua
syntropy.invoke_editor(path: string) -> integer
```

**Parameters:**
- `path` (string) - Path to the file to edit

**Returns:**
- `exit_code` (integer) - Exit code from the editor (clamped to POSIX range 0-255)

**Behavior:**
- Automatically detects the user's preferred editor in this order:
  1. `$EDITOR` environment variable
  2. `$VISUAL` environment variable (fallback)
  3. `vim` (default fallback)
- **TUI mode:** Suspends syntropy's TUI, gives full terminal control to the editor, then restores syntropy's TUI when editor exits
- **CLI mode:** Runs editor directly with inherited stdio
- **Blocking:** The plugin execution pauses until the editor exits
- **State preservation:** All Lua variables and execution state are preserved; execution resumes exactly where it left off

**Use cases:**
- Edit configuration files
- Modify notes or todo lists
- Edit temporary files for user input
- Open files selected from plugin items
- Allow user to compose messages or descriptions

**Examples:**

```lua
-- Edit a configuration file
local code = syntropy.invoke_editor(syntropy.expand_path("~/.config/app/config.toml"))
if code == 0 then
    print("Configuration updated")
end

-- Edit plugin-local file
local config_path = syntropy.expand_path("./plugin_config.json")
local code = syntropy.invoke_editor(config_path)

-- Edit selected item from list
function M.execute(items)
    if #items == 0 then
        return "No items selected", 1
    end

    local file_path = items[1]
    local code = syntropy.invoke_editor(file_path)

    if code == 0 then
        return "File edited successfully", 0
    else
        return "Editor exited with error", code
    end
end

-- Create and edit temporary file
local tmp_file = "/tmp/syntropy_note_" .. os.time() .. ".txt"
local code = syntropy.invoke_editor(tmp_file)
if code == 0 then
    -- Read the file content
    local file = io.open(tmp_file, "r")
    if file then
        local content = file:read("*all")
        file:close()
        -- Process content...
    end
end

-- Edit with error handling
local code = syntropy.invoke_editor("document.txt")
if code ~= 0 then
    return "Editor exited with code " .. code, code
end
```

**Important notes:**
- Works with any editor: vim, nvim, emacs, nano, helix, etc.
- Respects user's editor preference via `$EDITOR`
- Blocking behavior allows sequential editing workflow
- Path can be relative or absolute, supports all `expand_path` features
- Exit code 0 typically means successful save, non-zero may indicate cancellation

**Comparison with syntropy.shell:**
```lua
-- ❌ DON'T use shell for editors (broken terminal control):
syntropy.shell("$EDITOR " .. file)

-- ✅ DO use invoke_editor (proper terminal control):
syntropy.invoke_editor(file)
```

## Standard Lua Library

Syntropy provides **Lua 5.4 standard library** with safety restrictions.

### Available Modules

All standard modules except restricted functions:

- `string` - String manipulation
- `table` - Table operations
- `math` - Mathematical functions
- `io` - File I/O
- `os` - OS utilities (**except** `os.exit`, `os.execute`)
- `debug` - Debugging utilities
- `coroutine` - Coroutine support
- `utf8` - UTF-8 operations

### Restricted Functions

These functions are **removed** for security:

- `os.exit()` - Would terminate syntropy
- `os.execute()` - Use `syntropy.shell()` instead

### Common Standard Library Usage

**File I/O:**
```lua
-- Read file
local file = io.open("path.txt", "r")
if file then
    local content = file:read("*all")
    file:close()
end

-- Write file
local file = io.open("path.txt", "w")
file:write("content\n")
file:close()
```

**String manipulation:**
```lua
-- Format strings
local msg = string.format("Processed %d items", count)

-- Pattern matching
local name = string.match(line, "name: (.+)")

-- Split string
for part in string.gmatch(text, "[^,]+") do
    print(part)
end
```

**Table operations:**
```lua
-- Insert/remove
table.insert(items, "new item")
table.remove(items, 1)

-- Sort
table.sort(items)

-- Concatenate
local csv = table.concat(items, ",")
```

**OS utilities:**
```lua
-- Environment variables
local home = os.getenv("HOME")
local user = os.getenv("USER")

-- Date/time
local timestamp = os.time()
local date = os.date("%Y-%m-%d")

-- File operations
os.rename("old.txt", "new.txt")
os.remove("file.txt")
```

## Global Utilities

### merge

Deep merge two tables (used for plugin overrides).

```lua
local result = merge(base, override)
```

**Parameters:**
- `base` (table) - Base configuration
- `override` (table) - Override configuration

**Returns:**
- `result` (table) - Merged table

**Merge Rules:**
- Objects: Deep merge (recursive)
- Arrays: Replace entirely (override replaces base)
- Primitives: Override wins

**Examples:**

```lua
-- Merge objects
local base = {a = 1, b = 2}
local override = {b = 3, c = 4}
local result = merge(base, override)
-- Result: {a = 1, b = 3, c = 4}

-- Arrays are replaced, not merged
local base = {items = {"a", "b"}}
local override = {items = {"c"}}
local result = merge(base, override)
-- Result: {items = {"c"}}

-- Nested merging
local base = {
    metadata = {name = "test", icon = "T"},
    config = {debug = false}
}
local override = {
    metadata = {icon = "X"},
    config = {verbose = true}
}
local result = merge(base, override)
-- Result: {
--   metadata = {name = "test", icon = "X"},
--   config = {debug = false, verbose = true}
-- }
```

