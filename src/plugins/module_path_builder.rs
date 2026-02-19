use anyhow::Result;
use mlua::{Lua, Table};
use std::path::Path;

/// Configure Lua package.path for plugin module loading
///
/// Sets up resolution order:
/// 1. Plugin namespaced modules: <plugin_dir>/lua/?.lua and <plugin_dir>/lua/?/init.lua
/// 2. Shared syntropy modules: <syntropy_root>/plugins/shared/?.lua
///
/// # Arguments
/// * `lua` - Lua VM instance
/// * `plugin_dir` - Absolute path to plugin directory (containing plugin.lua)
/// * `syntropy_root` - Absolute path to syntropy repository root
///
/// # Example
/// ```
/// use mlua::Lua;
/// use std::path::Path;
///
/// # fn example() -> mlua::Result<()> {
/// let lua = Lua::new();
/// let plugin_dir = "/path/to/plugins/packages";
/// let syntropy_root = "/path/to/syntropy";
///
/// // This would configure Lua package.path for the plugin
/// // syntropy::lua::runtime::configure_module_paths(&lua, plugin_dir, syntropy_root)?;
/// # Ok(())
/// # }
/// ```
/// Builder for constructing Lua package.path strings
///
/// Consolidates module path configuration logic for plugins.
/// Supports both single-directory and multi-directory scenarios.
#[derive(Default)]
pub struct ModulePathBuilder {
    paths: Vec<String>,
}

impl ModulePathBuilder {
    /// Add a plugin directory's lua/ path if it exists
    ///
    /// Supports Neovim-style module structure:
    /// - lua/?.lua for require("pluginname.module")
    /// - lua/?/init.lua for require("pluginname")
    ///
    /// # Arguments
    /// * `plugin_dir` - Absolute path to plugin directory
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_plugin_dir(mut self, plugin_dir: &str) -> Self {
        let lua_dir = format!("{}/lua", plugin_dir);
        if Path::new(&lua_dir).exists() {
            // Add lua/?.lua for standard module files
            self.paths.push(format!("{}/?.lua", lua_dir));

            // Add lua/?/init.lua for directory modules
            self.paths.push(format!("{}/?/init.lua", lua_dir));
        }
        self
    }

    /// Add shared syntropy modules path if it exists
    ///
    /// # Arguments
    /// * `syntropy_root` - Absolute path to syntropy repository root
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_shared_modules(mut self, syntropy_root: &str) -> Self {
        let shared_path = format!("{}/plugins/shared/?.lua", syntropy_root);
        if Path::new(&format!("{}/plugins/shared", syntropy_root)).exists() {
            self.paths.push(shared_path);
        }
        self
    }

    /// Build the final package.path string (semicolon-separated)
    pub fn build(self) -> String {
        self.paths.join(";")
    }

    /// Apply the built path to a Lua runtime's package.path
    pub fn apply(self, lua: &Lua) -> Result<()> {
        let package: Table = lua.globals().get("package")?;
        package.set("path", self.build())?;
        Ok(())
    }
}
