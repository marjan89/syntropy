# Configuration Reference

Complete reference for syntropy's configuration file.

**Config location:** `~/.config/syntropy/config.toml`

## Table of Contents

- [Root Configuration](#root-configuration)
- [Plugin Management](#plugin-management)
- [Keybindings](#keybindings)
- [Styles](#styles)
- [Validation Rules](#validation-rules)
- [Complete Example](#complete-example)

## Root Configuration

Top-level configuration options.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `default_plugin` | string | (none) | Plugin to show on startup |
| `default_task` | string | (none) | Task to show on startup (requires `default_plugin`) |
| `default_plugin_icon` | string | `"⚒"` | Fallback icon for plugins without icon (must be 1 terminal cell) |
| `status_bar` | bool | `true` | Show status bar with breadcrumbs and hints |
| `search_bar` | bool | `true` | Show fuzzy search input at bottom |
| `show_preview_pane` | bool | `true` | Show preview pane for selected items |
| `exit_on_execute` | bool | `false` | Exit TUI after executing task |

### CLI Overrides

Several UI configuration options can be overridden at runtime using CLI flags, allowing you to customize behavior without modifying your config file:

| CLI Flag | Config Field | Description |
|----------|--------------|-------------|
| `--status-bar` | `status_bar` | Override status bar visibility |
| `--search-bar` | `search_bar` | Override search bar visibility |
| `--show-preview-pane` | `show_preview_pane` | Override preview pane visibility |
| `--exit-on-execute` | `exit_on_execute` | Override exit on execute behavior |

**Usage:**
```bash
# Disable status bar for this session only
syntropy --status-bar=false

# Enable exit-on-execute without changing config
syntropy --exit-on-execute=true

# Combine multiple overrides
syntropy --search-bar=false --show-preview-pane=false
```

CLI flags take precedence over config file settings and do not modify the config file.

### Plugin Discovery

Syntropy discovers plugins from both directories:
1. `~/.config/syntropy/plugins/` (user-created plugins)
2. `~/.local/share/syntropy/plugins/` (managed plugins installed via `syntropy plugins --install`)

**Plugin Precedence:**
When plugins with the same name exist in both directories, the config directory (`~/.config/syntropy/plugins/`) takes precedence. This allows you to:
- Override managed plugins with local customizations
- Test modifications to installed plugins
- Maintain personal forks of community plugins

The entire plugin directory is used from the precedence-winning location (not merged at the file level).

## Plugin Management

Declare git-based plugins to install via `syntropy plugins --install`.

### Plugin Declaration

```toml
[plugins.plugin-name]
git = "https://github.com/user/syntropy-plugin-name"
tag = "v1.0.0"  # OR use commit (not both)
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `git` | string | Yes | Git URL (must start with `https://` or `git@`) |
| `tag` | string | Conditional* | Git tag to checkout (e.g., `"v1.0.0"`) |
| `commit` | string | Conditional* | Git commit SHA to checkout (e.g., `"abc123"`) |

\* Must specify **exactly one** of `tag` or `commit` (not both, not neither)

**Validation:**
- Git URL must start with `https://` or `git@`
- Git URL cannot be empty
- Must specify exactly one of `tag` or `commit` (not both, not neither)

**Examples:**

```toml
# Using tag
[plugins.packages]
git = "https://github.com/user/syntropy-plugin-packages"
tag = "v2.1.0"

# Using commit
[plugins.backups]
git = "https://github.com/user/syntropy-plugin-backups"
commit = "a1b2c3d"
```

## Keybindings

Customize keyboard shortcuts.

### Default Keybindings

| Action | Field | Default | Description |
|--------|-------|---------|-------------|
| Navigate up | `select_previous` | `"<up>"` | Move selection up |
| Navigate down | `select_next` | `"<down>"` | Move selection down |
| Confirm | `confirm` | `"<enter>"` | Execute or navigate forward |
| Go back | `back` | `"<esc>"` | Return to previous screen |
| Toggle select | `select` | `"<tab>"` | Toggle item selection (multi-mode) |
| Scroll preview up | `scroll_preview_up` | `"<C-up>"` | Scroll preview pane up |
| Scroll preview down | `scroll_preview_down` | `"<C-down>"` | Scroll preview pane down |
| Toggle preview | `toggle_preview` | `"<C-p>"` | Show/hide preview pane |

### Key Binding Format

| Format | Example | Description |
|--------|---------|-------------|
| `<char>` | `a`, `b`, `1` | Single character |
| `<key>` | `<up>`, `<down>`, `<enter>`, `<esc>`, `<tab>` | Special keys (see below) |
| `<C-x>` | `<C-p>`, `<C-n>` | Ctrl + key |
| `<S-x>` | `<S-tab>` | Shift + key |
| `<A-x>` | `<A-q>` | Alt + key |

**Supported Special Keys:**

| Key | Aliases | Description |
|-----|---------|-------------|
| `<up>`, `<down>`, `<left>`, `<right>` | | Arrow keys |
| `<enter>` | | Enter/Return key |
| `<esc>` | | Escape key |
| `<tab>` | | Tab key |
| `<backspace>` | `<bs>` | Backspace key |
| `<delete>` | `<del>` | Delete key |
| `<home>`, `<end>` | | Home/End keys |
| `<pageup>`, `<pagedown>` | `<pgup>`, `<pgdn>` | Page Up/Page Down keys |
| `<f1>` through `<f12>` | | Function keys |

**Validation:**
- Cannot be empty (error: `"Empty key binding"`)
- Cannot have invalid format (error: `"Invalid key binding format: '<key>'"`)
- Cannot use unknown keys (error: `"Unknown key: '<key>'"`)
- Cannot use unknown modifiers (error: `"Unknown modifier: '<modifier>' (use C, S, or A)"`)
- Cannot duplicate same binding (error: `"Duplicate key bindings detected:\n  <details>"`)

**Example:**

```toml
[keybindings]
back = "<esc>"
select_previous = "k"  # Vim-style
select_next = "j"      # Vim-style
confirm = "<enter>"
select = "<space>"
```

## Styles

Customize TUI appearance.

### Screen Scaffold

Controls main screen split (left = list, right = preview).

```toml
[styles.screen_scaffold]
left_split = 50   # Percentage (0-100)
right_split = 50  # Percentage (0-100)
```

**Validation:** `left_split + right_split` must equal `100`

### Status Bar

```toml
[styles.status]
left_split = 70
right_split = 30
borders = ["all"]
font_weight = "bold"
breadcrumbs_separator = " → "
idle_icons = ["✔"]
error_icons = ["⛌"]
complete_icons = ["✔"]
running_icons = ["✴", "✵"]
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `left_split` | number | `50` | Left percentage (breadcrumbs) |
| `right_split` | number | `50` | Right percentage (hints) |
| `borders` | array | `["all"]` | Border sides (see Border Options) |
| `font_weight` | string | `"bold"` | `"bold"` or `"regular"` |
| `breadcrumbs_separator` | string | `" → "` | Separator between breadcrumb items |
| `idle_icons` | array | `["✔"]` | Icons when no task running |
| `error_icons` | array | `["⛌"]` | Icons when task failed |
| `complete_icons` | array | `["✔"]` | Icons when task succeeded |
| `running_icons` | array | `["✴", "✵"]` | Icons cycling during task execution |

**Validation:** `left_split + right_split` must equal `100`

### Modal

```toml
[styles.modal]
borders = ["all"]
font_weight = "regular"
show_title = true
scroll_offset = 2
vertical_size = 60
horizontal_size = 60
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `borders` | array | `["all"]` | Border sides |
| `font_weight` | string | `"regular"` | `"bold"` or `"regular"` |
| `show_title` | bool | `true` | Show modal title |
| `scroll_offset` | number | `2` | Lines to keep visible when scrolling |
| `vertical_size` | number | `60` | Height percentage (1-99) |
| `horizontal_size` | number | `60` | Width percentage (1-99) |

**Validation:** Both size fields must be `< 100` (recommend using values between 20-90 for practical usability)

### List

```toml
[styles.list]
highlight_symbol = "→"
icon_marked = "▣"
icon_unmarked = "□"
font_weight = "regular"
borders = ["all"]
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `highlight_symbol` | string | `"→"` | Symbol before selected item |
| `icon_marked` | string | `"▣"` | Icon for selected items (multi-mode) |
| `icon_unmarked` | string | `"□"` | Icon for unselected items (multi-mode) |
| `font_weight` | string | `"regular"` | `"bold"` or `"regular"` |
| `borders` | array | `["all"]` | Border sides |

### Preview

```toml
[styles.preview]
borders = ["all"]
font_weight = "regular"
show_title = true
scroll_offset = 2
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `borders` | array | `["all"]` | Border sides |
| `font_weight` | string | `"regular"` | `"bold"` or `"regular"` |
| `show_title` | bool | `true` | Show preview pane title |
| `scroll_offset` | number | `2` | Lines to keep visible when scrolling |

### Search Bar

```toml
[styles.search_bar]
borders = ["all"]
font_weight = "bold"
search_hint = ">"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `borders` | array | `["all"]` | Border sides |
| `font_weight` | string | `"bold"` | `"bold"` or `"regular"` |
| `search_hint` | string | `">"` | Prompt symbol for search input |

### Colors

Customize TUI colors. All fields are optional and default to terminal colors.

#### Global Colors

```toml
[styles.colors]
highlights_background = "terminal"  # Or hex: "#ff0000", named: "red"
highlights_text = "terminal"
borders = "terminal"
text = "terminal"
background = "terminal"
```

| Field | Default | Description |
|-------|---------|-------------|
| `highlights_background` | `"terminal"` | Background for selected items |
| `highlights_text` | `"terminal"` | Text color for selected items |
| `borders` | `"terminal"` | Global border color |
| `text` | `"terminal"` | Global text color |
| `background` | `"terminal"` | Global background color |

#### Component-Specific Colors (Optional Overrides)

Override global colors for specific components. Empty string `""` falls back to global.

**Border Colors:**

| Field | Fallback | Description |
|-------|----------|-------------|
| `borders_list` | `borders` | List border color |
| `borders_preview` | `borders` | Preview pane border color |
| `borders_search` | `borders` | Search bar border color |
| `borders_status` | `borders` | Status bar border color |
| `borders_modal` | `borders` | Modal border color |

**Text Colors:**

| Field | Fallback | Description |
|-------|----------|-------------|
| `text_list` | `text` | List text color |
| `text_preview` | `text` | Preview pane text color |
| `text_search` | `text` | Search bar text color |
| `text_status` | `text` | Status bar text color |
| `text_modal` | `text` | Modal text color |

**Background Colors:**

| Field | Fallback | Description |
|-------|----------|-------------|
| `background_list` | `background` | List background color |
| `background_preview` | `background` | Preview pane background color |
| `background_search` | `background` | Search bar background color |
| `background_status` | `background` | Status bar background color |
| `background_modal` | `background` | Modal background color |

**Color Value Formats:**

| Format | Example | Description |
|--------|---------|-------------|
| `"terminal"` | `"terminal"` | Use terminal default color |
| Hex | `"#ff0000"` | RGB hex color |
| Named | `"red"`, `"blue"`, `"green"` | Named color (depends on terminal) |

**Example:**

```toml
[styles.colors]
# Global colors
highlights_background = "#3b4252"
highlights_text = "#88c0d0"
borders = "#4c566a"
text = "#d8dee9"
background = "#2e3440"

# Override specific components
borders_list = "#5e81ac"
text_status = "#a3be8c"
```

### Border Options

Border enum values (used in all `borders` fields):

| Value | Description |
|-------|-------------|
| `"all"` | All four borders |
| `"top"` | Top border only |
| `"bottom"` | Bottom border only |
| `"left"` | Left border only |
| `"right"` | Right border only |

Can specify multiple: `borders = ["top", "bottom"]`

### Font Weight Options

| Value | Description |
|-------|-------------|
| `"regular"` | Normal text weight |
| `"bold"` | Bold text weight |

## Validation Rules

Syntropy validates config on load and shows errors for invalid values.

| Rule | Error Message |
|------|---------------|
| `default_task` requires `default_plugin` | `"default_task requires default_plugin to be set"` |
| `default_plugin_icon` must be 1 cell wide | `"Default plugin icon '...' must occupy a single terminal cell"` |
| Plugin git URL not empty | `"Plugin git URL cannot be empty"` |
| Plugin git URL format | `"Invalid git URL format: '<url>' (must start with https:// or git@)"` |
| Plugin must have tag or commit | `"Plugin must specify either tag or commit"` |
| Plugin tag XOR commit | `"Plugin must not declare both tag and commit - choose one"` |
| Screen scaffold splits sum to 100 | `"Screen scaffold style left and right split must amount to 100"` |
| Status splits sum to 100 | `"Status style left and right split must amount to 100"` |
| Modal sizes < 100 | `"Modal style vertical_size and horizontal_size must not exceed 100"` |
| Keybinding not empty | `"Empty keybinding"` |
| Keybinding no duplicates | `"Duplicate keybinding: <key>"` |
| Keybinding valid format | `"Invalid keybinding: <key>"` |

## Complete Example

```toml
# Root configuration
default_plugin = "packages"
default_task = "list"
default_plugin_icon = "⚒"

# UI options
status_bar = true
search_bar = true
show_preview_pane = true
exit_on_execute = false

# Keybindings
[keybindings]
back = "<esc>"
select_previous = "<up>"
select_next = "<down>"
scroll_preview_up = "["
scroll_preview_down = "]"
toggle_preview = "<C-p>"
select = "<tab>"
confirm = "<enter>"

# Plugin declarations
[plugins.packages]
git = "https://github.com/user/syntropy-plugin-packages"
tag = "v1.2.0"

[plugins.backups]
git = "https://github.com/user/syntropy-plugin-backups"
commit = "abc123"

# Styles
[styles.screen_scaffold]
left_split = 60
right_split = 40

[styles.status]
left_split = 70
right_split = 30
borders = ["all"]
font_weight = "bold"
breadcrumbs_separator = " > "
idle_icons = ["✔"]
error_icons = ["⛌"]
complete_icons = ["✔"]
running_icons = ["✴", "✵"]

[styles.modal]
vertical_size = 80
horizontal_size = 70
borders = ["all"]
font_weight = "regular"
show_title = true
scroll_offset = 2

[styles.list]
highlight_symbol = "▶"
icon_marked = "✓"
icon_unmarked = "○"
font_weight = "regular"
borders = ["all"]

[styles.preview]
borders = ["left", "right", "bottom"]
font_weight = "regular"
show_title = true
scroll_offset = 2

[styles.search_bar]
borders = ["all"]
search_hint = "🔍"
font_weight = "bold"

[styles.colors]
highlights_background = "#3b4252"
highlights_text = "#88c0d0"
borders = "#4c566a"
text = "#d8dee9"
background = "#2e3440"

# Override specific components
borders_list = "#5e81ac"
text_status = "#a3be8c"
```

---

For quick start examples, see the [Configuration section in README](../README.md#configuration).
