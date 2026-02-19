use std::{
    mem::replace,
    sync::{Arc, Mutex},
};

use anyhow::{Result, ensure};
use tokio::task::JoinHandle;

use crate::{
    execution::{
        RuntimeHandle, SharedLua,
        runner::{run_execute_pipeline, run_items_pipeline, run_preview_pipeline},
    },
    plugins::Task,
};

pub enum Operation {
    Items {
        task: Arc<Task>,
    },
    Preview {
        task: Arc<Task>,
        current_item: String,
    },
    Execute {
        task: Arc<Task>,
        selected_items: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum State {
    #[default]
    None,
    Running,
    Finished,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
    Items {
        items: Vec<String>,
        preselected_items: Vec<String>,
    },
    Preview(String),
    Output(String, i32),
    Error(String),
    None,
}

pub struct Handle {
    state: Arc<Mutex<State>>,
    result: Arc<Mutex<ExecutionResult>>,
    thread_handle: Option<JoinHandle<()>>,
    runtime_handle: RuntimeHandle,
    lua_runtime: SharedLua,
}

impl Handle {
    pub fn new(runtime_handle: RuntimeHandle, lua_runtime: &SharedLua) -> Self {
        Handle {
            state: Arc::new(Mutex::new(State::None)),
            result: Arc::new(Mutex::new(ExecutionResult::None)),
            thread_handle: None,
            runtime_handle,
            lua_runtime: Arc::clone(lua_runtime),
        }
    }

    async fn dispatch_task(operation: Operation, lua_runtime: SharedLua) -> ExecutionResult {
        match &operation {
            Operation::Items { task } => {
                let items = run_items_pipeline(lua_runtime, task).await;
                match items {
                    Ok((items, preselected_items)) => ExecutionResult::Items {
                        items,
                        preselected_items,
                    },
                    Err(output) => ExecutionResult::Error(format!("{:#}", output)),
                }
            }
            Operation::Preview { task, current_item } => {
                let output = run_preview_pipeline(lua_runtime, task, current_item).await;
                match output {
                    Ok(output) => ExecutionResult::Preview(output),
                    Err(output) => ExecutionResult::Error(format!("{:#}", output)),
                }
            }
            Operation::Execute {
                task,
                selected_items,
            } => {
                let output = run_execute_pipeline(lua_runtime, task, selected_items).await;
                match output {
                    Ok((output, exit_code)) => ExecutionResult::Output(output, exit_code),
                    Err(output) => ExecutionResult::Error(format!("{:#}", output)),
                }
            }
        }
    }
}

impl Handle {
    pub fn execute(&mut self, operation: Operation) -> Result<()> {
        ensure!(!self.is_executing(), "Execution in progress");

        // Join any previous thread before starting a new one
        if let Some(handle) = self.thread_handle.take() {
            handle.abort();
        }

        let state_clone = Arc::clone(&self.state);

        if let Ok(mut state_guard) = state_clone.lock() {
            *state_guard = State::Running;
        }

        let result_clone = Arc::clone(&self.result);
        let lua_runtime = Arc::clone(&self.lua_runtime);

        let handle = self.runtime_handle.spawn(async move {
            let result = Self::dispatch_task(operation, lua_runtime).await;

            let state_lock = state_clone.lock();
            let result_lock = result_clone.lock();

            if let (Ok(mut state_guard), Ok(mut result_guard)) = (state_lock, result_lock) {
                *result_guard = result;
                *state_guard = State::Finished;
            }
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    #[must_use = "State should be checked to determine execution status"]
    pub fn read_state(&self) -> State {
        match self.state.lock() {
            Ok(state) => state.clone(),
            Err(_) => State::Error,
        }
    }

    #[must_use = "Result should be consumed and handled"]
    pub fn consume_result(&mut self) -> ExecutionResult {
        if let Ok(mut state_guard) = self.state.lock()
            && let Ok(mut result_guard) = self.result.lock()
        {
            match *state_guard {
                State::Finished => {
                    let result = replace(&mut *result_guard, ExecutionResult::None);
                    *state_guard = State::None;
                    result
                }
                _ => ExecutionResult::None,
            }
        } else {
            ExecutionResult::None
        }
    }

    pub fn is_executing(&self) -> bool {
        self.state
            .lock()
            .map(|state| matches!(*state, State::Running))
            .unwrap_or(false)
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            handle.abort();
        }
    }
}
