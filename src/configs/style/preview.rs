use serde::{Deserialize, Serialize};

use crate::configs::style::{Borders, FontWeight};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Preview {
    pub borders: Vec<Borders>,
    pub font_weight: FontWeight,
    pub show_title: bool,
    pub scroll_offset: u16,
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            borders: vec![Borders::All],
            font_weight: FontWeight::Regular,
            show_title: true,
            scroll_offset: 2,
        }
    }
}
