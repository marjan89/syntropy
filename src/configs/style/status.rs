use serde::{Deserialize, Serialize};

use crate::configs::style::{Borders, FontWeight};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Status {
    pub left_split: u16,
    pub right_split: u16,
    pub borders: Vec<Borders>,
    pub font_weight: FontWeight,
    pub breadcrumbs_separator: String,
    pub idle_icons: Vec<String>,
    pub error_icons: Vec<String>,
    pub complete_icons: Vec<String>,
    pub running_icons: Vec<String>,
}

impl Default for Status {
    fn default() -> Self {
        let collect_strings =
            |strs: &[&str]| -> Vec<String> { strs.iter().map(|s| s.to_string()).collect() };
        Self {
            left_split: 50,
            right_split: 50,
            borders: vec![Borders::All],
            font_weight: FontWeight::Bold,
            breadcrumbs_separator: String::from(" → "),
            idle_icons: collect_strings(&["✔"]),
            error_icons: collect_strings(&["⛌"]),
            complete_icons: collect_strings(&["✔"]),
            running_icons: collect_strings(&["✴", "✵"]),
        }
    }
}
