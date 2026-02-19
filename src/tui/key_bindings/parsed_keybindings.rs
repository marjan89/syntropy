use anyhow::{Context, Result, bail};
use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

use crate::{configs::KeyBindings, tui::key_bindings::KeyBind};

#[derive(Debug, Clone)]
pub struct ParsedKeyBindings {
    pub back: KeyBind,
    pub select_previous: KeyBind,
    pub select_next: KeyBind,
    pub scroll_preview_up: KeyBind,
    pub scroll_preview_down: KeyBind,
    pub toggle_preview: KeyBind,
    pub select: KeyBind,
    pub confirm: KeyBind,
}

impl ParsedKeyBindings {
    pub fn from(key_bindings: &KeyBindings) -> Result<ParsedKeyBindings> {
        let parsed_keybindings = ParsedKeyBindings {
            back: KeyBind::parse(&key_bindings.back).with_context(|| {
                format!("Failed to parse 'back' keybinding '{}'", key_bindings.back)
            })?,
            select_previous: KeyBind::parse(&key_bindings.select_previous).with_context(|| {
                format!(
                    "Failed to parse 'select_previous' keybinding '{}'",
                    key_bindings.select_previous
                )
            })?,
            select_next: KeyBind::parse(&key_bindings.select_next).with_context(|| {
                format!(
                    "Failed to parse 'select_next' keybinding '{}'",
                    key_bindings.select_next
                )
            })?,
            scroll_preview_up: KeyBind::parse(&key_bindings.scroll_preview_up).with_context(
                || {
                    format!(
                        "Failed to parse 'scroll_preview_up' keybinding '{}'",
                        key_bindings.scroll_preview_up
                    )
                },
            )?,
            scroll_preview_down: KeyBind::parse(&key_bindings.scroll_preview_down).with_context(
                || {
                    format!(
                        "Failed to parse 'scroll_preview_down' keybinding '{}'",
                        key_bindings.scroll_preview_down
                    )
                },
            )?,
            toggle_preview: KeyBind::parse(&key_bindings.toggle_preview).with_context(|| {
                format!(
                    "Failed to parse 'toggle_preview' keybinding '{}'",
                    key_bindings.toggle_preview
                )
            })?,
            select: KeyBind::parse(&key_bindings.select).with_context(|| {
                format!(
                    "Failed to parse 'select' keybinding '{}'",
                    key_bindings.select
                )
            })?,
            confirm: KeyBind::parse(&key_bindings.confirm).with_context(|| {
                format!(
                    "Failed to parse 'confirm' keybinding '{}'",
                    key_bindings.confirm
                )
            })?,
        };

        // Check for duplicate key bindings
        check_for_duplicates(&parsed_keybindings)?;

        Ok(parsed_keybindings)
    }
}

fn check_for_duplicates(parsed: &ParsedKeyBindings) -> Result<()> {
    let mut binding_map: HashMap<(KeyCode, KeyModifiers), Vec<&str>> = HashMap::new();

    binding_map
        .entry((parsed.back.code, parsed.back.modifiers))
        .or_default()
        .push("back");
    binding_map
        .entry((
            parsed.select_previous.code,
            parsed.select_previous.modifiers,
        ))
        .or_default()
        .push("select_previous");
    binding_map
        .entry((parsed.select_next.code, parsed.select_next.modifiers))
        .or_default()
        .push("select_next");
    binding_map
        .entry((
            parsed.scroll_preview_up.code,
            parsed.scroll_preview_up.modifiers,
        ))
        .or_default()
        .push("scroll_preview_up");
    binding_map
        .entry((
            parsed.scroll_preview_down.code,
            parsed.scroll_preview_down.modifiers,
        ))
        .or_default()
        .push("scroll_preview_down");
    binding_map
        .entry((parsed.toggle_preview.code, parsed.toggle_preview.modifiers))
        .or_default()
        .push("toggle_preview");
    binding_map
        .entry((parsed.select.code, parsed.select.modifiers))
        .or_default()
        .push("select");
    binding_map
        .entry((parsed.confirm.code, parsed.confirm.modifiers))
        .or_default()
        .push("confirm");

    let conflicts: Vec<String> = binding_map
        .iter()
        .filter(|(_, actions)| actions.len() > 1)
        .map(|(key, actions)| format!("{:?} is bound to: {}", key, actions.join(", ")))
        .collect();

    if !conflicts.is_empty() {
        bail!(
            "Duplicate key bindings detected:\n  {}",
            conflicts.join("\n  ")
        );
    }

    Ok(())
}
