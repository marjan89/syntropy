use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

use crate::{
    configs::{KeyBindings, PluginDeclaration, Styles},
    tui::key_bindings::ParsedKeyBindings,
};
use anyhow::{Context, Result, ensure};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub plugins: HashMap<String, PluginDeclaration>,
    pub default_plugin: Option<String>,
    pub default_task: Option<String>,
    pub default_plugin_icon: String,
    pub keybindings: KeyBindings,
    pub styles: Styles,
    pub status_bar: bool,
    pub search_bar: bool,
    pub show_preview_pane: bool,
    pub exit_on_execute: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plugins: HashMap::default(),
            default_plugin: None,
            default_task: None,
            default_plugin_icon: String::from("⚒"),
            keybindings: KeyBindings::default(),
            styles: Styles::default(),
            status_bar: true,
            search_bar: true,
            show_preview_pane: true,
            exit_on_execute: false,
        }
    }
}

pub fn load_config(config_path: PathBuf) -> Result<Config> {
    let contents = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {:?}", config_path))?;

    let config: Config =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {:?}", config_path))?;

    Ok(config)
}

pub fn validate_config(config: &Config) -> Result<()> {
    for declaration in config.plugins.values() {
        declaration.validate()?;
    }

    let screen_scaffold_style = &config.styles.screen_scaffold;
    ensure!(
        screen_scaffold_style.left_split + screen_scaffold_style.right_split == 100,
        "Screen scaffold style left and right split must amount to 100"
    );

    let status_style = &config.styles.status;
    ensure!(
        status_style.left_split + status_style.right_split == 100,
        "Status style left and right split must amount to 100"
    );

    let modal_style = &config.styles.modal;
    ensure!(
        modal_style.vertical_size < 100 && modal_style.horizontal_size < 100,
        "Modal style vertical_size and horizontal_size must not exceed 100"
    );

    ensure!(
        config.default_plugin_icon.width() == 1,
        "Default plugin icon '{}' must occupy a single terminal cell",
        config.default_plugin_icon
    );

    ensure!(
        config.default_task.is_none() || config.default_plugin.is_some(),
        "default_task requires default_plugin to be set"
    );

    ParsedKeyBindings::from(&config.keybindings).context("Invalid keybinding configuration")?;

    Ok(())
}
