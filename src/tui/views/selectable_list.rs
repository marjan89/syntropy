use std::collections::HashSet;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::tui::views::{ColorStyle, style::ListStyle};

#[derive(Default)]
pub struct SelectionCountCache {
    item_count: usize,
    marked_item_count: usize,
    formatted_value: String,
}

impl SelectionCountCache {
    fn get_selection_count(&mut self, marked_item_count: usize, item_count: usize) -> &str {
        let cache_invalid =
            marked_item_count != self.marked_item_count || item_count != self.item_count;

        if cache_invalid {
            self.formatted_value = format!("[{}/{}]", marked_item_count, item_count);
            self.item_count = item_count;
            self.marked_item_count = marked_item_count;
        }

        &self.formatted_value
    }
}

pub struct SelectableList {
    list_state: ListState,
    multiselect: bool,
    selection_count_cache: SelectionCountCache,
}

impl SelectableList {
    pub fn new(multiselect: bool) -> Self {
        Self {
            list_state: ListState::default(),
            multiselect,
            selection_count_cache: SelectionCountCache::default(),
        }
    }

    pub fn set_multiselect_enable(&mut self, enabled: bool) {
        self.multiselect = enabled;
    }

    pub fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn select_first(&mut self) {
        self.list_state.select(Some(0));
    }

    pub fn select(&mut self, index: usize) {
        self.list_state.select(Some(index));
    }

    pub fn select_next(&mut self) {
        self.list_state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.list_state.select_previous();
    }

    pub fn reset_selected(&mut self) {
        self.list_state.select(None);
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        items: &[&String],
        list_style: &ListStyle,
        color_style: &ColorStyle,
        external_marks: Option<&HashSet<usize>>,
    ) {
        let empty_marks = HashSet::new();
        let marks = external_marks.unwrap_or(&empty_marks);
        let render_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| -> ListItem<'static> {
                let icon = if !self.multiselect {
                    ""
                } else if marks.contains(&idx) {
                    &list_style.icon_marked
                } else {
                    &list_style.icon_unmarked
                };
                ListItem::new(format!("{} {}", icon, item))
            })
            .collect();

        let apply_font_weight = |style: Style| -> Style {
            list_style
                .font_weight
                .map_or(style, |m| style.add_modifier(m))
        };

        let item_count = render_items.len();

        let list = List::new(render_items)
            .style(apply_font_weight(
                Style::default()
                    .fg(color_style.text_list)
                    .bg(color_style.background_list),
            ))
            .highlight_style(apply_font_weight(
                Style::default()
                    .bg(color_style.highlights_background)
                    .fg(color_style.highlights_text),
            ))
            .highlight_symbol(list_style.highlight_symbol.as_str());

        let mut outer_block = Block::default();

        if let Some(borders) = list_style.borders {
            outer_block = outer_block.borders(borders).border_style(
                Style::default()
                    .fg(color_style.borders_list)
                    .bg(color_style.background_list),
            );
        }

        let inner_area = outer_block.inner(area);

        frame.render_widget(outer_block, area);

        if self.multiselect {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(inner_area);

            let count_text = self
                .selection_count_cache
                .get_selection_count(marks.len(), item_count);

            let mut style = Style::default()
                .fg(color_style.text_list)
                .bg(color_style.background_list);

            if let Some(font_weight) = list_style.font_weight {
                style = style.add_modifier(font_weight);
            }

            let selection_count = Paragraph::new(count_text)
                .alignment(Alignment::Right)
                .style(style);

            frame.render_widget(selection_count, vertical_chunks[1]);
            frame.render_stateful_widget(list, vertical_chunks[0], &mut self.list_state);
        } else {
            frame.render_stateful_widget(list, inner_area, &mut self.list_state);
        }
    }
}
