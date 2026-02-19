# Plugin API Reference

Complete reference for the Lua API available to syntropy plugins.

> **New to syntropy plugins?** Start with the **[Plugin Tutorial](plugins.md)** for step-by-step guidance on building your first plugin. This API reference is for looking up detailed specifications, type definitions, and advanced features.

## Quick Navigation

| Section | Description | Best For |
|---------|-------------|----------|
| [Data Structures & Configuration](plugin-api-reference-section-data-structures.md) | Type definitions, metadata, configuration patterns | Plugin structure, type checking, config setup |
| [Task Definition](plugin-api-reference-section-tasks.md) | Task system, execution model, lifecycle, performance | Core plugin logic, understanding workflows |
| [Item Sources](plugin-api-reference-section-item-sources.md) | Item source configuration and validation | Data providers, item lists, validation |
| [API Functions & Utilities](plugin-api-reference-section-api-functions.md) | syntropy.* functions, Lua standard library, utilities | Function reference, API usage |
| [Examples](plugin-api-reference-section-examples.md) | Complete working plugin examples | Learning by example, templates |
| [Advanced Topics](plugin-api-reference-section-advanced.md) | Module system, plugin organization | Complex plugins, code reuse |

## Getting Started Path

**First-time plugin developers:**
1. **Start with [Plugin Tutorial](plugins.md)** - Step-by-step guide to building your first plugin
2. **Return here for reference** - Use this API reference to look up specific types, functions, and behaviors

**Experienced developers looking for reference material:**
1. **Read the [Overview](#overview)** below to understand the plugin environment
2. **Check [Examples](plugin-api-reference-section-examples.md)** - start with the Minimal Plugin example
3. **Study [Data Structures](plugin-api-reference-section-data-structures.md)** to understand plugin structure and types
4. **Learn [Task Definition](plugin-api-reference-section-tasks.md)** to understand task execution models
5. **Explore [API Functions](plugin-api-reference-section-api-functions.md)** for runtime utilities
6. **Dive into [Advanced Topics](plugin-api-reference-section-advanced.md)** when building complex plugins

## Overview

Plugins run in a sandboxed Lua 5.4 environment with:
- **Standard Lua library** (except `os.exit`, `os.execute`)
- **syntropy namespace** (`syntropy.shell`, `syntropy.expand_path`, `syntropy.invoke_tui`, `syntropy.invoke_editor`)
- **Global utilities** (`merge` function for plugin overrides)
- **Type annotations** (`PluginDefinition` for base plugins, `PluginOverride` for config overrides)

The **syntropy.*** functions are **async-aware** - you can call them normally, syntropy handles async execution. Regular Lua code is synchronous.

### Plugin Types

- Use `---@type PluginDefinition` for standalone/base plugins (both `metadata` and `tasks` required)
- Use `---@type PluginOverride` for config directory overrides (both fields optional)
- See [Data Structures](plugin-api-reference-section-data-structures.md) for complete type definitions

### Plugin Environment

**Sandboxing:**
- Plugins share a single Lua VM and can access each other's globals by plugin name
- Use `syntropy.shell()` for system operations (recommended over direct `io.*` calls)
- Cannot call blocking system functions (`os.exit`, `os.execute`)

**Execution Context:**
- CLI mode: Task executed once, then exits
- TUI mode: Task executed within interactive interface, items can be polled and refreshed

**Best Practices:**
- Keep task functions fast (< 100ms recommended for good UX)
- Use `item_polling_interval` for live-updating data
- Handle errors gracefully (return error messages, don't crash)
- Test plugins in both CLI and TUI modes

---

For detailed information, navigate to the specific sections above.

**Related Documentation:**
- [Plugin Tutorial](plugins.md) - Step-by-step guide to building plugins
- [Configuration Reference](config-reference.md) - Syntropy configuration options
- [Recipes](recipes.md) - Integration examples and workflows
