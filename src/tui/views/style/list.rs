use ratatui::{style::Modifier, widgets::Borders};

use crate::{configs::style::List, tui::views::style::borders::parse};

pub struct ListStyle {
    pub highlight_symbol: String,
    pub icon_marked: String,
    pub icon_unmarked: String,
    pub borders: Option<Borders>,
    pub font_weight: Option<Modifier>,
}

impl From<&List> for ListStyle {
    fn from(list_style: &List) -> Self {
        Self {
            highlight_symbol: list_style.highlight_symbol.clone(),
            icon_marked: list_style.icon_marked.clone(),
            icon_unmarked: list_style.icon_unmarked.clone(),
            borders: parse(&list_style.borders),
            font_weight: (&list_style.font_weight).into(),
        }
    }
}
