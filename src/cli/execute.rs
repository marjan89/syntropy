use anyhow::{Context, Result, bail, ensure};
use std::collections::HashSet;

use crate::{
    app::App,
    cli::ExecuteArgs,
    execution::{
        clamp_exit_code, run_execute_pipeline, run_items_pipeline, run_preview_pipeline,
        runner::parse_tag,
    },
    plugins::{Mode, Task},
};

/// Parses comma-separated items with support for escaped commas
///
/// Supports:
/// - `\,` - escaped comma (becomes part of item name)
/// - `\\` - escaped backslash (becomes literal backslash)
/// - `,` - item separator (unescaped comma)
///
/// # Examples
///
/// ```
/// use syntropy::cli::execute::parse_comma_separated_with_escapes;
///
/// let result = parse_comma_separated_with_escapes("item1,item2");
/// assert_eq!(result, vec!["item1", "item2"]);
///
/// let result = parse_comma_separated_with_escapes("item1,backup\\,2024,item3");
/// assert_eq!(result, vec!["item1", "backup,2024", "item3"]);
///
/// let result = parse_comma_separated_with_escapes("path\\\\to\\\\file,other");
/// assert_eq!(result, vec!["path\\to\\file", "other"]);
/// ```
#[doc(hidden)]
pub fn parse_comma_separated_with_escapes(input: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current_item = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match (ch, chars.peek()) {
            // Escaped comma: add comma to current item
            ('\\', Some(&',')) => {
                current_item.push(',');
                chars.next(); // consume the comma
            }
            // Escaped backslash: add backslash to current item
            ('\\', Some(&'\\')) => {
                current_item.push('\\');
                chars.next(); // consume the backslash
            }
            // Trailing backslash or unrecognized escape: keep the backslash
            ('\\', _) => {
                current_item.push('\\');
            }
            // Unescaped comma: separator between items
            (',', _) => {
                let trimmed = current_item.trim().to_string();
                if !trimmed.is_empty() {
                    items.push(trimmed);
                }
                current_item.clear();
            }
            // Regular character: add to current item
            _ => {
                current_item.push(ch);
            }
        }
    }

    // Don't forget the last item
    let trimmed = current_item.trim().to_string();
    if !trimmed.is_empty() {
        items.push(trimmed);
    }

    items
}

/// Handles item matching with three-tiered fallback strategy:
/// 1. Exact case-sensitive match
/// 2. Tag-stripped match (multi-source only)
/// 3. Case-insensitive match
#[doc(hidden)]
pub struct ItemMatcher<'a> {
    available_items: &'a [String],
    is_multi_source: bool,
    task_key: &'a str,
}

impl<'a> ItemMatcher<'a> {
    #[doc(hidden)]
    pub fn new(available_items: &'a [String], is_multi_source: bool, task_key: &'a str) -> Self {
        Self {
            available_items,
            is_multi_source,
            task_key,
        }
    }

    /// Matches a single requested item, returning the matched item or an error
    #[doc(hidden)]
    pub fn match_item(&self, requested_item: &str) -> Result<String> {
        let requested_item = requested_item.trim();
        ensure!(
            !requested_item.is_empty(),
            "--items cannot contain empty or whitespace-only values"
        );

        // Strategy 1: Exact case-sensitive match
        if let Some(exact_match) = self.try_exact_match(requested_item) {
            return Ok(exact_match);
        }

        // Strategy 2: Tag-stripped match (multi-source only)
        if self.is_multi_source
            && let Some(tagged_match) = self.try_tag_stripped_match(requested_item)?
        {
            return Ok(tagged_match);
        }

        // Strategy 3: Case-insensitive fallback
        if let Some(case_insensitive) = self.try_case_insensitive_match(requested_item) {
            return Ok(case_insensitive);
        }

        // No match found
        bail!(
            "Item '{}' not found in task '{}'. Available items:\n  {}",
            requested_item,
            self.task_key,
            self.available_items.join("\n  ")
        );
    }

    /// Attempts exact case-sensitive match
    #[doc(hidden)]
    pub fn try_exact_match(&self, requested_item: &str) -> Option<String> {
        self.available_items
            .iter()
            .find(|&item| item == requested_item)
            .cloned()
    }

    /// Attempts tag-stripped matching with ambiguity detection
    #[doc(hidden)]
    pub fn try_tag_stripped_match(&self, requested_item: &str) -> Result<Option<String>> {
        let matches: Vec<String> = self
            .available_items
            .iter()
            .filter(|item| {
                let (_, content) = parse_tag(item);
                content == requested_item
            })
            .cloned()
            .collect();

        match matches.len() {
            0 => Ok(None),
            1 => {
                eprintln!(
                    "Info: Matched '{}' to tagged item '{}'",
                    requested_item, matches[0]
                );
                Ok(Some(matches[0].clone()))
            }
            n => bail!(
                "Ambiguous item: '{}' matches {} items from different sources:\n  {}\n\
                 Use the full tagged format (e.g., '[packages] {}') to disambiguate.",
                requested_item,
                n,
                matches.join("\n  "),
                requested_item
            ),
        }
    }

    /// Attempts case-insensitive match
    #[doc(hidden)]
    pub fn try_case_insensitive_match(&self, requested_item: &str) -> Option<String> {
        let requested_lower = requested_item.to_lowercase();
        let matches: Vec<&String> = self
            .available_items
            .iter()
            .filter(|item| {
                let (_, content) = parse_tag(item);
                content.to_lowercase() == requested_lower
            })
            .collect();

        if matches.len() == 1 {
            eprintln!(
                "Info: Using case-insensitive match '{}' for '{}'",
                matches[0], requested_item
            );
            Some(matches[0].clone())
        } else {
            None
        }
    }

    /// Validates and matches all requested items
    #[doc(hidden)]
    pub fn match_all(&self, requested_items: &[&str]) -> Result<Vec<String>> {
        requested_items
            .iter()
            .map(|&item| self.match_item(item))
            .collect()
    }
}

/// Validates that items_arg is compatible with the task configuration
fn validate_items_arg_compatibility(
    items_arg: &[&str],
    task: &Task,
    preselected_items: &[String],
) -> Result<()> {
    if items_arg.is_empty() {
        return Ok(());
    }

    ensure!(
        task.item_sources.is_some(),
        "Task '{}' has no item sources (standalone execute-only task). \
         The --items flag cannot be used with this task.",
        task.task_key
    );

    if !preselected_items.is_empty() {
        eprintln!(
            "Warning: --items flag overrides preselected_items(). \
             Using {} specified item(s) instead of {} preselected item(s).",
            items_arg.len(),
            preselected_items.len()
        );
    }

    Ok(())
}

/// Resolves items based on task mode when no explicit items are specified
fn resolve_items_by_mode(
    task: &Task,
    items: &[String],
    preselected_items: &[String],
) -> Result<Vec<String>> {
    match task.mode {
        Mode::None => {
            if items.len() > 1 {
                bail!(
                    "Task '{}' has mode='none' which requires single-item selection. \
                     Use --items flag to specify which item to execute.\n  Available items:\n  {}",
                    task.task_key,
                    items.join("\n  ")
                );
            }
            Ok(items.to_vec())
        }
        Mode::Multi => {
            if !preselected_items.is_empty() {
                eprintln!(
                    "Executing with {} preselected item(s)",
                    preselected_items.len()
                );
                Ok(preselected_items.to_vec())
            } else {
                eprintln!("Executing with all {} item(s)", items.len());
                Ok(items.to_vec())
            }
        }
    }
}

fn validate_and_resolve_items(
    items_arg: &[&str],
    task: &Task,
    items: &[String],
    preselected_items: &[String],
) -> Result<Vec<String>> {
    // Early validation
    validate_items_arg_compatibility(items_arg, task, preselected_items)?;

    // If items explicitly specified, validate and match them
    if !items_arg.is_empty() {
        let is_multi_source = task
            .item_sources
            .as_ref()
            .map(|sources| sources.len() > 1)
            .unwrap_or(false);

        let matcher = ItemMatcher::new(items, is_multi_source, &task.task_key);
        return matcher.match_all(items_arg);
    }

    // Otherwise, resolve based on task mode
    resolve_items_by_mode(task, items, preselected_items)
}

/// Executes a task directly from CLI without launching the TUI
///
/// This function provides non-interactive task execution for use in scripts,
/// cron jobs, and CI/CD pipelines.
///
/// # Item Selection Logic
///
/// **With `--items` flag:**
/// - Validates that the specified items exist in the task's items
/// - Executes on those items (works for any mode)
/// - Supports comma-separated list: `--items "item1,item2,item3"`
/// - Overrides `preselected_items()` if present
///
/// **Without `--items` flag:**
/// - For `mode="none"` tasks with multiple items: Returns error (explicit selection required)
/// - For `mode="none"` tasks with single item: Executes on that item
/// - For `mode="multi"` tasks: Uses preselected items if any, otherwise all items
///
/// **For execute-only tasks (no item_sources):**
/// - Executes directly with empty items array
/// - `--items` flag is not applicable and will return error
///
/// # Arguments
///
/// * `app` - Application context with loaded plugins and configuration
/// * `execute_args` - Execute command arguments containing plugin, task, and optional items
///
/// # Returns
///
/// Returns `Ok(exit_code)` on successful execution, `Err` otherwise.
/// Exit code is propagated from the Lua execute function and clamped to valid POSIX range (0-255).
/// Invalid exit codes (<0 or >255) are clamped with warnings printed to stderr.
/// Output is printed to stdout, errors to stderr.
///
/// # Examples
///
/// ```bash
/// # Execute on specific items (any mode)
/// syntropy execute --plugin packages --task info --items git
/// syntropy execute --plugin packages --task info --items "git,npm,brew"
///
/// # Execute with default selection (multi mode)
/// syntropy execute --plugin packages --task export_bundle
///
/// # Error: mode=none requires --items when multiple items exist
/// syntropy execute --plugin packages --task export
/// ```
pub async fn execute_task_cli(app: App, execute_args: &ExecuteArgs) -> Result<i32> {
    let plugin_name = &execute_args.plugin;
    let task_key = &execute_args.task;

    // Parse comma-separated items if provided (with escape support for commas in item names)
    let items_arg: Vec<String> = execute_args
        .items
        .as_ref()
        .map(|s| parse_comma_separated_with_escapes(s))
        .unwrap_or_default();

    // Convert to Vec<&str> for validate_and_resolve_items
    let items_arg_refs: Vec<&str> = items_arg.iter().map(|s| s.as_str()).collect();

    // Validate that if --items was provided, it contained at least one non-empty value
    if execute_args.items.is_some() && items_arg.is_empty() {
        bail!("--items cannot be empty or whitespace-only");
    }

    let plugin = app
        .plugins
        .iter()
        .find(|p| p.metadata.name == *plugin_name)
        .with_context(|| {
            let available = app
                .plugins
                .iter()
                .map(|p| p.metadata.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "Plugin '{}' not found. Available plugins: {}",
                plugin_name, available
            )
        })?;

    let task = plugin.tasks.get(task_key).with_context(|| {
        let mut available: Vec<_> = plugin.tasks.keys().map(|k| k.as_str()).collect();
        // Sort task names alphabetically (case-insensitive) for consistent error messages
        available.sort_by_key(|a| a.to_lowercase());
        let available_str = available.join(", ");
        format!(
            "Task '{}' not found in plugin '{}'. Available tasks: {}",
            task_key, plugin_name, available_str
        )
    })?;

    // Handle --preview flag: generate preview for a single item
    if let Some(preview_item) = &execute_args.preview {
        ensure!(
            task.item_sources.is_some(),
            "Task '{}' has no item sources. The --preview flag requires a task with item sources.",
            task.task_key
        );

        let (items, _) = run_items_pipeline(app.lua_runtime.clone(), task)
            .await
            .context("Failed to fetch items from task")?;

        let is_multi_source = task.item_sources.as_ref().unwrap().len() > 1;
        let matcher = ItemMatcher::new(&items, is_multi_source, &task.task_key);
        let matched_item = matcher.match_item(preview_item)?;

        let preview_text = run_preview_pipeline(app.lua_runtime.clone(), task, &matched_item)
            .await
            .context("Failed to generate preview")?;

        println!("{}", preview_text);
        return Ok(0);
    }

    // Handle --produce-items flag: output all available items
    if execute_args.produce_items {
        ensure!(
            task.item_sources.is_some(),
            "Task '{}' has no item sources. The --produce-items flag requires a task with item sources.",
            task.task_key
        );

        let (items, _) = run_items_pipeline(app.lua_runtime.clone(), task)
            .await
            .context("Failed to fetch items from task")?;

        for item in items {
            println!("{}", item);
        }

        return Ok(0);
    }

    // Handle --produce-preselected-items flag: output preselected items
    if execute_args.produce_preselected_items {
        ensure!(
            task.item_sources.is_some(),
            "Task '{}' has no item sources. The --produce-preselected-items flag requires a task with item sources.",
            task.task_key
        );

        let (_, preselected_items) = run_items_pipeline(app.lua_runtime.clone(), task)
            .await
            .context("Failed to fetch items from task")?;

        for item in preselected_items {
            println!("{}", item);
        }

        return Ok(0);
    }

    // Handle --produce-preselection-matches flag: output items that match preselection
    if execute_args.produce_preselection_matches {
        ensure!(
            task.item_sources.is_some(),
            "Task '{}' has no item sources. The --produce-preselection-matches flag requires a task with item sources.",
            task.task_key
        );

        let (items, preselected_items) = run_items_pipeline(app.lua_runtime.clone(), task)
            .await
            .context("Failed to fetch items from task")?;

        // Calculate intersection: items that appear in both lists
        let preselected_set: HashSet<_> = preselected_items.into_iter().collect();
        for item in items {
            if preselected_set.contains(&item) {
                println!("{}", item);
            }
        }

        return Ok(0);
    }

    let selected_items = if task.item_sources.is_some() {
        let (items, preselected_items) = run_items_pipeline(app.lua_runtime.clone(), task)
            .await
            .context("Failed to fetch items from task")?;

        validate_and_resolve_items(&items_arg_refs, task, &items, &preselected_items)?
    } else {
        ensure!(
            items_arg_refs.is_empty(),
            "Task '{}' has no item sources (standalone execute-only task). The --items flag cannot be used with this task.",
            task.task_key
        );
        vec![]
    };

    let (output, exit_code) = run_execute_pipeline(app.lua_runtime.clone(), task, &selected_items)
        .await
        .context("Failed to execute task")?;

    if !output.is_empty() {
        println!("{}", output);
    }

    let clamped_exit_code = clamp_exit_code(exit_code);
    if clamped_exit_code != exit_code {
        eprintln!(
            "Warning: Exit code {} clamped to {}",
            exit_code, clamped_exit_code
        );
    }

    Ok(clamped_exit_code)
}
