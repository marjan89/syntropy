use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginDeclaration {
    pub git: String,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
}

impl PluginDeclaration {
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.git.is_empty(), "Plugin git URL cannot be empty");

        ensure!(
            self.git.starts_with("https://") || self.git.starts_with("git@"),
            "Invalid git URL format: '{}' (must start with https:// or git@)",
            self.git
        );

        match (&self.tag, &self.commit) {
            (Some(_), Some(_)) => {
                bail!("Plugin must not declare both tag and commit - choose one")
            }
            (None, None) => {
                bail!("Plugin must specify either tag or commit")
            }
            _ => Ok(()),
        }
    }
}
