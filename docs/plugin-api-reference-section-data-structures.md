# Plugin API Reference - Data Structures & Configuration

> Part of the [Plugin API Reference](plugin-api-reference.md)  
> **See also:** [Tasks](plugin-api-reference-section-tasks.md) | [Examples](plugin-api-reference-section-examples.md)

## Data Structures

### PluginDefinition

For base/standalone plugins. Both `metadata` and `tasks` are required.

```lua
---@class PluginDefinition
{
    metadata = Metadata,      -- Required: Plugin metadata
    tasks = table<string, Task>, -- Required: Task definitions
    config = table?,          -- Optional: Custom configuration table
    -- [any other custom fields] -- Optional: Plugins can have arbitrary custom fields
}
```

**Use when:**
- Creating a new plugin from scratch
- Plugin is NOT merging with another plugin
- You need to define all plugin fields (standalone/complete plugin)
- Typical location: `~/.local/share/syntropy/plugins/` (but can be anywhere)

### PluginOverride

For override plugins that merge with existing base plugins. Both `metadata` and `tasks` are optional - specify only what you want to override.

```lua
---@class PluginOverride
{
    metadata = MetadataOverride?,  -- Optional: Override plugin metadata
    tasks = table<string, Task>?, -- Optional: Override or add specific tasks
    config = table?,              -- Optional: Override configuration table
    -- [any other custom fields]  -- Optional: Override arbitrary custom fields
}
```

**Use when:**
- Merging with an existing base plugin (config overrides data)
- You only want to change specific fields (icon, task descriptions, add new tasks, etc.)
- You want to customize a base plugin without modifying the original
- Typical location: `~/.config/syntropy/plugins/` (to override plugins in data directory)

**Key differences:**
- `PluginDefinition`: Requires both `metadata` and `tasks` fields with complete data
- `PluginOverride`: All fields optional - only specify what you're changing
- Override plugins use `MetadataOverride` (only `name` required, other fields optional)
- Override plugins merge with base plugins (config overrides data)

**Custom fields:**
- Plugins can define arbitrary custom fields beyond `metadata` and `tasks`
- Common pattern: `config` table for user-configurable settings (storage paths, default values, etc.)
- Custom fields are deep merged when using PluginOverride pattern
- Access custom fields from task functions via module-level variables

### Metadata

For base/standalone plugins. Both `name` and `version` are required.

```lua
---@class Metadata
{
    name = "string",            -- Required
    version = "1.0.0",          -- Required (semver)
    icon = "P",                 -- Optional
    description = "...",        -- Optional
    platforms = {"macos"},      -- Optional
}
```

### MetadataOverride

For override plugins. Only `name` is required (must match base plugin name). All other fields are optional.

```lua
---@class MetadataOverride
{
    name = "string",            -- Required (must match base plugin name)
    version = "1.0.0",          -- Optional: Override version
    icon = "P",                 -- Optional: Override icon
    description = "...",        -- Optional: Override description
    platforms = {"macos"},      -- Optional: Override platforms
}
```

### Task

```lua
---@class Task
{
    description = "string",                 -- Required: Task description (non-empty)
    name = "string",                        -- Optional
    mode = "multi" | "none",                -- Optional
    execution_confirmation_message = "string", -- Optional
    suppress_success_notification = boolean, -- Optional (default: false)
    item_polling_interval = integer,        -- Optional (milliseconds, 0 = disabled)
    preview_polling_interval = integer,     -- Optional (milliseconds, 0 = disabled)
    item_sources = table<string, ItemSource>, -- Optional
    pre_run = function(),                   -- Optional
    post_run = function(),                  -- Optional
    execute = function(),                   -- Optional
    preview = function(item),               -- Optional
}
```

### ItemSource

```lua
---@class ItemSource
{
    tag = "s",                              -- Required if multiple sources
    items = function(),                     -- Required
    preselected_items = function(),         -- Optional
    preview = function(item),               -- Optional
    execute = function(items),              -- Optional
}
```

## Plugin Metadata

### Required Fields

```lua
metadata = {
    name = "plugin-name",     -- Required: Unique identifier (kebab-case recommended)
    version = "1.0.0",        -- Required: Semantic version (major.minor.patch)
}
```

### Optional Fields

```lua
metadata = {
    icon = "P",               -- Optional: Single char (Unicode/Nerd Font OK, no emojis)
    description = "...",      -- Optional: Short description
    platforms = {"macos"},    -- Optional: Platform filter (macos, linux, windows)
}
```

### Metadata Validation Rules

| Field | Type | Rules |
|-------|------|-------|
| `name` | string | Required, non-empty, unique across plugins |
| `version` | string | Required, valid semver (X.Y.Z) |
| `icon` | string | Optional, must occupy exactly 1 terminal cell (Unicode/Nerd Font OK, emojis forbidden) |
| `description` | string | Optional, any length |
| `platforms` | array | Optional, filter plugin by OS |

**Platform detection:**
- `macos` - macOS systems
- `linux` - Linux distributions
- `windows` - Windows systems

If `platforms` is omitted, plugin runs on all platforms.

## Plugin Configuration

Plugins can define custom configuration tables to store user-configurable settings. This is a common pattern for handling default values, file paths, and other settings that users may want to override.

### Custom Configuration Tables

Plugins can have arbitrary custom fields beyond the required `metadata` and `tasks` fields. The most common pattern is a `config` table:

```lua
---@type PluginDefinition
local plugin = {
    metadata = {
        name = "notes",
        version = "1.0.0",
    },
    config = {
        storage_dir = "~/.syntropy-notes",  -- Default storage location
        file_extension = ".md",         -- Default file extension
        editor = nil,                   -- nil = use $EDITOR
    },
    tasks = {
        -- ...tasks access plugin.config...
    },
}
return plugin
```

**Key points:**
- `config` is just a convention - you can use any field name
- Config fields can store any data: strings, numbers, tables, etc.
- Common use: storing unexpanded paths (tilde, env vars) for runtime expansion

### Runtime Path Expansion Pattern

**Important:** `syntropy.expand_path()` cannot be called at module level (top of plugin.lua). It only works inside runtime functions like `items()`, `execute()`, `preview()`, `pre_run()`, and `post_run()`.

**Why this limitation exists:**
- Plugin-relative paths (`./`, `../`) require plugin context which isn't available at module load time
- Module-level code runs during plugin loading, before runtime context is established
- Attempting to call `syntropy.expand_path("./file")` at module level will error: "Cannot resolve relative path: no plugin context"

**The solution:** Store unexpanded paths in your config table, then expand them at runtime:

```lua
---@type PluginDefinition
local plugin = {
    metadata = {name = "notes", version = "1.0.0"},

    -- Store unexpanded path in config
    config = {
        storage_dir = "~/.syntropy-notes",  -- NOT expanded yet
    },

    tasks = {
        list = {
            item_sources = {
                notes = {
                    items = function()
                        -- ✅ Expand at runtime inside function
                        local dir = syntropy.expand_path(plugin.config.storage_dir)

                        -- Now use the expanded path
                        local output, _ = syntropy.shell("ls " .. dir)
                        -- ...
                    end,
                },
            },
        },
    },
}
return plugin
```

**What NOT to do:**
```lua
---@type PluginDefinition
local plugin = {
    metadata = {name = "notes", version = "1.0.0"},
    config = {
        -- ❌ FAILS: Cannot call syntropy.expand_path at module level
        storage_dir = syntropy.expand_path("~/.syntropy-notes"),
    },
}
-- Error: Cannot resolve relative path: no plugin context
```

### Accessing Configuration in Tasks

Access config values from task functions using module-level variables:

```lua
---@type PluginDefinition
local plugin = {
    config = {
        storage_dir = "~/.notes",
        max_items = 100,
    },
    tasks = {
        list = {
            item_sources = {
                notes = {
                    items = function()
                        -- Access via plugin.config
                        local dir = syntropy.expand_path(plugin.config.storage_dir)
                        local max = plugin.config.max_items

                        local cmd = string.format("ls %s | head -n %d", dir, max)
                        local output, _ = syntropy.shell(cmd)
                        -- ...
                    end,
                },
            },
        },
    },
}
return plugin
```

### Overriding Configuration

Users can override config values by placing an override plugin in `~/.config/syntropy/plugins/`:

```lua
---@type PluginOverride
return {
    metadata = {
        name = "notes",  -- Must match base plugin name
    },
    config = {
        storage_dir = "~/Documents/Notes",  -- Override default location
        max_items = 50,                      -- Override default limit
        -- file_extension not specified = inherits from base
    },
}
```

**Merge behavior:**
- Config tables are deep merged (like all plugin fields)
- Override values replace base values
- Unspecified fields in override inherit from base
- Works for nested config structures too

**Example with nested config:**
```lua
-- Base plugin
config = {
    paths = {
        inbox = "~/notes/inbox",
        archive = "~/notes/archive",
    },
    display = {
        show_dates = true,
        show_tags = true,
    },
}

-- Override plugin
config = {
    paths = {
        inbox = "~/Documents/inbox",  -- Override inbox only
        -- archive inherits from base
    },
    -- display inherits entirely from base
}

-- Resulting merged config
config = {
    paths = {
        inbox = "~/Documents/inbox",   -- Overridden
        archive = "~/notes/archive",   -- Inherited
    },
    display = {
        show_dates = true,             -- Inherited
        show_tags = true,              -- Inherited
    },
}
```

### Best Practices

1. **Store unexpanded paths in config** - Let users provide tilde/env var paths, expand at runtime
2. **Use sensible defaults** - Base plugin should work without override
3. **Document config options** - Add comments explaining each config field
4. **Validate config values** - Check for required fields in `pre_run()` or `items()`
5. **Support nil for system defaults** - e.g., `editor = nil` means "use $EDITOR"

**Example with validation:**
```lua
tasks = {
    edit = {
        pre_run = function()
            -- Validate required config at runtime
            if not plugin.config.storage_dir then
                error("storage_dir not configured")
            end
        end,
        execute = function()
            local dir = syntropy.expand_path(plugin.config.storage_dir)
            -- ...
        end,
    },
}
```

