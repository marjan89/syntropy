use std::{
    collections::{HashMap, HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    app::App,
    execution::{ExecutionResult, Handle, Operation, State},
    plugins::{Mode, Task},
    tui::{
        events::InputEvent,
        fuzzy_searcher::FuzzySearcher,
        navigation::{Intent, ItemPayload},
        screens::{Screen, Status},
        strings::{ModalStrings, PreviewStrings},
        views::{Modal, ModalDialog, Preview, SelectableList, Styles, render_screen_scaffold},
    },
};
use mlua::Lua;
use ratatui::{Frame, layout::Rect};
use tokio::{runtime::Handle as RuntimeHandle, sync::Mutex};

#[derive(Default, PartialEq)]
struct ExecutionStates {
    execution: State,
    preview: State,
}

#[derive(Default)]
struct Cache {
    previews: HashMap<String, String>,
    status: Status,
    execution_states: ExecutionStates,
    instant_since_last_item_poll: Option<Instant>,
    instant_since_last_preview_poll: Option<Instant>,
    search_query: String,
    display_marked: HashSet<usize>,
    display_marked_dirty: bool,
    items_hash: u64,
    pending_execution_items: String,
}

impl Cache {
    fn clear(&mut self) {
        self.previews.clear();
        self.status = Status::default();
        self.execution_states = ExecutionStates::default();
        self.instant_since_last_item_poll = None;
        self.instant_since_last_preview_poll = None;
        self.search_query.clear();
        self.display_marked.clear();
        self.display_marked_dirty = false;
        self.items_hash = 0;
        self.pending_execution_items.clear();
    }
}

pub struct ItemListScreen {
    items: Vec<Rc<String>>,
    search_results: Vec<Rc<String>>,
    search_results_map: HashMap<Rc<String>, usize>,
    marked_items: HashSet<String>,
    selected_item: Rc<String>,
    pending_preview_item: Option<Rc<String>>,
    fuzzy_searcher: FuzzySearcher,
    selectable_list: SelectableList,
    preview: Preview,
    modal: Modal,
    modal_dialog: ModalDialog,
    show_preview: bool,
    execution_handle: Handle,
    preview_handle: Handle,
    cache: Cache,
    modal_content: Option<String>,
    modal_dialog_shown: bool,
    pending_execution_items: Vec<String>,
}

impl ItemListScreen {
    pub fn new(
        runtime_handle: RuntimeHandle,
        lua_runtime: &Arc<Mutex<Lua>>,
        show_preview_pane: bool,
    ) -> Self {
        Self {
            items: Vec::new(),
            search_results: Vec::new(),
            search_results_map: HashMap::new(),
            marked_items: HashSet::new(),
            selected_item: Rc::new(String::new()),
            fuzzy_searcher: FuzzySearcher::default(),
            selectable_list: SelectableList::new(true),
            show_preview: show_preview_pane,
            preview: Preview::default(),
            modal: Modal::default(),
            modal_dialog: ModalDialog::default(),
            execution_handle: Handle::new(runtime_handle.clone(), lua_runtime),
            preview_handle: Handle::new(runtime_handle.clone(), lua_runtime),
            pending_preview_item: None,
            pending_execution_items: Vec::new(),
            cache: Cache::default(),
            modal_content: None,
            modal_dialog_shown: false,
        }
    }

    fn poll_items(&mut self, app: &App, payload: &ItemPayload) {
        if !self.modal_dialog_shown
            && let Some(task) = app.get_task(payload.plugin_idx, payload.task_key.as_str())
            && task.item_polling_interval > 0
            && let Some(last_item_poll) = self.cache.instant_since_last_item_poll
            && last_item_poll.elapsed() >= Duration::from_millis(task.item_polling_interval as u64)
            && !self.execution_handle.is_executing()
        {
            let _ = self.execution_handle.execute(Operation::Items {
                task: Arc::clone(task),
            });
            self.cache.instant_since_last_item_poll = Some(Instant::now());
        }
    }

    fn update_preview(&mut self, task: &Arc<Task>) {
        let pending_cache = if let Some(pending_preview) = &self.pending_preview_item {
            pending_preview == &self.selected_item
        } else {
            false
        };

        let cache_valid = self.cache.previews.contains_key(&**self.selected_item)
            || pending_cache
            || self.preview_handle.is_executing();

        let should_invalidate_cache = task.preview_polling_interval > 0
            && self
                .cache
                .instant_since_last_preview_poll
                .is_some_and(|last_preview_poll| {
                    last_preview_poll.elapsed()
                        >= Duration::from_millis(task.preview_polling_interval as u64)
                });

        if cache_valid && !should_invalidate_cache {
            return;
        }

        if self
            .preview_handle
            .execute(Operation::Preview {
                task: Arc::clone(task),
                current_item: (*self.selected_item).clone(),
            })
            .is_ok()
        {
            self.pending_preview_item = Some(Rc::clone(&self.selected_item));
        }
        self.cache.instant_since_last_preview_poll = Some(Instant::now());
    }

    fn sync_selected_item(&mut self) {
        if !self.search_results.is_empty() {
            let selected_idx = self.selectable_list.selected();
            if let Some(item) = self.search_results.get(selected_idx) {
                self.selected_item = Rc::clone(item);
            }
        } else {
            self.selected_item = Rc::new(String::new());
        }
    }

    fn search(&mut self) {
        let previously_selected = if !self.selected_item.is_empty() {
            Some(Rc::clone(&self.selected_item))
        } else {
            None
        };

        let search_indexes = self
            .fuzzy_searcher
            .search(&self.items, self.cache.search_query.as_str());

        self.search_results = search_indexes
            .iter()
            .map(|index| self.items[*index].clone())
            .collect();

        self.search_results_map = self
            .search_results
            .iter()
            .enumerate()
            .map(|(idx, item)| (Rc::clone(item), idx))
            .collect();

        self.cache.display_marked_dirty = true;

        if !self.search_results.is_empty() {
            if let Some(prev_item) = previously_selected {
                if let Some(&new_idx) = self.search_results_map.get(&prev_item) {
                    self.selectable_list.select(new_idx);
                } else {
                    self.selectable_list.select_first();
                }
            } else {
                self.selectable_list.select_first();
            }
        }
        self.sync_selected_item();
    }

    fn execute(&mut self, task: &Arc<Task>) {
        self.cache.pending_execution_items.clear();
        let execution_items = self.pending_execution_items.clone();
        self.pending_execution_items.clear();
        let _ = self.execution_handle.execute(Operation::Execute {
            task: Arc::clone(task),
            selected_items: execution_items,
        });
    }
}

impl Screen<ItemPayload> for ItemListScreen {
    fn on_enter(&mut self, app: &App, payload: &ItemPayload) {
        let Some(task) = app.get_task(payload.plugin_idx, &payload.task_key) else {
            return;
        };
        if let Some(confirmation_message) = &task.execution_confirmation_message {
            self.modal_dialog.configure(
                confirmation_message.clone(),
                app.config.keybindings.confirm.clone(),
                app.config.keybindings.back.clone(),
            );
        };
        self.modal.configure(app.config.keybindings.confirm.clone());
        let _ = self.execution_handle.execute(Operation::Items {
            task: Arc::clone(task),
        });
        self.cache.instant_since_last_item_poll = Some(Instant::now());

        self.selectable_list
            .set_multiselect_enable(matches!(task.mode, Mode::Multi));

        self.selectable_list.select(0);
    }

    fn on_exit(&mut self) {
        self.cache.clear();
        self.items.clear();
        self.search_results.clear();
        self.search_results_map.clear();
        self.marked_items.clear();
        self.selected_item = Rc::new(String::new());
        self.selectable_list.reset_selected();
        self.pending_preview_item = None;
        self.pending_execution_items.clear();
        self.modal_content = None;
        self.modal_dialog_shown = false;
    }

    fn on_update(&mut self, app: &App, payload: &ItemPayload) -> Intent {
        self.poll_items(app, payload);
        match self.execution_handle.consume_result() {
            ExecutionResult::Items {
                items,
                preselected_items,
            } => {
                let mut hasher = DefaultHasher::new();
                for item in &items {
                    item.hash(&mut hasher);
                }
                let new_hash = hasher.finish();

                if new_hash != self.cache.items_hash {
                    self.items = items.into_iter().map(Rc::new).collect();
                    self.cache.items_hash = new_hash;
                    self.search();
                }

                preselected_items.iter().for_each(|preselected| {
                    self.marked_items.insert(preselected.clone());
                });
                self.cache.display_marked_dirty = true;
            }
            ExecutionResult::Output(output, exit_code) => {
                if app.config.exit_on_execute {
                    return Intent::Quit;
                } else {
                    let should_show_modal =
                        if let Some(task) = app.get_task(payload.plugin_idx, &payload.task_key) {
                            !task.suppress_success_notification || exit_code > 0
                        } else {
                            exit_code > 0
                        };

                    if should_show_modal {
                        self.modal_content = Some(output);
                    }
                    if let Some(task) = app.get_task(payload.plugin_idx, &payload.task_key) {
                        let _ = self.execution_handle.execute(Operation::Items {
                            task: Arc::clone(task),
                        });
                    }
                }
            }
            ExecutionResult::Error(output) => {
                if app.config.exit_on_execute {
                    return Intent::Quit;
                } else {
                    self.modal_content = Some(output);
                    if let Some(task) = app.get_task(payload.plugin_idx, &payload.task_key) {
                        let _ = self.execution_handle.execute(Operation::Items {
                            task: Arc::clone(task),
                        });
                    }
                }
            }
            _ => {}
        }

        if let ExecutionResult::Preview(output) | ExecutionResult::Error(output) =
            self.preview_handle.consume_result()
            && let Some(idx) = self.pending_preview_item.clone()
        {
            self.cache.previews.insert((*idx).clone(), output);
            self.pending_preview_item = None;
        }

        if let Some(task) = app.get_task(payload.plugin_idx, &payload.task_key) {
            self.update_preview(task);
        }

        Intent::None
    }

    fn handle_event(&mut self, event: InputEvent, app: &App, payload: &ItemPayload) -> Intent {
        if self.modal_content.is_some() {
            match event {
                InputEvent::Confirm => {
                    self.modal.reset_scroll();
                    self.modal_content = None;
                }
                InputEvent::ScrollPreviewUp => {
                    self.modal.scroll_up(app.config.styles.modal.scroll_offset);
                }
                InputEvent::ScrollPreviewDown => {
                    self.modal
                        .scroll_down(app.config.styles.modal.scroll_offset);
                }
                _ => {}
            };
            return Intent::None;
        }
        let Some(task) = app.get_task(payload.plugin_idx, &payload.task_key) else {
            return Intent::None;
        };
        if self.modal_dialog_shown {
            match event {
                InputEvent::Confirm => {
                    self.modal_dialog.reset_scroll();
                    self.modal_dialog_shown = false;
                    self.execute(task);
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
                self.sync_selected_item();
                self.preview.reset_scroll();
                self.update_preview(task);
            }
            InputEvent::PreviousItem => {
                self.selectable_list.select_previous();
                self.sync_selected_item();
                self.preview.reset_scroll();
                self.update_preview(task);
            }
            InputEvent::ScrollPreviewUp => {
                self.preview
                    .scroll_up(app.config.styles.preview.scroll_offset);
            }
            InputEvent::ScrollPreviewDown => {
                if self.modal_content.is_some() {
                    self.modal
                        .scroll_down(app.config.styles.modal.scroll_offset);
                } else {
                    self.preview
                        .scroll_down(app.config.styles.preview.scroll_offset);
                }
            }
            InputEvent::TogglePreview => {
                self.show_preview = !self.show_preview;
            }
            InputEvent::Select => {
                if matches!(task.mode, Mode::Multi) {
                    let selected_item = &self.selected_item;
                    if self.marked_items.contains(&**selected_item) {
                        self.marked_items.remove(&**selected_item);
                    } else {
                        self.marked_items.insert((**selected_item).clone());
                    }
                    self.cache.display_marked_dirty = true;
                    self.selectable_list.select_next();
                    self.sync_selected_item();
                }
            }
            InputEvent::Confirm => {
                self.pending_execution_items = match task.mode {
                    Mode::Multi => self.marked_items.iter().cloned().collect(),
                    Mode::None => {
                        if self.selected_item.is_empty() {
                            vec![]
                        } else {
                            vec![(*self.selected_item).clone()]
                        }
                    }
                };
                self.cache.pending_execution_items = self.pending_execution_items.join(", ");
                if task.execution_confirmation_message.is_some() {
                    self.modal_dialog_shown = true;
                } else {
                    self.execute(task);
                }
            }
            _ => {}
        }
        Intent::None
    }

    fn get_status(&mut self) -> &mut Status {
        let current_state = ExecutionStates {
            execution: self.execution_handle.read_state(),
            preview: self.preview_handle.read_state(),
        };
        if current_state != self.cache.execution_states {
            self.cache.status = resolve_status(&current_state);
            self.cache.execution_states = current_state;
        }
        &mut self.cache.status
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let display_items: Vec<&String> =
            self.search_results.iter().map(|rc| rc.as_ref()).collect();

        if self.cache.display_marked_dirty {
            self.cache.display_marked = self
                .search_results
                .iter()
                .enumerate()
                .filter_map(|(display_idx, result)| {
                    if self.marked_items.contains(&**result) {
                        Some(display_idx)
                    } else {
                        None
                    }
                })
                .collect();
            self.cache.display_marked_dirty = false;
        }

        let display_marked = &self.cache.display_marked;

        if self.show_preview {
            let preview = if !self.selected_item.is_empty()
                && let Some(cached) = self.cache.previews.get(&**self.selected_item)
            {
                cached.as_str()
            } else {
                PreviewStrings::LOADING
            };

            render_screen_scaffold(
                frame,
                area,
                &styles.screen_scaffold_style,
                |frame, left, right| -> () {
                    self.selectable_list.render(
                        frame,
                        left,
                        &display_items,
                        &styles.list,
                        &styles.colors,
                        Some(display_marked),
                    );
                    self.preview.render(
                        frame,
                        right,
                        preview,
                        self.selected_item.as_str(),
                        &styles.preview,
                        &styles.colors,
                    );
                },
            );
        } else {
            self.selectable_list.render(
                frame,
                area,
                &display_items,
                &styles.list,
                &styles.colors,
                Some(display_marked),
            );
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
                &self.cache.pending_execution_items,
                &styles.modal,
                &styles.colors,
            );
        }
    }

    fn on_search(&mut self, query: &str) {
        self.cache.search_query = query.to_string();
        self.selected_item = Rc::new(String::new());
        self.search();
    }

    fn consumed_event(&mut self, event: &InputEvent) -> bool {
        matches!(event, InputEvent::Back) && self.modal_dialog_shown
    }
}

fn resolve_status(state: &ExecutionStates) -> Status {
    match (&state.execution, &state.preview) {
        (State::Running, _) => Status::Running,
        (State::Error, _) => Status::Error,
        (State::Finished, State::None) => Status::Complete,
        (State::Finished, State::Running) => Status::Running,
        (State::Finished, State::Finished) => Status::Complete,
        (State::Finished, State::Error) => Status::Complete,
        (State::None, State::Running) => Status::Running,
        (State::None, _) => Status::Idle,
    }
}
