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
