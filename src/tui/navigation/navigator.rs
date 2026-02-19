use crate::tui::navigation::Intent;
use crate::tui::navigation::ItemPayload;
use crate::tui::navigation::Route;
use crate::tui::navigation::TaskPayload;

#[derive(Debug, PartialEq)]
pub struct StackEntry {
    pub route: Route,
    pub name: String,
}

impl StackEntry {
    pub fn new(route: Route, name: String) -> Self {
        Self { route, name }
    }
}

pub struct Navigator {
    stack: Vec<StackEntry>,
    breadcrumbs: String,
    breadcrumb_separator: String,
}

impl Navigator {
    pub fn new(route: Route, name: String, breadcrumb_separator: String) -> Self {
        let mut navigator = Self {
            stack: vec![StackEntry::new(route, name)],
            breadcrumbs: String::new(),
            breadcrumb_separator,
        };
        navigator.update_breadcrumbs();
        navigator
    }

    pub fn current(&self) -> &Route {
        &self
            .stack
            .last()
            .expect("Navigator stack should never be empty")
            .route
    }

    pub fn push(&mut self, route: Route, name: String) {
        self.stack.push(StackEntry::new(route, name));
        self.update_breadcrumbs();
    }

    pub fn pop(&mut self) -> Option<StackEntry> {
        if self.stack.len() > 1 {
            let popped = self.stack.pop();
            self.update_breadcrumbs();
            popped
        } else {
            None
        }
    }

    pub fn resolve_intent(&mut self, event: Intent) -> Option<Route> {
        match event {
            Intent::SelectPlugin { plugin_idx } => Some(Route::Task {
                payload: TaskPayload { plugin_idx },
            }),
            Intent::SelectTask {
                plugin_idx,
                task_key,
            } => Some(Route::Item {
                payload: ItemPayload {
                    plugin_idx,
                    task_key,
                },
            }),
            Intent::Quit | Intent::None => None,
        }
    }

    pub fn update_breadcrumbs(&mut self) {
        self.breadcrumbs = self
            .stack
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(self.breadcrumb_separator.as_str())
    }

    pub fn get_breadcrumbs(&self) -> &String {
        &self.breadcrumbs
    }
}
