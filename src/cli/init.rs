use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::configs::get_default_config_dir;

// Directory names
const PLUGINS_DIR_NAME: &str = "plugins";

// Template file names
const SYNTROPY_LUA_FILE: &str = "syntropy.lua";
const LUARC_JSON_FILE: &str = ".luarc.json";
const PLUGIN_LUA_FILE: &str = "plugin.lua";

// Embedded template contents
const SYNTROPY_LUA_TEMPLATE: &str = include_str!("../../scaffold_templates/syntropy.lua");
const LUARC_JSON_TEMPLATE: &str = include_str!("../../scaffold_templates/.luarc.json");
const PLUGIN_LUA_TEMPLATE: &str = include_str!("../../scaffold_templates/plugin.lua");

/// Writes template content to a file
fn write_template(content: &str, path: &Path) -> Result<()> {
    fs::write(path, content)
        .with_context(|| format!("Failed to write template to {}", path.display()))?;
    Ok(())
}

/// Initializes plugin directory structure at the given base directory
///
/// Creates plugins/ subdirectory, checks for existing files, and writes all template files.
///
/// Returns a vector of filenames that were overwritten.
fn initialize_plugin_directory(base_dir: &Path) -> Result<Vec<&'static str>> {
    let plugins_dir = base_dir.join(PLUGINS_DIR_NAME);

    // Create directory
    fs::create_dir_all(&plugins_dir).with_context(|| {
        format!(
            "Failed to create plugins directory at {}",
            plugins_dir.display()
        )
    })?;

    // Check if any files exist
    let files_to_write = [
        (SYNTROPY_LUA_FILE, plugins_dir.join(SYNTROPY_LUA_FILE)),
        (LUARC_JSON_FILE, plugins_dir.join(LUARC_JSON_FILE)),
        (PLUGIN_LUA_FILE, plugins_dir.join(PLUGIN_LUA_FILE)),
    ];

    let existing_files: Vec<&str> = files_to_write
        .iter()
        .filter(|(_, path)| path.exists())
        .map(|(name, _)| *name)
        .collect();

    // Write template files to plugins directory
    write_template(SYNTROPY_LUA_TEMPLATE, &plugins_dir.join(SYNTROPY_LUA_FILE))?;
    write_template(LUARC_JSON_TEMPLATE, &plugins_dir.join(LUARC_JSON_FILE))?;
    write_template(PLUGIN_LUA_TEMPLATE, &plugins_dir.join(PLUGIN_LUA_FILE))?;

    Ok(existing_files)
}

/// Creates the plugin development environment scaffold
///
/// Creates directory structure and template files at XDG config location:
/// - `$XDG_CONFIG_HOME/syntropy/plugins/` (default: `~/.config/syntropy/plugins/`)
///
/// Files created:
/// - `syntropy.lua` - Type hints for syntropy global namespace
/// - `.luarc.json` - Lua language server configuration
/// - `plugin.lua` - Plugin type definitions
///
/// Note: Installed plugins (via package managers or `syntropy --install`) will be
/// placed in `$XDG_DATA_HOME/syntropy/plugins/` (default: `~/.local/share/syntropy/plugins/`)
pub fn create_plugin_scaffold() -> Result<()> {
    let config_dir = get_default_config_dir().context("Failed to get config directory")?;

    // Initialize config directory (user development environment)
    let existing = initialize_plugin_directory(&config_dir)?;

    if !existing.is_empty() {
        eprintln!("Warning: The following files will be overwritten:");
        for file in &existing {
            eprintln!("  - {}", file);
        }
        eprintln!(
            "\nThese files are tied to syntropy's internal models and will be updated to match the current version."
        );
    }

    let plugins_dir = config_dir.join(PLUGINS_DIR_NAME);

    println!(
        "\
Plugin development environment initialized at:
  {}

Created files:
  - {} (type hints for syntropy namespace)
  - {} (Lua language server config)
  - {} (plugin type definitions)

Next steps:
  1. Create your plugin: mkdir {}/my-plugin
  2. Edit and run: syntropy

Note: Installed plugins will be placed in ~/.local/share/syntropy/plugins/",
        plugins_dir.display(),
        SYNTROPY_LUA_FILE,
        LUARC_JSON_FILE,
        PLUGIN_LUA_FILE,
        plugins_dir.display()
    );

    Ok(())
}
