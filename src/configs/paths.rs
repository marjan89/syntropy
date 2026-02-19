use anyhow::{Context, Result, anyhow};
use std::env;
use std::path::PathBuf;

const SYNTROPY_CONFIG_NAME: &str = "syntropy.toml";
const SYNTROPY_APP_NAME: &str = "syntropy";
const PLUGINS_DIR_NAME: &str = "plugins";

/// Expands a path with tilde (~) and environment variable substitution
///
/// This function handles shell-style path expansion:
/// - `~` → user's home directory
/// - `~/path` → path relative to home directory
/// - `$VAR/path` → expands environment variable VAR
/// - `${VAR}/path` → expands environment variable VAR (brace syntax)
///
/// # Arguments
///
/// * `path` - The path to expand
///
/// # Returns
///
/// * `Ok(PathBuf)` - The expanded absolute path
/// * `Err` - If expansion fails (e.g., undefined environment variable, invalid home directory)
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use syntropy::configs::expand_path;
///
/// # fn main() -> anyhow::Result<()> {
/// let expanded = expand_path(PathBuf::from("~/config.toml"))?;
/// // Returns: /Users/username/config.toml
///
/// let expanded = expand_path(PathBuf::from("$HOME/.config/syntropy.toml"))?;
/// // Returns: /Users/username/.config/syntropy.toml
/// # Ok(())
/// # }
/// ```
pub fn expand_path(path: PathBuf) -> Result<PathBuf> {
    let path_str = path
        .to_str()
        .context("Path contains invalid UTF-8 characters")?;

    let expanded = shellexpand::full(path_str).context("Failed to expand path")?;

    Ok(PathBuf::from(expanded.as_ref()))
}

/// Returns the default config directory based on platform conventions
///
/// Respects XDG Base Directory Specification:
/// - Checks `$XDG_CONFIG_HOME` environment variable
/// - Falls back to `~/.config/syntropy` if:
///   - XDG_CONFIG_HOME is not set
///   - XDG_CONFIG_HOME is empty string
///   - XDG_CONFIG_HOME is relative path (must be absolute per XDG spec)
/// - Uses XDG-style paths on all platforms (Linux, macOS, Windows)
pub fn get_default_config_dir() -> Result<PathBuf> {
    // Check XDG_CONFIG_HOME environment variable first (Linux standard)
    if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        // XDG spec: empty string should be treated as unset
        if !xdg_config.is_empty() {
            let path = PathBuf::from(&xdg_config);
            // XDG spec: path must be absolute
            if path.is_absolute() {
                return Ok(path.join(SYNTROPY_APP_NAME));
            }
            // Relative path: fall through to default
        }
    }

    // Fallback to ~/.config/syntropy on all platforms (XDG-style)
    dirs::home_dir()
        .map(|dir| dir.join(".config").join(SYNTROPY_APP_NAME))
        .context("Failed to determine home directory")
}

/// Returns the default data directory based on platform conventions
///
/// Respects XDG Base Directory Specification:
/// - Checks `$XDG_DATA_HOME` environment variable
/// - Falls back to `~/.local/share/syntropy` if:
///   - XDG_DATA_HOME is not set
///   - XDG_DATA_HOME is empty string
///   - XDG_DATA_HOME is relative path (must be absolute per XDG spec)
/// - Uses XDG-style paths on all platforms (Linux, macOS, Windows)
pub fn get_default_data_dir() -> Result<PathBuf> {
    // Check XDG_DATA_HOME environment variable first (Linux standard)
    if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
        // XDG spec: empty string should be treated as unset
        if !xdg_data.is_empty() {
            let path = PathBuf::from(&xdg_data);
            // XDG spec: path must be absolute
            if path.is_absolute() {
                return Ok(path.join(SYNTROPY_APP_NAME));
            }
            // Relative path: fall through to default
        }
    }

    // Fallback to ~/.local/share/syntropy on all platforms (XDG-style)
    dirs::home_dir()
        .map(|dir| dir.join(".local").join("share").join(SYNTROPY_APP_NAME))
        .context("Failed to determine home directory")
}

/// Finds the config file using the following search order:
///
/// 1. CLI argument path (if provided) - returns error if specified but doesn't exist
/// 2. XDG config directory: `~/.config/syntropy/syntropy.toml`
/// 3. Current directory: `./syntropy.toml`
///
/// Returns `Ok(Some(path))` if config found, `Ok(None)` if no config found via auto-discovery,
/// or `Err` if CLI path was explicitly specified but doesn't exist.
pub fn find_config_file(cli_path: Option<PathBuf>) -> Result<Option<PathBuf>> {
    // Priority 1: CLI argument - error if explicitly specified but missing
    if let Some(path) = cli_path {
        if path.exists() {
            return Ok(Some(path));
        } else {
            return Err(anyhow!("Specified config file does not exist: {:?}", path));
        }
    }

    // Priority 2: XDG config directory
    let xdg_config_path = get_default_config_dir()?.join(SYNTROPY_CONFIG_NAME);
    if xdg_config_path.exists() {
        return Ok(Some(xdg_config_path));
    }

    // Priority 3: Current directory
    let local_config_path = PathBuf::from(".").join(SYNTROPY_CONFIG_NAME);
    if local_config_path.exists() {
        return Ok(Some(local_config_path));
    }

    // No config found via auto-discovery - not an error
    Ok(None)
}

/// Resolves plugin directory paths using XDG Base Directory specification
///
/// # Behavior
///
/// Returns both default plugin directories:
/// - `~/.config/syntropy/plugins/` (user-created plugins)
/// - `~/.local/share/syntropy/plugins/` (managed plugins installed via `syntropy plugins --install`)
///
/// Plugins with the same name are merged, with config directory taking precedence.
pub fn resolve_plugin_paths() -> Result<Vec<PathBuf>> {
    let config_plugins = get_default_config_dir()?.join(PLUGINS_DIR_NAME);
    let data_plugins = get_default_data_dir()?.join(PLUGINS_DIR_NAME);
    Ok(vec![config_plugins, data_plugins])
}
