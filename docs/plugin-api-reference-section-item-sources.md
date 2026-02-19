# Plugin API Reference - Item Sources

> Part of the [Plugin API Reference](plugin-api-reference.md)  
> **See also:** [Tasks](plugin-api-reference-section-tasks.md) | [API Functions](plugin-api-reference-section-api-functions.md)

## Item Source Definition

Item sources provide data for tasks.

### Item Source Structure

```lua
item_sources = {
    source_key = {
        tag = "s",                              -- Required: Short identifier
        items = function() ... end,             -- Required: Return items array
        preselected_items = function() ... end, -- Optional: Return preselected items
        preview = function(item) ... end,       -- Optional: Return preview text
        execute = function(items) ... end,      -- Optional: Execute selected items
    },
}
```

### Required Fields

**`tag`** - Short identifier for item source

```lua
tag = "f"  -- Displayed as "[f] filename"
```

- **Type:** string
- **Required:** Yes (for all item sources)
- **Format:** Short identifier (1-3 chars recommended)

**`items()`** - Return array of items

```lua
items = function()
    return {"item1", "item2", "item3"}
end
```

**Parameters:**
- None

**Returns:**
- `string[]` - Array of items to display

### Optional Fields

**`preselected_items()`** - Items selected by default

```lua
preselected_items = function()
    return {"item1"}  -- Pre-select item1
end
```

**Parameters:**
- None

**Returns:**
- `string[]` - Array of items to pre-select

**Note:** Items must exist in `items()` result

**`preview(item)`** - Show preview for selected item

```lua
preview = function(item)
    return "Preview for: " .. item
end
```

**Parameters:**
- `item` (string) - Currently selected item

**Returns:**
- `string | nil` - Preview text (nil = no preview)

**`execute(items)`** - Execute action on selected items

```lua
execute = function(items)
    local count = #items
    return "Processed " .. count .. " items", 0
end
```

**Parameters:**
- `items` (string[]) - Array of selected items

**Returns:**
- `output` (string) - Execution output message
- `exit_code` (integer) - Exit code (0 = success)

### Validation Rules

Understanding validation requirements for item sources ensures your plugins work correctly and avoid common errors.

#### Tag Validation

**Requirements:**
- **Required** for all item sources (plugin will fail to load without it)
- **Type:** String
- **Cannot be empty string** when task has multiple item sources
- **Length:** No hard limit, but 1-3 characters recommended for UI readability
- **Characters:** Alphanumeric recommended (a-z, A-Z, 0-9)
- **Special characters:** Allowed but may affect UI display
- **Uniqueness:** Tags must be unique within a single task (not across tasks)

**Examples:**

```lua
-- ✅ GOOD: Multiple sources with tags
item_sources = {
    files = {
        tag = "f",  -- Single character, clear
        items = function() return find_files() end,
    },
    directories = {
        tag = "d",  -- Single character, clear
        items = function() return find_directories() end,
    },
}

-- ✅ GOOD: Single source with tag
item_sources = {
    notes = {
        tag = "n",  -- Required even for single source
        items = function() return list_notes() end,
    },
}

-- ✅ ACCEPTABLE: Longer tags
item_sources = {
    local_branches = {
        tag = "loc",  -- 3 characters
        items = function() return get_local_branches() end,
    },
    remote_branches = {
        tag = "rem",  -- 3 characters
        items = function() return get_remote_branches() end,
    },
}

-- ❌ BAD: Duplicate tags
item_sources = {
    files = {
        tag = "f",
        items = function() return get_txt_files() end,
    },
    folders = {
        tag = "f",  -- ERROR: Duplicate tag!
        items = function() return get_folders() end,
    },
}

-- ❌ BAD: Missing tags with multiple sources
item_sources = {
    source1 = {
        -- Missing tag - required for multiple sources
        items = function() return {...} end,
    },
    source2 = {
        -- Missing tag - required for multiple sources
        items = function() return {...} end,
    },
}
```

**UI Display:**
- Tags shown as prefix: `[f] filename.txt`
- Longer tags take more horizontal space: `[local] branch-name`
- Keep tags short for better readability

#### Item String Validation

**Requirements:**
- **Type:** Array of strings (Lua table)
- **Empty strings:** Allowed (shows blank line in UI, not recommended)
- **Empty array:** Allowed (shows "No items" in TUI, task cannot execute)
- **Nil values:** Not allowed (will cause errors)
- **Whitespace:** Not automatically trimmed, included in item identity
- **Newlines in strings:** Allowed but will break UI display (split into multiple lines)
- **Unicode/Emojis:** Fully supported in item strings

**Best Practices:**

```lua
-- ✅ GOOD: Clean item strings
items = function()
    return {
        "file1.txt",
        "file2.txt",
        "README.md",
    }
end

-- ✅ ACCEPTABLE: Empty array for "no results"
items = function()
    local output, code = syntropy.shell("find . -name '*.tmp'")
    if code ~= 0 or output == "" then
        return {}  -- Shows "No items" in UI
    end
    return parse_lines(output)
end

-- ⚠️ ALLOWED BUT NOT RECOMMENDED: Empty strings
items = function()
    return {
        "file1.txt",
        "",  -- Shows as blank line
        "file2.txt",
    }
end

-- ❌ BAD: Nil values in array
items = function()
    return {
        "file1.txt",
        nil,  -- ERROR: Will cause runtime error
        "file2.txt",
    }
end

-- ❌ BAD: Newlines in item strings
items = function()
    return {
        "file1.txt\nline2",  -- Breaks UI display
    }
end

-- ❌ BAD: Mixed types
items = function()
    return {
        "string item",
        123,  -- ERROR: Not a string
        true, -- ERROR: Not a string
    }
end
```

**Whitespace Handling:**

```lua
-- Item strings are NOT trimmed automatically
items = function()
    return {
        "  file.txt  ",  -- Leading/trailing spaces preserved
        "file.txt",      -- Different item from above!
    }
end

-- ✅ GOOD: Explicitly trim if needed
items = function()
    local output, _ = syntropy.shell("ls")
    local items = {}
    for line in output:gmatch("[^\n]+") do
        local trimmed = line:match("^%s*(.-)%s*$")  -- Trim whitespace
        if trimmed ~= "" then
            table.insert(items, trimmed)
        end
    end
    return items
end
```

#### Item Stability for Polling

When using `item_polling_interval`, item strings serve as **identity keys** for selection and marking persistence.

**Requirements:**
- Item strings must remain **stable** across poll cycles
- Same logical item = same string value
- Different string = different item (selection lost)

**Guidelines:**

```lua
-- ❌ BAD: Item identity changes with dynamic data
items = function()
    local processes = {}
    for line in ps_output:gmatch("[^\n]+") do
        local pid, cpu, cmd = line:match("(%d+)%s+([%d.]+)%%%s+(.+)")
        -- CPU % changes every poll → new item identity
        table.insert(processes, pid .. " " .. cpu .. "% " .. cmd)
    end
    return processes
end
-- Problem: User selects "[123] 45% vim", next poll it's "[123] 47% vim" → selection lost

-- ✅ GOOD: Stable item identity
items = function()
    local processes = {}
    for line in ps_output:gmatch("[^\n]+") do
        local pid, cmd = line:match("(%d+)%s+.-%s+(.+)")
        -- PID + command is stable
        table.insert(processes, "[" .. pid .. "] " .. cmd)
    end
    return processes
end
-- User selects "[123] vim", next poll still "[123] vim" → selection preserved
-- Show dynamic data (CPU %) in preview instead
```

**Testing Item Stability:**
1. Enable polling for your task
2. Select an item in TUI
3. Wait for a poll cycle
4. Verify item remains selected
5. If selection disappears, item identity is not stable

#### Validation Rules Summary

| Aspect | Requirement | Default | Validation |
|--------|-------------|---------|------------|
| **Tag** | Required for all sources | N/A | Must be unique within task; cannot be empty string with multiple sources |
| **Items array** | Must be Lua table | Required | Type check on return |
| **Item strings** | Must be strings | Required | Type check per item |
| **Empty array** | Allowed | Shows "No items" | Valid but task cannot execute |
| **Empty string** | Allowed | Shows blank line | Valid but not recommended |
| **Nil in array** | Not allowed | Runtime error | Checked at runtime |
| **Whitespace** | Not trimmed | Preserved as-is | Part of item identity |
| **Newlines in item** | Allowed | Breaks UI display | Avoid in item strings |
| **Unicode/Emoji** | Supported | Displays correctly | Fully supported |
| **Item stability** | Required for polling | N/A | User-tested (selection persistence) |

#### Common Validation Mistakes

**Mistake 1: Forgetting to validate items() return type**
```lua
-- ❌ BAD: Returns nil on error
items = function()
    local file = io.open("data.txt", "r")
    if not file then
        return nil  -- ERROR: Must return array
    end
    -- ...
end

-- ✅ GOOD: Always return array
items = function()
    local file = io.open("data.txt", "r")
    if not file then
        return {}  -- Valid empty array
    end
    -- ...
end
```

**Mistake 2: Not handling empty command output**
```lua
-- ❌ BAD: Empty output returns nil or single empty item
items = function()
    local output, _ = syntropy.shell("find . -name '*.tmp'")
    return {output}  -- Single item with empty string if no files found
end

-- ✅ GOOD: Parse properly, handle empty output
items = function()
    local output, _ = syntropy.shell("find . -name '*.tmp'")
    if output == "" or output == nil then
        return {}
    end

    local items = {}
    for line in output:gmatch("[^\n]+") do
        if line ~= "" then
            table.insert(items, line)
        end
    end
    return items
end
```

**Mistake 3: Including dynamic data in item string for polled tasks**
```lua
-- ❌ BAD: Timestamp/counter in item string (polling task)
item_polling_interval = 1000

items = function()
    local timestamp = os.time()
    return {
        "file1.txt [" .. timestamp .. "]",  -- Changes every poll!
        "file2.txt [" .. timestamp .. "]",
    }
end

-- ✅ GOOD: Static identifiers, dynamic data in preview
item_polling_interval = 1000

items = function()
    return {"file1.txt", "file2.txt"}  -- Stable
end

preview = function(item)
    local stat_out, _ = syntropy.shell("stat " .. item)
    return "Last modified: " .. parse_mtime(stat_out)
end
```

**Mistake 4: Not validating preselected items exist**
```lua
-- ❌ BAD: Preselecting items that don't exist
items = function()
    return {"file1.txt", "file2.txt"}
end

preselected_items = function()
    return {"file3.txt"}  -- Not in items list!
end

-- ✅ GOOD: Only preselect items that exist
items = function()
    return {"file1.txt", "file2.txt"}
end

preselected_items = function()
    return {"file1.txt"}  -- Exists in items list
end
```

