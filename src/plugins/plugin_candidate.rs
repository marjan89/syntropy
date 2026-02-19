use anyhow::{Context, Result};
use std::path::PathBuf;

use mlua::{Lua, Table};

/// Represents a plugin file discovered during directory scanning
///
/// Caches the plugin name and file contents to enable single-evaluation loading.
#[derive(Debug, Clone)]
pub struct PluginCandidate {
    /// Absolute path to plugin.lua file
    pub path: PathBuf,

    /// Plugin name extracted from metadata.name (cached from peek)
    pub name: String,

    /// Cached file contents from disk read
    ///
    /// This is read once during peek and reused during load,
    /// eliminating redundant file I/O.
    pub cached_contents: String,
}

impl PluginCandidate {
    /// Peek at a plugin file to extract its name and cache contents
    ///
    /// **Important**: The returned candidate contains cached file contents
    /// that will be evaluated only once during loading.
    pub fn peek(lua_runtime: &Lua, path: PathBuf) -> Result<Self> {
        let cached_contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read plugin file '{}'", path.display()))?;

        let plugin_table: Table = lua_runtime
            .load(&cached_contents)
            .set_name(path.to_str().with_context(|| {
                format!("Plugin path contains invalid UTF-8: {}", path.display())
            })?)
            .eval()
            .with_context(|| format!("Failed to evaluate plugin '{}'", path.display()))?;

        let metadata_table: Table = plugin_table
            .get("metadata")
            .with_context(|| format!("Plugin '{}' missing 'metadata' table", path.display()))?;

        let name: String = metadata_table
            .get("name")
            .with_context(|| format!("Plugin '{}' missing 'name' in metadata", path.display()))?;

        Ok(Self {
            path,
            name,
            cached_contents,
        })
    }

    /// Evaluate the cached contents into a Lua table
    ///
    /// This is the second (and final) evaluation of the plugin file.
    pub fn evaluate(&self, lua: &Lua) -> Result<Table> {
        let plugin_table: Table = lua
            .load(&self.cached_contents)
            .set_name(self.path.to_str().with_context(|| {
                format!(
                    "Plugin path contains invalid UTF-8: {}",
                    self.path.display()
                )
            })?)
            .eval()
            .with_context(|| format!("Failed to evaluate plugin '{}'", self.path.display()))?;

        Ok(plugin_table)
    }
}
