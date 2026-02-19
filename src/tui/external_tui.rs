use anyhow::Result;
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::sync::oneshot;

use crate::execution::clamp_exit_code;

/// Request to run an external TUI application with full terminal control
#[derive(Debug)]
pub struct ExternalTuiRequest {
    pub command: String,
    pub args: Vec<String>,
    pub response: oneshot::Sender<i32>,
}

pub type TuiRequestSender = tokio::sync::mpsc::UnboundedSender<ExternalTuiRequest>;
pub type TuiRequestReceiver = tokio::sync::mpsc::UnboundedReceiver<ExternalTuiRequest>;

// Global TUI request channel sender - initialized by TUI, used by Lua
static TUI_SENDER: OnceLock<TuiRequestSender> = OnceLock::new();

pub fn create_tui_channel() -> (TuiRequestSender, TuiRequestReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}

pub fn set_tui_sender(sender: TuiRequestSender) -> Result<()> {
    TUI_SENDER
        .set(sender)
        .map_err(|_| anyhow::anyhow!("TUI sender already initialized"))
}

pub fn get_tui_sender() -> Option<&'static TuiRequestSender> {
    TUI_SENDER.get()
}

/// Runs an external TUI command with full terminal control (blocking)
/// Returns the exit code from the command (clamped to POSIX range 0-255)
pub fn run_tui_command_blocking(command: &str, args: &[String]) -> Result<i32> {
    let status = std::process::Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(clamp_exit_code(status.code().unwrap_or(-1)))
}
