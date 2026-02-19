use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Paragraph},
};

use crate::tui::views::{ColorStyle, style::PreviewStyle};

#[derive(Default)]
pub struct Preview {
    scroll_offset: u16,
}
impl Preview {
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
        preview: &str,
        title: &str,
        preview_style: &PreviewStyle,
        color_style: &ColorStyle,
    ) {
        let mut block = Block::default();

        if let Some(borders) = preview_style.borders {
            block = block.borders(borders);
        }

        if preview_style.show_title {
            block = block.title(title);
        }

        if let Some(font_weight) = preview_style.font_weight {
            block = block.add_modifier(font_weight);
        }

        block = block.border_style(Style::default().fg(color_style.borders_preview));

        let paragraph = Paragraph::new(preview)
            .block(block)
            .style(
                Style::default()
                    .fg(color_style.text_preview)
                    .bg(color_style.background_preview),
            )
            .scroll((self.scroll_offset, 0));
        frame.render_widget(paragraph, area);
    }
}
