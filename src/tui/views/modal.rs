use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::tui::{
    strings::ModalStrings,
    views::{ColorStyle, style::ModalStyle},
};

#[derive(Default)]
pub struct Modal {
    scroll_offset: u16,
    confirm_key_binding: String,
}

impl Modal {
    pub fn configure(&mut self, confirm_key_binding: String) {
        self.confirm_key_binding = format!(
            "{} ({})",
            ModalStrings::LABEL_BUTTON_DISMISS,
            confirm_key_binding
        );
    }

    pub fn scroll_up(&mut self, offset: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(offset);
    }

    pub fn scroll_down(&mut self, offset: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(offset);
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        content: &str,
        title: &str,
        modal_style: &ModalStyle,
        color_style: &ColorStyle,
    ) {
        let modal_area =
            centered_rect(modal_style.horizontal_size, modal_style.vertical_size, area);

        frame.render_widget(Clear, modal_area);

        let mut outer_block = Block::default();

        if let Some(borders) = modal_style.borders {
            outer_block = outer_block.borders(borders);
        }

        if modal_style.show_title {
            outer_block = outer_block.title(title);
        }

        if let Some(font_weight) = modal_style.font_weight {
            outer_block = outer_block.add_modifier(font_weight);
        }

        outer_block = outer_block
            .style(Style::default().bg(color_style.background_modal))
            .border_style(Style::default().fg(color_style.borders_modal));

        let inner_area = outer_block.inner(modal_area);

        frame.render_widget(outer_block, modal_area);

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(color_style.text_modal))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, vertical_chunks[0]);

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(vertical_chunks[1]);

        let mut dismiss_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(color_style.borders_modal))
            .style(Style::default().bg(color_style.background_modal));

        if let Some(font_weight) = modal_style.font_weight {
            dismiss_block = dismiss_block.add_modifier(font_weight);
        }

        let dismiss_button = Paragraph::new(self.confirm_key_binding.as_str())
            .block(dismiss_block)
            .style(Style::default().fg(color_style.text_modal))
            .alignment(Alignment::Center);

        frame.render_widget(dismiss_button, button_chunks[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
