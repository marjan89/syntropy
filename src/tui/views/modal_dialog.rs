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
pub struct ModalDialog {
    scroll_offset: u16,
    content: String,
    confirm_key_binding: String,
    cancel_key_binding: String,
}

impl ModalDialog {
    pub fn configure(
        &mut self,
        content: String,
        confirm_key_binding: String,
        cancel_key_binding: String,
    ) {
        self.content = content;
        self.confirm_key_binding = format!(
            "{} ({})",
            ModalStrings::LABEL_BUTTON_CONFIRM,
            confirm_key_binding
        );
        self.cancel_key_binding = format!(
            "{} ({})",
            ModalStrings::LABEL_BUTTON_CANCEL,
            cancel_key_binding
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
        title: &str,
        item: &str,
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

        let paragraph = Paragraph::new(format!("{} {}", self.content, &item))
            .style(Style::default().fg(color_style.text_modal))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, vertical_chunks[0]);

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Percentage(45),
                Constraint::Length(2),
                Constraint::Percentage(45),
                Constraint::Length(2),
            ])
            .split(vertical_chunks[1]);

        let mut cancel_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(color_style.borders_modal))
            .style(Style::default().bg(color_style.background_modal));

        if let Some(font_weight) = modal_style.font_weight {
            cancel_block = cancel_block.add_modifier(font_weight);
        }

        let cancel_button = Paragraph::new(self.cancel_key_binding.as_str())
            .block(cancel_block)
            .style(Style::default().fg(color_style.text_modal))
            .alignment(Alignment::Center);

        frame.render_widget(cancel_button, button_chunks[1]);

        let mut confirm_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(color_style.borders_modal))
            .style(Style::default().bg(color_style.background_modal));

        if let Some(font_weight) = modal_style.font_weight {
            confirm_block = confirm_block.add_modifier(font_weight);
        }

        let confirm_button = Paragraph::new(self.confirm_key_binding.as_str())
            .block(confirm_block)
            .style(Style::default().fg(color_style.text_modal))
            .alignment(Alignment::Center);

        frame.render_widget(confirm_button, button_chunks[3]);
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
