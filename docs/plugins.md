# Syntropy Plugins

Plugins are Lua scripts that define tasks for syntropy. This guide covers the plugin model, development workflow, and common patterns.

> **📖 API Reference:** For detailed type definitions, function signatures, and comprehensive specifications, see the [Plugin API Reference](plugin-api-reference.md).

## Table of Contents

- [Plugin Model Concepts](#plugin-model-concepts)
- [Quick Example](#quick-example)
- [Tutorial: Build a Bookmarks Plugin](#tutorial-build-a-bookmarks-plugin)
- [Common Patterns](#common-patterns)
- [Debugging](#debugging)

## Plugin Model Concepts

### What is a Plugin?

A plugin is a Lua script (`plugin.lua`) that defines:
- **Metadata**: Name, version, icon, description
- **Tasks**: User-facing operations (list items, execute actions)

Plugins are first-class citizens in syntropy:
- Auto-discovered from `~/.config/syntropy/plugins/` and `~/.local/share/syntropy/plugins/`
- Support merging (config overrides data)
- Hot-reloadable during development

### Plugin Structure

Every plugin is a single `plugin.lua` file returning a table:

```lua
---@type PluginDefinition
return {
    metadata = {
        name = "example",        -- Required: unique identifier
        version = "1.0.0",       -- Required: semver
        icon = "E",              -- Optional: single char (Unicode/Nerd Font OK, no emojis)
        description = "...",     -- Optional
        platforms = {"macos"},   -- Optional: filter by platform
    },
    tasks = {
        task_key = {
            -- Task definition here
        },
    },
}
```

### Plugin Lifecycle

```
Discovery → Loading → Validation → Execution → Rendering
```

**1. Discovery**
- Syntropy scans plugin directories on startup
- Finds all `plugin.lua` files
- Checks for duplicate names (merge candidates)

**2. Loading**
- Execute Lua script in sandboxed environment
- If duplicates exist: deep merge (config overrides data)
- Parse metadata and tasks

**3. Validation**
- Ensure required fields exist (`metadata.name`, `metadata.version`)
- Verify semver format
- Validate task structure
- Check icons occupy single terminal cell

**4. Execution** (when user selects task)
- Call `pre_run()` hook if defined
- Fetch items from all item sources via `items()` functions
- User selects items in TUI
- Call `execute()` with selected items
- Call `post_run()` hook if defined
- **TUI only:** Automatically refetch items (call `items()` again) after execution completes, regardless of success or error

**5. Rendering**
- TUI displays items, preview, execution output
- CLI prints output to stdout with exit code

### Task Model

Tasks define what users can do. Two main patterns:

**Pattern 1: Task with Item Sources** (select items → execute)
```lua
task_key = {
    name = "Display Name",
    description = "Task description shown in preview pane",
    mode = "multi",  -- "multi" (select many) or "none" (select one)
    item_sources = {
        source_key = {
            tag = "s",
            items = function() return {"item1", "item2"} end,
            execute = function(items) return "Done", 0 end,
        },
    },
}
```

**Pattern 2: Execute-Only Task** (no items, just run)
```lua
task_key = {
    name = "Do Something",
    description = "Execute a command without selecting items",
    execute = function()
        local out, code = syntropy.shell("echo hello")
        return out, code
    end,
}
```

> **For complete task specifications** including all optional fields (polling, confirmation, notifications, lifecycle hooks), see [Task Definition](plugin-api-reference-section-tasks.md).

### Item Sources

Item sources provide the data for tasks. Each source needs:
- `items()` function - Returns array of strings
- `tag` - Short identifier (required if task has multiple sources)

Tasks can combine data from multiple sources:

```lua
item_sources = {
    files = {
        tag = "f",
        items = function() return {"file1", "file2"} end,
    },
    folders = {
        tag = "d",
        items = function() return {"dir1", "dir2"} end,
    },
}
```

TUI displays items as `[f] file1`, `[d] dir1` for disambiguation.

> **For complete item source specifications** including optional fields (`preselected_items`, `preview`, `execute`) and validation rules, see [Item Source Definition](plugin-api-reference-section-item-sources.md).

### Task Modes

Tasks have different selection behaviors:

- **`mode = "multi"`** - Select multiple items (batch operations)
- **`mode = "none"`** - Single item selection with immediate execution
- **No item sources** - Execute-only tasks (no item selection)

> **For detailed mode behavior** including CLI vs TUI differences and execution patterns, see [Task Modes](plugin-api-reference-section-tasks.md#task-modes).

### Execution Confirmation

Tasks can display a confirmation dialog before execution to prevent accidental actions:

```lua
execution_confirmation_message = "Are you sure you want to delete:"
```

**How it works:**
- When set, a modal dialog appears immediately before execution
- Format: `"{execution_confirmation_message} {list of items to execute}"`
- User must confirm or cancel
- Only applies in TUI mode (CLI executes without confirmation)

**Example:**
```lua
tasks = {
    delete = {
        name = "Delete Files",
        description = "Delete selected files from the filesystem",
        mode = "multi",
        execution_confirmation_message = "Are you sure you want to delete:",
        item_sources = {
            files = {
                tag = "f",
                items = function() return {"file1.txt", "file2.txt"} end,
                execute = function(items)
                    for _, file in ipairs(items) do
                        syntropy.shell("rm " .. file)
                    end
                    return "Deleted " .. #items .. " files", 0
                end,
            },
        },
    },
}
```

If user selects `file1.txt` and `file2.txt`, the dialog shows:
```
Are you sure you want to delete: file1.txt, file2.txt
```

**Use for:**
- Destructive operations (delete, remove, uninstall)
- Non-reversible actions
- Operations with significant side effects

### Suppressing Success Notifications

Tasks can suppress the success modal that appears after successful execution by setting `suppress_success_notification = true`.

**How it works:**
- When set to `true`, no success modal is shown after the task completes successfully
- **Errors are still displayed** - silent failures are never acceptable
- Only applies in TUI mode (CLI always prints to stdout regardless)
- Default is `false` (success modals are shown)

**When to use:**
- Tasks that open external TUI applications (`invoke_editor`, `invoke_tui`)
- Tasks where the external application provides its own feedback
- Tasks where showing a success modal is redundant or interrupts the workflow
- Situations where the user already knows the task succeeded from context

**Example - Edit files without success modal:**

```lua
tasks = {
    edit = {
        name = "Edit File",
        description = "Open selected file in editor",
        mode = "none",
        suppress_success_notification = true,  -- No modal after editor closes

        item_sources = {
            files = {
                tag = "f",
                items = function()
                    local output, _ = syntropy.shell("ls *.md")
                    local files = {}
                    for line in output:gmatch("[^\n]+") do
                        table.insert(files, line)
                    end
                    return files
                end,
                execute = function(items)
                    local code = syntropy.invoke_editor(items[1])
                    if code == 0 then
                        return "File edited", 0
                    else
                        return "Editor exited with code " .. code, code
                    end
                end,
            },
        },
    },
}
```

In this example, when the user closes the editor:
- If successful (code 0): **No modal shown** - user returns to syntropy seamlessly
- If error (code ≠ 0): **Error modal shown** - user is informed of the problem

**Use for:**
- Editor integrations (`invoke_editor`)
- TUI application launchers (`invoke_tui` with htop, fzf, ranger, etc.)
- Tasks where the action itself provides clear feedback
- Workflows where modals would be disruptive

**Avoid using when:**
- The task output contains important information the user needs to see
- The task performs destructive or non-obvious operations
- The user needs confirmation that the task completed
- The success message contains useful data (counts, summaries, etc.)

### Lifecycle Hooks

**`pre_run()` (optional)**
- Runs before items are fetched each time the task executes
- **In TUI**: Called every time you navigate to the task screen (not just once)
- **In CLI**: Called once before items fetch
- Use for: cache invalidation, state initialization, setup, validation, notifications

**`post_run()` (optional)**
- Runs after execution completes
- **In TUI**: Called after each execution
- **In CLI**: Called once after execution
- Use for: cleanup, logging, notifications

```lua
local cache = {}

task_key = {
    pre_run = function()
        print("Starting task...")
        cache = {}  -- Reset cache on each screen entry
    end,
    post_run = function()
        print("Task complete!")
    end,
    item_sources = { ... },
}
```

**Important:** In TUI mode, plugins are loaded once at startup and persist until app exit. Module-level variables persist across all screen navigations, but `items()` is called fresh every time you enter/re-enter the task screen. This makes `pre_run()` ideal for cache invalidation.

> **For complete lifecycle hook specifications** including CLI vs TUI differences, see [Lifecycle Hooks](plugin-api-reference-section-tasks.md#lifecycle-hooks).

### Post-Execution Item Refresh

**Automatic item refresh after execution (TUI mode only):**

When a task's `execute()` function completes in TUI mode, syntropy automatically calls `items()` to refresh the item list. This happens for **both successful and failed executions**.

**Behavior:**
- ✅ **Happens in TUI mode** - Items automatically refresh after execution
- ❌ **Does NOT happen in CLI mode** - One-shot execution, no refresh
- ✅ **Happens on success AND error** - Refresh occurs regardless of exit code
- ❌ **Does NOT call `pre_run()`** - Only `items()` is called, not the full lifecycle
- ✅ **Module-level caches persist** - Only `items()` runs fresh, module state remains

**Why this matters:**

This enables operations that modify underlying data to immediately show updated state:
- **Delete operations** - Removed items disappear from the list
- **Add operations** - New items appear automatically
- **Update operations** - Modified items reflect changes
- **Toggle operations** - State changes are visible
- **Any data mutation** - Changes are reflected without manual refresh

**Example - Delete with auto-refresh:**

```lua
tasks = {
    delete = {
        name = "Delete Files",
        description = "Delete selected files (list auto-refreshes)",
        mode = "multi",
        item_sources = {
            files = {
                tag = "f",
                items = function()
                    -- This is called:
                    -- 1. When entering the task screen
                    -- 2. After each execution completes
                    local output, _ = syntropy.shell("ls *.txt")
                    local files = {}
                    for line in output:gmatch("[^\n]+") do
                        table.insert(files, line)
                    end
                    return files
                end,
                execute = function(items)
                    for _, file in ipairs(items) do
                        syntropy.shell("rm " .. file)
                    end
                    return "Deleted " .. #items .. " files", 0
                    -- After returning, items() is automatically called again
                    -- The deleted files will no longer appear in the list
                end,
            },
        },
    },
}
```

**Example - Add with auto-refresh:**

```lua
tasks = {
    add = {
        name = "Add Bookmark",
        description = "Add a bookmark (list auto-refreshes to show new item)",
        execute = function()
            -- Prompt for URL
            local url = "https://example.com"  -- Simplified

            -- Add to file
            local file = io.open(syntropy.expand_path("~/.bookmarks"), "a")
            file:write(url .. "\n")
            file:close()

            return "Added bookmark", 0
            -- items() will be called automatically
        end,
    },
    list = {
        name = "Browse Bookmarks",
        mode = "none",
        item_sources = {
            urls = {
                items = function()
                    -- This refreshes after add task completes
                    -- (if user navigates back to this task)
                    local file = io.open(syntropy.expand_path("~/.bookmarks"), "r")
                    if not file then return {} end
                    local bookmarks = {}
                    for line in file:lines() do
                        table.insert(bookmarks, line)
                    end
                    file:close()
                    return bookmarks
                end,
            },
        },
    },
}
```

**Cache considerations:**

Since `pre_run()` is NOT called during post-execution refresh, module-level caches are not reset:

```lua
local cache = {}  -- Module-level cache

tasks = {
    my_task = {
        pre_run = function()
            -- Called when entering screen, NOT after execution
            cache = {}
        end,

        item_sources = {
            source = {
                items = function()
                    -- Called both:
                    -- - When entering screen (after pre_run)
                    -- - After execution (WITHOUT pre_run)

                    -- If you need fresh data, query it here
                    local fresh_data = fetch_from_source()
                    return fresh_data
                end,
            },
        },
    },
}
```

**Best practices:**

1. **Don't rely on stale caches** - If `items()` uses cached data, ensure it's fresh or re-query
2. **Stateless is safer** - Query fresh data in `items()` rather than relying on module-level caches
3. **Use for mutations** - Design delete/add/update operations knowing the list will refresh
4. **Combine with polling** - Use `item_polling_interval` for external changes, post-execution refresh handles operation results

### Automatic Polling

Tasks can automatically refresh their data at regular intervals, perfect for monitoring dynamic content.

**When to use polling:**
- **Process monitors** - Track running processes, CPU/memory usage
- **Active window lists** - Monitor currently open windows/applications
- **File watchers** - Detect file system changes
- **System stats** - Display real-time metrics (disk usage, network traffic)
- **Log viewers** - Show live log updates
- **Any dynamic data** - Content that changes externally over time

**Configuration:**

```lua
task_key = {
    name = "Process Monitor",
    description = "Monitor and manage running processes",
    item_polling_interval = 2000,       -- Refresh items every 2 seconds
    preview_polling_interval = 1000,    -- Refresh preview every 1 second
    item_sources = { ... },
}
```

**How it works:**
- **`item_polling_interval`** - Milliseconds between automatic `items()` calls (0 = disabled)
- **`preview_polling_interval`** - Milliseconds between automatic `preview()` calls (0 = disabled)
- **Preserves** - Search query and selected item position during refreshes
- **Smart timing** - Only polls when not already executing an operation
- **Stable identities** - Item strings must remain consistent for selection persistence to work

**Performance considerations:**
- Use reasonable intervals (1000ms+ for most use cases)
- Very small intervals (< 100ms) may impact TUI responsiveness
- Consider the cost of your data fetching operations
- Disable polling (set to 0) when data doesn't change dynamically

> **For complete polling specifications** including item identity stability and performance guidelines, see [Automatic Polling](plugin-api-reference-section-tasks.md#automatic-polling).

**Example:**

```lua
tasks = {
    windows = {
        name = "Active Windows",
        description = "Switch to an active window",
        mode = "none",
        item_polling_interval = 1500,  -- Refresh window list every 1.5 seconds

        item_sources = {
            win = {
                tag = "w",
                items = function()
                    local output, _ = syntropy.shell("wmctrl -l")
                    local windows = {}
                    for line in output:gmatch("[^\n]+") do
                        table.insert(windows, line)
                    end
                    return windows
                end,
                execute = function(items)
                    -- Switch to selected window
                    local win_id = items[1]:match("^(0x%x+)")
                    syntropy.shell("wmctrl -i -a " .. win_id)
                    return "Switched to window", 0
                end,
            },
        },
    },
}
```

### Configuration and Merging

Plugins in `~/.config/syntropy/plugins/` can override plugins in `~/.local/share/syntropy/plugins/` using deep merge:

```lua
---@type PluginOverride
return {
    metadata = {name = "pkg", icon = "K"},  -- Change icon only
    tasks = {
        export = {name = "My Export"},  -- Override task name
        new_task = {...},  -- Add new task
    },
}
```

**Merge behavior:** Objects are deep merged, arrays replaced, overrides win.

> **For complete merge rules and configuration patterns**, see [Plugin Configuration](plugin-api-reference-section-data-structures.md#plugin-configuration).

### Base Plugins vs Override Plugins

Syntropy supports two plugin types:

**`---@type PluginDefinition`** - For creating new plugins from scratch
- Both `metadata` and `tasks` are required
- Location: `~/.local/share/syntropy/plugins/` or `~/.config/syntropy/plugins/`

**`---@type PluginOverride`** - For customizing existing plugins
- All fields optional - specify only what you want to change
- Must match base plugin's `metadata.name`
- Location: typically `~/.config/syntropy/plugins/` (to override data directory plugins)

**Quick example - Override just the icon:**
```lua
---@type PluginOverride
return {
    metadata = {
        name = "pkg",  -- Must match base plugin
        icon = "📦",   -- Override just the icon
    },
    -- All other fields inherited from base plugin
}
```

> **For complete override patterns and deep merge behavior**, see [Plugin Configuration](plugin-api-reference-section-data-structures.md#plugin-configuration) and [PluginOverride examples](plugin-api-reference-section-examples.md).

## Quick Example: Base Plugin

**Minimal working plugin** (`~/.config/syntropy/plugins/hello/plugin.lua`):

This is a base plugin (not an override). It defines all required fields from scratch.

```lua
---@type PluginDefinition
return {
    metadata = {
        name = "hello",
        version = "1.0.0",
    },
    tasks = {
        greet = {
            name = "Say Hello",
            description = "Greet people by name",
            item_sources = {
                names = {
                    tag = "n",
                    items = function()
                        return {"Alice", "Bob", "Charlie"}
                    end,
                    execute = function(items)
                        for _, name in ipairs(items) do
                            print("Hello, " .. name .. "!")
                        end
                        return "Greeted " .. #items .. " people", 0
                    end,
                },
            },
        },
    },
}
```

**Usage:**
```bash
# TUI mode: Select names and execute
syntropy

# CLI mode: Execute directly
syntropy execute --plugin hello --task greet
```

## Tutorial: Build a Bookmarks Plugin

We'll build a plugin to manage URL bookmarks. Features:
- List bookmarks
- Add new bookmarks
- Delete bookmarks
- Open in browser

**Note:** This tutorial builds a **base plugin** (not an override). For overriding existing plugins, use `---@type PluginOverride` and provide only the fields you want to change. See the [Base Plugins vs Override Plugins](#base-plugins-vs-override-plugins) section for details.

### Step 1: Create Plugin Structure

```bash
mkdir -p ~/.config/syntropy/plugins/bookmarks
cd ~/.config/syntropy/plugins/bookmarks
```

Create `plugin.lua`:

```lua
---@type PluginDefinition
local bookmarks = {
    metadata = {
        name = "bookmarks",
        version = "1.0.0",
        icon = "B",
        description = "Manage URL bookmarks",
        platforms = {"macos", "linux"},
    },
    tasks = {},
}

return bookmarks
```

### Step 2: Define Data Storage

Add helper functions:

```lua
---@type PluginDefinition
local bookmarks = {
    metadata = { ... },
    tasks = {},
}

-- Helper functions (not exposed to syntropy)
local function get_bookmarks_file()
    local home = os.getenv("HOME")
    return home .. "/.local/share/syntropy/bookmarks.txt"
end

local function read_bookmarks()
    local file = io.open(get_bookmarks_file(), "r")
    if not file then return {} end

    local lines = {}
    for line in file:lines() do
        if line ~= "" then
            table.insert(lines, line)
        end
    end
    file:close()
    return lines
end

local function write_bookmarks(bookmarks_list)
    local file = io.open(get_bookmarks_file(), "w")
    if not file then
        error("Failed to write bookmarks file")
    end

    for _, bookmark in ipairs(bookmarks_list) do
        file:write(bookmark .. "\n")
    end
    file:close()
end

return bookmarks
```

### Step 3: Add "List" Task

```lua
tasks = {
    list = {
        name = "Browse Bookmarks",
        description = "Open a saved bookmark in your browser",
        mode = "none",
        item_sources = {
            urls = {
                tag = "b",
                items = function()
                    return read_bookmarks()
                end,
                preview = function(item)
                    return "URL: " .. item
                end,
                execute = function(items)
                    local url = items[1]
                    local output, code = syntropy.shell("open " .. url)
                    return "Opened: " .. url, code
                end,
            },
        },
    },
},
```

### Step 4: Add "Add Bookmark" Task

```lua
tasks = {
    list = { ... },

    add = {
        name = "Add Bookmark",
        description = "Add a new bookmark to the list",
        execute = function()
            -- Prompt user for URL (in real plugin, pass via CLI args)
            print("Enter URL to bookmark:")
            local url = io.read()

            if not url or url == "" then
                return "No URL provided", 1
            end

            -- Read existing bookmarks
            local current = read_bookmarks()

            -- Check for duplicates
            for _, bookmark in ipairs(current) do
                if bookmark == url then
                    return "Already bookmarked: " .. url, 1
                end
            end

            -- Add and save
            table.insert(current, url)
            write_bookmarks(current)

            return "Added: " .. url, 0
        end,
    },
},
```

### Step 5: Add "Delete" Task

```lua
tasks = {
    list = { ... },
    add = { ... },

    delete = {
        name = "Delete Bookmarks",
        description = "Remove bookmarks from the list",
        mode = "multi",
        item_sources = {
            urls = {
                tag = "b",
                items = function()
                    return read_bookmarks()
                end,
                execute = function(items)
                    local current = read_bookmarks()

                    -- Filter out deleted items
                    local new_bookmarks = {}
                    for _, bookmark in ipairs(current) do
                        local should_keep = true
                        for _, to_delete in ipairs(items) do
                            if bookmark == to_delete then
                                should_keep = false
                                break
                            end
                        end
                        if should_keep then
                            table.insert(new_bookmarks, bookmark)
                        end
                    end

                    write_bookmarks(new_bookmarks)
                    return "Deleted " .. #items .. " bookmark(s)", 0
                end,
            },
        },
    },
},
```

### Step 6: Test the Plugin

```bash
# Validate plugin structure
syntropy validate --plugin ~/.config/syntropy/plugins/bookmarks/plugin.lua

# Run in TUI
syntropy

# Or test CLI execution
syntropy execute --plugin bookmarks --task add
```

### Complete Plugin

<details>
<summary>Click to see full bookmarks plugin code</summary>

```lua
---@type PluginDefinition
local bookmarks = {
    metadata = {
        name = "bookmarks",
        version = "1.0.0",
        icon = "B",
        description = "Manage URL bookmarks",
        platforms = {"macos", "linux"},
    },
    tasks = {
        list = {
            name = "Browse Bookmarks",
            description = "Open a saved bookmark in your browser",
            mode = "none",
            item_sources = {
                urls = {
                    tag = "b",
                    items = function() return read_bookmarks() end,
                    preview = function(item) return "URL: " .. item end,
                    execute = function(items)
                        local url = items[1]
                        local output, code = syntropy.shell("open " .. url)
                        return "Opened: " .. url, code
                    end,
                },
            },
        },
        add = {
            name = "Add Bookmark",
            description = "Add a new bookmark to the list",
            execute = function()
                print("Enter URL:")
                local url = io.read()
                if not url or url == "" then return "No URL", 1 end

                local current = read_bookmarks()
                for _, b in ipairs(current) do
                    if b == url then return "Duplicate", 1 end
                end

                table.insert(current, url)
                write_bookmarks(current)
                return "Added: " .. url, 0
            end,
        },
        delete = {
            name = "Delete Bookmarks",
            description = "Remove bookmarks from the list",
            mode = "multi",
            item_sources = {
                urls = {
                    tag = "b",
                    items = function() return read_bookmarks() end,
                    execute = function(items)
                        local current = read_bookmarks()
                        local new = {}
                        for _, b in ipairs(current) do
                            local keep = true
                            for _, d in ipairs(items) do
                                if b == d then keep = false break end
                            end
                            if keep then table.insert(new, b) end
                        end
                        write_bookmarks(new)
                        return "Deleted " .. #items, 0
                    end,
                },
            },
        },
    },
}

-- Helper functions
local function get_bookmarks_file()
    return os.getenv("HOME") .. "/.local/share/syntropy/bookmarks.txt"
end

local function read_bookmarks()
    local file = io.open(get_bookmarks_file(), "r")
    if not file then return {} end
    local lines = {}
    for line in file:lines() do
        if line ~= "" then table.insert(lines, line) end
    end
    file:close()
    return lines
end

local function write_bookmarks(list)
    local file = io.open(get_bookmarks_file(), "w")
    if not file then error("Failed to write") end
    for _, b in ipairs(list) do file:write(b .. "\n") end
    file:close()
end

return bookmarks
```

</details>

## Common Patterns

### Calling Shell Commands

```lua
-- Simple command
local output, exit_code = syntropy.shell("ls -la")

-- With error handling
local out, code = syntropy.shell("git status")
if code ~= 0 then
    return "Git command failed: " .. out, code
end

-- Multi-line commands
local script = [[
cd ~/project
git pull origin main
cargo build --release
]]
local output, code = syntropy.shell(script)
```

### Using Automatic Polling for Real-Time Data

For plugins that display dynamic data, use polling to automatically refresh content without user interaction.

**Pattern: Process Monitor with Live Updates**

```lua
tasks = {
    processes = {
        name = "Process Monitor",
        description = "Monitor and manage running processes with real-time updates",
        mode = "multi",
        item_polling_interval = 2000,       -- Refresh list every 2 seconds
        preview_polling_interval = 1000,    -- Refresh details every 1 second

        item_sources = {
            procs = {
                tag = "p",
                items = function()
                    -- This runs automatically every 2 seconds
                    local output, _ = syntropy.shell("ps aux --sort=-%cpu | head -20 | tail -n +2")
                    local processes = {}
                    for line in output:gmatch("[^\n]+") do
                        local user, pid, cpu, mem, cmd = line:match("^(%S+)%s+(%d+)%s+([%d.]+)%s+([%d.]+)%s+.*%s+(.+)$")
                        if pid then
                            table.insert(processes, string.format("[%s] %s - CPU: %s%%, MEM: %s%%", pid, cmd, cpu, mem))
                        end
                    end
                    return processes
                end,

                preview = function(item)
                    -- This runs automatically every 1 second for selected item
                    local pid = item:match("^%[(%d+)%]")
                    if not pid then return "No process selected" end

                    local output, code = syntropy.shell("ps -p " .. pid .. " -o pid,user,%cpu,%mem,vsz,rss,stat,start,time,command")
                    if code ~= 0 then
                        return "Process no longer exists"
                    end
                    return output
                end,

                execute = function(items)
                    local killed = 0
                    for _, item in ipairs(items) do
                        local pid = item:match("^%[(%d+)%]")
                        if pid then
                            local _, code = syntropy.shell("kill " .. pid)
                            if code == 0 then killed = killed + 1 end
                        end
                    end
                    return "Killed " .. killed .. " processes", 0
                end,
            },
        },
    },
}
```

**Pattern: Live Log Viewer**

```lua
tasks = {
    logs = {
        name = "System Logs",
        description = "View and monitor system log files in real-time",
        mode = "none",
        item_polling_interval = 5000,  -- Refresh log list every 5 seconds

        item_sources = {
            logfiles = {
                tag = "l",
                items = function()
                    local output, _ = syntropy.shell("ls -t /var/log/*.log 2>/dev/null | head -10")
                    local logs = {}
                    for line in output:gmatch("[^\n]+") do
                        table.insert(logs, line)
                    end
                    return logs
                end,

                preview = function(item)
                    -- Show last 50 lines of log file
                    local output, _ = syntropy.shell("tail -50 " .. item)
                    return output
                end,

                execute = function(items)
                    -- Open log in editor
                    local code = syntropy.invoke_editor(items[1])
                    return "Opened log file", code
                end,
            },
        },
    },
}
```

**Best Practices:**

- **Choose appropriate intervals:**
  - Fast-changing data (CPU, memory): 1000-2000ms
  - Moderate updates (window lists, files): 2000-5000ms
  - Slow-changing data (disk usage, logs): 5000-10000ms

- **Keep item identities stable:**

  Item strings serve as identity keys for tracking selections across polls. Dynamic data breaks persistence.

  - ❌ **Avoid**: `"pid: 300 | cpu: 23% | tmux"` (CPU changes = different item, selection lost)
  - ✅ **Use**: `"[300] tmux"` or `"300 tmux"` (stable across polls, selection preserved)
  - Display dynamic data (CPU%, memory, status) in the **preview** pane
  - Sort items by dynamic properties if needed (highest CPU first, etc.)

  **Example - Stable process items:**
  ```lua
  items = function()
      local output = syntropy.shell("ps aux --sort=-%cpu | head -20")
      local procs = {}
      for line in output:gmatch("[^\n]+") do
          local pid, cpu, cmd = line:match("^%S+%s+(%d+)%s+([%d.]+)%s+.-%s+(.+)$")
          -- ✅ Use stable PID + command as item (not CPU%)
          table.insert(procs, string.format("[%s] %s", pid, cmd))
      end
      return procs
  end,

  preview = function(item)
      -- Show dynamic CPU% in preview instead
      local pid = item:match("^%[(%d+)%]")
      local output = syntropy.shell("ps -p " .. pid .. " -o %cpu,%mem,time,command")
      return output
  end
  ```

- **Handle missing data gracefully:**
  ```lua
  items = function()
      local output, code = syntropy.shell("get-data")
      if code ~= 0 then
          return {"Error fetching data"}
      end
      -- Parse and return items
  end
  ```

- **Preserve user experience:**
  - Search queries are automatically preserved
  - Selected items are maintained when possible
  - Use consistent formatting for stable item identities

- **Optimize performance:**
  - Cache expensive computations within poll cycles
  - Use efficient shell commands (avoid processing large outputs)
  - Consider disabling preview polling if previews are expensive

### Launching External TUI Applications

Plugins can interrupt execution to launch external TUI (Text User Interface) applications with full terminal control. This is useful for integrating editors, file managers, process monitors, fuzzy finders, and other interactive tools.

**Key Concepts:**

- **Blocking is intentional:** When you launch an external TUI app, the plugin execution pauses and the syntropy TUI suspends
- **State preservation:** All Lua variables and execution state are preserved; execution resumes exactly where it left off
- **Complete terminal control:** The external app receives full terminal access (raw mode, alternate screen, stdin/stdout/stderr)
- **Exit code handling:** Functions return the exit code so you can detect success/cancellation

> **For complete syntropy.invoke_editor() and syntropy.invoke_tui() specifications**, see [API Functions](plugin-api-reference-section-api-functions.md).

#### Using syntropy.invoke_editor()

The most common use case is opening files in the user's configured editor:

```lua
return {
    plugin = "notes",
    tasks = {
        edit = {
            description = "Edit note files",
            suppress_success_notification = true,  -- No success modal after editor closes

            items = function()
                local notes_dir = syntropy.expand_path("~/notes")
                local output, _ = syntropy.shell("ls " .. notes_dir .. "/*.md")
                local files = {}
                for line in output:gmatch("[^\n]+") do
                    table.insert(files, line)
                end
                return files
            end,

            execute = function(items)
                if #items == 0 then
                    return "No files selected", 1
                end

                -- Open file in user's editor ($EDITOR, $VISUAL, or vim)
                local code = syntropy.invoke_editor(items[1])

                if code == 0 then
                    return "File edited successfully", 0
                else
                    return "Editor exited with code " .. code, code
                end
            end,
        },
    },
}
```

**Multi-file editing:**

```lua
execute = function(items)
    local edited_count = 0

    for _, file in ipairs(items) do
        local code = syntropy.invoke_editor(file)
        if code == 0 then
            edited_count = edited_count + 1
        else
            return "Cancelled at " .. file, code
        end
    end

    return "Edited " .. edited_count .. " files", 0
end
```

#### Using syntropy.invoke_tui()

For more general TUI applications, use `syntropy.invoke_tui()`:

**Example: File selection with fzf**

```lua
execute = function(items)
    -- Let user select files with fzf
    local code = syntropy.invoke_tui("fzf", {
        "--multi",
        "--preview", "bat --color=always {}",
        "--preview-window", "right:60%"
    })

    if code == 0 then
        return "Files selected", 0
    else
        return "Selection cancelled", code
    end
end
```

**Example: Process monitor integration**

```lua
return {
    plugin = "system",
    tasks = {
        processes = {
            description = "Monitor system processes",
            suppress_success_notification = true,  -- No modal after htop exits
            execute = function()
                -- Launch htop for interactive process management
                local code = syntropy.invoke_tui("htop", {})
                return "Process monitor closed", code
            end,
        },
    },
}
```

**Example: Git TUI integration**

```lua
return {
    plugin = "git",
    tasks = {
        history = {
            description = "Browse git history",
            suppress_success_notification = true,  -- No modal after tig exits
            execute = function()
                -- Launch tig for git history browsing
                local code = syntropy.invoke_tui("tig", {"--all"})
                return "Git browser closed", code
            end,
        },

        interactive_commit = {
            description = "Interactive staging",
            suppress_success_notification = true,  -- No modal after lazygit exits
            execute = function()
                -- Launch lazygit for interactive git operations
                local code = syntropy.invoke_tui("lazygit", {})
                return "Git operations complete", code
            end,
        },
    },
}
```

**Example: File manager integration**

```lua
tasks = {
    browse = {
        description = "Browse files with ranger",
        suppress_success_notification = true,  -- No modal after ranger exits
        execute = function()
            local dir = syntropy.expand_path("~/projects")

            -- Launch ranger file manager
            local code = syntropy.invoke_tui("ranger", {dir})

            if code == 0 then
                return "File manager closed", 0
            else
                return "File manager exited with error", code
            end
        end,
    },
}
```

#### Handling Temporary Files

A common pattern is to create a temporary file, let the user edit it, then read the result:

```lua
execute = function()
    -- Create temporary file for user input
    local tmp_file = "/tmp/syntropy_input_" .. os.time() .. ".txt"

    -- Pre-populate with template
    local template = "# Enter your notes here\n\n"
    local file = io.open(tmp_file, "w")
    file:write(template)
    file:close()

    -- Let user edit
    local code = syntropy.invoke_editor(tmp_file)

    if code ~= 0 then
        return "Edit cancelled", code
    end

    -- Read the edited content
    file = io.open(tmp_file, "r")
    local content = file:read("*all")
    file:close()

    -- Process the content
    -- ... do something with content ...

    -- Cleanup
    os.remove(tmp_file)

    return "Notes saved", 0
end
```

#### Error Handling

Always handle exit codes to detect cancellation or errors:

```lua
execute = function(items)
    local file = items[1]
    local code = syntropy.invoke_editor(file)

    if code == 0 then
        -- Success - user saved the file
        return "Changes saved", 0
    elseif code == 1 then
        -- Common exit code for "quit without saving"
        return "No changes made", 0
    else
        -- Other error
        return "Editor failed with code " .. code, code
    end
end
```

#### Best Practices

- **Validate paths before editing:**
  ```lua
  local file_path = syntropy.expand_path("./config.json")
  local file = io.open(file_path, "r")
  if not file then
      return "File not found: " .. file_path, 1
  end
  file:close()
  syntropy.invoke_editor(file_path)
  ```

- **Use invoke_editor for files, invoke_tui for everything else:**
  - ✅ `syntropy.invoke_editor(file)` - Respects user's $EDITOR
  - ❌ `syntropy.invoke_tui("vim", {file})` - Ignores user preference

- **Don't mix with syntropy.shell for terminal apps:**
  - ❌ `syntropy.shell("$EDITOR " .. file)` - Broken terminal control
  - ✅ `syntropy.invoke_editor(file)` - Proper terminal handoff

- **Check command availability:**
  ```lua
  local _, code = syntropy.shell("command -v fzf")
  if code ~= 0 then
      return "fzf not installed", 1
  end
  syntropy.invoke_tui("fzf", {"--multi"})
  ```

- **Provide meaningful error messages:**
  ```lua
  if code ~= 0 then
      return "Editor exited without saving (code: " .. code .. ")", code
  end
  ```

### Reading and Writing Files

```lua
-- Read file
local function read_file(path)
    local file = io.open(path, "r")
    if not file then return nil end
    local content = file:read("*all")
    file:close()
    return content
end

-- Write file
local function write_file(path, content)
    local file = io.open(path, "w")
    if not file then error("Cannot write to " .. path) end
    file:write(content)
    file:close()
end

-- Append to file
local function append_file(path, line)
    local file = io.open(path, "a")
    file:write(line .. "\n")
    file:close()
end
```

### Parsing Structured Data

```lua
-- Parse CSV-like data
local function parse_line(line)
    local parts = {}
    for part in string.gmatch(line, "[^,]+") do
        table.insert(parts, part)
    end
    return parts
end

-- Parse key=value format
local function parse_env(text)
    local vars = {}
    for line in string.gmatch(text, "[^\n]+") do
        local key, value = string.match(line, "(.+)=(.+)")
        if key then vars[key] = value end
    end
    return vars
end

```

### Organizing Code with Modules

For complex plugins, you can split code across multiple Lua files using `require()`.

**Important:** As of v0.3.4, syntropy uses Neovim-style module structure with mandatory namespacing for plugin modules.

> **For complete module loading specifications** including search paths and precedence rules, see [Advanced Topics - Module Loading](plugin-api-reference-section-advanced.md).

**Plugin-specific modules** (`lua/` directory):

Syntropy supports two module patterns:

**1. Standard module files** (`lua/pluginname/module.lua`):
```lua
-- Create: ~/.local/share/syntropy/plugins/my-plugin/lua/my-plugin/utils.lua
local utils = {}

function utils.parse_csv(line)
    local parts = {}
    for part in string.gmatch(line, "[^,]+") do
        table.insert(parts, part)
    end
    return parts
end

return utils
```

**2. Directory-style modules** (`lua/pluginname/module/init.lua`):
```lua
-- Create: ~/.local/share/syntropy/plugins/my-plugin/lua/my-plugin/parser/init.lua
local parser = {}

function parser.parse(data)
    return {parsed = data}
end

return parser
```

Use in your plugin with **namespaced imports**:
```lua
-- In ~/.local/share/syntropy/plugins/my-plugin/plugin.lua
-- MUST use namespaced require with plugin name
local utils = require("my-plugin.utils")      -- Loads from lua/my-plugin/utils.lua
local parser = require("my-plugin.parser")    -- Loads from lua/my-plugin/parser/init.lua

tasks = {
    process = {
        description = "Process CSV data using utility functions",
        execute = function()
            local data = utils.parse_csv("a,b,c")
            return "Parsed: " .. #data .. " items", 0
        end,
    },
}
```

### Module Import Rules

**Plugin-specific modules** (in plugin's `lua/` directory):
- MUST be imported with plugin namespace: `require("pluginname.module")`
- This ensures plugin isolation and prevents naming conflicts
- Example: For plugin "myapp" with `lua/myapp/utils.lua`, use `require("myapp.utils")`

**Why namespacing?**
- Prevents module name conflicts between plugins
- Makes code more explicit about dependencies
- Follows Neovim-style module organization pattern

**Complete example** - Split plugin across files:

```lua
-- lua/example/formatter.lua
local formatter = {}

function formatter.format_list(items)
    return "Items:\n- " .. table.concat(items, "\n- ")
end

return formatter
```

```lua
-- plugin.lua
local formatter = require("example.formatter")  -- Namespaced import required

return {
    metadata = {name = "example", version = "1.0.0"},
    tasks = {
        show = {
            description = "Display formatted list of items",
            execute = function()
                local items = {"one", "two", "three"}
                return formatter.format_list(items), 0
            end,
        },
    },
}
```

### Multi-Step Actions

```lua
execute = function(items)
    -- Step 1: Validate
    if #items == 0 then
        return "No items selected", 1
    end

    -- Step 2: Process each item
    local results = {}
    for _, item in ipairs(items) do
        local out, code = syntropy.shell("process " .. item)
        if code ~= 0 then
            return "Failed on " .. item .. ": " .. out, code
        end
        table.insert(results, item)
    end

    -- Step 3: Summary
    return "Processed " .. #results .. " items", 0
end
```

### Conditional Logic

```lua
-- Platform-specific behavior
local function get_open_command()
    local platform = os.getenv("OS") or ""
    if platform:match("Windows") then
        return "start"
    else
        return "open"  -- macOS/Linux (xdg-open on Linux)
    end
end

-- Check file existence
local function file_exists(path)
    local file = io.open(path, "r")
    if file then
        file:close()
        return true
    end
    return false
end

-- Conditional item sources
items = function()
    if file_exists("/etc/debian_version") then
        return {"apt", "dpkg"}
    elseif file_exists("/etc/redhat-release") then
        return {"yum", "rpm"}
    else
        return {"brew"}
    end
end
```

### Error Handling in Lua

```lua
-- Using pcall for error handling
execute = function(items)
    local success, result = pcall(function()
        local data = read_file("/etc/config")
        return process_data(data)
    end)

    if not success then
        return "Error: " .. tostring(result), 1
    end

    return "Success: " .. result, 0
end

-- Explicit error checking
execute = function(items)
    local file = io.open("data.txt", "r")
    if not file then
        return "File not found", 1
    end

    local content = file:read("*all")
    file:close()

    if content == "" then
        return "File is empty", 1
    end

    return "Read " .. #content .. " bytes", 0
end
```

### Managing State and Caches

Plugins in syntropy have persistent state across task executions. Understanding the lifecycle helps you manage caches and state effectively.

#### State Persistence Model

**Key behaviors:**
- Plugins are loaded **once at startup** and persist until app exit
- Module-level variables persist across **all** screen navigations
- `items()` is called **fresh** on every screen entry (both TUI and CLI)
- `items()` is **always called before execute()** (guaranteed in both modes)
- In TUI: Rust screen state is cleared on exit, but Lua state persists
- In CLI: Plugin executes once and exits

```lua
-- Module-level variable persists across ALL executions
local persistent_cache = {}
local call_count = 0

tasks = {
    demo = {
        description = "Demonstrate persistent cache across executions",
        item_sources = {
            source = {
                items = function()
                    call_count = call_count + 1
                    print("Items called, count: " .. call_count)
                    -- This will increment each time you enter the screen
                    return {"item1", "item2"}
                end,
            },
        },
    },
}
```

#### Cache Management Patterns

When building plugins that cache data for use in both `items()` and `execute()`, you have three approaches:

**Pattern 1: Reset Cache in pre_run()** (Recommended for clarity)

Best when you want explicit cache lifecycle management.

```lua
local cache = {}

tasks = {
    my_task = {
        description = "Process items using cached data",
        pre_run = function()
            -- Called every time screen is entered (TUI) or before execution (CLI)
            cache = {}  -- Clear cache before fetching items
        end,

        item_sources = {
            source = {
                items = function()
                    -- Fetch and populate cache
                    local data = fetch_expensive_data()
                    for _, item in ipairs(data) do
                        cache[item.id] = item
                    end
                    return extract_display_names(data)
                end,

                execute = function(items)
                    -- Use cached data (populated in items())
                    for _, item_name in ipairs(items) do
                        local full_data = cache[item_name]
                        process(full_data)
                    end
                    return "Processed " .. #items, 0
                end,
            },
        },
    },
}
```

**Pros:**
- Clear separation of concerns (lifecycle vs logic)
- Easy to understand when cache resets
- Works identically in TUI and CLI

**Cons:**
- Extra function definition
- Cache reset logic is separate from data fetching

**Pattern 2: Reset Cache in items()** (Simpler)

Best for straightforward cases where cache reset and population happen together.

```lua
local cache = {}

tasks = {
    my_task = {
        description = "Process items with inline cache reset",
        item_sources = {
            source = {
                items = function()
                    -- Reset and populate cache inline
                    cache = {}  -- Clear before fetching

                    local data = fetch_expensive_data()
                    for _, item in ipairs(data) do
                        cache[item.id] = item
                    end
                    return extract_display_names(data)
                end,

                execute = function(items)
                    -- Use cache populated in items()
                    for _, item_name in ipairs(items) do
                        local full_data = cache[item_name]
                        process(full_data)
                    end
                    return "Processed " .. #items, 0
                end,
            },
        },
    },
}
```

**Pros:**
- Simpler (one less function)
- Cache reset happens exactly when data is fetched
- Guaranteed fresh data (items() always called first)

**Cons:**
- Mixes cache management with data fetching
- Less obvious that cache is being reset

**Pattern 3: Stateless Re-query** (Best Practice)

Best approach for data that's cheap to fetch or when you want guaranteed consistency.

```lua
-- No shared cache needed
local function fetch_items()
    -- Fetch data fresh each time
    local output, _ = syntropy.shell("list-items")
    return parse_output(output)
end

tasks = {
    my_task = {
        description = "Process items with fresh data each time",
        item_sources = {
            source = {
                items = function()
                    return fetch_items():map(function(item) return item.name end)
                end,

                execute = function(items)
                    -- Re-query fresh data (no stale cache risk)
                    local all_data = fetch_items()

                    -- Find selected items in fresh data
                    for _, item_name in ipairs(items) do
                        for _, data_item in ipairs(all_data) do
                            if data_item.name == item_name then
                                process(data_item)
                                break
                            end
                        end
                    end
                    return "Processed " .. #items, 0
                end,
            },
        },
    },
}
```

**Pros:**
- No stale cache issues
- Simpler mental model (no shared state)
- Always works correctly
- Best for data that changes externally

**Cons:**
- Fetches data twice (once for items, once for execute)
- May be slower if data fetch is expensive

#### When to Use Each Pattern

| Pattern | Use When | Avoid When |
|---------|----------|------------|
| **pre_run() reset** | Cache logic is complex, needs explicit control | Simple caching scenarios |
| **items() reset** | Cache and fetch naturally go together | Cache needs setup before items() |
| **Stateless re-query** | Data is cheap to fetch or changes externally | Data fetch is very expensive |

#### Common Pitfall: Stale Cache

**Problem:** Cache persists across screen navigations in TUI

```lua
-- WRONG: Cache never resets in TUI
local cache = {}
local initialized = false

items = function()
    if not initialized then
        -- Only runs ONCE per app session, not per screen entry!
        cache = fetch_data()
        initialized = true
    end
    return cache
end
```

**Solution:** Always reset cache in `pre_run()` or `items()`

```lua
-- CORRECT: Cache resets on every screen entry
local cache = {}

pre_run = function()
    cache = {}  -- Reset each time
end

items = function()
    cache = fetch_data()
    return cache
end
```

#### Execution Guarantee

**Important:** `items()` is **always** called before `execute()` in both TUI and CLI modes:

- **In TUI:** Every time you enter the task screen, `items()` is called
- **In CLI:** `items()` is called once before execution
- **Guarantee:** You can safely cache in `items()` and use in `execute()`

This means Pattern 2 (reset cache in `items()`) is **safe and reliable** in both modes.

```lua
-- This pattern is guaranteed to work
local cache = {}

items = function()
    cache = {}  -- Always called first
    cache = fetch_and_parse_data()
    return get_item_names(cache)
end

execute = function(items)
    -- Cache is guaranteed to be populated
    use_cache(cache)
end
```

## Debugging

### Validation

```bash
# Validate plugin structure
syntropy validate --plugin path/to/plugin.lua

# Common errors:
# - Missing metadata.name or metadata.version
# - Invalid semver format
# - Icon too wide (must occupy exactly 1 terminal cell - Unicode/Nerd Font OK, no emojis)
# - Task has no execute or item_sources
# - Multiple item sources without tags
```

### Print Debugging

```lua
-- Use print() for debugging (visible in TUI output)
items = function()
    print("Fetching items...")
    local result = get_items()
    print("Found " .. #result .. " items")
    return result
end
```

### Testing in CLI Mode

```bash
# Test task execution directly
syntropy execute --plugin your-plugin --task your-task

# See exit codes
syntropy execute --plugin test --task demo
echo $?  # Print exit code
```

### Common Issues

**Plugin not showing up:**
- Check plugin is in `~/.config/syntropy/plugins/<name>/plugin.lua`
- Run `syntropy validate --plugin path/to/plugin.lua`
- Ensure `metadata.name` and `metadata.version` are set

**Items not appearing:**
- Check `items()` function returns array of strings
- Use `print()` to debug inside `items()`
- Verify no Lua syntax errors (check validation output)

**Execute failing silently:**
- Return `output, exit_code` (both required)
- Check for Lua errors (use `pcall`)
- Verify shell commands work (`syntropy.shell("test")`)

**Preview not showing:**
- Ensure `preview()` returns string (not nil)
- Check item passed to preview matches item from `items()`

---

Next steps: Check out the [Plugin API Reference](plugin-api-reference.md) for complete API documentation.
