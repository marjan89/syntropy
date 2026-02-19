mod args;
pub mod completions;
pub mod execute;
pub mod init;
pub mod plugins;
pub mod validate;

pub use args::{Args, Commands, ExecuteArgs, PluginsArgs};
pub use plugins::handle_plugins_command;
