use anyhow::{Context, Result, bail, ensure};
use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    configs::{
        expand_path, get_default_config_dir, get_default_data_dir, load_config, validate_config,
    },
    lua::create_lua_vm,
    plugins::{ModulePathBuilder, load_plugin, merge_and_validate_plugins, validate_plugin},
};

const DEFAULT_PLUGIN_ICON: &str = "⚒";

/// Location of a plugin file
#[derive(Debug, PartialEq, Clone, Copy)]
enum PluginLocation {
    ConfigDir, // ~/.config/syntropy/plugins/
    DataDir,   // ~/.local/share/syntropy/plugins/
    Custom,    // Any other location
}

/// Detects which standard directory (if any) contains the plugin
fn detect_plugin_location(lua_path: &Path) -> Result<PluginLocation> {
    let config_plugins = get_default_config_dir()?.join("plugins");
    let data_plugins = get_default_data_dir()?.join("plugins");

    if lua_path.starts_with(&config_plugins) {
        Ok(PluginLocation::ConfigDir)
    } else if lua_path.starts_with(&data_plugins) {
        Ok(PluginLocation::DataDir)
    } else {
        Ok(PluginLocation::Custom)
    }
}

/// Extracts plugin name from the directory structure
///
/// Example: ~/.config/syntropy/plugins/notes/plugin.lua → "notes"
fn extract_plugin_name(lua_path: &Path) -> Result<String> {
    lua_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(String::from)
        .context("Could not extract plugin name from path")
}

/// Finds a merge candidate for the given plugin
///
/// Returns:
/// - Some(path) if a corresponding base/override exists
/// - None if no merge candidate found (standalone plugin)
fn find_merge_candidate(
    plugin_name: &str,
    current_location: PluginLocation,
) -> Result<Option<PathBuf>> {
    match current_location {
        PluginLocation::ConfigDir => {
            // Override in config dir - look for base in data dir
            let data_plugins = get_default_data_dir()?.join("plugins");
            let base_path = data_plugins.join(plugin_name).join("plugin.lua");
            Ok(if base_path.exists() {
                Some(base_path)
            } else {
                None
            })
        }
        PluginLocation::DataDir => {
            // Base in data dir - look for override in config dir
            let config_plugins = get_default_config_dir()?.join("plugins");
            let override_path = config_plugins.join(plugin_name).join("plugin.lua");
            Ok(if override_path.exists() {
                Some(override_path)
            } else {
                None
            })
        }
        PluginLocation::Custom => {
            // Custom location - no merge detection
            Ok(None)
        }
    }
}

/// Validates a plugin at the specified path
///
/// Accepts either:
/// - A directory containing plugin.lua
/// - A direct path to plugin.lua
///
/// Performs complete validation including:
/// - Lua syntax checking
/// - Structure parsing
/// - Metadata validation (name, version, icon)
/// - Task validation (item sources, tags)
///
/// If the plugin is in a standard directory and has a merge candidate,
/// validates the merged result instead of the standalone plugin.
pub fn validate_plugin_cli(plugin_path: PathBuf) -> Result<()> {
    let plugin_path = expand_path(plugin_path).context("Failed to expand plugin path")?;

    let lua_path = if plugin_path.is_dir() {
        plugin_path.join("plugin.lua")
    } else if plugin_path.file_name().and_then(|n| n.to_str()) == Some("plugin.lua") {
        plugin_path
    } else {
        bail!(
            "Path must be a directory containing plugin.lua or a direct path to plugin.lua\nProvided: {}",
            plugin_path.display()
        );
    };

    ensure!(
        lua_path.exists(),
        "Plugin file not found: {}",
        lua_path.display()
    );

    // Detect if this plugin is part of a merge scenario
    let location = detect_plugin_location(&lua_path)?;
    let plugin_name = extract_plugin_name(&lua_path)?;
    let merge_candidate = find_merge_candidate(&plugin_name, location)?;

    let lua_runtime = create_lua_vm().context("Failed to create Lua runtime")?;

    let syntropy_root = env::current_dir()
        .context("Failed to get current directory")?
        .to_str()
        .context("Current directory path contains invalid UTF-8")?
        .to_string();

    // Configure module paths before loading plugin
    let plugin_dir = lua_path
        .parent()
        .context("Plugin path has no parent directory")?
        .to_str()
        .context("Plugin directory path contains invalid UTF-8")?;

    let mut path_builder = ModulePathBuilder::default()
        .with_plugin_dir(plugin_dir)
        .with_shared_modules(&syntropy_root);

    // If merging, also add the merge candidate's directory to module paths
    if let Some(ref candidate_path) = merge_candidate
        && let Some(candidate_dir) = candidate_path.parent()
        && let Some(candidate_dir_str) = candidate_dir.to_str()
    {
        path_builder = path_builder.with_plugin_dir(candidate_dir_str);
    }

    path_builder
        .apply(&lua_runtime)
        .context("Failed to configure Lua module paths")?;

    if let Some(candidate_path) = merge_candidate {
        // MERGED VALIDATION
        let (base_path, override_path) = match detect_plugin_location(&lua_path)? {
            PluginLocation::ConfigDir => {
                // Current is override, candidate is base
                (candidate_path, lua_path.clone())
            }
            PluginLocation::DataDir => {
                // Current is base, candidate is override
                (lua_path.clone(), candidate_path)
            }
            PluginLocation::Custom => {
                unreachable!("Custom location should not have merge candidate")
            }
        };

        println!("Validating plugin '{}'...", plugin_name);
        println!("  ✓ Found base plugin at {}", base_path.display());
        println!("  ✓ Found override at {}", override_path.display());

        // Validate base plugin first
        let base_plugin = load_plugin(&lua_runtime, &base_path, DEFAULT_PLUGIN_ICON, None)
            .with_context(|| format!("Failed to load base plugin from {}", base_path.display()))?;

        if let Err(e) = validate_plugin(&base_plugin) {
            bail!(
                "✗ Base plugin is invalid:\n  {}\n✗ Cannot validate override because base plugin is invalid",
                e
            );
        }

        // Merge and validate
        let merged_plugin = merge_and_validate_plugins(
            &lua_runtime,
            &base_path,
            &override_path,
            &plugin_name,
            DEFAULT_PLUGIN_ICON,
        )
        .context("Failed to merge and validate plugins")?;

        println!(
            "✓ Plugin '{}' (v{}) is valid (merged configuration)",
            merged_plugin.metadata.name, merged_plugin.metadata.version
        );
    } else {
        // STANDALONE VALIDATION
        if matches!(location, PluginLocation::Custom) {
            println!("ℹ Plugin not in standard directory - validating as standalone");
        }

        let plugin = load_plugin(&lua_runtime, &lua_path, DEFAULT_PLUGIN_ICON, None)
            .context("Failed to load plugin")?;

        validate_plugin(&plugin)
            .with_context(|| format!("validation failed for plugin {}", plugin.metadata.name))?;

        println!(
            "✓ Plugin '{}' (v{}) is valid",
            plugin.metadata.name, plugin.metadata.version
        );
    }

    Ok(())
}

/// Validates a config file at the specified path
///
/// Performs complete validation including:
/// - TOML syntax checking
/// - Structure parsing
/// - Style split percentages (must sum to 100)
/// - Modal size constraints (< 100)
/// - Default plugin icon width (must be 1 cell)
///
/// Note: load_config() already performs validation internally,
/// so we don't need to call validate_config() separately.
pub fn validate_config_cli(config_path: PathBuf) -> Result<()> {
    let config_path = expand_path(config_path).context("Failed to expand config path")?;

    ensure!(
        config_path.exists(),
        "Config file not found: {}",
        config_path.display()
    );

    ensure!(
        config_path.is_file(),
        "Path must be a file, not a directory: {}",
        config_path.display()
    );

    let config = load_config(config_path.clone()).context("Failed to load config")?;

    validate_config(&config)?;

    println!("✓ Config file is valid");

    Ok(())
}
