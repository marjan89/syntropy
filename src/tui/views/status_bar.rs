use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::tui::{
    screens::Status,
    views::{ColorStyle, style::StatusStyle},
};

#[derive(Default)]
pub struct StatusBar {
    pub last_keyframe: u64,
    pub cached_status_line: String,
}

impl StatusBar {
    pub fn get_status_line(
        &mut self,
        status: &Status,
        keyframe: u64,
        status_style: &StatusStyle,
    ) -> &String {
        if self.last_keyframe != keyframe {
            self.cached_status_line = format!(
                "{} {} ",
                status,
                self.get_icon(status, keyframe, status_style)
            );
        }
        self.last_keyframe = keyframe;
        &self.cached_status_line
    }

    fn get_icon<'a>(
        &self,
        status: &Status,
        keyframe: u64,
        status_style: &'a StatusStyle,
    ) -> &'a str {
        let icons = match status {
            Status::Idle => &status_style.idle_icons,
            Status::Error => &status_style.error_icons,
            Status::Running => &status_style.running_icons,
            Status::Complete => &status_style.complete_icons,
        };
        if icons.is_empty() {
            return " ";
        }
        let index = (keyframe as usize) % icons.len();
        icons[index].as_str()
    }

    pub fn render(
        &mut self,
        frame: &mut Frame<'_>,
        status: &mut Status,
        breadcrumbs: &str,
        keyframe: u64,
        area: Rect,
        status_style: &StatusStyle,
        color_style: &ColorStyle,
    ) {
        let mut outer_block = Block::default();

        if let Some(borders) = status_style.borders {
            outer_block = outer_block.borders(borders);
        }

        outer_block = outer_block.border_style(
            Style::default()
                .fg(color_style.borders_status)
                .bg(color_style.background_status),
        );

        let inner_area = outer_block.inner(area);

        frame.render_widget(outer_block, area);

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner_area);

        let status_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(status_style.left_split),
                Constraint::Percentage(status_style.right_split),
            ])
            .split(vertical_chunks[1]);

        let mut left_status = Paragraph::new(breadcrumbs).alignment(Alignment::Left);

        let status_line = self.get_status_line(status, keyframe, status_style);

        let mut right_status = Paragraph::new(&status_line[..]).alignment(Alignment::Right);

        let mut text_style = Style::default()
            .fg(color_style.text_status)
            .bg(color_style.background_status);

        if let Some(font_weight) = status_style.font_weight {
            text_style = text_style.patch(font_weight);
        }

        left_status = left_status.style(text_style);
        right_status = right_status.style(text_style);

        frame.render_widget(left_status, status_chunks[0]);
        frame.render_widget(right_status, status_chunks[1]);
    }
}
