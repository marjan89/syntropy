use clap::Command;
use clap_complete::{Shell, generate};
use std::io;

/// Generates shell completion scripts to stdout
///
/// Outputs shell-specific completion scripts that can be redirected to the
/// appropriate completion directory for each shell.
///
/// # Examples
///
/// ```bash
/// # Zsh
/// syntropy completions zsh > ~/.zfunc/_syntropy
///
/// # Bash
/// syntropy completions bash > ~/.local/share/bash-completion/completions/syntropy
///
/// # Fish
/// syntropy completions fish > ~/.config/fish/completions/syntropy.fish
/// ```
pub fn generate_completions(shell: Shell, cmd: &mut Command) {
    generate(shell, cmd, "syntropy", &mut io::stdout());
}
