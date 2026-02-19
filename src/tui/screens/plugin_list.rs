use crate::{
    app::App,
    tui::{
        events::InputEvent,
        fuzzy_searcher::FuzzySearcher,
        navigation::{Intent, PluginPayload},
        screens::{Screen, Status},
        strings::PreviewStrings,
        views::{Preview, SelectableList, Styles, render_screen_scaffold},
    },
};
use core::str;
use ratatui::{Frame, layout::Rect};
use std::collections::HashMap;

#[derive(Default)]
struct Cache {
    status: Status,
    previews: HashMap<usize, String>,
    plugin_names: Vec<String>,
    title: String,
}

pub struct PluginListScreen {
    selectable_list: SelectableList,
    preview: Preview,
    show_preview: bool,
    cache: Cache,
    fuzzy_searcher: FuzzySearcher,
    item_indices: Vec<usize>,
}

impl PluginListScreen {
    pub fn new(show_preview_pane: bool) -> Self {
        let mut plugin_list_screen = Self {
            selectable_list: SelectableList::new(false),
            preview: Preview::default(),
            show_preview: show_preview_pane,
            cache: Cache::default(),
            fuzzy_searcher: FuzzySearcher::default(),
            item_indices: Vec::new(),
        };

        plugin_list_screen.selectable_list.select(0);

        plugin_list_screen
    }

    fn original_index(&self) -> Option<usize> {
        self.item_indices
            .get(self.selectable_list.selected())
            .copied()
    }

    fn update_preview(&mut self, app: &App) {
        let Some(original_idx) = self.original_index() else {
            return;
        };
        let Some(plugin) = app.get_plugin(original_idx) else {
            return;
        };
        if self.cache.title != plugin.metadata.name {
            self.cache.title = plugin.metadata.name.clone();
        }
        if self.cache.previews.contains_key(&original_idx) {
            return;
        };
        self.cache.previews.insert(
            original_idx,
            format!(
                "{}: {}\n{}: {}\n\n{}: {}\n\n{}: {}\n\n{}:\n{}",
                PreviewStrings::PLUGIN,
                plugin.metadata.name,
                PreviewStrings::VERSION,
                plugin.metadata.version,
                PreviewStrings::DESCRIPTION,
                plugin.metadata.description,
                PreviewStrings::PLATFORMS,
                plugin.metadata.platforms.join(", "),
                PreviewStrings::TASKS,
                {
                    // Collect and sort task keys for consistent display order
                    let mut task_keys: Vec<_> = plugin.tasks.keys().collect();
                    task_keys.sort_by_key(|a| a.to_lowercase());
                    task_keys
                        .iter()
                        .map(|k| format!("  • {}", k))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            ),
        );
    }
}
impl Screen<PluginPayload> for PluginListScreen {
    fn on_enter(&mut self, app: &App, _payload: &PluginPayload) {
        self.cache.plugin_names = app
            .plugins
            .iter()
            .map(|p| format!("{} {}", p.metadata.icon, p.metadata.name))
            .collect();
        self.item_indices = (0..self.cache.plugin_names.len()).collect();
        self.selectable_list.select(0);
        self.update_preview(app);
    }

    fn on_exit(&mut self) {
        self.cache.previews.clear();
        self.item_indices.clear();
        self.selectable_list.reset_selected();
    }

    fn handle_event(&mut self, event: InputEvent, app: &App, _payload: &PluginPayload) -> Intent {
        match event {
            InputEvent::NextItem => {
                self.selectable_list.select_next();
                self.preview.reset_scroll();
                self.update_preview(app);
            }
            InputEvent::PreviousItem => {
                self.selectable_list.select_previous();
                self.preview.reset_scroll();
                self.update_preview(app);
            }
            InputEvent::ScrollPreviewUp => {
                self.preview
                    .scroll_up(app.config.styles.preview.scroll_offset);
            }
            InputEvent::ScrollPreviewDown => {
                self.preview
                    .scroll_down(app.config.styles.preview.scroll_offset);
            }
            InputEvent::TogglePreview => {
                self.show_preview = !self.show_preview;
            }
            InputEvent::Confirm => {
                if let Some(original_idx) = self.original_index()
                    && app.get_plugin(original_idx).is_some()
                {
                    return Intent::SelectPlugin {
                        plugin_idx: original_idx,
                    };
                }
            }
            _ => {}
        }
        Intent::None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let items: Vec<&String> = self
            .item_indices
            .iter()
            .map(|&idx| &self.cache.plugin_names[idx])
            .collect();

        if self.show_preview {
            let original_idx = self.original_index().unwrap_or(0);
            let preview = self
                .cache
                .previews
                .get(&original_idx)
                .map_or("", |s| s.as_str());
            render_screen_scaffold(
                frame,
                area,
                &styles.screen_scaffold_style,
                |frame, left, right| -> () {
                    self.selectable_list.render(
                        frame,
                        left,
                        &items,
                        &styles.list,
                        &styles.colors,
                        None,
                    );
                    self.preview.render(
                        frame,
                        right,
                        preview,
                        &self.cache.title,
                        &styles.preview,
                        &styles.colors,
                    );
                },
            );
        } else {
            self.selectable_list
                .render(frame, area, &items, &styles.list, &styles.colors, None);
        }
    }

    fn get_status(&mut self) -> &mut Status {
        &mut self.cache.status
    }

    fn on_search(&mut self, query: &str) {
        self.item_indices = self.fuzzy_searcher.search(&self.cache.plugin_names, query);

        if !self.item_indices.is_empty() {
            self.selectable_list.select_first();
        }
    }
}
