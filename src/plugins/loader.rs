use anyhow::{Context, Result, bail, ensure};
use indexmap::{IndexMap, IndexSet};
use mlua::{Lua, Table, Value};
use semver::Version;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    configs::Config,
    lua::MERGE_LUA_FN_KEY,
    plugins::{
        ItemSource, Metadata, Mode, ModulePathBuilder, Plugin, PluginSource, Task, TaskMap,
        plugin_candidate::PluginCandidate,
    },
};
use tokio::sync::Mutex;
use unicode_width::UnicodeWidthStr;

pub fn load_plugins(
    plugin_paths: &[PathBuf],
    config: &Config,
    lua_runtime: Arc<Mutex<Lua>>,
) -> Result<Vec<Plugin>> {
    let lua_runtime = lua_runtime.blocking_lock();

    // Configure package.path ONCE for ALL plugins before any evaluation
    // This ensures require() works during peek() and subsequent evaluations
    let mut path_builder = ModulePathBuilder::default();

    // STEP 1: Add all plugin lua/ directories
    for plugin_dir_path in plugin_paths {
        if !plugin_dir_path.exists() {
            continue;
        }

        let plugin_dir = fs::read_dir(plugin_dir_path).with_context(|| {
            format!(
                "Failed to read plugins directory at: {}",
                plugin_dir_path.display()
            )
        })?;

        let mut entries: Vec<_> = plugin_dir
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read directory entries")?;
        entries.sort_by_key(|entry| entry.path());

        for dir in entries {
            let path = dir.path();
            if !path.is_dir() {
                continue;
            }

            if !path.join("plugin.lua").exists() {
                continue;
            }

            let plugin_dir_str = path
                .to_str()
                .context("Plugin directory path contains invalid UTF-8")?;

            path_builder = path_builder.with_plugin_dir(plugin_dir_str);
        }
    }

    // STEP 2: Add shared/ directories from each plugin root
    // Extract unique parent directories and check for shared/ subdirectories
    // Using IndexSet to preserve insertion order (config before data)
    let mut shared_dirs = IndexSet::new();
    for plugin_dir_path in plugin_paths {
        if !plugin_dir_path.exists() {
            continue;
        }

        // Check if this plugin directory has a shared/ subdirectory
        let shared_path = plugin_dir_path.join("shared");
        if shared_path.exists()
            && shared_path.is_dir()
            && let Some(shared_str) = shared_path.to_str()
        {
            shared_dirs.insert(shared_str.to_string());
        }
    }

    // Add all found shared directories to the module path
    for shared_dir in shared_dirs {
        // Strip "/shared" suffix to get the root directory
        // ModulePathBuilder::with_shared_modules expects the root and adds "/plugins/shared"
        // But our shared_dir is already "<root>/plugins/shared"
        // So we need to extract just the root part
        if let Some(plugins_idx) = shared_dir.rfind("/plugins/shared") {
            let root = &shared_dir[..plugins_idx];
            path_builder = path_builder.with_shared_modules(root);
        }
    }

    path_builder
        .apply(&lua_runtime)
        .context("Failed to configure Lua module paths")?;

    // PASS 1: Collect all plugin candidates by name
    // This allows us to detect when the same plugin exists in multiple directories
    // Use IndexMap to preserve directory order (config dir before data dir)
    let mut plugin_map: IndexMap<String, Vec<PluginCandidate>> = IndexMap::new();

    for plugin_dir_path in plugin_paths {
        // Skip if directory doesn't exist (allows optional directories)
        if !plugin_dir_path.exists() {
            continue;
        }

        let plugin_dir = fs::read_dir(plugin_dir_path).with_context(|| {
            format!(
                "Failed to read plugins directory at: {}",
                plugin_dir_path.display()
            )
        })?;

        // Collect and sort directory entries for deterministic ordering across platforms
        let mut entries: Vec<_> = plugin_dir
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read directory entries")?;
        entries.sort_by_key(|entry| entry.path());

        for dir in entries {
            let path = dir.path();
            if !path.is_dir() {
                continue;
            }
            let lua_plugin_path = path.join("plugin.lua");

            if !lua_plugin_path.exists() {
                continue;
            }

            // Create candidate by peeking (caches name)
            let candidate = PluginCandidate::peek(&lua_runtime, lua_plugin_path)
                .with_context(|| format!("Failed to peek plugin at {:?}", path))?;

            plugin_map
                .entry(candidate.name.clone())
                .or_default()
                .push(candidate);
        }
    }

    // PASS 2: Load plugins (with merging if multiple sources exist)
    let mut plugins: Vec<Plugin> = Vec::new();

    for (plugin_name, candidates) in plugin_map {
        let paths: Vec<PathBuf> = candidates.iter().map(|c| c.path.clone()).collect();
        let source = PluginSource::from_paths(paths)?;

        let plugin = if source.needs_merge() {
            // Evaluate cached contents from candidates
            let tables: Vec<Table> = candidates
                .iter()
                .map(|c| c.evaluate(&lua_runtime))
                .collect::<Result<Vec<_>>>()?;

            load_and_merge_plugin(
                &lua_runtime,
                &source,
                &plugin_name,
                &config.default_plugin_icon,
                tables,
            )
            .with_context(|| format!("Failed to merge plugin '{}'", plugin_name))?
        } else {
            // Single source - load normally (existing behavior)
            let path = match &source {
                PluginSource::Single(p) => p,
                _ => unreachable!(),
            };

            let cached_table = candidates[0].evaluate(&lua_runtime)?;

            load_plugin(
                &lua_runtime,
                path,
                &config.default_plugin_icon,
                Some(cached_table),
            )
            .with_context(|| format!("Failed to load plugin '{}' from {:?}", plugin_name, path))?
        };

        validate_plugin(&plugin).context("Plugin validation failed")?;

        plugins.push(plugin);
    }

    Ok(plugins)
}

/// Evaluates a plugin.lua file and returns the plugin table
///
/// This helper function:
/// 1. Uses cached table if provided (eliminates re-evaluation)
/// 2. Otherwise reads the plugin file from disk
/// 3. Evaluates the Lua code
/// 4. Returns the plugin table (does NOT store in globals)
///
/// Note: Module paths must be configured before calling this function
fn evaluate_plugin_file(
    lua_runtime: &Lua,
    lua_path: &Path,
    cached_table: Option<Table>,
) -> Result<Table> {
    // Extract plugin directory for __plugin_dir metadata
    let plugin_dir = lua_path
        .parent()
        .with_context(|| {
            format!(
                "Plugin path has no parent directory: {}",
                lua_path.display()
            )
        })?
        .to_str()
        .with_context(|| format!("Path contains invalid UTF-8: {}", lua_path.display()))?;

    // If we have a cached table, use it instead of re-evaluating
    if let Some(table) = cached_table {
        // Still need to set __plugin_dir for the cached table
        table
            .set("__plugin_dir", plugin_dir)
            .context("Failed to set __plugin_dir in cached plugin table")?;
        return Ok(table);
    }

    // No cache - evaluate from file (fallback for legacy code paths)
    let plugin_contents = std::fs::read_to_string(lua_path)
        .with_context(|| format!("Failed to read plugin file '{}'", lua_path.display()))?;

    let plugin_table: Table = lua_runtime
        .load(&plugin_contents)
        .set_name(lua_path.to_str().with_context(|| {
            format!("Plugin path contains invalid UTF-8: {}", lua_path.display())
        })?)
        .eval()
        .with_context(|| format!("Failed to evaluate plugin '{}'", lua_path.display()))?;

    // Store plugin directory in the plugin table for expand_path to use
    plugin_table
        .set("__plugin_dir", plugin_dir)
        .context("Failed to set __plugin_dir in plugin table")?;

    Ok(plugin_table)
}

/// Merge two plugin tables using Lua merge function
fn merge_plugin_tables(
    lua_runtime: &Lua,
    base_table: &Table,
    override_table: &Table,
) -> Result<Table> {
    let merge_fn: mlua::Function = lua_runtime.globals().get(MERGE_LUA_FN_KEY).context(
        "merge function not found in Lua globals (should be injected at runtime creation)",
    )?;

    merge_fn
        .call((base_table, override_table))
        .context("Failed to call merge function")
}

/// Store merged plugin table in Lua globals for runtime access
fn store_plugin_in_globals(
    lua_runtime: &Lua,
    plugin_name: &str,
    plugin_table: &Table,
) -> Result<()> {
    lua_runtime
        .globals()
        .set(plugin_name, plugin_table.clone())
        .with_context(|| format!("Failed to store plugin '{}' in Lua globals", plugin_name))
}

/// Parse and validate merged plugin structure
fn parse_merged_plugin(
    merged_table: &Table,
    plugin_name: &str,
    default_plugin_icon: &str,
) -> Result<Plugin> {
    let metadata_table: Table = merged_table
        .get("metadata")
        .with_context(|| format!("Merged plugin '{}' missing 'metadata' table", plugin_name))?;

    let metadata = parse_metadata(&metadata_table, default_plugin_icon)?;

    // Verify merged plugin name matches expected name
    ensure!(
        metadata.name == plugin_name,
        "Override plugin has name '{}' but expected '{}'. \
         Override plugins must use the same metadata.name as the base plugin.",
        metadata.name,
        plugin_name
    );

    let tasks_table: Table = merged_table
        .get("tasks")
        .with_context(|| format!("Merged plugin '{}' missing 'tasks' table", plugin_name))?;

    let tasks = parse_tasks(&tasks_table, &metadata.name)?;

    Ok(Plugin { metadata, tasks })
}

/// Loads and merges multiple plugin sources (base + overrides)
///
/// When a plugin exists in multiple directories, this function:
/// 1. Merges cached tables using Lua merge function
/// 2. Stores merged result in Lua globals
/// 3. Parses and returns the merged Plugin struct
///
/// # Arguments
/// * `source` - PluginSource::Merge with base/override paths
/// * `cached_tables` - Pre-evaluated plugin tables (same order as source paths)
///
/// Note: Module paths must be configured before calling this function
fn load_and_merge_plugin(
    lua_runtime: &Lua,
    source: &PluginSource,
    plugin_name: &str,
    default_plugin_icon: &str,
    cached_tables: Vec<Table>,
) -> Result<Plugin> {
    let (_base, _override_path, ignored) = match source {
        PluginSource::Merge {
            base,
            override_path,
            ignored,
        } => (base, override_path, ignored),
        _ => bail!("load_and_merge_plugin requires PluginSource::Merge"),
    };

    ensure!(
        cached_tables.len() >= 2,
        "load_and_merge_plugin requires at least 2 cached tables"
    );

    // Warn if paths ignored
    if !ignored.is_empty() {
        eprintln!(
            "Warning: Plugin '{}' found in {} locations. Only first (override) and last (base) will be merged.",
            plugin_name,
            2 + ignored.len()
        );
        for path in ignored {
            eprintln!("  [IGNORED] {}", path.display());
        }
    }

    // Step 1: Extract tables (base = last, override = first)
    let base_table = &cached_tables[cached_tables.len() - 1];
    let override_table = &cached_tables[0];

    // Step 2: Merge tables
    let merged_table = merge_plugin_tables(lua_runtime, base_table, override_table)?;

    // Step 3: Add plugin directory (use override path as it has highest priority)
    let override_plugin_dir = _override_path
        .parent()
        .with_context(|| format!("Override path has no parent: {}", _override_path.display()))?
        .to_str()
        .with_context(|| format!("Path contains invalid UTF-8: {}", _override_path.display()))?;
    merged_table
        .set("__plugin_dir", override_plugin_dir)
        .context("Failed to set __plugin_dir in merged plugin table")?;

    // Step 4: Store in globals (required for runtime function calls)
    store_plugin_in_globals(lua_runtime, plugin_name, &merged_table)?;

    // Step 5: Parse and return Plugin struct
    parse_merged_plugin(&merged_table, plugin_name, default_plugin_icon)
}

/// Merges and validates two plugin files (base + override)
///
/// This function provides a public API for merge-aware validation:
/// 1. Loads both base and override plugin files
/// 2. Merges them using the Lua merge function
/// 3. Stores merged result in Lua globals
/// 4. Parses and validates the merged Plugin struct
///
/// # Arguments
/// * `lua_runtime` - The Lua runtime with merge function loaded
/// * `base_path` - Path to the base plugin file
/// * `override_path` - Path to the override plugin file
/// * `plugin_name` - Expected plugin name (for validation)
/// * `default_icon` - Default icon if not specified
///
/// Note: Module paths must be configured before calling this function
pub fn merge_and_validate_plugins(
    lua_runtime: &Lua,
    base_path: &Path,
    override_path: &Path,
    plugin_name: &str,
    default_icon: &str,
) -> Result<Plugin> {
    // Load both plugin tables
    let base_table = evaluate_plugin_file(lua_runtime, base_path, None)?;
    let override_table = evaluate_plugin_file(lua_runtime, override_path, None)?;

    // Merge tables
    let merged_table = merge_plugin_tables(lua_runtime, &base_table, &override_table)?;

    // Set __plugin_dir to override path (has higher priority)
    let override_plugin_dir = override_path
        .parent()
        .with_context(|| format!("Override path has no parent: {}", override_path.display()))?
        .to_str()
        .with_context(|| format!("Path contains invalid UTF-8: {}", override_path.display()))?;
    merged_table
        .set("__plugin_dir", override_plugin_dir)
        .context("Failed to set __plugin_dir in merged plugin table")?;

    // Store in globals (required for runtime function calls)
    store_plugin_in_globals(lua_runtime, plugin_name, &merged_table)?;

    // Parse and validate merged plugin
    let plugin = parse_merged_plugin(&merged_table, plugin_name, default_icon)?;
    validate_plugin(&plugin)?;

    Ok(plugin)
}

pub fn load_plugin(
    lua_runtime: &Lua,
    lua_path: &Path,
    default_plugin_icon: &str,
    cached_table: Option<Table>,
) -> Result<Plugin> {
    // Evaluate plugin file to get table (uses cache if provided)
    let plugin_table = evaluate_plugin_file(lua_runtime, lua_path, cached_table)?;

    let metadata_table: Table = plugin_table
        .get("metadata")
        .with_context(|| format!("Plugin '{}' missing 'metadata' table", lua_path.display()))?;

    let tasks_table: Table = plugin_table
        .get("tasks")
        .with_context(|| format!("Plugin '{}' missing 'tasks' table", lua_path.display()))?;

    let metadata = parse_metadata(&metadata_table, default_plugin_icon)?;

    lua_runtime
        .globals()
        .set(metadata.name.as_str(), plugin_table)
        .with_context(|| format!("Failed to store plugin '{}' in Lua globals", metadata.name))?;

    let tasks = parse_tasks(&tasks_table, &metadata.name)?;

    Ok(Plugin { metadata, tasks })
}

fn parse_metadata(metadata_table: &Table, default_plugin_icon: &str) -> Result<Metadata> {
    let platforms = match metadata_table.get::<Value>("platforms") {
        Ok(Value::Nil) => Vec::new(), // Field not present - default to empty
        Ok(Value::Table(table)) => {
            // Validate it's an array (sequential integer keys), not a map
            // Use sequence_values() which only works on array-like tables
            let platforms: Vec<String> = table
                .sequence_values()
                .collect::<mlua::Result<Vec<String>>>()
                .context("platforms array must contain only strings")?;
            platforms
        }
        Ok(value) => {
            bail!(
                "platforms field must be an array, got {}",
                value.type_name()
            )
        }
        Err(_) => Vec::new(),
    };

    Ok(Metadata {
        name: metadata_table.get("name").unwrap_or_default(),
        version: metadata_table.get("version").unwrap_or_default(),
        description: metadata_table.get("description").unwrap_or_default(),
        icon: metadata_table
            .get("icon")
            .unwrap_or(default_plugin_icon.to_string()),
        platforms,
    })
}

fn parse_tasks(tasks_table: &Table, plugin_name: &str) -> Result<TaskMap> {
    let mut tasks = HashMap::new();

    for key_table_pair in tasks_table.pairs::<String, Table>() {
        let (task_key, task_table) = key_table_pair
            .with_context(|| format!("Failed to parse task for plugin {}", plugin_name))?;

        let item_polling_interval: usize = task_table.get("item_polling_interval").unwrap_or(0);
        let preview_polling_interval: usize =
            task_table.get("preview_polling_interval").unwrap_or(0);
        let execution_confirmation_message: Option<String> =
            task_table.get("execution_confirmation_message").ok();
        let description: String = task_table.get("description").unwrap_or_default();
        let suppress_success_notification: bool = task_table
            .get("suppress_success_notification")
            .ok()
            .unwrap_or(false);

        let task = Task {
            task_key: task_key.clone(),
            plugin_name: plugin_name.to_string(),
            name: task_table.get("name").unwrap_or_else(|_| task_key.clone()),
            description,
            mode: parse_mode(&task_table)?,
            item_sources: parse_item_sources(&task_table, &task_key)?,
            item_polling_interval,
            preview_polling_interval,
            execution_confirmation_message,
            suppress_success_notification,
        };

        validate_task(&task_table, &task_key)?;

        tasks.insert(task_key, Arc::new(task));
    }

    Ok(tasks)
}

fn parse_mode(task_table: &Table) -> Result<Mode> {
    let mode_str: String = task_table
        .get("mode")
        .unwrap_or_else(|_| "none".to_string());

    match mode_str.as_str() {
        "multi" => Ok(Mode::Multi),
        "none" => Ok(Mode::None),
        _ => bail!("Invalid mode '{}' (must be 'multi' or 'none')", mode_str),
    }
}

fn parse_item_sources(
    task_table: &Table,
    task_key: &str,
) -> Result<Option<HashMap<String, ItemSource>>> {
    let sources_table = task_table.get::<Table>("item_sources").ok();

    if let Some(sources_table) = sources_table {
        let mut sources = HashMap::new();

        for key_table_pair in sources_table.pairs() {
            let (item_source_key, source_table): (String, Table) = key_table_pair
                .with_context(|| format!("Failed to parse item source for task {}", task_key))?;

            let tag: String = source_table
                .get("tag")
                .with_context(|| format!("Item source {} missing 'tag' field", item_source_key))?;

            ensure!(
                source_table.get::<mlua::Function>("items").is_ok(),
                "Item source '{}' in task '{}' must define an 'items' function",
                item_source_key,
                task_key
            );

            sources.insert(
                item_source_key.clone(),
                ItemSource {
                    tag,
                    item_source_key,
                },
            );
        }

        Ok(Some(sources))
    } else {
        Ok(None)
    }
}

pub fn validate_plugin(plugin: &Plugin) -> Result<()> {
    ensure!(!plugin.metadata.name.is_empty(), "Plugin must have a name");
    ensure!(
        !plugin.metadata.version.is_empty(),
        "Plugin ({}) must have a specified version",
        plugin.metadata.name
    );

    Version::parse(&plugin.metadata.version).map_err(|_| {
        anyhow::anyhow!(
            "Plugin ({}) version '{}' has invalid format - must follow semantic versioning (e.g., '1.0.0', '2.5.1-beta')",
            plugin.metadata.name,
            plugin.metadata.version,
        )
    })?;

    ensure!(
        plugin.metadata.icon.width() == 1,
        "Plugin ({}) icon '{}' must occupy a single terminal cell",
        plugin.metadata.name,
        plugin.metadata.icon,
    );

    ensure!(
        !plugin.tasks.is_empty(),
        "Plugin ({}) must define at least one task",
        plugin.metadata.name
    );

    for (task_key, task) in &plugin.tasks {
        if let Some(item_sources) = &task.item_sources {
            ensure!(
                item_sources.is_empty()
                    || item_sources.len() == 1
                    || item_sources.values().all(|s| !s.tag.is_empty()),
                "Task ({}) {} has multiple item sources so every item source needs to declare a tag",
                plugin.metadata.name,
                task_key
            )
        }
    }
    Ok(())
}

fn validate_task(task_table: &Table, task_key: &str) -> Result<()> {
    let has_item_sources = task_table.get::<Table>("item_sources").is_ok();
    let has_execute = task_table
        .get::<mlua::Function>(Task::LUA_FN_NAME_EXECUTE)
        .is_ok();

    ensure!(
        has_item_sources || has_execute,
        "Task '{}' must have either 'item_sources' or 'execute' function",
        task_key
    );

    let description: String = task_table.get("description").unwrap_or_default();
    ensure!(
        !description.is_empty(),
        "Task '{}' must have a 'description' field",
        task_key
    );

    Ok(())
}
