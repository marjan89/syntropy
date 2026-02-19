# Plugin API Reference - Task Definition

> Part of the [Plugin API Reference](plugin-api-reference.md)  
> **See also:** [Data Structures](plugin-api-reference-section-data-structures.md) | [Item Sources](plugin-api-reference-section-item-sources.md) | [API Functions](plugin-api-reference-section-api-functions.md)

## Task Definition

Tasks are the user-facing operations in your plugin.

### CLI vs TUI Execution Model

Syntropy plugins execute differently depending on whether they're invoked from the command line (CLI) or within the terminal user interface (TUI). Understanding these differences is critical for building plugins that work correctly in both modes.

#### Behavioral Differences Matrix

| Aspect | CLI Mode | TUI Mode |
|--------|----------|----------|
| **Invocation** | `syntropy plugin-name task-name` | Interactive menu navigation |
| **Item Selection** | Via `--items` flag or pre-selection | Interactive search and multi-select UI |
| **Execution Trigger** | Automatic (runs immediately) | User presses Enter/Return to execute |
| **Execution Confirmation** | Not shown (executes without prompt) | Modal dialog shown if configured |
| **Success Notification** | Printed to stdout | Modal dialog (unless suppressed) |
| **Error Display** | Printed to stderr | Red error modal |
| **Items Refresh** | Not applicable (one-shot) | Automatic after execution |
| **pre_run() calls** | Once before items fetch | Every time task screen entered |
| **post_run() calls** | Once after execution | After each execution |
| **Polling** | Not supported (items fetched once) | Full support for live updates |
| **Preview** | Not shown | Displayed in right pane |

#### Execution Flow Diagrams

**CLI Mode Flow:**
```
syntropy plugin task [--items item1,item2]
  ↓
pre_run() called once
  ↓
items() called once
  ↓
Apply --items filter or use preselected_items
  ↓
execute(selected_items) immediately
  ↓
post_run() called once
  ↓
Print output to stdout/stderr
  ↓
Exit with code from execute()
```

**TUI Mode Flow:**
```
User navigates to task
  ↓
pre_run() called
  ↓
items() called
  ↓
Display items list + search UI + preview pane
  ↓
User searches, selects items, navigates
  ↓
[If polling enabled: items() called periodically]
  ↓
User presses Enter
  ↓
[If confirmation configured: show modal, wait for confirm]
  ↓
execute(selected_items)
  ↓
post_run() called
  ↓
items() called again (refresh list)
  ↓
Return to items list, user can execute again
```

#### Key Behavioral Differences Explained

**1. Execution Confirmation**

- **CLI:** `execution_confirmation_message` is ignored - commands execute immediately without prompting
  - Reason: CLI is designed for scripting and automation
  - Workaround: Use shell confirmation (`read -p`) if needed
- **TUI:** Modal dialog shown with message and selected items
  - User can confirm (Enter) or cancel (Esc)

```lua
execution_confirmation_message = "Are you sure you want to delete:"
-- TUI: Shows modal with selected items listed
-- CLI: Ignored, executes immediately
```

**2. Items Refresh After Execution**

- **CLI:** Not applicable (process exits)
- **TUI:** `items()` automatically called after `execute()` completes
  - Happens for both successful and failed executions
  - **Does call `pre_run()` again** before refreshing items
  - Allows delete/add/update operations to show changes immediately

**3. Hook Execution Frequency**

- **CLI:**
  - `pre_run()`: Called once before initial `items()` fetch
  - `post_run()`: Called once after execution completes
- **TUI:**
  - `pre_run()`: Called **every time** you navigate to task screen (re-entering counts)
  - `post_run()`: Called after **each** execution (user can execute multiple times)
  - Perfect for cache invalidation patterns

**4. Polling Support**

- **CLI:** Polling fields ignored
  - `item_polling_interval`: No effect (items fetched once)
  - `preview_polling_interval`: No effect (preview never shown)
- **TUI:** Full polling support
  - Items refresh at specified interval
  - Preview refreshes independently
  - Search query and selection preserved

**5. Preview Display**

- **CLI:** `preview()` function never called
  - Preview pane doesn't exist in CLI
  - Preview code not executed (optimization)
- **TUI:** Preview shown in right pane
  - Updates when selection changes
  - Can refresh with `preview_polling_interval`
  - Can return nil for "no preview"

**6. Success Notification Suppression**

- **CLI:** `suppress_success_notification` has no effect
  - Output always printed to stdout regardless of setting
  - Exit code always returned to shell
- **TUI:** Controls modal display
  - `true`: No modal shown after success
  - `false`: Success modal with output message
  - Errors always show modal regardless of setting

#### Cross-Mode Compatibility Guidelines

**Design for both modes:**

1. **Always return meaningful output messages**
   - CLI prints them to stdout
   - TUI shows them in modals
   - Don't return empty strings

2. **Use proper exit codes**
   - 0 = success, non-zero = failure
   - CLI exits with this code
   - TUI uses it to determine success/error modal

3. **Don't rely on TUI-only features in critical logic**
   - Polling doesn't work in CLI
   - Confirmation dialogs don't show in CLI
   - Preview functions not called in CLI

4. **Test both modes**
   ```bash
   # Test TUI mode
   syntropy plugin-name task-name

   # Test CLI mode
   syntropy plugin-name task-name --items item1,item2
   ```

5. **Document mode-specific behavior**
   - If a task is TUI-only (uses polling, interactive), document it
   - If a task is CLI-friendly (quick automation), highlight it

#### Mode-Specific Optimizations

**Optimize for CLI:**
- Minimize expensive operations in `items()` if using `--items` flag (items filtered anyway)
- Avoid unnecessary `preview()` computation (never called in CLI)
- Return concise output messages (printed directly to terminal)

**Optimize for TUI:**
- Implement preview for better UX
- Use polling for dynamic data
- Use `exit_on_execute` appropriately
- Keep item strings stable for selection persistence

### Task Structure

```lua
tasks = {
    task_key = {
        name = "Display Name",           -- Optional: Defaults to task_key
        description = "Task description", -- Required: Shown in preview pane
        mode = "multi",                  -- Optional: "multi" | "none" | default (none)
        execution_confirmation_message = "string",  -- Optional: Show confirmation dialog (default: not shown)
        suppress_success_notification = false,      -- Optional: Suppress success modal (default: false)

        -- Automatic polling
        item_polling_interval = 0,       -- Optional: Milliseconds between item refreshes (default: 0 = disabled)
        preview_polling_interval = 0,    -- Optional: Milliseconds between preview refreshes (default: 0 = disabled)

        -- Lifecycle hooks
        pre_run = function() end,        -- Optional: Before items() called (default: not defined)
        post_run = function() end,       -- Optional: After execute() completes (default: not defined)

        -- Option 1: Task with item sources
        item_sources = { ... },

        -- Option 2: Task-level functions (no item sources)
        execute = function() ... end,    -- Required if no item_sources
        preview = function(item) ... end -- Optional (default: not defined)
    },
}
```

**Task Field Defaults:**

| Field | Required? | Default Value | Notes |
|-------|-----------|---------------|-------|
| `name` | No | `task_key` | Uses the task's key as display name if not specified |
| `description` | Yes | N/A | Must be provided - shown in preview pane |
| `mode` | No | `"none"` | No selection mode (execute directly) |
| `execution_confirmation_message` | No | `nil` | No confirmation dialog shown |
| `suppress_success_notification` | No | `false` | Show success modal in TUI |
| `item_polling_interval` | No | `0` | Polling disabled |
| `preview_polling_interval` | No | `0` | Preview polling disabled |
| `item_sources` | No | `nil` | No item sources (task-level execution) |
| `pre_run` | No | `nil` | No pre-run hook |
| `post_run` | No | `nil` | No post-run hook |
| `execute` | Conditional | `nil` | Required if no `item_sources` defined |
| `preview` | No | `nil` | No preview (or fallback to task-level) |

### Task-Level vs Item-Source Execution Patterns

Syntropy supports two distinct patterns for implementing task execution. Understanding when to use each pattern is essential for clean plugin architecture.

#### The Two Patterns

**Pattern 1: Item-Source Functions** (Recommended for most tasks)
```lua
tasks = {
    my_task = {
        description = "Task with item sources",
        item_sources = {
            source1 = {
                tag = "s",
                items = function() return {...} end,
                preview = function(item) return "..." end,      -- Optional
                execute = function(items) return "...", 0 end,  -- Optional
            },
        },
    },
}
```

**Pattern 2: Task-Level Functions** (For simple, no-items tasks)
```lua
tasks = {
    my_task = {
        description = "Simple task without item sources",
        execute = function() return "Done", 0 end,  -- Required
        preview = function(item) return "..." end,  -- Optional, acts as fallback
    },
}
```

#### Execution Routing Logic

Syntropy uses the following precedence rules to determine which functions to call:

**For items() function:**
- If `item_sources` exist: Call `items()` from each source, combine results
- If no `item_sources`: No items available (task executes without selection)

**For execute() function:**
- If `item_sources` exist AND source has `execute()`: Call source-specific `execute()`
- If `item_sources` exist BUT source lacks `execute()`: Error (no-op)
- If no `item_sources` AND task has `execute()`: Call task-level `execute()`
- If neither exists: Task cannot execute (error)

**For preview() function:**
- If item source has `preview()`: Use source-specific preview
- Else if task has `preview()`: Use task-level preview (fallback)
- Else: No preview shown

#### Precedence Rules Summary

| Function | Item Source Level | Task Level | Resolution |
|----------|------------------|------------|------------|
| `items()` | ✅ Defined | N/A | Source function called |
| `items()` | ❌ Not applicable | N/A | No items (execute without selection) |
| `execute()` | ✅ Defined | N/A | Source function called |
| `execute()` | ❌ Missing | ✅ Defined | Task function called |
| `execute()` | ❌ Missing | ❌ Missing | Error: Task cannot execute |
| `preview()` | ✅ Defined | N/A | Source function used |
| `preview()` | ❌ Missing | ✅ Defined | Task function used (fallback) |
| `preview()` | ❌ Missing | ❌ Missing | No preview shown |

**Important:** Task-level `execute()` and `preview()` are **only** used when `item_sources` is not defined or as fallback for preview. You cannot mix item-source execution with task-level execution.

#### Pattern Selection Guide

**Use Item-Source Pattern when:**

1. **Task has selectable items** - Users choose from a list
   - File browsers, process managers, Git branches
2. **Multiple item types** - Different sources in one task
   - Files + directories, local + remote branches
3. **Per-source logic** - Different handling for each source
   - Different execute behavior for files vs directories
4. **Need preview** - Show content for selected item
   - File preview, process details, branch info

**Use Task-Level Pattern when:**

1. **No items needed** - Direct action without selection
   - Run script, show report, check status
2. **Simple workflow** - Single action, no choices
   - Rebuild project, clear cache, sync data
3. **Report-style tasks** - Display information only
   - Show system info, list config, display stats
4. **Editor/TUI launchers** - Just invoke external tool
   - Open editor, launch htop, run fzf

#### Complete Examples

**Example 1: Item-Source Pattern (File Manager)**

```lua
tasks = {
    browse = {
        name = "Browse Files",
        description = "Browse and delete files in current directory",
        mode = "multi",

        item_sources = {
            files = {
                tag = "f",
                items = function()
                    local output, _ = syntropy.shell("find . -type f -maxdepth 1")
                    local items = {}
                    for line in output:gmatch("[^\n]+") do
                        if line ~= "." then
                            table.insert(items, line)
                        end
                    end
                    return items
                end,

                preview = function(item)
                    -- Source-specific preview for files
                    local file = io.open(item, "r")
                    if not file then return "Cannot read file" end
                    local content = file:read("*all")
                    file:close()
                    return content
                end,

                execute = function(items)
                    -- Source-specific execution for files
                    for _, file in ipairs(items) do
                        syntropy.shell("rm " .. file)
                    end
                    return "Deleted " .. #items .. " files", 0
                end,
            },

            directories = {
                tag = "d",
                items = function()
                    local output, _ = syntropy.shell("find . -type d -maxdepth 1")
                    local items = {}
                    for line in output:gmatch("[^\n]+") do
                        if line ~= "." then
                            table.insert(items, line)
                        end
                    end
                    return items
                end,

                preview = function(item)
                    -- Source-specific preview for directories
                    local output, _ = syntropy.shell("ls -la " .. item)
                    return output
                end,

                execute = function(items)
                    -- Source-specific execution for directories
                    for _, dir in ipairs(items) do
                        syntropy.shell("rm -rf " .. dir)
                    end
                    return "Deleted " .. #items .. " directories", 0
                end,
            },
        },
    },
}
```

**Example 2: Task-Level Pattern (Simple Script)**

```lua
tasks = {
    rebuild = {
        name = "Rebuild Project",
        description = "Clean and rebuild the entire project",

        -- No item_sources - direct execution
        execute = function()
            -- Clean
            local clean_out, clean_code = syntropy.shell("cargo clean")
            if clean_code ~= 0 then
                return "Clean failed: " .. clean_out, clean_code
            end

            -- Build
            local build_out, build_code = syntropy.shell("cargo build --release")
            if build_code ~= 0 then
                return "Build failed: " .. build_out, build_code
            end

            return "Project rebuilt successfully", 0
        end,
        -- No preview needed (no items to preview)
    },
}
```

**Example 3: Mixed Pattern (Task-Level Preview Fallback)**

```lua
tasks = {
    processes = {
        name = "Process Manager",
        description = "Monitor and kill processes",
        mode = "multi",

        item_sources = {
            procs = {
                tag = "p",
                items = function()
                    local output, _ = syntropy.shell("ps aux | tail -n +2")
                    local items = {}
                    for line in output:gmatch("[^\n]+") do
                        table.insert(items, line)
                    end
                    return items
                end,

                -- No preview at source level - falls back to task level

                execute = function(items)
                    for _, item in ipairs(items) do
                        local pid = item:match("^%S+%s+(%d+)")
                        syntropy.shell("kill " .. pid)
                    end
                    return "Killed " .. #items .. " processes", 0
                end,
            },
        },

        -- Task-level preview acts as fallback
        preview = function(item)
            local pid = item:match("^%S+%s+(%d+)")
            local output, _ = syntropy.shell("ps -p " .. pid .. " -o %cpu,%mem,command")
            return output
        end,
    },
}
```

**Example 4: Multiple Sources with Shared Preview Fallback**

```lua
tasks = {
    search = {
        name = "Search Files",
        description = "Search by name or content",
        mode = "multi",

        item_sources = {
            by_name = {
                tag = "n",
                items = function()
                    local output, _ = syntropy.shell("find . -name '*.txt'")
                    return parse_lines(output)
                end,
                -- No source-specific preview - uses task fallback
                execute = function(items)
                    return "Found " .. #items .. " files by name", 0
                end,
            },

            by_content = {
                tag = "c",
                items = function()
                    local output, _ = syntropy.shell("grep -rl 'pattern' .")
                    return parse_lines(output)
                end,
                -- No source-specific preview - uses task fallback
                execute = function(items)
                    return "Found " .. #items .. " files by content", 0
                end,
            },
        },

        -- Shared preview for both sources
        preview = function(item)
            local file = io.open(item, "r")
            if not file then return "Cannot read: " .. item end
            local content = file:read("*all")
            file:close()
            return content
        end,
    },
}
```

#### Common Mistakes

**Mistake 1: Defining both item_sources and task-level execute()**
```lua
-- ❌ BAD: Task-level execute() never called when item_sources exist
tasks = {
    bad_task = {
        item_sources = {
            source1 = {
                items = function() return {"a", "b"} end,
                -- Missing execute() here
            },
        },
        execute = function() return "...", 0 end,  -- Never called!
    },
}

-- ✅ GOOD: Choose one pattern
tasks = {
    good_task = {
        item_sources = {
            source1 = {
                items = function() return {"a", "b"} end,
                execute = function(items) return "...", 0 end,  -- Source-level
            },
        },
        -- No task-level execute()
    },
}
```

**Mistake 2: Expecting items without item_sources**
```lua
-- ❌ BAD: No items available, but execute expects them
tasks = {
    bad_task = {
        -- No item_sources defined
        execute = function(items)
            -- items is always empty here!
            for _, item in ipairs(items) do
                -- Never runs
            end
        end,
    },
}

-- ✅ GOOD: Task-level execute() has no items parameter
tasks = {
    good_task = {
        execute = function()  -- No items parameter
            -- Direct execution logic
            return "Done", 0
        end,
    },
}
```

**Mistake 3: Missing execute() entirely**
```lua
-- ❌ BAD: Item source without execute()
tasks = {
    bad_task = {
        item_sources = {
            source1 = {
                items = function() return {"a"} end,
                -- Missing execute() - task cannot execute!
            },
        },
        -- No task-level execute either
    },
}

-- ✅ GOOD: Execute() at source level
tasks = {
    good_task = {
        item_sources = {
            source1 = {
                items = function() return {"a"} end,
                execute = function(items) return "Done", 0 end,
            },
        },
    },
}
```

#### Best Practices

1. **Be consistent** - Use one pattern per task, don't mix unnecessarily
2. **Use item-sources for selection** - If users pick from items, use item-source pattern
3. **Use task-level for actions** - If no selection needed, use task-level pattern
4. **Leverage preview fallback** - Share preview logic across sources when appropriate
5. **Document your choice** - Comment why you chose a particular pattern
6. **Validate appropriately** - Task-level execute() doesn't receive items, source-level does

### Task Modes

| Mode | Behavior | Use Case |
|------|----------|----------|
| `"multi"` | Select multiple items, execute all together | Batch operations, multi-select lists |
| `"none"` | Select one item, execute immediately | Navigation, single actions |
| (omitted) | No selection, just execute | Scripts, reports |

### Execution Confirmation

Tasks can optionally display a confirmation dialog before execution by setting the `execution_confirmation_message` field.

```lua
execution_confirmation_message = "Are you sure you want to delete:"
```

**Behavior:**
- When set, a modal dialog appears immediately before execution
- The message is displayed with the format: `"{execution_confirmation_message} {list of items to execute}"`
- User can confirm or cancel the execution
- Only applies in TUI mode (CLI mode executes without confirmation)

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
                items = function()
                    return {"file1.txt", "file2.txt"}
                end,
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

If the user selects `file1.txt` and `file2.txt`, the dialog will show:
```
Are you sure you want to delete: file1.txt, file2.txt
```

**Use cases:**
- Destructive operations (delete, remove, uninstall)
- Operations that cannot be undone
- Actions with significant side effects

### Success Notification Suppression

Tasks can suppress the success modal displayed after execution by setting `suppress_success_notification`.

```lua
suppress_success_notification = true
```

**Parameters:**
- **Type:** `boolean`
- **Default:** `false`

**Behavior:**
- When `true`: No success modal shown after successful execution
- When `false` (default): Success modal displays the return output
- **Errors are always shown** - failed executions always display an error modal
- Only applies in TUI mode (CLI always prints to stdout)

**Returns:** None (this is a configuration field, not a function)

**Use cases:**
- Tasks using `syntropy.invoke_editor()` - The editor provides its own feedback
- Tasks using `syntropy.invoke_tui()` - External TUI apps provide their own interaction
- Tasks where success is obvious from context (e.g., returning to a refreshed list)
- Workflows where modals interrupt the user experience

**Example:**

```lua
tasks = {
    edit = {
        name = "Edit Configuration",
        description = "Open config file in editor",
        suppress_success_notification = true,  -- No modal after editor closes
        execute = function()
            local code = syntropy.invoke_editor(syntropy.expand_path("~/.config/app/config.toml"))
            if code == 0 then
                return "Configuration updated", 0
            else
                return "Editor exited with error", code
            end
        end,
    },
}
```

In this example:
- **Success (code 0):** User returns to syntropy silently (no modal)
- **Error (code ≠ 0):** Error modal shows "Editor exited with error"

**When NOT to use:**
- The task output contains important information the user must see
- Success confirmation is important for the user's workflow
- The task performs non-obvious operations
- The return message includes useful data (counts, summaries, results)

### Lifecycle Hooks

```lua
pre_run = function()
    -- Runs before items() is called each time the task executes
    -- Use for: cache invalidation, state initialization, setup, validation
end

post_run = function()
    -- Runs after all executions complete
    -- Use for: cleanup, logging, notifications
end
```

**Execution Behavior:**

| Hook | TUI Mode | CLI Mode |
|------|----------|----------|
| `pre_run()` | Called **every time** you navigate to the task screen | Called **once** before items fetch |
| `post_run()` | Called after **each** execution | Called **once** after execution |

**pre_run() Details:**
- In **TUI**: Called every time the screen is entered (not just once per app session)
- In **CLI**: Called once before `items()` is fetched
- Perfect for cache invalidation when re-entering screens
- No parameters, no return value expected

**post_run() Details:**
- Runs after all `execute()` functions complete
- In **TUI**: Called after each execution (user may return and execute again)
- In **CLI**: Called once after execution completes
- No parameters, no return value expected

**State Persistence:**
- Plugins load once at startup and persist until app exit
- Module-level variables persist across all task executions
- `items()` is called fresh every time (TUI: on screen enter, CLI: before execution)
- Use `pre_run()` to reset caches and state for fresh data

**Execution Order:**
1. `pre_run()` (if defined)
2. `items()` for all item sources
3. User selects items (TUI only)
4. `execute()` for selected item sources
5. `post_run()` (if defined)
6. **TUI only:** `pre_run()` called again, then `items()` called to refresh the list

**Important:** `items()` is **always** called before `execute()` in both TUI and CLI modes. You can safely populate caches in `items()` and use them in `execute()`.

**Post-Execution Item Refresh (TUI only):**

After `execute()` completes in TUI mode, syntropy automatically calls `pre_run()` (if defined) and then `items()` to refresh the item list. This happens for both successful (exit code 0) and failed executions (exit code > 0).

- ✅ **Automatic in TUI** - Items refresh after every execution
- ❌ **Not in CLI** - One-shot execution, no refresh
- ✅ **Does call `pre_run()`** - Full pipeline runs again (`pre_run()` → `items()`)
- ✅ **Module state persists** - Caches and variables remain unchanged unless cleared in `pre_run()`

This enables delete/add/update operations to immediately show changes:

```lua
-- After deletion, pre_run() and items() are called automatically to show updated list
execute = function(items)
    for _, item in ipairs(items) do
        syntropy.shell("rm " .. item)
    end
    return "Deleted " .. #items .. " files", 0
    -- pre_run() then items() will be called here automatically in TUI
end
```

**Return value:** None (hooks don't return anything)

### Automatic Polling

Tasks can automatically refresh their items and previews at regular intervals without user interaction. This is useful for displaying dynamic data that changes over time.

**`item_polling_interval`** - Automatic item list refresh

```lua
item_polling_interval = 1000  -- Refresh items every 1000ms (1 second)
```

- **Type:** integer (milliseconds)
- **Default:** `0` (disabled)
- **Behavior:** When > 0, automatically calls `items()` function at the specified interval
- **Preserves:** Search query and selected item position across refreshes
- **Use cases:** Process monitors, active window lists, file watchers, system stats

**`preview_polling_interval`** - Automatic preview refresh

```lua
preview_polling_interval = 500  -- Refresh preview every 500ms
```

- **Type:** integer (milliseconds)
- **Default:** `0` (disabled)
- **Behavior:** When > 0, automatically calls `preview()` function for the selected item at the specified interval
- **Invalidates:** Preview cache at each interval, forcing fresh data
- **Use cases:** Live logs, dynamic content, real-time status

**Example - Process Monitor:**

```lua
tasks = {
    processes = {
        name = "Process Monitor",
        description = "Monitor and manage running processes",
        mode = "multi",
        item_polling_interval = 2000,       -- Refresh process list every 2 seconds
        preview_polling_interval = 1000,    -- Refresh process details every 1 second

        item_sources = {
            procs = {
                tag = "p",
                items = function()
                    local output, _ = syntropy.shell("ps aux | tail -n +2 | awk '{print $2, $11}'")
                    local processes = {}
                    for line in output:gmatch("[^\n]+") do
                        table.insert(processes, line)
                    end
                    return processes
                end,
                preview = function(item)
                    local pid = item:match("^(%d+)")
                    local output, _ = syntropy.shell("ps -p " .. pid .. " -o %cpu,%mem,etime,command")
                    return output
                end,
                execute = function(items)
                    for _, item in ipairs(items) do
                        local pid = item:match("^(%d+)")
                        syntropy.shell("kill " .. pid)
                    end
                    return "Killed " .. #items .. " processes", 0
                end,
            },
        },
    },
}
```

**Important Notes:**
- Polling only occurs when not already executing an operation
- Very small intervals (< 100ms) may impact performance
- Polling is independent for items and preview (can use different intervals)
- Search filters are preserved during item polling
- Selected item is maintained when possible during refreshes

### Performance Best Practices

Plugin performance directly impacts user experience, especially for frequently-polled tasks and large item lists. Follow these guidelines to keep your plugins responsive.

#### Polling Interval Recommendations

**Item Polling (`item_polling_interval`):**

| Update Frequency | Interval (ms) | Use Case | Example |
|------------------|---------------|----------|---------|
| Real-time | 100-500 | Live metrics, active processes | CPU monitor, running timers |
| Fast updates | 1000-2000 | File watchers, network status | Active window, Bluetooth devices |
| Moderate updates | 3000-5000 | System info, resource usage | Disk space, memory usage |
| Slow updates | 10000+ | Infrequent changes | Git status, package updates |
| No polling | 0 (default) | Static data, manual refresh | File lists, notes, bookmarks |

**Preview Polling (`preview_polling_interval`):**

| Update Frequency | Interval (ms) | Use Case | Example |
|------------------|---------------|----------|---------|
| Real-time | 100-500 | Live logs, streaming data | Tail -f output, live metrics |
| Fast updates | 500-1000 | Process details, dynamic content | Process CPU/memory, file size |
| Moderate updates | 2000-5000 | Occasional changes | File metadata, Git blame |
| No polling | 0 (default) | Static content | File preview, help text |

**Guidelines:**
- Preview polling can be faster than item polling (preview updates don't rebuild entire list)
- Don't poll faster than your data source can provide meaningful updates
- Very fast polling (< 100ms) can cause UI lag and high CPU usage
- Consider user's typical workflow - do they need instant updates or periodic refresh?

#### Items Array Size Guidance

**Recommended Limits:**

| Item Count | Performance | User Experience | Recommendation |
|------------|-------------|-----------------|----------------|
| < 100 | Excellent | Instant | Ideal for most tasks |
| 100-500 | Good | Fast | Acceptable, consider pagination |
| 500-1000 | Moderate | Noticeable delay | Implement filtering/search hints |
| 1000-5000 | Slow | Laggy scrolling | Add aggressive filtering or pagination |
| > 5000 | Poor | Very slow | Strongly recommend pagination or limiting |

**Mitigation Strategies:**

1. **Limit results** - Use `head` to cap output:
   ```lua
   items = function()
       local output, _ = syntropy.shell("find . -type f | head -n 1000")
       -- Limit to 1000 files
   end
   ```

2. **Smart defaults** - Start with filtered view:
   ```lua
   items = function()
       -- Only show recent files by default
       local output, _ = syntropy.shell("find . -type f -mtime -7")
       -- User can remove time filter if needed
   end
   ```

3. **Pagination** - Use multiple item sources:
   ```lua
   item_sources = {
       recent = {
           tag = "r",
           items = function() return find_recent(100) end,
       },
       all = {
           tag = "a",
           items = function() return find_all(1000) end,
       },
   }
   ```

#### Shell Command Performance Gotchas

**Gotcha 1: Unnecessary command invocations**

```lua
-- ❌ BAD: Runs command for every item
preview = function(item)
    -- This runs ps for EVERY item in the list!
    local output, _ = syntropy.shell("ps aux | grep " .. item)
    return output
end

-- ✅ GOOD: Run command once, cache results
local process_cache = {}
local cache_time = 0

items = function()
    local output, _ = syntropy.shell("ps aux")
    process_cache = parse_ps_output(output)
    cache_time = os.time()
    return get_process_names(process_cache)
end

preview = function(item)
    -- Look up from cache instead of running ps again
    return process_cache[item] or "Not found"
end
```

**Gotcha 2: Expensive parsing in polling loop**

```lua
-- ❌ BAD: Reparsing entire file every poll
item_polling_interval = 1000  -- Every second

items = function()
    local file = io.open("large-log.txt", "r")
    local content = file:read("*all")  -- Reads entire file every second!
    file:close()
    return parse_all_lines(content)
end

-- ✅ GOOD: Only read new data
local last_position = 0

items = function()
    local file = io.open("large-log.txt", "r")
    file:seek("set", last_position)
    local new_content = file:read("*all")
    last_position = file:seek()
    file:close()
    return parse_new_lines(new_content)
end
```

**Gotcha 3: Synchronous network calls**

```lua
-- ❌ BAD: Blocks UI on network request
items = function()
    -- If network is slow, UI freezes
    local output, _ = syntropy.shell("curl https://api.example.com/data")
    return parse_json(output)
end

-- ✅ GOOD: Use timeout and cache
local api_cache = nil
local api_cache_time = 0
local CACHE_TTL = 60  -- 1 minute

items = function()
    local now = os.time()
    if api_cache and (now - api_cache_time) < CACHE_TTL then
        return api_cache  -- Return cached data
    end

    -- Fetch with timeout
    local output, code = syntropy.shell("curl --max-time 5 https://api.example.com/data")
    if code == 0 then
        api_cache = parse_json(output)
        api_cache_time = now
        return api_cache
    else
        -- Return cached data on failure, or empty array
        return api_cache or {"Error: API timeout"}
    end
end
```

#### When to Cache vs Compute

**Cache when:**
- Data is expensive to compute (parsing large files, complex calculations)
- Data doesn't change frequently (configuration, file lists without polling)
- Same data needed across multiple functions (items + preview + execute)
- Network/API calls involved

**Compute when:**
- Data is trivial to generate (simple string formatting)
- Data changes constantly (real-time metrics)
- Caching adds complexity without benefit
- Memory usage is a concern

**Complete Caching Pattern Example:**

```lua
---@type PluginDefinition
local plugin = {
    metadata = {
        name = "process-monitor",
        version = "1.0.0",
    },
    tasks = {
        monitor = {
            name = "Process Monitor",
            description = "Monitor system processes with caching",
            mode = "multi",
            item_polling_interval = 2000,  -- Refresh every 2 seconds

            item_sources = {
                procs = {
                    tag = "p",
                    items = function()
                        -- Parse ps output and cache results
                        local output, _ = syntropy.shell("ps aux")

                        -- Module-level cache (persists across calls)
                        plugin.process_cache = {}

                        for line in output:gmatch("[^\n]+") do
                            local pid, cpu, mem, cmd = line:match("^%S+%s+(%d+)%s+([%d.]+)%s+([%d.]+)%s+.-%s+(.+)$")
                            if pid then
                                local key = "[" .. pid .. "] " .. cmd
                                plugin.process_cache[key] = {
                                    pid = pid,
                                    cpu = cpu,
                                    mem = mem,
                                    cmd = cmd,
                                }
                            end
                        end

                        -- Return stable identifiers
                        local items = {}
                        for key, _ in pairs(plugin.process_cache) do
                            table.insert(items, key)
                        end
                        return items
                    end,

                    preview = function(item)
                        -- Look up from cache instead of running ps again
                        local proc = plugin.process_cache[item]
                        if not proc then return "Process not found" end

                        return string.format(
                            "PID: %s\nCPU: %s%%\nMemory: %s%%\nCommand: %s",
                            proc.pid, proc.cpu, proc.mem, proc.cmd
                        )
                    end,

                    execute = function(items)
                        local killed = 0
                        for _, item in ipairs(items) do
                            local proc = plugin.process_cache[item]
                            if proc then
                                syntropy.shell("kill " .. proc.pid)
                                killed = killed + 1
                            end
                        end
                        return "Killed " .. killed .. " processes", 0
                    end,
                },
            },
        },
    },
}

return plugin
```

#### Polling vs pre_run() for Cache Invalidation

**Use polling when:**
- Data changes frequently and users need to see updates
- User stays on task screen for extended periods
- Real-time monitoring is the primary use case

**Use pre_run() when:**
- Data only needs refresh when user enters screen
- Polling would waste resources
- User workflow is enter → select → execute → leave

**Example: pre_run() cache invalidation**

```lua
local file_list_cache = nil

tasks = {
    browse = {
        pre_run = function()
            -- Clear cache every time user enters this task
            file_list_cache = nil
        end,

        item_sources = {
            files = {
                items = function()
                    if file_list_cache then
                        return file_list_cache
                    end

                    local output, _ = syntropy.shell("find . -type f")
                    file_list_cache = parse_lines(output)
                    return file_list_cache
                end,
            },
        },
    },
}
```

**Item Identity and Persistence:**

For selection and marking persistence to work across polls, item strings must remain **stable** and serve as identity keys.

- ❌ **Bad**: Include dynamic data in item string
  - `"300 23% tmux"` → `"300 1% tmux"` → `"300 24% tmux"` (treated as different items, selection lost)
- ✅ **Good**: Use stable identifiers
  - `"300 tmux"` → `"300 tmux"` → `"300 tmux"` (same item, selection preserved)

**Guidelines:**
- Show dynamic data (CPU%, memory, status) in the **preview** instead
- Use **sorting** to emphasize dynamic properties (highest CPU first, etc.)
- Keep the item string consistent across poll cycles

**Example - Stable vs Unstable Items:**

```lua
-- ❌ BAD: CPU percentage changes item identity
items = function()
    local output = syntropy.shell("ps aux --sort=-%cpu | head -20")
    local procs = {}
    for line in output:gmatch("[^\n]+") do
        local pid, cpu, cmd = line:match("^%S+%s+(%d+)%s+([%d.]+)%s+.-%s+(.+)$")
        -- Don't include CPU% in item string - it changes constantly
        table.insert(procs, pid .. " " .. cpu .. "% " .. cmd)  -- ❌ Bad
    end
    return procs
end

-- ✅ GOOD: Stable identifiers, dynamic data in preview
items = function()
    local output = syntropy.shell("ps aux --sort=-%cpu | head -20")
    local procs = {}
    for line in output:gmatch("[^\n]+") do
        local pid, cmd = line:match("^%S+%s+(%d+)%s+.-%s+(.+)$")
        -- Use stable PID + command as item identity
        table.insert(procs, "[" .. pid .. "] " .. cmd)  -- ✅ Good
    end
    return procs
end,

preview = function(item)
    -- Show dynamic CPU/memory data in preview instead
    local pid = item:match("^%[(%d+)%]")
    local output = syntropy.shell("ps -p " .. pid .. " -o %cpu,%mem,vsz,rss,time")
    return output
end
```

### Error Handling and Validation Patterns

Understanding how syntropy handles errors and validation failures is critical for building robust plugins.

#### Error Propagation Model

Syntropy distinguishes between two types of failures:

1. **Lua errors** (runtime exceptions) - Unhandled errors that crash the function
2. **Exit codes** (controlled failures) - Non-zero exit codes returned from `execute()`

**Lua Errors:**
- Thrown by `error()`, failed assertions, or runtime exceptions (nil access, type errors, etc.)
- Immediately stop execution and display error modal in TUI
- In CLI mode, print error to stderr and exit with non-zero code
- Error message shown to user exactly as provided

**Exit Codes:**
- Returned as second value from `execute()`: `return message, exit_code`
- Exit code 0 = success, any other value = failure
- Output message shown in modal (TUI) or printed to stdout (CLI)
- Non-zero codes show red error modal in TUI
- Allows graceful failure handling with custom user messages

#### items() Function Error Handling

The `items()` function is expected to always return an array of strings. Error handling patterns:

**Pattern 1: Return empty array**
```lua
items = function()
    local output, code = syntropy.shell("ls /nonexistent")
    if code ~= 0 then
        return {}  -- Empty list shown in TUI
    end
    -- Parse output...
end
```

**Behavior:**
- TUI: Shows empty list with "No items" message
- CLI: No items available for selection, task cannot execute

**Pattern 2: Return error message as single item**
```lua
items = function()
    local file = io.open("required-file.txt", "r")
    if not file then
        return {"Error: required-file.txt not found"}
    end
    -- Parse file...
end
```

**Behavior:**
- TUI: Shows error message in item list
- User sees the error but cannot execute (error item selected would pass to execute())
- Consider validating in `execute()` to prevent execution

**Pattern 3: Throw Lua error (recommended for critical failures)**
```lua
items = function()
    local config_dir = syntropy.expand_path("./config")
    local file = io.open(config_dir .. "/required.json", "r")
    if not file then
        error("Configuration file not found: " .. config_dir .. "/required.json")
    end
    -- Parse file...
end
```

**Behavior:**
- TUI: Shows error modal with message, task screen cannot be entered
- CLI: Prints error to stderr, exits with error code
- **Use for:** Missing dependencies, invalid configuration, broken plugin state

**Best practice:** Use pattern 3 (throw error) for initialization failures, pattern 1 (empty array) for "no results" scenarios.

#### execute() Function Error Patterns

The `execute()` function should always return `(string, integer)` - output message and exit code.

**Pattern 1: Simple validation**
```lua
execute = function(items)
    if #items == 0 then
        return "No items selected", 1
    end

    -- Process items...
    return "Processed " .. #items .. " items", 0
end
```

**Pattern 2: Command failure handling**
```lua
execute = function(items)
    local output, code = syntropy.shell("git status")
    if code ~= 0 then
        return "Git command failed: " .. output, code
    end

    -- Continue processing...
    return "Success", 0
end
```

**Pattern 3: Partial failure reporting**
```lua
execute = function(items)
    local succeeded = 0
    local failed = 0

    for _, item in ipairs(items) do
        local _, code = syntropy.shell("process " .. item)
        if code == 0 then
            succeeded = succeeded + 1
        else
            failed = failed + 1
        end
    end

    if failed > 0 then
        return string.format("Processed %d items (%d failed)", succeeded + failed, failed), 1
    end

    return "Processed " .. succeeded .. " items successfully", 0
end
```

**Pattern 4: User-friendly error messages**
```lua
execute = function(items)
    local file_path = items[1]

    -- Validate before operation
    local file = io.open(file_path, "r")
    if not file then
        return "Cannot open file: " .. file_path .. "\nCheck file permissions or path.", 1
    end
    file:close()

    -- Perform operation...
    return "File processed", 0
end
```

#### preview() Function Error Handling

Preview functions should handle errors gracefully since they're called frequently and automatically.

**Pattern 1: Return error message as preview text**
```lua
preview = function(item)
    local file = io.open(item, "r")
    if not file then
        return "Error: Cannot read file " .. item
    end

    local content = file:read("*all")
    file:close()
    return content
end
```

**Pattern 2: Return nil for no preview**
```lua
preview = function(item)
    -- Only show preview for text files
    if not item:match("%.txt$") then
        return nil  -- No preview shown
    end

    local file = io.open(item, "r")
    if not file then return nil end

    local content = file:read("*all")
    file:close()
    return content
end
```

**Best practice:** Never throw errors from `preview()` - return error message as string or nil instead.

#### Common Validation Mistakes

**Mistake 1: Not validating empty item selection**
```lua
-- ❌ BAD: Assumes items exist
execute = function(items)
    local file = items[1]  -- Crashes if items is empty!
    -- ...
end

-- ✅ GOOD: Validate first
execute = function(items)
    if #items == 0 then
        return "No items selected", 1
    end
    local file = items[1]
    -- ...
end
```

**Mistake 2: Not checking shell command exit codes**
```lua
-- ❌ BAD: Ignores failures
execute = function(items)
    local output, code = syntropy.shell("rm " .. items[1])
    return "Deleted file", 0  -- Says success even if rm failed!
end

-- ✅ GOOD: Check exit code
execute = function(items)
    local output, code = syntropy.shell("rm " .. items[1])
    if code ~= 0 then
        return "Delete failed: " .. output, code
    end
    return "Deleted file", 0
end
```

**Mistake 3: Not validating file I/O**
```lua
-- ❌ BAD: Assumes file exists
items = function()
    local file = io.open("data.txt", "r")
    local content = file:read("*all")  -- Crashes if file is nil!
    file:close()
    -- ...
end

-- ✅ GOOD: Check for nil
items = function()
    local file = io.open("data.txt", "r")
    if not file then
        error("data.txt not found")
    end
    local content = file:read("*all")
    file:close()
    -- ...
end
```

**Mistake 4: Throwing errors from execute() instead of returning exit codes**
```lua
-- ❌ BAD: Error modal is unclear
execute = function(items)
    if #items == 0 then
        error("No items!")  -- Generic error modal
    end
    -- ...
end

-- ✅ GOOD: Controlled failure with clear message
execute = function(items)
    if #items == 0 then
        return "No items selected. Please select at least one item.", 1
    end
    -- ...
end
```

#### Error Display Reference

| Error Type | TUI Display | CLI Display | When to Use |
|------------|-------------|-------------|-------------|
| `error("msg")` from items() | Error modal, cannot enter task | stderr, exit 1 | Missing dependencies, invalid config |
| `error("msg")` from execute() | Error modal with stack trace | stderr, exit 1 | Should not use - prefer exit codes |
| `return msg, 0` from execute() | Success modal (or suppressed) | stdout | Successful execution |
| `return msg, 1` from execute() | Red error modal | stdout, exit 1 | Validation failure, operation failed |
| Return `nil` from preview() | No preview shown | N/A | Preview unavailable |
| Return error string from preview() | Error shown in preview pane | N/A | Preview generation failed |

#### Debugging Tips

1. **Test both TUI and CLI modes** - Error behavior differs
2. **Check exit codes** - Use `echo $?` after CLI execution to verify exit codes
3. **Validate early** - Check inputs at the start of functions
4. **Provide context** - Include file paths, command output in error messages
5. **Use descriptive messages** - "Cannot open file: missing.txt" > "Error"
6. **Handle partial failures** - Report counts of succeeded/failed operations

### Task-Level Execute

For tasks without item sources:

```lua
execute = function()
    -- No items to select, just run the task
    local output = "Task completed"
    local exit_code = 0
    return output, exit_code
end
```

**Parameters:**
- None

**Returns:**
- `output` (string) - Task output message
- `exit_code` (integer) - Exit code (0 = success)

### Task-Level Preview

Fallback preview when item source doesn't define one:

```lua
preview = function(item)
    return "Preview for: " .. item
end
```

**Parameters:**
- `item` (string) - The selected item

**Returns:**
- `string | nil` - Preview text (nil = no preview)

