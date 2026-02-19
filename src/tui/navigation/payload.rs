#[derive(Debug, Clone, PartialEq)]
pub struct PluginPayload;

#[derive(Debug, Clone, PartialEq)]
pub struct TaskPayload {
    pub plugin_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemPayload {
    pub plugin_idx: usize,
    pub task_key: String,
}
