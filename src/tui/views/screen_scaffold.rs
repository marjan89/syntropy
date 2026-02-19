use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::tui::views::style::ScreenScaffoldStyle;

pub fn render_screen_scaffold<F>(
    frame: &mut Frame,
    area: Rect,
    screen_scaffold_style: &ScreenScaffoldStyle,
    callback: F,
) where
    F: FnOnce(&mut Frame<'_>, Rect, Rect),
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(screen_scaffold_style.left_split),
            Constraint::Percentage(screen_scaffold_style.right_split),
        ])
        .split(area);

    callback(frame, chunks[0], chunks[1])
}
