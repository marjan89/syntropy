use anyhow::{Result, bail};
use std::path::PathBuf;

/// Represents the source(s) of a plugin's Lua files
///
/// Handles both single-source and multi-source (merge) scenarios with
/// clear base/override semantics.
#[derive(Debug, Clone)]
pub enum PluginSource {
    /// Single source - load normally without merging
    Single(PathBuf),

    /// Multiple sources - merge with override precedence
    ///
    /// # Merge Order
    /// - `base` - Lowest priority (typically data directory)
    /// - `override_path` - Highest priority (typically config directory)
    /// - `ignored` - Additional paths that won't be used
    Merge {
        base: PathBuf,
        override_path: PathBuf,
        ignored: Vec<PathBuf>,
    },
}

impl PluginSource {
    /// Create from a vector of paths (preserves directory scan order)
    ///
    /// # Path Ordering Convention
    /// - `paths[0]` = config dir (highest priority)
    /// - `paths[last]` = data dir (lowest priority)
    /// - `paths[1..last-1]` = ignored
    pub fn from_paths(paths: Vec<PathBuf>) -> Result<Self> {
        match paths.len() {
            0 => bail!("PluginSource::from_paths requires at least 1 path"),
            1 => Ok(PluginSource::Single(paths[0].clone())),
            _ => {
                let override_path = paths[0].clone();
                let base = paths[paths.len() - 1].clone();
                let ignored = paths[1..paths.len() - 1].to_vec();
                Ok(PluginSource::Merge {
                    base,
                    override_path,
                    ignored,
                })
            }
        }
    }

    /// Returns true if this source requires merging
    pub fn needs_merge(&self) -> bool {
        matches!(self, PluginSource::Merge { .. })
    }
}
