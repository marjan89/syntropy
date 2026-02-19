# Plugin API Reference - Advanced Topics

> Part of the [Plugin API Reference](plugin-api-reference.md)  
> **See also:** [Data Structures](plugin-api-reference-section-data-structures.md) | [Examples](plugin-api-reference-section-examples.md)

## Advanced Topics

### Module Loading

Plugins can organize code across multiple Lua files using the standard `require()` function. Syntropy uses a Neovim-style module structure with mandatory namespacing for plugin isolation.

#### Plugin Isolation and Namespacing

**Important:** As of version 0.3.4, syntropy uses Neovim-style plugin structure with mandatory namespacing.

**Why namespacing?**
- **Plugin isolation**: Each plugin's modules are namespaced under the plugin name
- **Prevents conflicts**: Multiple plugins can have modules with the same name without colliding
- **Explicit dependencies**: Makes it clear which plugin a module belongs to

**Namespace requirement:**
- Plugin-specific modules MUST be imported with namespace: `require("pluginname.module")`

#### require

Load external Lua modules from plugin-specific directories.

```lua
-- For plugin-specific modules, MUST use namespaced imports:
local module = require("pluginname.module_name")
```

**Parameters:**
- `module_name` (string) - Module name (without `.lua` extension)

**Returns:**
- Module's return value (typically a table)

**Module Search Paths:**

Syntropy configures a **global `package.path`** at startup for all plugins. The search order is:

1. **All plugin lua directories** (in plugin discovery order):
   - Each plugin's `<plugin-dir>/lua/?.lua` and `<plugin-dir>/lua/?/init.lua` paths
   - Plugin-specific modules (private to the plugin, must use namespaced imports)
   - Example: `~/.config/syntropy/plugins/my-plugin/lua/my-plugin/utils.lua`
   - Also supports directory-style modules: `<plugin-dir>/lua/?/init.lua`
   - Example: `~/.config/syntropy/plugins/my-plugin/lua/my-plugin/init.lua`
   - This allows `require("my-plugin")` to load the init.lua file

2. **Lua standard library**: Built-in Lua modules

**Note:** All plugins share the same `package.path` configured once at startup, not per-plugin. First match wins across all paths.

**Precedence Rules:**

Module resolution follows these precedence rules:

1. **Plugin-specific modules are namespaced and isolated**: Plugin modules in `lua/pluginname/` are accessed via `require("pluginname.module")` and cannot conflict with other plugins due to namespacing
2. **First match wins**: Once a module is found, the search stops

**Directory Structure Example:**
```
~/.config/syntropy/plugins/
└── my-plugin/
    ├── plugin.lua
    └── lua/
        └── my-plugin/          # Namespace matches plugin name
            ├── utils.lua       # require("my-plugin.utils")
            ├── parser.lua      # require("my-plugin.parser")
            └── helpers/
                └── init.lua    # require("my-plugin.helpers")
```

**Examples:**

**Example: Plugin-specific module**

```lua
-- File: ~/.config/syntropy/plugins/my-plugin/lua/my-plugin/parser.lua
local parser = {}
function parser.parse(text)
    local result = {}
    for item in text:gmatch("[^,]+") do
        table.insert(result, item)
    end
    return result
end
return parser

-- File: ~/.config/syntropy/plugins/my-plugin/plugin.lua
local parser = require("my-plugin.parser")  -- MUST use namespaced import
local data = parser.parse("a,b,c")  -- {"a", "b", "c"}
```

**Notes:**
- Modules are cached after first load (standard Lua behavior)
- Module paths are configured automatically during plugin loading
- No environment variables or configuration needed
- Plugin modules MUST use namespaced imports: `require("pluginname.module")`
- Module files must return a value (typically a table)

---

For conceptual overview and tutorial, see [Plugin Development Guide](plugins.md).
