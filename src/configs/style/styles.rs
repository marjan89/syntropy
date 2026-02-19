use serde::{Deserialize, Serialize};

use crate::configs::style::{Colors, List, Modal, Preview, ScreenScaffold, SearchBar, Status};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde[rename_all = "lowercase"]]
pub enum FontWeight {
    Bold,
    Regular,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde[rename_all = "lowercase"]]
pub enum Borders {
    Top,
    Left,
    Right,
    Bottom,
    All,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde[default, deny_unknown_fields]]
pub struct Styles {
    pub list: List,
    pub preview: Preview,
    pub modal: Modal,
    pub status: Status,
    pub screen_scaffold: ScreenScaffold,
    pub search_bar: SearchBar,
    pub colors: Colors,
}
