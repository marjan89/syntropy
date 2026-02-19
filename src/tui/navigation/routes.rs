use std::fmt::Display;

use crate::tui::{
    navigation::{ItemPayload, PluginPayload, TaskPayload},
    strings::RouteStrings,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Plugin { payload: PluginPayload },
    Task { payload: TaskPayload },
    Item { payload: ItemPayload },
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Route::Plugin { .. } => write!(f, "{}", RouteStrings::PLUGIN),
            Route::Task { .. } => write!(f, "{}", RouteStrings::TASK),
            Route::Item { .. } => write!(f, "{}", RouteStrings::ITEM),
        }
    }
}
