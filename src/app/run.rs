use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser};
use std::{path::PathBuf, process::exit, sync::Arc};
use tokio::{runtime::Builder, sync::Mutex};

use crate::{
    Config,
    app::App,
    cli::handle_plugins_command,
    cli::{
        Args, Commands,
        completions::generate_completions,
        execute::execute_task_cli,
        init::create_plugin_scaffold,
        validate::{validate_config_cli, validate_plugin_cli},
    },
    configs::{
        expand_path, find_config_file, get_default_config_dir, load_config, resolve_plugin_paths,
        validate_config,
    },
    lua::create_lua_vm,
    plugins::load_plugins,
    tui::TuiApp,
};

/// Main entry point for the Syntropy application.
///
/// This function handles the complete application lifecycle from command-line
/// argument parsing through validation and execution. It implements a multi-stage
/// validation pipeline before initializing the application environment:
///
/// 1. **Parse CLI arguments**: Uses clap to parse and validate command-line arguments
/// 2. **Handle subcommands**: Processes `init`, `completions`, `validate`, and `plugins` commands (exits early if present)
/// 3. **Setup and run**: Initializes application environment and runs TUI or `execute` subcommand
///
/// # Execution Flow
///
/// The function uses an early-exit pattern where some subcommands (init, completions, validate)
/// complete and exit before the main application runs. Other operations (TUI mode, execute subcommand)
/// require full environment setup and are handled in `setup_the_environment_and_run`.
///
/// # Returns
///
/// Returns `Ok(())` if:
/// - A subcommand executes successfully and exits
/// - The application runs and exits normally (TUI mode or execute mode with exit code 0)
///
/// # Errors
///
/// Returns an error if any stage fails:
/// - CLI argument parsing fails (invalid arguments, missing required values)
/// - Subcommand execution fails (file I/O errors during `init`, validation failures, etc.)
/// - Application setup or execution fails (see `setup_the_environment_and_run` for details)
///
/// # Notes
///
/// - In CLI execution mode (`execute` subcommand), non-zero exit codes cause `exit()` to be called
///   within `setup_the_environment_and_run`, which does not return normally
/// - Some subcommand operations complete without loading plugins or initializing
///   the full application environment for performance
pub fn run() -> Result<()> {
    let cli_args = Args::parse();

    if handle_cli_commands(&cli_args.command, &cli_args)? {
        return Ok(());
    }

    setup_the_environment_and_run(&cli_args)?;

    Ok(())
}

// Loads config, resolves plugin paths, initializes Lua runtime and plugins, then
// dispatches to either CLI execution mode (execute subcommand) or interactive TUI mode.
// In CLI mode with non-zero exit code, calls exit() and does not return.
fn setup_the_environment_and_run(cli_args: &Args) -> Result<()> {
    let (config, _config_path) = handle_config(cli_args)?;

    let plugin_paths = resolve_plugin_paths().context("Failed to resolve plugin paths")?;

    let lua_runtime = Arc::new(Mutex::new(create_lua_vm()?));

    let plugins = load_plugins(&plugin_paths, &config, Arc::clone(&lua_runtime))
        .context("Failed to load plugins")?;

    let app = App::new(config, plugins, lua_runtime);
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;

    if let Some(Commands::Execute(execute_args)) = &cli_args.command {
        let exit_code = runtime.block_on(execute_task_cli(app, execute_args))?;
        if exit_code != 0 {
            exit(exit_code);
        }
    } else {
        let mut tui_app = TuiApp::new(app, runtime.handle().clone())
            .context("Failed to initialize TUI application")?;
        tui_app.run()?;
    }

    Ok(())
}

// Loads and validates the config file using XDG-compliant path resolution.
// Applies CLI overrides: --plugin sets default_plugin, --task sets default_task,
// and boolean flags override their respective config values.
// Returns error if --task is specified without --plugin.
fn handle_config(cli_args: &Args) -> Result<(Config, PathBuf)> {
    let expanded_config = cli_args
        .config
        .clone()
        .map(expand_path)
        .transpose()
        .context("Failed to expand config path")?;

    let config_path_opt =
        find_config_file(expanded_config).context("Failed to find config file")?;

    let (mut config, config_path) = match config_path_opt {
        Some(path) => {
            let config = load_config(path.clone()).context("Failed to load config file")?;
            (config, path)
        }
        None => {
            // No config file found - use defaults (expected for new users)
            (Config::default(), PathBuf::from("<no config file>"))
        }
    };

    if let Some(ref plugin_name) = cli_args.plugin
        && !plugin_name.trim().is_empty()
    {
        config.default_plugin = Some(plugin_name.trim().to_string());
        if let Some(ref task_name) = cli_args.task
            && !task_name.trim().is_empty()
        {
            config.default_task = Some(task_name.trim().to_string());
        } else {
            config.default_task = None;
        }
    } else if let Some(ref task_name) = cli_args.task
        && !task_name.trim().is_empty()
    {
        bail!("--task requires --plugin to be specified");
    }

    // Apply CLI overrides for boolean config fields
    if let Some(status_bar) = cli_args.status_bar {
        config.status_bar = status_bar;
    }
    if let Some(search_bar) = cli_args.search_bar {
        config.search_bar = search_bar;
    }
    if let Some(show_preview_pane) = cli_args.show_preview_pane {
        config.show_preview_pane = show_preview_pane;
    }
    if let Some(exit_on_execute) = cli_args.exit_on_execute {
        config.exit_on_execute = exit_on_execute;
    }

    validate_config(&config)?;

    Ok((config, config_path))
}

// Handles subcommands that exit immediately without launching TUI.
// Returns Ok(false) if no subcommand or if subcommand needs environment (Execute)
// Returns Ok(true) if subcommand was handled and app should exit
fn handle_cli_commands(command: &Option<Commands>, cli_args: &Args) -> Result<bool> {
    let Some(command) = &command else {
        return Ok(false);
    };
    match command {
        Commands::Execute(_) => {
            // Execute requires full environment setup, handle in setup_the_environment_and_run
            Ok(false)
        }
        Commands::Init => {
            create_plugin_scaffold()?;
            Ok(true)
        }
        Commands::Completions { shell } => {
            generate_completions(*shell, &mut Args::command());
            Ok(true)
        }
        Commands::Validate { plugin, config } => {
            if let Some(plugin_path) = plugin {
                validate_plugin_cli(plugin_path.clone())?;
            } else if let Some(config_paths) = config {
                let config_path = if config_paths.is_empty() {
                    match find_config_file(cli_args.config.clone())? {
                        Some(path) => path,
                        None => {
                            let xdg_path = get_default_config_dir()?.join("syntropy.toml");
                            bail!(
                                "No config file found to validate. Searched:\n  - {:?}\n  - ./syntropy.toml",
                                xdg_path
                            );
                        }
                    }
                } else {
                    config_paths[0].clone()
                };
                validate_config_cli(config_path)?;
            } else {
                bail!("validate command requires either --plugin or --config flag");
            }
            Ok(true)
        }
        Commands::Plugins(plugin_params) => {
            let (config, _config_path) = handle_config(cli_args)?;
            handle_plugins_command(plugin_params, config)?;
            Ok(true)
        }
    }
}
