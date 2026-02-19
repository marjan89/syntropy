use ratatui::{Frame, layout::Rect};

use crate::{
    app::App,
    tui::{
        events::InputEvent,
        navigation::{Intent, Route},
        screens::{ItemListScreen, PluginListScreen, Screen, Status, TaskListScreen},
        views::Styles,
    },
};

pub struct ScreenDispatcher {
    pub plugin_screen: PluginListScreen,
    pub task_screen: TaskListScreen,
    pub item_screen: ItemListScreen,
}

impl ScreenDispatcher {
    pub fn on_enter(&mut self, route: &Route, app: &App) {
        match route {
            Route::Plugin { payload } => self.plugin_screen.on_enter(app, payload),
            Route::Task { payload } => self.task_screen.on_enter(app, payload),
            Route::Item { payload } => self.item_screen.on_enter(app, payload),
        }
    }

    pub fn on_exit(&mut self, route: &Route) {
        match route {
            Route::Plugin { .. } => self.plugin_screen.on_exit(),
            Route::Task { .. } => self.task_screen.on_exit(),
            Route::Item { .. } => self.item_screen.on_exit(),
        }
    }

    pub fn handle_event(&mut self, route: &Route, event: InputEvent, app: &App) -> Intent {
        match route {
            Route::Plugin { payload } => self.plugin_screen.handle_event(event, app, payload),
            Route::Task { payload } => self.task_screen.handle_event(event, app, payload),
            Route::Item { payload } => self.item_screen.handle_event(event, app, payload),
        }
    }

    pub fn render(&mut self, route: &Route, rect: Rect, frame: &mut Frame<'_>, styles: &Styles) {
        match route {
            Route::Plugin { .. } => self.plugin_screen.render(frame, rect, styles),
            Route::Task { .. } => self.task_screen.render(frame, rect, styles),
            Route::Item { .. } => self.item_screen.render(frame, rect, styles),
        }
    }

    pub fn update(&mut self, route: &Route, app: &App) -> Intent {
        match route {
            Route::Plugin { payload } => self.plugin_screen.on_update(app, payload),
            Route::Task { payload } => self.task_screen.on_update(app, payload),
            Route::Item { payload } => self.item_screen.on_update(app, payload),
        }
    }

    pub fn get_status(&mut self, route: &Route) -> &mut Status {
        match route {
            Route::Plugin { .. } => self.plugin_screen.get_status(),
            Route::Task { .. } => self.task_screen.get_status(),
            Route::Item { .. } => self.item_screen.get_status(),
        }
    }

    pub fn on_search(&mut self, route: &Route, query: &str) {
        match route {
            Route::Plugin { .. } => self.plugin_screen.on_search(query),
            Route::Task { .. } => self.task_screen.on_search(query),
            Route::Item { .. } => self.item_screen.on_search(query),
        }
    }

    pub fn consumed_event(&mut self, route: &Route, event: &InputEvent) -> bool {
        match route {
            Route::Plugin { .. } => self.plugin_screen.consumed_event(event),
            Route::Task { .. } => self.task_screen.consumed_event(event),
            Route::Item { .. } => self.item_screen.consumed_event(event),
        }
    }
}
