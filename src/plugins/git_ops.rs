use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, ensure};

/// Clones a git repository and checks out the specified ref
///
/// # Arguments
///
/// * `git_url` - The git repository URL (https:// or git@)
/// * `dest` - The destination directory for the clone
/// * `ref_spec` - The tag or commit to checkout after cloning
///
/// # Errors
///
/// Returns an error if:
/// - git command is not available
/// - Clone operation fails
/// - Checkout operation fails
pub fn clone_plugin(git_url: &str, dest: &Path, ref_spec: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", "--quiet", git_url])
        .arg(dest.as_os_str())
        .output()
        .context("Failed to execute git clone (is git installed?)")?;

    ensure!(
        output.status.success(),
        "git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    checkout_ref(dest, ref_spec)?;
    Ok(())
}

/// Checks out a specific tag or commit
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
/// * `ref_spec` - The tag or commit to checkout
///
/// # Errors
///
/// Returns an error if:
/// - The repository path does not exist
/// - The ref_spec does not exist in the repository
/// - Checkout operation fails
pub fn checkout_ref(repo_path: &Path, ref_spec: &str) -> Result<()> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["checkout", "--quiet", ref_spec])
        .output()
        .context("Failed to execute git checkout")?;

    ensure!(
        output.status.success(),
        "git checkout '{}' failed: {}",
        ref_spec,
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

/// Fetches latest tags from remote
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
///
/// # Errors
///
/// Returns an error if:
/// - The repository path does not exist
/// - Network is unavailable
/// - Fetch operation fails
pub fn git_fetch(repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["fetch", "--tags", "--quiet"])
        .output()
        .context("Failed to execute git fetch")?;

    ensure!(
        output.status.success(),
        "git fetch failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

/// Gets the latest tag sorted by version
///
/// Uses `git tag --sort=-version:refname` to get tags sorted by semantic version
/// in descending order. Returns the first (latest) tag.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
///
/// # Returns
///
/// Returns `Ok(Some(tag))` if tags exist, `Ok(None)` if no tags exist
///
/// # Errors
///
/// Returns an error if the git command fails
pub fn get_latest_tag(repo_path: &Path) -> Result<Option<String>> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["tag", "--sort=-version:refname"])
        .output()
        .context("Failed to execute git tag")?;

    ensure!(
        output.status.success(),
        "git tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tags = String::from_utf8(output.stdout)?;
    Ok(tags.lines().next().map(|s| s.to_string()))
}

/// Gets the current tag if HEAD is exactly on a tag
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
///
/// # Returns
///
/// Returns `Ok(Some(tag))` if HEAD is exactly on a tag, `Ok(None)` otherwise
///
/// # Errors
///
/// Returns an error if the git command fails (not including "not on a tag" case)
pub fn get_current_tag(repo_path: &Path) -> Result<Option<String>> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .context("Failed to execute git describe")?;

    if !output.status.success() {
        // Not on a tag is not an error condition
        return Ok(None);
    }

    let tag = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(Some(tag))
}

/// Checks if a directory is a git repository
///
/// # Arguments
///
/// * `path` - Path to check
///
/// # Returns
///
/// Returns `true` if the directory contains a `.git` subdirectory
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}
