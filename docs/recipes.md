# Syntropy Recipes

Practical integration examples for using syntropy.

## macOS Hotkey Integration with skhd

Trigger syntropy tasks via keyboard shortcuts using [skhd](https://github.com/koekeishiya/skhd).

### Setup

```bash
brew install koekeishiya/formulae/skhd
brew services start skhd
```

Grant skhd Accessibility permissions in System Settings → Privacy & Security → Accessibility.

### Example `.skhdrc`

```bash
# Execute task directly
cmd + shift - b : syntropy execute --plugin backups --task create
cmd + shift - c : syntropy execute --plugin system --task cleanup

# Execute on specific items
cmd + shift - r : syntropy execute --plugin docker --task restart --items "api,frontend"

# Open TUI at plugin/task
cmd + shift - p : kitty -e syntropy --plugin packages
cmd + shift - l : kitty -e syntropy --plugin logs --task view

# Execute with notification
cmd + shift - s : syntropy execute --plugin services --task restart && \
                  osascript -e 'display notification "Done" with title "Syntropy"'

# Export items to clipboard
cmd + shift - e : syntropy execute --plugin bookmarks --task list --produce-items | pbcopy
```

### Tips

- Use `syntropy execute` for headless execution (faster, no window)
- Set `suppress_success_notification = true` in task config to avoid modals
- Set `exit_on_execute = true` in config.toml for one-shot workflows

## Terminal Hotkey Window Configuration

When launching syntropy from a hotkey, the default terminal window may not suit a quick popup interaction. Most terminal emulators support alternate config files that let you define a purpose-built window — fixed size, minimal chrome, auto-close on exit.

### Concept

Create a separate config that your terminal loads only for hotkey-spawned windows. This config typically:

- Sets a **fixed window size** (e.g. 80×25 characters) instead of remembering the last size
- Pins **window placement** (e.g. top-left corner) so it appears predictably
- Strips distractions like background images
- Enables **close-on-exit** so the window disappears when syntropy finishes
- Optionally overrides colors or theme for visual distinction

### Example: kitty

kitty supports loading an alternate config with `--config`:

```bash
# In .skhdrc — launch syntropy in a purpose-built kitty window
cmd + shift - p : kitty --config ~/.config/kitty/hotkey.conf -e syntropy --plugin packages
```

The referenced `hotkey.conf` includes the main config, then overrides what matters:

```conf
include kitty.conf

# Fixed size and position
placement_strategy top-left
remember_window_size no
initial_window_width 80c
initial_window_height 25c

# Clean up for popup use
background_image none
close_on_child_death yes
macos_quit_when_last_window_closed yes
```

### Other terminals

The same idea applies to most terminals — look for launch flags or profile selectors:

- **Alacritty** — `alacritty --config-file ~/.config/alacritty/hotkey.toml`
- **WezTerm** — `wezterm --config-file ~/.config/wezterm/hotkey.lua`
- **iTerm2** — create a dedicated Profile and launch with a specific profile via `open -a iTerm --args --profile Hotkey`
- **Ghostty** — `ghostty --config-file=~/.config/ghostty/hotkey`

The key dimensions to tune are window size, placement, and close-on-exit behavior. Keep it small enough to feel like a popup, large enough to display your plugin output without scrolling.
