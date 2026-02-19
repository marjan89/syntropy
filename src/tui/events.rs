use crossterm::event::KeyEvent;

use crate::tui::key_bindings::ParsedKeyBindings;

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Back,
    PreviousItem,
    NextItem,
    ScrollPreviewUp,
    ScrollPreviewDown,
    TogglePreview,
    Confirm,
    Select,
}

pub fn handle_key(key: &KeyEvent, bindings: &ParsedKeyBindings) -> Option<InputEvent> {
    match () {
        _ if bindings.back.matches(key) => Some(InputEvent::Back),
        _ if bindings.select_previous.matches(key) => Some(InputEvent::PreviousItem),
        _ if bindings.select_next.matches(key) => Some(InputEvent::NextItem),
        _ if bindings.scroll_preview_up.matches(key) => Some(InputEvent::ScrollPreviewUp),
        _ if bindings.scroll_preview_down.matches(key) => Some(InputEvent::ScrollPreviewDown),
        _ if bindings.toggle_preview.matches(key) => Some(InputEvent::TogglePreview),
        _ if bindings.confirm.matches(key) => Some(InputEvent::Confirm),
        _ if bindings.select.matches(key) => Some(InputEvent::Select),
        _ => None,
    }
}
