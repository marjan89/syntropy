use ratatui::widgets::Borders;

use crate::{configs::style::Preview, tui::views::style::borders::parse};

pub struct PreviewStyle {
    pub borders: Option<Borders>,
    pub font_weight: Option<ratatui::style::Modifier>,
    pub show_title: bool,
}

impl From<&Preview> for PreviewStyle {
    fn from(preview_style: &Preview) -> Self {
        Self {
            borders: parse(&preview_style.borders),
            font_weight: (&preview_style.font_weight).into(),
            show_title: preview_style.show_title,
        }
    }
}
