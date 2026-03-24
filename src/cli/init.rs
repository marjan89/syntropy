use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::configs::get_default_config_dir;

// Directory names
const PLUGINS_DIR_NAME: &str = "plugins";
const DOCS_DIR_NAME: &str = "docs";

// Plugin template file names
const SYNTROPY_LUA_FILE: &str = "syntropy.lua";
const LUARC_JSON_FILE: &str = ".luarc.json";
const PLUGIN_LUA_FILE: &str = "plugin.lua";

// Doc file names
const README_FILE: &str = "README.md";
const PLUGINS_MD_FILE: &str = "plugins.md";
const CONFIG_REFERENCE_FILE: &str = "config-reference.md";
const API_REFERENCE_FILE: &str = "plugin-api-reference.md";
const API_ADVANCED_FILE: &str = "plugin-api-reference-section-advanced.md";
const API_FUNCTIONS_FILE: &str = "plugin-api-reference-section-api-functions.md";
const API_DATA_STRUCTURES_FILE: &str = "plugin-api-reference-section-data-structures.md";
const API_EXAMPLES_FILE: &str = "plugin-api-reference-section-examples.md";
const API_ITEM_SOURCES_FILE: &str = "plugin-api-reference-section-item-sources.md";
const API_TASKS_FILE: &str = "plugin-api-reference-section-tasks.md";
const AVAILABLE_PLUGINS_FILE: &str = "available-plugins.md";
const RECIPES_FILE: &str = "recipes.md";

// Embedded plugin template contents
const SYNTROPY_LUA_TEMPLATE: &str = include_str!("../../scaffold_templates/syntropy.lua");
const LUARC_JSON_TEMPLATE: &str = include_str!("../../scaffold_templates/.luarc.json");
const PLUGIN_LUA_TEMPLATE: &str = include_str!("../../scaffold_templates/plugin.lua");

// Embedded doc contents
const README_CONTENT: &str = include_str!("../../README.md");
const PLUGINS_MD_CONTENT: &str = include_str!("../../docs/plugins.md");
const CONFIG_REFERENCE_CONTENT: &str = include_str!("../../docs/config-reference.md");
const API_REFERENCE_CONTENT: &str = include_str!("../../docs/plugin-api-reference.md");
const API_ADVANCED_CONTENT: &str =
    include_str!("../../docs/plugin-api-reference-section-advanced.md");
const API_FUNCTIONS_CONTENT: &str =
    include_str!("../../docs/plugin-api-reference-section-api-functions.md");
const API_DATA_STRUCTURES_CONTENT: &str =
    include_str!("../../docs/plugin-api-reference-section-data-structures.md");
const API_EXAMPLES_CONTENT: &str =
    include_str!("../../docs/plugin-api-reference-section-examples.md");
const API_ITEM_SOURCES_CONTENT: &str =
    include_str!("../../docs/plugin-api-reference-section-item-sources.md");
const API_TASKS_CONTENT: &str = include_str!("../../docs/plugin-api-reference-section-tasks.md");
const AVAILABLE_PLUGINS_CONTENT: &str = include_str!("../../docs/available-plugins.md");
const RECIPES_CONTENT: &str = include_str!("../../docs/recipes.md");

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

/// Initializes docs directory at the given base directory
///
/// Creates docs/ subdirectory, checks for existing files, and writes all doc files.
///
/// Returns a vector of filenames that were overwritten.
fn initialize_docs_directory(base_dir: &Path) -> Result<Vec<&'static str>> {
    let docs_dir = base_dir.join(DOCS_DIR_NAME);

    fs::create_dir_all(&docs_dir)
        .with_context(|| format!("Failed to create docs directory at {}", docs_dir.display()))?;

    let files_to_write = [
        (README_FILE, docs_dir.join(README_FILE)),
        (PLUGINS_MD_FILE, docs_dir.join(PLUGINS_MD_FILE)),
        (CONFIG_REFERENCE_FILE, docs_dir.join(CONFIG_REFERENCE_FILE)),
        (API_REFERENCE_FILE, docs_dir.join(API_REFERENCE_FILE)),
        (API_ADVANCED_FILE, docs_dir.join(API_ADVANCED_FILE)),
        (API_FUNCTIONS_FILE, docs_dir.join(API_FUNCTIONS_FILE)),
        (
            API_DATA_STRUCTURES_FILE,
            docs_dir.join(API_DATA_STRUCTURES_FILE),
        ),
        (API_EXAMPLES_FILE, docs_dir.join(API_EXAMPLES_FILE)),
        (API_ITEM_SOURCES_FILE, docs_dir.join(API_ITEM_SOURCES_FILE)),
        (API_TASKS_FILE, docs_dir.join(API_TASKS_FILE)),
        (
            AVAILABLE_PLUGINS_FILE,
            docs_dir.join(AVAILABLE_PLUGINS_FILE),
        ),
        (RECIPES_FILE, docs_dir.join(RECIPES_FILE)),
    ];

    let existing_files: Vec<&str> = files_to_write
        .iter()
        .filter(|(_, path)| path.exists())
        .map(|(name, _)| *name)
        .collect();

    write_template(README_CONTENT, &docs_dir.join(README_FILE))?;
    write_template(PLUGINS_MD_CONTENT, &docs_dir.join(PLUGINS_MD_FILE))?;
    write_template(
        CONFIG_REFERENCE_CONTENT,
        &docs_dir.join(CONFIG_REFERENCE_FILE),
    )?;
    write_template(API_REFERENCE_CONTENT, &docs_dir.join(API_REFERENCE_FILE))?;
    write_template(API_ADVANCED_CONTENT, &docs_dir.join(API_ADVANCED_FILE))?;
    write_template(API_FUNCTIONS_CONTENT, &docs_dir.join(API_FUNCTIONS_FILE))?;
    write_template(
        API_DATA_STRUCTURES_CONTENT,
        &docs_dir.join(API_DATA_STRUCTURES_FILE),
    )?;
    write_template(API_EXAMPLES_CONTENT, &docs_dir.join(API_EXAMPLES_FILE))?;
    write_template(
        API_ITEM_SOURCES_CONTENT,
        &docs_dir.join(API_ITEM_SOURCES_FILE),
    )?;
    write_template(API_TASKS_CONTENT, &docs_dir.join(API_TASKS_FILE))?;
    write_template(
        AVAILABLE_PLUGINS_CONTENT,
        &docs_dir.join(AVAILABLE_PLUGINS_FILE),
    )?;
    write_template(RECIPES_CONTENT, &docs_dir.join(RECIPES_FILE))?;

    Ok(existing_files)
}

/// Creates the plugin development environment scaffold
///
/// Creates directory structure and template files at XDG config location:
/// - `$XDG_CONFIG_HOME/syntropy/plugins/` (default: `~/.config/syntropy/plugins/`)
/// - `$XDG_CONFIG_HOME/syntropy/docs/` (default: `~/.config/syntropy/docs/`)
///
/// Files created in plugins/:
/// - `syntropy.lua` - Type hints for syntropy global namespace
/// - `.luarc.json` - Lua language server configuration
/// - `plugin.lua` - Plugin type definitions
///
/// Files created in docs/:
/// - Full plugin development documentation (11 reference files)
///
/// Note: Installed plugins (via package managers or `syntropy --install`) will be
/// placed in `$XDG_DATA_HOME/syntropy/plugins/` (default: `~/.local/share/syntropy/plugins/`)
pub fn create_plugin_scaffold() -> Result<()> {
    let config_dir = get_default_config_dir().context("Failed to get config directory")?;

    let mut existing = initialize_plugin_directory(&config_dir)?;
    existing.extend(initialize_docs_directory(&config_dir)?);

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
    let docs_dir = config_dir.join(DOCS_DIR_NAME);

    println!(
        "\
Plugin development environment initialized at:
  {}

Created directories:
  - {}
  - {}

Created files in plugins/:
  - {} (type hints for syntropy namespace)
  - {} (Lua language server config)
  - {} (plugin type definitions)

Created files in docs/:
  - {} (readme / quick start)
  - {} (plugin authoring guide)
  - {} (configuration reference)
  - {} (API reference index)
  - 8 additional API reference sections

Next steps:
  1. Create your plugin: mkdir {}/my-plugin
  2. Edit and run: syntropy

Note: Installed plugins will be placed in ~/.local/share/syntropy/plugins/",
        config_dir.display(),
        plugins_dir.display(),
        docs_dir.display(),
        SYNTROPY_LUA_FILE,
        LUARC_JSON_FILE,
        PLUGIN_LUA_FILE,
        README_FILE,
        PLUGINS_MD_FILE,
        CONFIG_REFERENCE_FILE,
        API_REFERENCE_FILE,
        plugins_dir.display()
    );

    Ok(())
}
