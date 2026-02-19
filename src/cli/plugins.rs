use std::{
    collections::HashSet,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use crate::{Config, cli::PluginsArgs, configs::paths::resolve_plugin_paths, plugins::git_ops};
use anyhow::{Context, Result, bail, ensure};

struct PluginPaths {
    user: PathBuf,
    managed: PathBuf,
}

fn resolve_plugin_directories() -> Result<PluginPaths> {
    let resolved = resolve_plugin_paths()?;

    match resolved.len() {
        1 => Ok(PluginPaths {
            user: resolved[0].clone(),
            managed: resolved[0].clone(),
        }),
        2 => Ok(PluginPaths {
            user: resolved[0].clone(),
            managed: resolved[1].clone(),
        }),
        _ => bail!("Invalid plugin path resolution"),
    }
}

pub fn handle_plugins_command(plugin_params: &PluginsArgs, config: Config) -> Result<()> {
    let flags_set = [
        plugin_params.remove,
        plugin_params.install,
        plugin_params.upgrade,
        plugin_params.list,
    ]
    .iter()
    .filter(|&&flag| flag)
    .count();

    ensure!(
        flags_set == 1,
        "Exactly one operation flag must be specified (--install, --remove, --upgrade, or --list)"
    );

    if plugin_params.plugin.is_some() && !plugin_params.upgrade {
        bail!("--plugin can only be used with --upgrade")
    }

    let paths = resolve_plugin_directories()?;

    if plugin_params.remove {
        remove_plugins(config, &paths)?
    } else if plugin_params.install {
        install_plugins(config, &paths)?
    } else if plugin_params.upgrade {
        upgrade_plugins(config, &paths, &plugin_params.plugin)?
    } else if plugin_params.list {
        list_plugins(config, &paths)?
    }

    Ok(())
}

fn get_plugin_names_in_dir(dir: &PathBuf) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
            && path.join("plugin.lua").exists()
        {
            plugins.push(name.to_string());
        }
    }

    Ok(plugins)
}

fn list_plugins(config: Config, paths: &PluginPaths) -> Result<()> {
    let user_plugins = get_plugin_names_in_dir(&paths.user)?;
    let managed_plugins = get_plugin_names_in_dir(&paths.managed)?;
    let declared_plugins: HashSet<_> = config.plugins.keys().collect();

    println!("User plugins installed at: {:?}", &paths.user);
    println!("Managed plugins installed at: {:?}", &paths.managed);
    println!();

    if !user_plugins.is_empty() {
        println!("User plugins:");
        for plugin in &user_plugins {
            let warning = if managed_plugins.contains(plugin) {
                " ⚠ overrides managed plugin"
            } else {
                ""
            };
            println!("  {}{}", plugin, warning);
        }
        println!();
    }

    let managed_with_decl: Vec<_> = managed_plugins
        .iter()
        .filter(|name| declared_plugins.contains(name))
        .collect();

    if !managed_with_decl.is_empty() {
        println!("Managed plugins:");
        for plugin in managed_with_decl {
            let decl = &config.plugins[plugin];
            let version_info = match (&decl.tag, &decl.commit) {
                (Some(tag), None) => format!("tag={}", tag),
                (None, Some(commit)) => {
                    let short_commit = &commit[..8.min(commit.len())];
                    format!("commit={}", short_commit)
                }
                _ => "unknown".to_string(),
            };

            let override_warning = if user_plugins.contains(plugin) {
                " (overridden by user plugin)"
            } else {
                ""
            };

            println!("  {} ({}){}", plugin, version_info, override_warning);
        }
        println!();
    }

    let orphaned: Vec<_> = managed_plugins
        .iter()
        .filter(|name| !declared_plugins.contains(name))
        .collect();

    if !orphaned.is_empty() {
        println!("Orphaned plugins:");
        for plugin in orphaned {
            println!("  {} ← candidate for removal", plugin);
        }
        println!();
    }

    if user_plugins.is_empty() && managed_plugins.is_empty() {
        println!("No plugins found.");
    }

    Ok(())
}

fn install_plugins(config: Config, paths: &PluginPaths) -> Result<()> {
    let data_dir = &paths.managed;
    let managed_plugins = get_plugin_names_in_dir(&paths.managed)?;

    fs::create_dir_all(data_dir).context("Failed to create data plugins directory")?;

    let to_install: Vec<_> = config
        .plugins
        .iter()
        .filter(|(name, _)| !managed_plugins.contains(name))
        .collect();

    if to_install.is_empty() {
        println!("All declared plugins already installed.");
        return Ok(());
    }

    println!("Installing {} plugin(s)...", to_install.len());

    for (name, decl) in to_install {
        print!("  {} ... ", name);
        io::stdout().flush()?;

        let plugin_dir = data_dir.join(name);

        let ref_spec = match (&decl.tag, &decl.commit) {
            (Some(tag), None) => tag,
            (None, Some(commit)) => commit,
            _ => {
                println!("ERROR: must specify either tag or commit");
                continue;
            }
        };

        match git_ops::clone_plugin(&decl.git, &plugin_dir, ref_spec) {
            Ok(_) => println!("✓ installed ({})", ref_spec),
            Err(e) => {
                println!("✗ failed: {:#}", e);
                let _ = fs::remove_dir_all(&plugin_dir);
            }
        }
    }

    Ok(())
}

fn remove_plugins(config: Config, paths: &PluginPaths) -> Result<()> {
    let managed_plugins = get_plugin_names_in_dir(&paths.managed)?;
    let declared_plugins: HashSet<_> = config.plugins.keys().collect();
    let user_plugins: HashSet<_> = get_plugin_names_in_dir(&paths.user)?.into_iter().collect();

    let orphaned: Vec<_> = managed_plugins
        .iter()
        .filter(|name| !declared_plugins.contains(name))
        .collect();

    if orphaned.is_empty() {
        println!("No orphaned plugins to remove.");
        return Ok(());
    }

    println!("The following plugins will be removed:");

    let mut removable = Vec::new();
    let mut blocked = Vec::new();

    for name in orphaned {
        if user_plugins.contains(name) {
            println!(
                "  {} - ⚠ user override exists in XDG_CONFIG, remove manually",
                name
            );
            blocked.push(name);
        } else {
            println!("  {}", name);
            removable.push(name);
        }
    }

    if removable.is_empty() {
        println!("\nNo plugins can be removed (all have user overrides).");
        return Ok(());
    }

    print!("\nContinue? (y/n): ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("Aborted.");
        return Ok(());
    }

    for name in removable {
        let plugin_dir = paths.managed.join(name);
        match fs::remove_dir_all(&plugin_dir) {
            Ok(_) => println!("  ✓ {} removed", name),
            Err(e) => println!("  ✗ {} failed: {:#}", name, e),
        }
    }

    Ok(())
}

fn upgrade_plugins(config: Config, paths: &PluginPaths, plugin: &Option<String>) -> Result<()> {
    let plugins_to_upgrade: Vec<String> = if let Some(name) = plugin {
        vec![name.clone()]
    } else {
        config.plugins.keys().cloned().collect()
    };

    println!(
        "Checking {} plugin(s) for upgrades...",
        plugins_to_upgrade.len()
    );

    for name in plugins_to_upgrade {
        let decl = config
            .plugins
            .get(&name)
            .with_context(|| format!("Plugin '{}' not declared in config", name))?;

        let plugin_dir = paths.managed.join(&name);

        if !plugin_dir.exists() {
            println!("  {} - not installed, skipping", name);
            continue;
        }

        if decl.commit.is_some() {
            println!("  {} - declaration uses commit, nothing to do", name);
            continue;
        }

        let declared_tag = decl
            .tag
            .as_ref()
            .context("Plugin must specify tag for upgrade")?;

        if let Err(e) = git_ops::git_fetch(&plugin_dir) {
            println!("  {} - fetch failed: {:#}", name, e);
            continue;
        }

        let latest_tag = match git_ops::get_latest_tag(&plugin_dir)? {
            Some(tag) => tag,
            None => {
                println!("  {} - no tags found in repository", name);
                continue;
            }
        };

        use std::cmp::Ordering;
        match compare_tags(declared_tag, &latest_tag) {
            Ordering::Greater => {
                print!("  {} - upgrading to {} ... ", name, declared_tag);
                io::stdout().flush()?;
                match git_ops::checkout_ref(&plugin_dir, declared_tag) {
                    Ok(_) => println!("✓"),
                    Err(e) => println!("✗ {:#}", e),
                }
            }
            Ordering::Equal => {
                println!("  {} - already up to date ({})", name, declared_tag);
            }
            Ordering::Less => {
                println!(
                    "  {} - ⚠ TOML declares {} but {} is available (not upgrading to older version)",
                    name, declared_tag, latest_tag
                );
            }
        }
    }

    Ok(())
}

pub fn compare_tags(tag1: &str, tag2: &str) -> std::cmp::Ordering {
    use semver::Version;

    let v1_clean = tag1.strip_prefix('v').unwrap_or(tag1);
    let v2_clean = tag2.strip_prefix('v').unwrap_or(tag2);

    if let (Ok(ver1), Ok(ver2)) = (Version::parse(v1_clean), Version::parse(v2_clean)) {
        return ver1.cmp(&ver2);
    }

    tag1.cmp(tag2)
}
