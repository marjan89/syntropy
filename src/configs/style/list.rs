use crate::configs::style::styles::{Borders, FontWeight};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct List {
    pub highlight_symbol: String,
    pub icon_marked: String,
    pub icon_unmarked: String,
    // bold, regular
    pub font_weight: FontWeight,
    // top, left, right, bottom, all
    pub borders: Vec<Borders>,
}

impl Default for List {
    fn default() -> Self {
        Self {
            highlight_symbol: String::from("→"),
            icon_marked: String::from("▣"),
            icon_unmarked: String::from("□"),
            borders: vec![Borders::All],
            font_weight: FontWeight::Regular,
        }
    }
}
