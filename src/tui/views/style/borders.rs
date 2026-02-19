use crate::configs::style;
use ratatui::widgets::Borders;

impl From<&style::Borders> for Borders {
    fn from(config_borders: &style::Borders) -> Self {
        match config_borders {
            style::Borders::Top => Borders::TOP,
            style::Borders::Left => Borders::LEFT,
            style::Borders::Right => Borders::RIGHT,
            style::Borders::Bottom => Borders::BOTTOM,
            style::Borders::All => Borders::ALL,
        }
    }
}

pub fn parse(config_borders: &[style::Borders]) -> Option<Borders> {
    config_borders
        .iter()
        .map(Borders::from)
        .reduce(|acc, b| acc | b)
}
