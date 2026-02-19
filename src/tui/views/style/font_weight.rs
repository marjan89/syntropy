use ratatui::style::Modifier;

use crate::configs::style::FontWeight;

impl From<&FontWeight> for Option<Modifier> {
    fn from(font_weight: &FontWeight) -> Option<Modifier> {
        match font_weight {
            FontWeight::Bold => Some(Modifier::BOLD),
            FontWeight::Regular => None,
        }
    }
}
