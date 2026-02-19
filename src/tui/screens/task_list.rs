use mlua::Lua;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{runtime::Handle as RuntimeHandle, sync::Mutex};

use crate::{
    app::App,
    execution::{ExecutionResult, Handle, Operation, State},
    plugins::Task,
    tui::{
        events::InputEvent,
        fuzzy_searcher::FuzzySearcher,
        navigation::{Intent, TaskPayload},
        screens::{Screen, Status},
        strings::ModalStrings,
        views::{Modal, ModalDialog, Preview, SelectableList, Styles, render_screen_scaffold},
    },
};
use ratatui::{Frame, layout::Rect};

#[derive(Default)]
struct Cache {
    status: Status,
    previews: HashMap<usize, String>,
    title: String,
    execution_state: State,
}

pub struct TaskListScreen {
    selectable_list: SelectableList,
    preview: Preview,
    show_preview: bool,
    task_keys: Vec<String>,
    cache: Cache,
    fuzzy_searcher: FuzzySearcher,
    items_indices: Vec<usize>,
    modal: Modal,
    modal_content: Option<String>,
    execution_handle: Handle,
    modal_dialog: ModalDialog,
    modal_dialog_shown: bool,
}

impl TaskListScreen {
    pub fn new(
        runtime_handle: RuntimeHandle,
        lua_runtime: &Arc<Mutex<Lua>>,
        show_preview_pane: bool,
    ) -> Self {
        Self {
            selectable_list: SelectableList::new(false),
            preview: Preview::default(),
            show_preview: show_preview_pane,
            task_keys: Vec::new(),
            cache: Cache::default(),
            fuzzy_searcher: FuzzySearcher::default(),
            items_indices: Vec::new(),
            modal: Modal::default(),
            modal_content: None,
            execution_handle: Handle::new(runtime_handle.clone(), lua_runtime),
            modal_dialog: ModalDialog::default(),
            modal_dialog_shown: false,
        }
    }

    fn original_index(&self) -> Option<usize> {
        self.items_indices
            .get(self.selectable_list.selected())
            .copied()
    }

    fn update_preview(&mut self, app: &App, payload: &TaskPayload) {
        let Some(original_idx) = self.original_index() else {
            return;
        };
        if let Some(task_key) = self.task_keys.get(original_idx)
            && let Some(task) = app.get_task(payload.plugin_idx, task_key)
        {
            if self.cache.title != task.name {
                self.cache.title = task.name.clone();
            }
            if self.cache.previews.contains_key(&original_idx) {
                return;
            };
            self.cache
                .previews
                .insert(original_idx, task.description.clone());
        }
    }

    fn execute(&mut self, task: &Arc<Task>) {
        let _ = self.execution_handle.execute(Operation::Execute {
            task: Arc::clone(task),
            selected_items: vec![],
        });
    }
}

impl Screen<TaskPayload> for TaskListScreen {
    fn on_enter(&mut self, app: &App, payload: &TaskPayload) {
        if let Some(plugin) = app.get_plugin(payload.plugin_idx) {
            self.task_keys = plugin.tasks.keys().cloned().collect();
            // Sort task keys alphabetically (case-insensitive) for consistent display order
            self.task_keys.sort_by_key(|a| a.to_lowercase());
            self.items_indices = (0..self.task_keys.len()).collect();
            self.selectable_list.select(0);
            self.update_preview(app, payload);
        }
        if let Some(original_idx) = self.original_index()
            && let Some(selected_task_key) = self.task_keys.get(original_idx)
            && let Some(task) = app.get_task(payload.plugin_idx, selected_task_key)
            && let Some(confirmation_message) = &task.execution_confirmation_message
        {
            self.modal_dialog.configure(
                confirmation_message.clone(),
                app.config.keybindings.confirm.clone(),
                app.config.keybindings.back.clone(),
            );
        };
        self.modal.configure(app.config.keybindings.confirm.clone());
    }

    fn on_exit(&mut self) {
        self.cache.previews.clear();
        self.task_keys.clear();
        self.selectable_list.reset_selected();
        self.modal_content = None;
        self.modal_dialog_shown = false;
    }

    fn on_update(&mut self, app: &App, payload: &TaskPayload) -> Intent {
        match self.execution_handle.consume_result() {
            ExecutionResult::Output(output, exit_code) => {
                if app.config.exit_on_execute {
                    return Intent::Quit;
                } else {
                    let should_show_modal = if let Some(original_idx) = self.original_index()
                        && let Some(selected_task_key) = self.task_keys.get(original_idx)
                        && let Some(task) = app.get_task(payload.plugin_idx, selected_task_key)
                    {
                        !task.suppress_success_notification || exit_code > 0
                    } else {
                        exit_code > 0
                    };
                    if should_show_modal {
                        self.modal_content = Some(output);
                    }
                }
            }
            ExecutionResult::Error(output) => {
                if app.config.exit_on_execute {
                    return Intent::Quit;
                } else {
                    self.modal_content = Some(output);
                }
            }
            _ => {}
        }
        Intent::None
    }

    fn handle_event(&mut self, event: InputEvent, app: &App, payload: &TaskPayload) -> Intent {
        if self.modal_content.is_some() {
            return match event {
                InputEvent::Confirm => {
                    self.modal.reset_scroll();
                    self.modal_content = None;
                    Intent::None
                }
                InputEvent::ScrollPreviewUp => {
                    self.modal.scroll_up(app.config.styles.modal.scroll_offset);
                    Intent::None
                }
                InputEvent::ScrollPreviewDown => {
                    self.modal
                        .scroll_down(app.config.styles.modal.scroll_offset);
                    Intent::None
                }
                _ => Intent::None,
            };
        }
        if self.modal_dialog_shown {
            match event {
                InputEvent::Confirm => {
                    if let Some(original_idx) = self.original_index()
                        && let Some(selected_task_key) = self.task_keys.get(original_idx)
                        && let Some(task) = app.get_task(payload.plugin_idx, selected_task_key)
                    {
                        self.modal_dialog.reset_scroll();
                        self.modal_dialog_shown = false;
                        self.execute(task);
                    }
                }
                InputEvent::ScrollPreviewUp => {
                    self.modal_dialog
                        .scroll_up(app.config.styles.modal.scroll_offset);
                }
                InputEvent::ScrollPreviewDown => {
                    self.modal_dialog
                        .scroll_down(app.config.styles.modal.scroll_offset);
                }
                InputEvent::Back => {
                    self.modal_dialog_shown = false;
                }
                _ => {}
            };
            return Intent::None;
        }
        match event {
            InputEvent::NextItem => {
                self.selectable_list.select_next();
                self.preview.reset_scroll();
                self.update_preview(app, payload);
            }
            InputEvent::PreviousItem => {
                self.selectable_list.select_previous();
                self.preview.reset_scroll();
                self.update_preview(app, payload);
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
                    && let Some(selected_task_key) = self.task_keys.get(original_idx)
                    && let Some(task) = app.get_task(payload.plugin_idx, selected_task_key)
                    && task.item_sources.is_none()
                {
                    if task.execution_confirmation_message.is_some() {
                        self.modal_dialog_shown = true;
                    } else {
                        self.execute(task);
                    }
                } else if let Some(original_idx) = self.original_index()
                    && let Some(selected_task_key) = self.task_keys.get(original_idx)
                {
                    return Intent::SelectTask {
                        plugin_idx: payload.plugin_idx,
                        task_key: selected_task_key.clone(),
                    };
                }
            }
            _ => {}
        }
        Intent::None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let items: Vec<&String> = self
            .items_indices
            .iter()
            .map(|&idx| &self.task_keys[idx])
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

        if let Some(content) = &self.modal_content {
            self.modal.render(
                frame,
                area,
                content,
                ModalStrings::TITLE_MODAL_RESULT,
                &styles.modal,
                &styles.colors,
            );
        }

        if self.modal_dialog_shown {
            self.modal_dialog.render(
                frame,
                area,
                ModalStrings::TITLE_MODAL_DIALOG_CONFIRM,
                "",
                &styles.modal,
                &styles.colors,
            );
        }
    }

    fn get_status(&mut self) -> &mut Status {
        let current_state = self.execution_handle.read_state();
        if current_state != self.cache.execution_state {
            self.cache.status = match &current_state {
                State::None => Status::Idle,
                State::Running => Status::Running,
                State::Finished => Status::Complete,
                State::Error => Status::Error,
            };
            self.cache.execution_state = current_state;
        }
        &mut self.cache.status
    }

    fn on_search(&mut self, query: &str) {
        self.items_indices = self.fuzzy_searcher.search(&self.task_keys, query);
        if !self.items_indices.is_empty() {
            self.selectable_list.select_first();
        }
    }
    fn consumed_event(&mut self, event: &InputEvent) -> bool {
        matches!(event, InputEvent::Back) && self.modal_dialog_shown
    }
}
