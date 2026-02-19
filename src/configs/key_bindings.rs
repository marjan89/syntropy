use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyBindings {
    pub back: String,
    pub select_previous: String,
    pub select_next: String,
    pub scroll_preview_up: String,
    pub scroll_preview_down: String,
    pub toggle_preview: String,
    pub select: String,
    pub confirm: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            back: "<esc>".to_string(),
            select_previous: "<up>".to_string(),
            select_next: "<down>".to_string(),
            scroll_preview_up: "<C-up>".to_string(),
            scroll_preview_down: "<C-down>".to_string(),
            toggle_preview: "<C-p>".to_string(),
            select: "<tab>".to_string(),
            confirm: "<enter>".to_string(),
        }
    }
}
