#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    SelectPlugin { plugin_idx: usize },
    SelectTask { plugin_idx: usize, task_key: String },
    Quit,
    None,
}
