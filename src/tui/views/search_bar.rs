use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::tui::views::{ColorStyle, style::SearchBarStyle};

#[derive(Default)]
pub struct SearchBar {
    input: Input,
}

impl SearchBar {
    pub fn handle_event(&mut self, event: &Event) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(_) => {
                    if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT {
                        self.input.handle_event(event);
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Backspace
                | KeyCode::Delete
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End => {
                    self.input.handle_event(event);
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn value(&self) -> &str {
        self.input.value().trim()
    }

    pub fn is_empty(&self) -> bool {
        self.input.value().is_empty()
    }

    pub fn clear(&mut self) {
        self.input = Input::default();
    }

    pub fn render(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        search_bar_style: &SearchBarStyle,
        color_style: &ColorStyle,
    ) {
        let text = if self.is_empty() {
            search_bar_style.search_hint.to_string()
        } else {
            self.input.value().to_string()
        };

        let mut paragraph_block = Block::default();

        if let Some(borders) = search_bar_style.borders {
            paragraph_block = paragraph_block.borders(borders);
        }

        paragraph_block =
            paragraph_block.border_style(Style::default().fg(color_style.borders_search));

        let mut paragraph = Paragraph::new(text).block(paragraph_block);

        let mut style = Style::default()
            .fg(color_style.text_search)
            .bg(color_style.background_search);

        if let Some(font_weight) = search_bar_style.font_weight {
            style = style.patch(font_weight);
        }

        paragraph = paragraph.style(style);

        frame.render_widget(paragraph, area);
    }
}
