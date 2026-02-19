# Plugin API Reference - Examples

> Part of the [Plugin API Reference](plugin-api-reference.md)  
> **See also:** [Data Structures](plugin-api-reference-section-data-structures.md) | [Tasks](plugin-api-reference-section-tasks.md)

## Examples

### Minimal Plugin

```lua
---@type PluginDefinition
return {
    metadata = {
        name = "minimal",
        version = "1.0.0",
    },
    tasks = {
        hello = {
            execute = function()
                return "Hello, World!", 0
            end,
        },
    },
}
```

### Complete Plugin with All Features

```lua
---@type PluginDefinition
return {
    metadata = {
        name = "example",
        version = "2.1.0",
        icon = "E",
        description = "Example plugin showing all features",
        platforms = {"macos", "linux"},
    },
    tasks = {
        list_and_process = {
            name = "Process Items",
            description = "List and process text files with real-time updates",
            mode = "multi",
            exit_on_execute = false,           -- Stay open after execution (default)
            execution_confirmation_message = "Are you sure you want to process:",
            suppress_success_notification = false,  -- Show success modal (default)
            item_polling_interval = 3000,      -- Refresh items every 3 seconds
            preview_polling_interval = 1000,   -- Refresh preview every 1 second

            pre_run = function()
                print("Starting task...")
            end,

            post_run = function()
                print("Task complete!")
            end,

            item_sources = {
                files = {
                    tag = "f",
                    items = function()
                        local out, _ = syntropy.shell("ls *.txt")
                        local items = {}
                        for line in out:gmatch("[^\n]+") do
                            table.insert(items, line)
                        end
                        return items
                    end,

                    preselected_items = function()
                        return {"README.txt"}
                    end,

                    preview = function(item)
                        local path = syntropy.expand_path("~/" .. item)
                        local file = io.open(path, "r")
                        if not file then return "File not found" end
                        local content = file:read("*all")
                        file:close()
                        return content
                    end,

                    execute = function(items)
                        local count = 0
                        for _, item in ipairs(items) do
                            local _, code = syntropy.shell("process " .. item)
                            if code == 0 then count = count + 1 end
                        end
                        return "Processed " .. count .. " files", 0
                    end,
                },
            },
        },
    },
}
```

### Plugin with Custom Configuration

This example shows the recommended pattern for plugins with user-configurable settings.

**Base plugin** (`~/.local/share/syntropy/plugins/syntropy-notes/plugin.lua`):

```lua
---@type PluginDefinition
local plugin = {
    metadata = {
        name = "syntropy-notes",
        version = "1.0.0",
        icon = "N",
        description = "Manage markdown notes",
    },

    -- Custom configuration table with defaults
    config = {
        storage_dir = "~/.syntropy-notes",  -- Unexpanded path (will expand at runtime)
        file_extension = ".md",
        editor = nil,                   -- nil = use $EDITOR
    },

    tasks = {
        list = {
            name = "List Notes",
            description = "Browse and edit notes",
            mode = "none",

            item_sources = {
                notes = {
                    tag = "n",
                    items = function()
                        -- ✅ Expand path at runtime inside function
                        local dir = syntropy.expand_path(plugin.config.storage_dir)
                        local ext = plugin.config.file_extension

                        -- Ensure storage directory exists
                        syntropy.shell("mkdir -p " .. dir)

                        -- List notes with extension filter
                        local cmd = string.format("ls -1 %s/*%s 2>/dev/null | xargs -n1 basename", dir, ext)
                        local output, code = syntropy.shell(cmd)

                        if code ~= 0 then
                            return {}
                        end

                        local notes = {}
                        for line in output:gmatch("[^\n]+") do
                            table.insert(notes, line)
                        end
                        return notes
                    end,

                    preview = function(item)
                        local dir = syntropy.expand_path(plugin.config.storage_dir)
                        local path = dir .. "/" .. item

                        local file = io.open(path, "r")
                        if not file then
                            return "Error: Could not read " .. item
                        end

                        local content = file:read("*all")
                        file:close()
                        return content
                    end,

                    execute = function(items)
                        if #items == 0 then
                            return "No note selected", 1
                        end

                        local dir = syntropy.expand_path(plugin.config.storage_dir)
                        local path = dir .. "/" .. items[1]

                        -- Use configured editor or system default
                        local code = syntropy.invoke_editor(path)

                        if code == 0 then
                            return "Note edited", 0
                        else
                            return "Editor exited with error", code
                        end
                    end,
                },
            },
        },

        new = {
            name = "New Note",
            description = "Create a new note",

            execute = function()
                local dir = syntropy.expand_path(plugin.config.storage_dir)
                local ext = plugin.config.file_extension

                -- Ensure directory exists
                syntropy.shell("mkdir -p " .. dir)

                -- Generate filename with timestamp
                local filename = os.date("%Y%m%d-%H%M%S") .. ext
                local path = dir .. "/" .. filename

                -- Create empty file
                local file = io.open(path, "w")
                if not file then
                    return "Error: Could not create note", 1
                end
                file:write("# New Note\n\n")
                file:close()

                -- Open in editor
                local code = syntropy.invoke_editor(path)

                if code == 0 then
                    return "Note created: " .. filename, 0
                else
                    return "Editor exited with error", code
                end
            end,
        },
    },
}

return plugin
```

**Override plugin** (`~/.config/syntropy/plugins/syntropy-notes/plugin.lua`):

```lua
---@type PluginOverride
return {
    metadata = {
        name = "syntropy-notes",  -- Must match base plugin name
    },

    -- Override configuration - change storage location
    config = {
        storage_dir = "~/Documents/Notes",  -- User's preferred location
        -- file_extension and editor inherit from base plugin
    },
}
```

**Result after merging:**

The final merged plugin will have:
- `config.storage_dir = "~/Documents/Notes"` (from override)
- `config.file_extension = ".md"` (inherited from base)
- `config.editor = nil` (inherited from base)

**Key patterns demonstrated:**
1. **Unexpanded paths in config** - Paths use `~` and are stored as-is
2. **Runtime expansion** - `syntropy.expand_path()` called inside `items()` and `execute()`
3. **Config access** - Task functions access `plugin.config.*` values
4. **Override merging** - Override only specifies changed values, inherits the rest
5. **Sensible defaults** - Base plugin works out-of-the-box, override is optional

### Multi-Source Task

```lua
---@type PluginDefinition
return {
    metadata = {
        name = "file-manager",
        version = "1.0.0",
    },
    tasks = {
        manage_items = {
            name = "Manage Files and Folders",
            description = "Delete files or directories in the current directory",
            mode = "multi",
            item_sources = {
            files = {
                tag = "f",
                items = function()
                    local out, _ = syntropy.shell("find . -type f -maxdepth 1")
                    return parse_lines(out)
                end,
                execute = function(items)
                    for _, file in ipairs(items) do
                        syntropy.shell("rm " .. file)
                    end
                    return "Deleted " .. #items .. " files", 0
                end,
            },
            directories = {
                tag = "d",
                items = function()
                    local out, _ = syntropy.shell("find . -type d -maxdepth 1")
                    return parse_lines(out)
                end,
                execute = function(items)
                    for _, dir in ipairs(items) do
                        syntropy.shell("rm -rf " .. dir)
                    end
                    return "Deleted " .. #items .. " directories", 0
                end,
            },
        },
    },
}

function parse_lines(text)
    local lines = {}
    for line in text:gmatch("[^\n]+") do
        if line ~= "" and line ~= "." then
            table.insert(lines, line)
        end
    end
    return lines
end
}
```

### Override Plugin Examples

These examples show different patterns for overriding existing plugins.

**Example 1: Minimal Override (Change Icon Only)**

```lua
---@type PluginOverride
return {
    metadata = {
        name = "git",     -- Must match base plugin name
        icon = "🔀",      -- Override just the icon
    },
    -- No tasks field - inherit all tasks from base plugin
}
```

**Example 2: Override Task Description**

```lua
---@type PluginOverride
return {
    metadata = {
        name = "docker",
    },
    tasks = {
        ps = {
            description = "My custom description for docker ps",  -- Override just this field
        },
        -- Other tasks (build, run, etc.) inherit from base
    },
}
```

**Example 3: Override Plugin Configuration**

```lua
---@type PluginOverride
return {
    metadata = {
        name = "syntropy-notes",
    },
    config = {
        storage_dir = "~/Documents/Notes",  -- Override default storage location
        file_extension = ".txt",            -- Override default extension
        -- Other config fields inherit from base plugin
    },
}
```

This pattern allows users to customize plugin behavior without modifying the base plugin code. The config table is deep merged, so you only need to specify the values you want to change.

**Example 4: Add New Task to Existing Plugin**

```lua
---@type PluginOverride
return {
    metadata = {
        name = "git",
    },
    tasks = {
        -- Existing tasks (status, commit, push) inherited from base
        my_workflow = {  -- Add a completely new task
            name = "My Workflow",
            description = "Custom git workflow: pull, rebase, push",
            execute = function()
                local out1, code1 = syntropy.shell("git pull --rebase")
                if code1 ~= 0 then
                    return "Pull failed: " .. out1, code1
                end

                local out2, code2 = syntropy.shell("git push")
                if code2 ~= 0 then
                    return "Push failed: " .. out2, code2
                end

                return "Workflow complete", 0
            end,
        },
    },
}
```

**Example 5: Override Multiple Fields**

```lua
---@type PluginOverride
return {
    metadata = {
        name = "npm",
        icon = "N",
        description = "My customized npm plugin",
    },
    tasks = {
        install = {
            -- Override the execution confirmation message
            execution_confirmation_message = "Install these packages:",
        },
        dev = {
            -- Override the task name
            name = "Start Dev Server",
            description = "Run npm run dev with my custom settings",
        },
        -- Add new task
        clean_install = {
            name = "Clean Install",
            description = "Remove node_modules and reinstall",
            execute = function()
                syntropy.shell("rm -rf node_modules package-lock.json")
                local out, code = syntropy.shell("npm install")
                return out, code
            end,
        },
    },
}
```

**Example 6: Omit Tasks Entirely (Metadata-Only Override)**

```lua
---@type PluginOverride
return {
    metadata = {
        name = "kubernetes",
        icon = "☸",
        description = "Kubernetes management (custom icon)",
    },
    -- No tasks field at all - inherit everything from base plugin
}
```

**Key Patterns:**

1. **Metadata-only override** - Change plugin appearance without touching tasks
2. **Selective task override** - Modify specific properties of existing tasks
3. **Config override** - Customize plugin settings (paths, defaults) without modifying code
4. **Task addition** - Add new tasks while inheriting existing ones
5. **Hybrid approach** - Combine metadata changes, task overrides, config changes, and new tasks

**Remember:**
- `metadata.name` must match the base plugin name for merging to work
- Omitted fields inherit from the base plugin
- Specified fields in override take precedence over base
- Deep merge applies to nested objects (like task properties, config tables, and other custom fields)

