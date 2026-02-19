use serde::{Deserialize, Serialize};

pub const DEFAULT_COLOR: &str = "terminal";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Colors {
    pub highlights_background: String,
    pub highlights_text: String,
    pub borders: String,
    pub borders_list: String,
    pub borders_preview: String,
    pub borders_search: String,
    pub borders_status: String,
    pub borders_modal: String,
    pub text: String,
    pub text_list: String,
    pub text_preview: String,
    pub text_search: String,
    pub text_status: String,
    pub text_modal: String,
    pub background: String,
    pub background_list: String,
    pub background_preview: String,
    pub background_search: String,
    pub background_status: String,
    pub background_modal: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            highlights_text: DEFAULT_COLOR.to_string(),
            highlights_background: DEFAULT_COLOR.to_string(),
            borders: DEFAULT_COLOR.to_string(),
            text: DEFAULT_COLOR.to_string(),
            background: DEFAULT_COLOR.to_string(),
            borders_list: String::new(),
            borders_preview: String::new(),
            borders_search: String::new(),
            borders_status: String::new(),
            borders_modal: String::new(),
            text_list: String::new(),
            text_preview: String::new(),
            text_search: String::new(),
            text_status: String::new(),
            text_modal: String::new(),
            background_list: String::new(),
            background_preview: String::new(),
            background_search: String::new(),
            background_status: String::new(),
            background_modal: String::new(),
        }
    }
}
