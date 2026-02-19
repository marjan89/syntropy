pub mod exit_code;
mod handle;
mod lua;
pub mod runner;

use std::sync::Arc;

pub use exit_code::clamp_exit_code;
pub use handle::{ExecutionResult, Handle, Operation, State};
pub(crate) use lua::{
    call_item_source_execute, call_item_source_preselected_items, call_item_source_preview,
    call_task_post_run, call_task_pre_run, call_task_preview, has_item_source_execute,
};
pub use lua::{call_item_source_items, call_task_execute};
use mlua::Lua;
pub use runner::{run_execute_pipeline, run_items_pipeline, run_preview_pipeline};

type SharedLua = Arc<tokio::sync::Mutex<Lua>>;
type RuntimeHandle = tokio::runtime::Handle;
