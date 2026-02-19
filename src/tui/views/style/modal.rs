use ratatui::widgets::Borders;

use crate::{configs::style::Modal, tui::views::style::borders::parse};

pub struct ModalStyle {
    pub borders: Option<Borders>,
    pub font_weight: Option<ratatui::style::Modifier>,
    pub show_title: bool,
    pub vertical_size: u16,
    pub horizontal_size: u16,
}

impl From<&Modal> for ModalStyle {
    fn from(modal_style: &Modal) -> Self {
        Self {
            borders: parse(&modal_style.borders),
            font_weight: (&modal_style.font_weight).into(),
            show_title: modal_style.show_title,
            vertical_size: modal_style.vertical_size,
            horizontal_size: modal_style.horizontal_size,
        }
    }
}
