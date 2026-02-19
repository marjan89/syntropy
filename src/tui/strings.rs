pub struct StatusStrings;

impl StatusStrings {
    pub const IDLE: &str = "";
    pub const ERROR: &str = "Error";
    pub const RUNNING: &str = "Running";
    pub const COMPLETE: &str = "Complete";
}

pub struct RouteStrings;

impl RouteStrings {
    pub const PLUGIN: &str = "Plugin";
    pub const TASK: &str = "Task";
    pub const ITEM: &str = "Item";
}

pub struct PreviewStrings;

impl PreviewStrings {
    pub const LOADING: &str = "Loading preview...";
    pub const PLUGIN: &str = "Plugin";
    pub const VERSION: &str = "Version";
    pub const PLATFORMS: &str = "Platforms";
    pub const DESCRIPTION: &str = "Description";
    pub const TASKS: &str = "Tasks";
}

pub struct ModalStrings;

impl ModalStrings {
    pub const TITLE_MODAL_RESULT: &str = "Task result";
    pub const TITLE_MODAL_DIALOG_CONFIRM: &str = "Confirm execution";
    pub const LABEL_BUTTON_CONFIRM: &str = "Confirm";
    pub const LABEL_BUTTON_DISMISS: &str = "Dismiss";
    pub const LABEL_BUTTON_CANCEL: &str = "Cancel";
}
