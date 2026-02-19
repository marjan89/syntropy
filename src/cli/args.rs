use clap::{Args as ClapArgs, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "syntropy")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Specify a custom config path to use with this instance
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Navigate to specific plugin (without executing)
    #[arg(long, value_name = "NAME")]
    pub plugin: Option<String>,

    /// Navigate to specific task (requires --plugin, without executing)
    #[arg(long, value_name = "NAME")]
    pub task: Option<String>,

    /// Override status bar visibility
    #[arg(long, value_name = "BOOL")]
    pub status_bar: Option<bool>,

    /// Override search bar visibility
    #[arg(long, value_name = "BOOL")]
    pub search_bar: Option<bool>,

    /// Override preview pane visibility
    #[arg(long, value_name = "BOOL")]
    pub show_preview_pane: Option<bool>,

    /// Override exit on execute behavior
    #[arg(long, value_name = "BOOL")]
    pub exit_on_execute: Option<bool>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(ClapArgs, Debug)]
pub struct ExecuteArgs {
    /// Plugin name
    #[arg(long, value_name = "NAME")]
    pub plugin: String,

    /// Task name
    #[arg(long, value_name = "NAME")]
    pub task: String,

    /// Specify specific items to execute on (comma-separated)
    #[arg(long, value_name = "NAMES", conflicts_with_all = ["produce_items", "produce_preselected_items", "produce_preselection_matches"])]
    pub items: Option<String>,

    /// Output items list (for debugging/scripting)
    #[arg(long, conflicts_with_all = ["items", "produce_preselected_items", "produce_preselection_matches"])]
    pub produce_items: bool,

    /// Output preselected items list
    #[arg(long, conflicts_with_all = ["items", "produce_items", "produce_preselection_matches"])]
    pub produce_preselected_items: bool,

    /// Output items matching preselection
    #[arg(long, conflicts_with_all = ["items", "produce_items", "produce_preselected_items"])]
    pub produce_preselection_matches: bool,

    /// Generate preview for an item
    #[arg(long, conflicts_with_all = ["items", "produce_items", "produce_preselected_items", "produce_preselection_matches"])]
    pub preview: Option<String>,
}

#[derive(ClapArgs, Debug)]
pub struct PluginsArgs {
    /// Remove installed plugins not present in config file
    #[arg(long)]
    pub remove: bool,

    /// Install missing plugins declared in config file
    #[arg(long)]
    pub install: bool,

    /// List all available plugins
    #[arg(long)]
    pub list: bool,

    /// Upgrade selected plugin to the version declared in config file. If no plugin is specified, tries to upgrade all plugins
    #[arg(long)]
    pub upgrade: bool,

    /// Plugin to upgrade (requires --upgrade)
    #[arg(long, value_name = "NAME")]
    pub plugin: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a task directly without launching TUI
    Execute(ExecuteArgs),

    /// Initialize a new plugin scaffold
    Init,

    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Validate plugin or configuration files
    Validate {
        /// Validate a plugin file
        #[arg(long, value_name = "PATH", conflicts_with = "config")]
        plugin: Option<PathBuf>,

        /// Validate configuration file. If no path provided, validates the default config
        #[arg(long, value_name = "PATH", num_args = 0..=1, conflicts_with = "plugin")]
        config: Option<Vec<PathBuf>>,
    },

    /// Manage plugins (install, remove, upgrade, list)
    ///
    /// - Managed plugins: Installed at XDG_DATA_HOME, managed by config file with [plugins] declaration
    ///
    /// - User plugins: Installed at XDG_CONFIG_HOME, not managed by plugin manager
    ///
    /// - Orphan plugins: Installed at XDG_DATA_HOME but have no declaration in config
    Plugins(PluginsArgs),
}
