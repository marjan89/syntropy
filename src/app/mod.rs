mod run;

pub use run::run;

use std::sync::Arc;

use mlua::Lua;
use tokio::sync::Mutex;

use crate::{
    configs::Config,
    plugins::{Plugin, Task},
};

pub struct App {
    pub config: Config,
    pub plugins: Vec<Plugin>,
    pub lua_runtime: Arc<Mutex<Lua>>,
}

impl App {
    pub fn new(config: Config, plugins: Vec<Plugin>, lua_runtime: Arc<Mutex<Lua>>) -> App {
        Self {
            config,
            plugins,
            lua_runtime,
        }
    }
}

impl App {
    pub fn get_plugin(&self, idx: usize) -> Option<&Plugin> {
        self.plugins.get(idx)
    }

    pub fn get_task(&self, plugin_idx: usize, task_key: &str) -> Option<&Arc<Task>> {
        self.get_plugin(plugin_idx)
            .and_then(|plugin| plugin.tasks.get(task_key))
    }
}
