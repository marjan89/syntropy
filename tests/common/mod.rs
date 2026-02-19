#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        fs::create_dir_all(&config_dir).expect("Failed to create config dir");
        fs::create_dir_all(&data_dir).expect("Failed to create data dir");

        Self {
            temp_dir,
            config_dir,
            data_dir,
        }
    }

    pub fn create_plugin(&self, name: &str, content: &str) {
        let plugin_path = self
            .data_dir
            .join("syntropy")
            .join("plugins")
            .join(name)
            .join("plugin.lua");
        fs::create_dir_all(plugin_path.parent().unwrap()).expect("Failed to create plugin dir");
        fs::write(plugin_path, content).expect("Failed to write plugin file");
    }

    pub fn create_plugin_override(&self, name: &str, content: &str) {
        let plugin_path = self
            .config_dir
            .join("syntropy")
            .join("plugins")
            .join(name)
            .join("plugin.lua");
        fs::create_dir_all(plugin_path.parent().unwrap()).expect("Failed to create plugin dir");
        fs::write(plugin_path, content).expect("Failed to write plugin file");
    }

    /// Create a lua/ module for a plugin in the data directory using Neovim-style structure
    pub fn create_lib_module(&self, plugin_name: &str, module_name: &str, content: &str) {
        let module_path = self
            .data_dir
            .join("syntropy")
            .join("plugins")
            .join(plugin_name)
            .join("lua")
            .join(plugin_name)
            .join(format!("{}.lua", module_name));
        fs::create_dir_all(module_path.parent().unwrap()).expect("Failed to create lua dir");
        fs::write(module_path, content).expect("Failed to write lua module");
    }

    /// Create a lua/ module for a plugin in the config directory (override) using Neovim-style structure
    pub fn create_lib_module_override(&self, plugin_name: &str, module_name: &str, content: &str) {
        let module_path = self
            .config_dir
            .join("syntropy")
            .join("plugins")
            .join(plugin_name)
            .join("lua")
            .join(plugin_name)
            .join(format!("{}.lua", module_name));
        fs::create_dir_all(module_path.parent().unwrap()).expect("Failed to create lua dir");
        fs::write(module_path, content).expect("Failed to write lua module");
    }

    /// Create a shared module in the data directory
    /// Shared modules are available to all plugins via require()
    pub fn create_shared_module(&self, module_name: &str, content: &str) {
        let module_path = self
            .data_dir
            .join("syntropy")
            .join("plugins")
            .join("shared")
            .join(format!("{}.lua", module_name));
        fs::create_dir_all(module_path.parent().unwrap()).expect("Failed to create shared dir");
        fs::write(module_path, content).expect("Failed to write shared module");
    }

    /// Create a shared module in the config directory (override)
    /// Config shared modules have precedence over data shared modules
    pub fn create_shared_module_override(&self, module_name: &str, content: &str) {
        let module_path = self
            .config_dir
            .join("syntropy")
            .join("plugins")
            .join("shared")
            .join(format!("{}.lua", module_name));
        fs::create_dir_all(module_path.parent().unwrap()).expect("Failed to create shared dir");
        fs::write(module_path, content).expect("Failed to write shared module");
    }

    pub fn create_config(&self, filename: &str, content: &str) {
        let config_path = self.config_dir.join("syntropy").join(filename);
        fs::create_dir_all(config_path.parent().unwrap()).expect("Failed to create config dir");
        fs::write(config_path, content).expect("Failed to write config file");
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.clone()
    }

    pub fn data_path(&self) -> PathBuf {
        self.data_dir.clone()
    }
}

pub fn sample_plugin() -> &'static str {
    r#"
return {
    metadata = {
        name = "test-plugin",
        version = "1.0.0",
        icon = "🔧",
        description = "Test plugin for integration tests",
        platforms = {"macos", "linux"},
    },
    tasks = {
        test_task = {
            name = "Test Task",
            description = "Test task for integration tests",
            mode = "none",
            item_sources = {
                test_source = {
                    tag = "t",
                    items = function()
                        return {"item1", "item2", "item3"}
                    end,
                    execute = function(items)
                        return "Executed " .. #items .. " items", 0
                    end,
                },
            },
        },
    },
}
"#
}
