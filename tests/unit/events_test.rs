//! Unit tests for TUI event handling
//!
//! Tests the pure function that maps crossterm KeyEvents to InputEvents.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use syntropy::tui::events::{InputEvent, handle_key};
use syntropy::tui::key_bindings::{KeyBind, ParsedKeyBindings};

// Helper to create test bindings with default configuration
fn create_test_bindings() -> ParsedKeyBindings {
    ParsedKeyBindings {
        back: KeyBind::parse("<esc>").unwrap(),
        select_previous: KeyBind::parse("<up>").unwrap(),
        select_next: KeyBind::parse("<down>").unwrap(),
        scroll_preview_up: KeyBind::parse("<C-u>").unwrap(),
        scroll_preview_down: KeyBind::parse("<C-d>").unwrap(),
        toggle_preview: KeyBind::parse("p").unwrap(),
        confirm: KeyBind::parse("<enter>").unwrap(),
        select: KeyBind::parse("<tab>").unwrap(),
    }
}

// ============================================================================
// Basic Input Event Mapping Tests
// ============================================================================

#[test]
fn test_handle_key_back() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Back));
}

#[test]
fn test_handle_key_previous_item() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::PreviousItem)
    );
}

#[test]
fn test_handle_key_next_item() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::NextItem));
}

#[test]
fn test_handle_key_scroll_preview_up() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::ScrollPreviewUp)
    );
}

#[test]
fn test_handle_key_scroll_preview_down() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::ScrollPreviewDown)
    );
}

#[test]
fn test_handle_key_toggle_preview() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty());
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::TogglePreview)
    );
}

#[test]
fn test_handle_key_confirm() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Confirm));
}

#[test]
fn test_handle_key_select() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Select));
}

// ============================================================================
// Unknown Key Tests
// ============================================================================

#[test]
fn test_handle_key_unknown_returns_none() {
    let bindings = create_test_bindings();
    let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), None);
}

#[test]
fn test_handle_key_modifier_mismatch() {
    let bindings = create_test_bindings();
    // Binding is 'p' (no modifier), event has Ctrl
    let event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
    assert_eq!(handle_key(&event, &bindings), None);
}

// ============================================================================
// Custom Binding Tests
// ============================================================================

#[test]
fn test_handle_key_with_custom_bindings() {
    let mut bindings = create_test_bindings();
    // Override back to use 'q' instead of Esc
    bindings.back = KeyBind::parse("q").unwrap();

    let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Back));

    // Esc should no longer trigger Back
    let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    assert_eq!(handle_key(&esc_event, &bindings), None);
}

#[test]
fn test_handle_key_with_ctrl_modifier() {
    let mut bindings = create_test_bindings();
    bindings.confirm = KeyBind::parse("<C-enter>").unwrap();

    let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Confirm));
}

// ============================================================================
// Comprehensive InputEvent Variant Coverage
// ============================================================================

#[test]
fn test_all_input_event_variants_mappable() {
    // Ensure all 8 InputEvent variants can be returned
    let bindings = ParsedKeyBindings {
        back: KeyBind::parse("1").unwrap(),
        select_previous: KeyBind::parse("2").unwrap(),
        select_next: KeyBind::parse("3").unwrap(),
        scroll_preview_up: KeyBind::parse("4").unwrap(),
        scroll_preview_down: KeyBind::parse("5").unwrap(),
        toggle_preview: KeyBind::parse("6").unwrap(),
        select: KeyBind::parse("7").unwrap(),
        confirm: KeyBind::parse("8").unwrap(),
    };

    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::Back)
    );
    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('2'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::PreviousItem)
    );
    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('3'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::NextItem)
    );
    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('4'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::ScrollPreviewUp)
    );
    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('5'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::ScrollPreviewDown)
    );
    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('6'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::TogglePreview)
    );
    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('7'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::Select)
    );
    assert_eq!(
        handle_key(
            &KeyEvent::new(KeyCode::Char('8'), KeyModifiers::empty()),
            &bindings
        ),
        Some(InputEvent::Confirm)
    );
}

// ============================================================================
// Special Key Tests
// ============================================================================

#[test]
fn test_handle_key_with_function_keys() {
    let mut bindings = create_test_bindings();
    bindings.back = KeyBind::parse("<F1>").unwrap();
    bindings.confirm = KeyBind::parse("<F2>").unwrap();

    let event = KeyEvent::new(KeyCode::F(1), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Back));

    let event = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Confirm));
}

#[test]
fn test_handle_key_with_home_end_keys() {
    let mut bindings = create_test_bindings();
    bindings.select_previous = KeyBind::parse("<home>").unwrap();
    bindings.select_next = KeyBind::parse("<end>").unwrap();

    let event = KeyEvent::new(KeyCode::Home, KeyModifiers::empty());
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::PreviousItem)
    );

    let event = KeyEvent::new(KeyCode::End, KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::NextItem));
}

#[test]
fn test_handle_key_with_page_up_down() {
    let mut bindings = create_test_bindings();
    bindings.scroll_preview_up = KeyBind::parse("<pageup>").unwrap();
    bindings.scroll_preview_down = KeyBind::parse("<pagedown>").unwrap();

    let event = KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty());
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::ScrollPreviewUp)
    );

    let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty());
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::ScrollPreviewDown)
    );
}

// ============================================================================
// Modifier Combination Tests
// ============================================================================

#[test]
fn test_handle_key_with_shift_modifier() {
    let mut bindings = create_test_bindings();
    bindings.select_next = KeyBind::parse("<S-j>").unwrap();

    let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::SHIFT);
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::NextItem));
}

#[test]
fn test_handle_key_with_alt_modifier() {
    let mut bindings = create_test_bindings();
    bindings.toggle_preview = KeyBind::parse("<A-p>").unwrap();

    let event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::ALT);
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::TogglePreview)
    );
}

#[test]
fn test_handle_key_with_multiple_modifiers() {
    let mut bindings = create_test_bindings();
    bindings.confirm = KeyBind::parse("<C-S-enter>").unwrap();

    let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Confirm));
}

// ============================================================================
// Binding Priority Tests (First Match Wins)
// ============================================================================

#[test]
fn test_handle_key_first_match_wins() {
    // If two bindings map to same key, first check wins
    let bindings = ParsedKeyBindings {
        back: KeyBind::parse("q").unwrap(),
        select_previous: KeyBind::parse("<up>").unwrap(),
        select_next: KeyBind::parse("<down>").unwrap(),
        scroll_preview_up: KeyBind::parse("<C-u>").unwrap(),
        scroll_preview_down: KeyBind::parse("<C-d>").unwrap(),
        toggle_preview: KeyBind::parse("p").unwrap(),
        confirm: KeyBind::parse("q").unwrap(), // Duplicate of back!
        select: KeyBind::parse("<tab>").unwrap(),
    };

    // 'q' should map to Back (checked first), not Confirm
    let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Back));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_handle_key_case_sensitive() {
    let mut bindings = create_test_bindings();
    bindings.back = KeyBind::parse("q").unwrap();
    bindings.confirm = KeyBind::parse("Q").unwrap();

    // Lowercase 'q' should map to back
    let event_lower = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
    assert_eq!(handle_key(&event_lower, &bindings), Some(InputEvent::Back));

    // Uppercase 'Q' (char 'q' with SHIFT) should map to confirm
    let event_upper = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::SHIFT);
    assert_eq!(
        handle_key(&event_upper, &bindings),
        Some(InputEvent::Confirm)
    );
}

#[test]
fn test_handle_key_space_character() {
    let mut bindings = create_test_bindings();
    bindings.select = KeyBind::parse("<space>").unwrap();

    let event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Select));
}

#[test]
fn test_handle_key_symbols() {
    let mut bindings = create_test_bindings();
    bindings.back = KeyBind::parse("?").unwrap();
    bindings.confirm = KeyBind::parse("/").unwrap();

    let event = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Back));

    let event = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::Confirm));
}

// ============================================================================
// Real-World Vim-Like Bindings
// ============================================================================

#[test]
fn test_handle_key_vim_navigation() {
    let bindings = ParsedKeyBindings {
        back: KeyBind::parse("<esc>").unwrap(),
        select_previous: KeyBind::parse("k").unwrap(),
        select_next: KeyBind::parse("j").unwrap(),
        scroll_preview_up: KeyBind::parse("<C-u>").unwrap(),
        scroll_preview_down: KeyBind::parse("<C-d>").unwrap(),
        toggle_preview: KeyBind::parse("p").unwrap(),
        confirm: KeyBind::parse("<enter>").unwrap(),
        select: KeyBind::parse("<space>").unwrap(),
    };

    // Test j/k navigation
    let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
    assert_eq!(handle_key(&event, &bindings), Some(InputEvent::NextItem));

    let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::PreviousItem)
    );

    // Test Ctrl-u/Ctrl-d for scrolling
    let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::ScrollPreviewUp)
    );

    let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_eq!(
        handle_key(&event, &bindings),
        Some(InputEvent::ScrollPreviewDown)
    );
}
