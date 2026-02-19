# Available Syntropy Plugins

A curated list of syntropy plugins to extend functionality and workflows.

## Table of Contents

- [System Management](#system-management)
- [Media Control](#media-control)
- [Productivity](#productivity)
- [Contributing](#contributing)

## System Management

### [syntropy-display](https://github.com/marjan89/syntropy-display)

Display configuration and management for macOS. Control screen brightness, resolution, and arrangement.

**Installation:**
```toml
[plugins.syntropy-display]
git = "https://github.com/marjan89/display.git"
tag = "v1.0.0"
```

### [syntropy-trash](https://github.com/marjan89/syntropy-trash)

Trash management for macOS. View, restore, and permanently delete items from the Trash.

**Installation:**
```toml
[plugins.syntropy-trash]
git = "https://github.com/marjan89/trash.git"
tag = "v1.0.0"
```

### [syntropy-bluetooth](https://github.com/marjan89/syntropy-bluetooth)

Bluetooth device management for macOS. Connect, disconnect, and manage Bluetooth devices.

**Installation:**
```toml
[plugins.syntropy-bluetooth]
git = "https://github.com/marjan89/bluetooth.git"
tag = "v1.0.0"
```

### [appswitch](https://github.com/marjan89/appswitch)

Application switcher for macOS. Quickly switch between running applications.

**Installation:**
```toml
[plugins.appswitch]
git = "https://github.com/marjan89/appswitch.git"
tag = "v1.0.0"
```

## Media Control

### [syntropy-audio](https://github.com/marjan89/syntropy-audio)

Audio device and volume management for macOS. Control input/output devices and system volume.

**Installation:**
```toml
[plugins.syntropy-audio]
git = "https://github.com/marjan89/audio.git"
tag = "v1.0.0"
```

## Productivity

### [syntropy-emoji](https://github.com/marjan89/syntropy-emoji)

Emoji picker and search. Quickly find and copy emojis to clipboard.

**Installation:**
```toml
[plugins.syntropy-emoji]
git = "https://github.com/marjan89/emoji.git"
tag = "v1.0.0"
```

### [syntropy-notes](https://github.com/marjan89/syntropy-notes)

Note-taking and management. Create, edit, and organize markdown notes.

**Installation:**
```toml
[plugins.syntropy-notes]
git = "https://github.com/marjan89/notes.git"
tag = "v1.0.0"
```

## Contributing

Want to add your plugin to this list? Please:

1. Ensure your plugin follows the [Plugin Development Guide](plugins.md)
2. Add documentation with usage examples
3. Include installation instructions
4. Submit a pull request with your addition

**Plugin submission template:**

```markdown
### [plugin-name](https://github.com/user/plugin-name)

Brief description of what the plugin does.

**Installation:**
\`\`\`toml
[plugins.plugin-name]
git = "https://github.com/user/plugin-name.git"
tag = "v1.0.0"
\`\`\`
```

---

**See also:**
- [Plugin Development Guide](plugins.md) - Learn how to create your own plugins
- [Plugin API Reference](plugin-api-reference.md) - Complete API documentation
- [Configuration Reference](config-reference.md) - Configure syntropy and plugins
