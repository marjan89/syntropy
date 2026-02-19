use serde::{Deserialize, Serialize};

use crate::configs::style::{Borders, FontWeight};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Modal {
    pub borders: Vec<Borders>,
    pub font_weight: FontWeight,
    pub show_title: bool,
    pub scroll_offset: u16,
    pub vertical_size: u16,
    pub horizontal_size: u16,
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            borders: vec![Borders::All],
            font_weight: FontWeight::Regular,
            show_title: true,
            scroll_offset: 2,
            vertical_size: 60,
            horizontal_size: 60,
        }
    }
}
