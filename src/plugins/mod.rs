pub mod git_ops;
mod loader;
mod module_path_builder;
mod plugin;
mod plugin_candidate;
mod plugin_source;

use std::{collections::HashMap, sync::Arc};

pub use loader::{load_plugin, load_plugins, merge_and_validate_plugins, validate_plugin};
pub use module_path_builder::ModulePathBuilder;
pub use plugin::{ItemSource, Metadata, Mode, Plugin, Task};
use plugin_source::PluginSource;

type TaskMap = HashMap<String, Arc<Task>>;
