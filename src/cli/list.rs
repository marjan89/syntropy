use anyhow::{Context, Result};

use crate::{app::App, cli::ListArgs};

pub fn list_cli(app: &App, args: &ListArgs) -> Result<()> {
    match (&args.plugin, &args.task) {
        (None, _) => list_plugins(app),
        (Some(plugin_name), None) => list_tasks(app, plugin_name),
        (Some(plugin_name), Some(task_key)) => show_task_detail(app, plugin_name, task_key),
    }
}

fn list_plugins(app: &App) -> Result<()> {
    let mut plugins: Vec<_> = app.plugins.iter().collect();
    plugins.sort_by_key(|p| p.metadata.name.to_lowercase());
    if plugins.is_empty() {
        println!("No plugins found.");
        return Ok(());
    }
    for plugin in plugins {
        println!(
            "{} (v{}) - {}",
            plugin.metadata.name, plugin.metadata.version, plugin.metadata.description
        );
    }
    Ok(())
}

fn list_tasks(app: &App, plugin_name: &str) -> Result<()> {
    let plugin = app
        .plugins
        .iter()
        .find(|p| p.metadata.name == plugin_name)
        .with_context(|| {
            let mut names: Vec<_> = app
                .plugins
                .iter()
                .map(|p| p.metadata.name.as_str())
                .collect();
            names.sort_by_key(|n| n.to_lowercase());
            format!(
                "Plugin '{}' not found. Available plugins: {}",
                plugin_name,
                names.join(", ")
            )
        })?;

    let mut tasks: Vec<_> = plugin.tasks.values().collect();
    tasks.sort_by_key(|t| t.task_key.to_lowercase());

    for task in tasks {
        println!("{} - {}", task.task_key, task.description);
    }
    Ok(())
}

fn show_task_detail(app: &App, plugin_name: &str, task_key: &str) -> Result<()> {
    let plugin = app
        .plugins
        .iter()
        .find(|p| p.metadata.name == plugin_name)
        .with_context(|| {
            let mut names: Vec<_> = app
                .plugins
                .iter()
                .map(|p| p.metadata.name.as_str())
                .collect();
            names.sort_by_key(|n| n.to_lowercase());
            format!(
                "Plugin '{}' not found. Available plugins: {}",
                plugin_name,
                names.join(", ")
            )
        })?;

    let task = plugin.tasks.get(task_key).with_context(|| {
        let mut available: Vec<_> = plugin.tasks.keys().map(|k| k.as_str()).collect();
        available.sort_by_key(|k| k.to_lowercase());
        format!(
            "Task '{}' not found in plugin '{}'. Available tasks: {}",
            task_key,
            plugin_name,
            available.join(", ")
        )
    })?;

    let name = if task.name.is_empty() {
        task.task_key.as_str()
    } else {
        task.name.as_str()
    };
    let item_sources_count = task.item_sources.as_ref().map_or(0, |m| m.len());

    println!("key: {}", task.task_key);
    println!("name: {}", name);
    println!("description: {}", task.description);
    println!("mode: {}", task.mode);
    println!("item_sources: {}", item_sources_count);
    Ok(())
}
