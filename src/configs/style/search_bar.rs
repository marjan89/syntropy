use crate::configs::style::{Borders, FontWeight};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SearchBar {
    pub borders: Vec<Borders>,
    pub font_weight: FontWeight,
    pub search_hint: String,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self {
            borders: vec![Borders::All],
            font_weight: FontWeight::Bold,
            search_hint: String::from(">"),
        }
    }
}
