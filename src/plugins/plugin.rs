use std::collections::HashMap;

use crate::plugins::TaskMap;
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Mode {
    Multi,
    #[default]
    None,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub metadata: Metadata,
    pub tasks: TaskMap,
}

impl Plugin {
    pub const LUA_PROPERTY_TASKS: &str = "tasks";
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub icon: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub plugin_name: String,

    pub task_key: String,

    pub name: String,

    pub description: String,

    pub item_sources: Option<HashMap<String, ItemSource>>,

    pub mode: Mode,

    pub preview_polling_interval: usize,

    pub item_polling_interval: usize,

    pub execution_confirmation_message: Option<String>,

    pub suppress_success_notification: bool,
}

impl Task {
    pub const LUA_FN_NAME_PRE_RUN: &str = "pre_run";
    pub const LUA_FN_NAME_POST_RUN: &str = "post_run";
    pub const LUA_FN_NAME_PREVIEW: &str = "preview";
    pub const LUA_FN_NAME_EXECUTE: &str = "execute";
    pub const LUA_PROPERTY_ITEM_SOURCES: &str = "item_sources";
}

#[derive(Debug, Clone)]
pub struct ItemSource {
    pub item_source_key: String,

    pub tag: String,
}

impl ItemSource {
    pub const LUA_FN_NAME_EXECUTE: &str = "execute";
    pub const LUA_FN_NAME_ITEMS: &str = "items";
    pub const LUA_FN_NAME_PRESELECTED_ITEMS: &str = "preselected_items";
    pub const LUA_FN_NAME_PREVIEW: &str = "preview";
}
