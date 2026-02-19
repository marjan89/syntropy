use ratatui::{style::Modifier, widgets::Borders};

use crate::configs::style::SearchBar;
use crate::tui::views::style::borders::parse;

pub struct SearchBarStyle {
    pub borders: Option<Borders>,
    pub font_weight: Option<Modifier>,
    pub search_hint: String,
}

impl From<&SearchBar> for SearchBarStyle {
    fn from(search_bar_style: &SearchBar) -> Self {
        Self {
            borders: parse(&search_bar_style.borders),
            font_weight: (&search_bar_style.font_weight).into(),
            search_hint: (search_bar_style.search_hint.clone()),
        }
    }
}
