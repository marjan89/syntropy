use ratatui::{style::Modifier, widgets::Borders};

use crate::{configs::style::Status, tui::views::style::borders::parse};

pub struct StatusStyle {
    pub left_split: u16,
    pub right_split: u16,
    pub borders: Option<Borders>,
    pub font_weight: Option<Modifier>,
    pub idle_icons: Vec<String>,
    pub error_icons: Vec<String>,
    pub complete_icons: Vec<String>,
    pub running_icons: Vec<String>,
}

impl From<&Status> for StatusStyle {
    fn from(status_style: &Status) -> Self {
        Self {
            left_split: status_style.left_split,
            right_split: status_style.right_split,
            borders: parse(&status_style.borders),
            font_weight: (&status_style.font_weight).into(),
            idle_icons: status_style.idle_icons.clone(),
            error_icons: status_style.error_icons.clone(),
            complete_icons: status_style.complete_icons.clone(),
            running_icons: status_style.running_icons.clone(),
        }
    }
}
