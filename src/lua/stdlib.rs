use mlua::{Error as LuaError, Lua, Result as LuaResult, Table as LuaTable};
use std::{env, process::Stdio};
use tokio::{io::AsyncBufReadExt, join};

use crate::execution::clamp_exit_code;
use crate::tui::{ExternalTuiRequest, get_tui_sender};

pub fn register_syntropy_stdlib(lua: &Lua) -> LuaResult<()> {
    let syntropy_table = lua.create_table()?;

    let shell_fn = lua.create_async_function(|_, cmd: String| async move {
        let (output, exit_code) = execute_shell_async(&cmd)
            .await
            .map_err(LuaError::external)?;

        Ok((output, exit_code))
    })?;

    syntropy_table.set("shell", shell_fn)?;

    // invoke_tui: Run any external TUI application with full terminal control
    let invoke_tui_fn =
        lua.create_async_function(|_, (command, args_table): (String, LuaTable)| async move {
            let exit_code = invoke_tui(command, args_table)
                .await
                .map_err(LuaError::external)?;

            Ok(exit_code)
        })?;

    syntropy_table.set("invoke_tui", invoke_tui_fn)?;

    // invoke_editor: Convenience wrapper for $EDITOR
    let invoke_editor_fn = lua.create_async_function(|_, path: String| async move {
        let exit_code = invoke_editor(path).await.map_err(LuaError::external)?;

        Ok(exit_code)
    })?;

    syntropy_table.set("invoke_editor", invoke_editor_fn)?;

    let expand_path_fn = lua.create_function(|lua_ctx, path: String| {
        // Handle ./ and ../ as plugin-relative paths
        if path.starts_with("./") || path.starts_with("../") {
            // Get current plugin name from registry
            let plugin_name: String = lua_ctx
                .named_registry_value("__syntropy_current_plugin__")
                .map_err(|_| {
                    LuaError::external(
                        "Cannot resolve relative path: no plugin context (expand_path called outside plugin execution)"
                    )
                })?;

            // Get plugin table from globals
            let plugin_table: mlua::Table = lua_ctx
                .globals()
                .get(plugin_name.as_str())
                .map_err(|e| {
                    LuaError::external(format!("Failed to get plugin '{}': {}", plugin_name, e))
                })?;

            // Get plugin directory from plugin table
            let plugin_dir: String = plugin_table
                .get("__plugin_dir")
                .map_err(|_| {
                    LuaError::external(format!(
                        "Plugin '{}' missing __plugin_dir (this is a syntropy bug)",
                        plugin_name
                    ))
                })?;

            // Join relative path with plugin directory
            let resolved = std::path::Path::new(&plugin_dir).join(&path);

            // Convert to string
            let resolved_str = resolved
                .to_str()
                .ok_or_else(|| LuaError::external("Resolved path contains invalid UTF-8"))?;

            return Ok(resolved_str.to_string());
        }

        // Handle tilde and environment variable expansion
        let expanded = expand_tilde(&path).map_err(LuaError::external)?;
        Ok(expanded)
    })?;

    syntropy_table.set("expand_path", expand_path_fn)?;
    lua.globals().set("syntropy", syntropy_table)?;
    Ok(())
}

pub async fn invoke_tui(command: String, args_table: LuaTable) -> Result<i32, String> {
    // Convert Lua table to Vec<String>
    let args: Vec<String> = args_table
        .sequence_values()
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| format!("Failed to parse args table: {}", e))?;

    // Check if we're in TUI mode or CLI mode
    if let Some(sender) = get_tui_sender() {
        // TUI mode: send request to main thread and wait for response
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let request = ExternalTuiRequest {
            command: command.clone(),
            args,
            response: response_tx,
        };

        sender
            .send(request)
            .map_err(|_| "Failed to send TUI request to main loop".to_string())?;

        // Wait for TUI to complete the command invocation
        let exit_code = response_rx
            .await
            .map_err(|_| "Failed to receive TUI response from main loop".to_string())?;

        Ok(exit_code)
    } else {
        // CLI mode: run command directly (blocking)
        let status = tokio::process::Command::new(&command)
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .map_err(|e| format!("Failed to spawn command '{}': {}", command, e))?;

        Ok(clamp_exit_code(status.code().unwrap_or(-1)))
    }
}

pub async fn invoke_editor(path: String) -> Result<i32, String> {
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vim".to_string());

    // Check if we're in TUI mode or CLI mode
    if let Some(sender) = get_tui_sender() {
        // TUI mode: send request to main thread and wait for response
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let request = ExternalTuiRequest {
            command: editor.clone(),
            args: vec![path.clone()],
            response: response_tx,
        };

        sender
            .send(request)
            .map_err(|_| "Failed to send editor request to TUI".to_string())?;

        // Wait for TUI to complete the editor invocation
        let exit_code = response_rx
            .await
            .map_err(|_| "Failed to receive editor response from TUI".to_string())?;

        Ok(exit_code)
    } else {
        // CLI mode: run editor directly (blocking)
        let status = tokio::process::Command::new(&editor)
            .arg(&path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .map_err(|e| format!("Failed to spawn editor '{}': {}", editor, e))?;

        Ok(clamp_exit_code(status.code().unwrap_or(-1)))
    }
}

/// Executes a shell command asynchronously using tokio.
/// Uses `sh -c` to support complex shell syntax (pipes, redirects, etc.).
/// Returns (exit_code, output_lines) on success.
pub async fn execute_shell_async(command: &str) -> Result<(String, i32), String> {
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let mut stdout_lines = tokio::io::BufReader::new(stdout).lines();
    let mut stderr_lines = tokio::io::BufReader::new(stderr).lines();

    let (stdout_result, stderr_result) = join!(
        async {
            let mut lines = Vec::new();
            while let Some(line) = stdout_lines.next_line().await.transpose() {
                lines.push(line?);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        },
        async {
            let mut lines = Vec::new();
            while let Some(line) = stderr_lines.next_line().await.transpose() {
                lines.push(line?);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        }
    );

    let mut output = stdout_result.map_err(|e| format!("Failed to read stdout: {}", e))?;
    output.extend(stderr_result.map_err(|e| format!("Failed to read stderr: {}", e))?);

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for command: {}", e))?;

    let exit_code = clamp_exit_code(status.code().unwrap_or(-1));

    Ok((output.join("\n"), exit_code))
}

fn expand_tilde(path: &str) -> Result<String, String> {
    shellexpand::full(path)
        .map(|expanded| expanded.to_string())
        .map_err(|e| format!("Failed to expand path: {}", e))
}
