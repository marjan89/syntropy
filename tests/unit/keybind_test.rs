use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use syntropy::tui::key_bindings::keybind::{KeyBind, ParseError};

// ============================================================================
// Simple Character Tests
// ============================================================================

#[test]
fn test_simple_lowercase() {
    let kb = KeyBind::parse("v").unwrap();
    assert_eq!(kb.code, KeyCode::Char('v'));
    assert_eq!(kb.modifiers, KeyModifiers::empty());
}

#[test]
fn test_simple_uppercase() {
    // Uppercase letters are normalized to lowercase + SHIFT
    // "D" is equivalent to "<S-d>"
    let kb = KeyBind::parse("D").unwrap();
    assert_eq!(kb.code, KeyCode::Char('d'));
    assert_eq!(kb.modifiers, KeyModifiers::SHIFT);
}

#[test]
fn test_simple_digit() {
    let kb = KeyBind::parse("1").unwrap();
    assert_eq!(kb.code, KeyCode::Char('1'));
    assert_eq!(kb.modifiers, KeyModifiers::empty());
}

#[test]
fn test_simple_symbol() {
    let kb = KeyBind::parse(":").unwrap();
    assert_eq!(kb.code, KeyCode::Char(':'));
    assert_eq!(kb.modifiers, KeyModifiers::empty());
}

// ============================================================================
// Special Keys Tests
// ============================================================================

#[test]
fn test_special_space() {
    let kb = KeyBind::parse("<space>").unwrap();
    assert_eq!(kb.code, KeyCode::Char(' '));
    assert_eq!(kb.modifiers, KeyModifiers::empty());
}

#[test]
fn test_special_enter() {
    let kb = KeyBind::parse("<enter>").unwrap();
    assert_eq!(kb.code, KeyCode::Enter);

    let kb = KeyBind::parse("<return>").unwrap();
    assert_eq!(kb.code, KeyCode::Enter);
}

#[test]
fn test_special_escape() {
    let kb = KeyBind::parse("<esc>").unwrap();
    assert_eq!(kb.code, KeyCode::Esc);

    let kb = KeyBind::parse("<escape>").unwrap();
    assert_eq!(kb.code, KeyCode::Esc);
}

#[test]
fn test_special_tab() {
    let kb = KeyBind::parse("<tab>").unwrap();
    assert_eq!(kb.code, KeyCode::Tab);
}

#[test]
fn test_special_backspace() {
    let kb = KeyBind::parse("<backspace>").unwrap();
    assert_eq!(kb.code, KeyCode::Backspace);

    let kb = KeyBind::parse("<bs>").unwrap();
    assert_eq!(kb.code, KeyCode::Backspace);
}

#[test]
fn test_special_delete() {
    let kb = KeyBind::parse("<delete>").unwrap();
    assert_eq!(kb.code, KeyCode::Delete);

    let kb = KeyBind::parse("<del>").unwrap();
    assert_eq!(kb.code, KeyCode::Delete);
}

#[test]
fn test_special_arrows() {
    let kb = KeyBind::parse("<up>").unwrap();
    assert_eq!(kb.code, KeyCode::Up);

    let kb = KeyBind::parse("<down>").unwrap();
    assert_eq!(kb.code, KeyCode::Down);

    let kb = KeyBind::parse("<left>").unwrap();
    assert_eq!(kb.code, KeyCode::Left);

    let kb = KeyBind::parse("<right>").unwrap();
    assert_eq!(kb.code, KeyCode::Right);
}

#[test]
fn test_special_home_end() {
    let kb = KeyBind::parse("<home>").unwrap();
    assert_eq!(kb.code, KeyCode::Home);

    let kb = KeyBind::parse("<end>").unwrap();
    assert_eq!(kb.code, KeyCode::End);
}

#[test]
fn test_special_page_keys() {
    let kb = KeyBind::parse("<pageup>").unwrap();
    assert_eq!(kb.code, KeyCode::PageUp);

    let kb = KeyBind::parse("<pgup>").unwrap();
    assert_eq!(kb.code, KeyCode::PageUp);

    let kb = KeyBind::parse("<pagedown>").unwrap();
    assert_eq!(kb.code, KeyCode::PageDown);

    let kb = KeyBind::parse("<pgdn>").unwrap();
    assert_eq!(kb.code, KeyCode::PageDown);
}

#[test]
fn test_special_function_keys() {
    let kb = KeyBind::parse("<f1>").unwrap();
    assert_eq!(kb.code, KeyCode::F(1));

    let kb = KeyBind::parse("<f5>").unwrap();
    assert_eq!(kb.code, KeyCode::F(5));

    let kb = KeyBind::parse("<f12>").unwrap();
    assert_eq!(kb.code, KeyCode::F(12));
}

// ============================================================================
// Single Modifier Tests
// ============================================================================

#[test]
fn test_ctrl_modifier() {
    let kb = KeyBind::parse("<C-k>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('k'));
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL);

    let kb = KeyBind::parse("<ctrl-x>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('x'));
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL);

    let kb = KeyBind::parse("<Ctrl-z>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('z'));
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL);
}

#[test]
fn test_shift_modifier() {
    let kb = KeyBind::parse("<S-a>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('a'));
    assert_eq!(kb.modifiers, KeyModifiers::SHIFT);

    let kb = KeyBind::parse("<shift-b>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('b'));
    assert_eq!(kb.modifiers, KeyModifiers::SHIFT);
}

#[test]
fn test_alt_modifier() {
    let kb = KeyBind::parse("<A-x>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('x'));
    assert_eq!(kb.modifiers, KeyModifiers::ALT);

    let kb = KeyBind::parse("<alt-y>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('y'));
    assert_eq!(kb.modifiers, KeyModifiers::ALT);
}

// ============================================================================
// Multiple Modifier Tests
// ============================================================================

#[test]
fn test_ctrl_shift() {
    let kb = KeyBind::parse("<C-S-k>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('k'));
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL | KeyModifiers::SHIFT);

    let kb = KeyBind::parse("<S-C-k>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('k'));
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL | KeyModifiers::SHIFT);
}

#[test]
fn test_ctrl_alt() {
    let kb = KeyBind::parse("<C-A-x>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('x'));
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL | KeyModifiers::ALT);
}

#[test]
fn test_shift_alt() {
    let kb = KeyBind::parse("<S-A-space>").unwrap();
    assert_eq!(kb.code, KeyCode::Char(' '));
    assert_eq!(kb.modifiers, KeyModifiers::SHIFT | KeyModifiers::ALT);
}

#[test]
fn test_all_modifiers() {
    let kb = KeyBind::parse("<C-S-A-k>").unwrap();
    assert_eq!(kb.code, KeyCode::Char('k'));
    assert_eq!(
        kb.modifiers,
        KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT
    );
}

// ============================================================================
// Modifiers with Special Keys
// ============================================================================

#[test]
fn test_ctrl_special_keys() {
    let kb = KeyBind::parse("<C-space>").unwrap();
    assert_eq!(kb.code, KeyCode::Char(' '));
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL);

    let kb = KeyBind::parse("<C-enter>").unwrap();
    assert_eq!(kb.code, KeyCode::Enter);
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL);

    let kb = KeyBind::parse("<C-up>").unwrap();
    assert_eq!(kb.code, KeyCode::Up);
    assert_eq!(kb.modifiers, KeyModifiers::CONTROL);
}

#[test]
fn test_shift_arrows() {
    let kb = KeyBind::parse("<S-up>").unwrap();
    assert_eq!(kb.code, KeyCode::Up);
    assert_eq!(kb.modifiers, KeyModifiers::SHIFT);

    let kb = KeyBind::parse("<S-down>").unwrap();
    assert_eq!(kb.code, KeyCode::Down);
    assert_eq!(kb.modifiers, KeyModifiers::SHIFT);
}

// ============================================================================
// Case Insensitivity Tests
// ============================================================================

#[test]
fn test_case_insensitive_keys() {
    let kb1 = KeyBind::parse("<SPACE>").unwrap();
    let kb2 = KeyBind::parse("<space>").unwrap();
    assert_eq!(kb1.code, kb2.code);

    let kb1 = KeyBind::parse("<ESC>").unwrap();
    let kb2 = KeyBind::parse("<esc>").unwrap();
    assert_eq!(kb1.code, kb2.code);

    let kb1 = KeyBind::parse("<ENTER>").unwrap();
    let kb2 = KeyBind::parse("<enter>").unwrap();
    assert_eq!(kb1.code, kb2.code);
}

#[test]
fn test_case_insensitive_modifiers() {
    let kb1 = KeyBind::parse("<C-k>").unwrap();
    let kb2 = KeyBind::parse("<c-k>").unwrap();
    assert_eq!(kb1, kb2);

    let kb1 = KeyBind::parse("<CTRL-k>").unwrap();
    let kb2 = KeyBind::parse("<ctrl-k>").unwrap();
    assert_eq!(kb1, kb2);
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_error_empty() {
    let result = KeyBind::parse("");
    assert!(matches!(result, Err(ParseError::Empty)));
}

#[test]
fn test_error_whitespace() {
    let result = KeyBind::parse("   ");
    assert!(matches!(result, Err(ParseError::Empty)));
}

#[test]
fn test_error_multiple_chars() {
    let result = KeyBind::parse("abc");
    assert!(matches!(result, Err(ParseError::InvalidFormat(_))));
}

#[test]
fn test_error_unknown_modifier() {
    let result = KeyBind::parse("<X-k>");
    assert!(matches!(result, Err(ParseError::UnknownModifier(_))));

    let result = KeyBind::parse("<Meta-k>");
    assert!(matches!(result, Err(ParseError::UnknownModifier(_))));
}

#[test]
fn test_error_unknown_key() {
    let result = KeyBind::parse("<unknown>");
    assert!(matches!(result, Err(ParseError::UnknownKey(_))));

    let result = KeyBind::parse("<f13>");
    assert!(matches!(result, Err(ParseError::UnknownKey(_))));
}

#[test]
fn test_error_incomplete_brackets() {
    let result = KeyBind::parse("<space");
    assert!(result.is_err());

    let result = KeyBind::parse("space>");
    assert!(result.is_err());
}

// ============================================================================
// Match Tests
// ============================================================================

#[test]
fn test_matches_simple() {
    let kb = KeyBind::parse("v").unwrap();
    assert!(kb.matches(&KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty())));
    assert!(!kb.matches(&KeyEvent::new(KeyCode::Char('V'), KeyModifiers::empty())));
    assert!(!kb.matches(&KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL)));
}

#[test]
fn test_matches_with_modifiers() {
    let kb = KeyBind::parse("<C-k>").unwrap();
    assert!(kb.matches(&KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)));
    assert!(!kb.matches(&KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty())));
    assert!(!kb.matches(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL)));
    assert!(!kb.matches(&KeyEvent::new(
        KeyCode::Char('k'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT
    )));
}

#[test]
fn test_matches_multiple_modifiers() {
    let kb = KeyBind::parse("<C-S-k>").unwrap();
    assert!(kb.matches(&KeyEvent::new(
        KeyCode::Char('k'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT
    )));
    assert!(!kb.matches(&KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)));
    assert!(!kb.matches(&KeyEvent::new(KeyCode::Char('k'), KeyModifiers::SHIFT)));
}

#[test]
fn test_matches_event() {
    let kb = KeyBind::parse("<C-k>").unwrap();
    let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    assert!(kb.matches(&event));

    let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
    assert!(!kb.matches(&event));
}

// ============================================================================
// Uppercase Letters with Shift Modifier Tests
// ============================================================================
// NOTE: These tests validate a normalization workaround in the KeyBind system
// Parser behavior: "K" → KeyCode::Char('k') + SHIFT (stores lowercase)
// Terminal behavior: Shift+K → KeyCode::Char('K') + SHIFT (uppercase KeyCode)
// Matcher workaround: matches() normalizes uppercase chars to lowercase before comparison
// This inconsistency works in practice but may be confusing. Consider refactoring parser
// to store uppercase KeyCode for uppercase letters instead of lowercase + SHIFT modifier.

#[test]
fn test_uppercase_k_matches_shift_k_event() {
    // When user types Shift+k, terminal sends 'K' with SHIFT modifier
    let kb = KeyBind::parse("K").unwrap();
    let event = KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT);
    assert!(
        kb.matches(&event),
        "Uppercase 'K' should match Shift+K event"
    );
}

// TODO: Case normalization workaround - see section comment above
#[test]
fn test_shift_lowercase_k_matches_shift_k_event() {
    // When user types Shift+k, terminal sends 'K' with SHIFT modifier
    let kb = KeyBind::parse("<S-k>").unwrap();
    let event = KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT);
    assert!(kb.matches(&event), "<S-k> should match Shift+K event");
}

// TODO: Case normalization workaround - see section comment above
#[test]
fn test_uppercase_j_matches_shift_j_event() {
    let kb = KeyBind::parse("J").unwrap();
    let event = KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT);
    assert!(
        kb.matches(&event),
        "Uppercase 'J' should match Shift+J event"
    );
}

// TODO: Case normalization workaround - see section comment above
#[test]
fn test_shift_lowercase_j_matches_shift_j_event() {
    let kb = KeyBind::parse("<S-j>").unwrap();
    let event = KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT);
    assert!(kb.matches(&event), "<S-j> should match Shift+J event");
}

#[test]
fn test_lowercase_k_does_not_match_shift_k_event() {
    // Lowercase 'k' should NOT match when shift is pressed
    let kb = KeyBind::parse("k").unwrap();
    let event = KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT);
    assert!(
        !kb.matches(&event),
        "Lowercase 'k' should not match Shift+K event"
    );
}

// ============================================================================
// Edge Case Tests - Parser Validation
// ============================================================================

// Regression test: Bracketed notation should only be for special keys and modifiers
#[test]
fn test_parse_error_bracketed_simple_char() {
    // Simple characters like 'k' should be written without brackets
    let result = KeyBind::parse("<k>");
    assert!(
        result.is_err(),
        "Simple character in brackets should be invalid"
    );
    match result {
        Err(ParseError::UnknownKey(key)) => {
            assert_eq!(key, "k", "Error should indicate unknown key 'k'");
        }
        _ => panic!("Expected ParseError::UnknownKey, got {:?}", result),
    }
}

// Regression test: Uppercase in brackets should be treated as lowercase + SHIFT
#[test]
fn test_parse_uppercase_in_brackets() {
    // <C-K> (uppercase K) should be equivalent to <C-S-k>
    let result_upper = KeyBind::parse("<C-K>");
    let result_explicit = KeyBind::parse("<C-S-k>");

    // Both should parse successfully
    assert!(
        result_upper.is_ok(),
        "Uppercase in brackets should be valid: {:?}",
        result_upper
    );
    assert!(
        result_explicit.is_ok(),
        "Explicit shift notation should be valid"
    );

    // Both should create equivalent keybinds
    let kb_upper = result_upper.unwrap();
    let kb_explicit = result_explicit.unwrap();

    assert_eq!(
        kb_upper.code, kb_explicit.code,
        "Both should have same KeyCode"
    );
    assert_eq!(
        kb_upper.modifiers, kb_explicit.modifiers,
        "Both should have same modifiers (CTRL + SHIFT)"
    );
}

#[test]
fn test_parse_error_empty_brackets() {
    // Empty bracketed notation should be rejected
    let result = KeyBind::parse("<>");
    assert!(result.is_err(), "Empty brackets should be invalid");

    // Parser returns UnknownKey("") instead of Empty
    // Both are acceptable error variants for this case
    match result {
        Err(ParseError::Empty) => {
            // Expected error type
        }
        Err(ParseError::UnknownKey(s)) if s.is_empty() => {
            // Also acceptable (current behavior)
        }
        other => panic!(
            "Expected ParseError::Empty or UnknownKey(\"\"), got {:?}",
            other
        ),
    }
}

#[test]
fn test_parse_error_modifier_without_key() {
    // Modifier without a key should be invalid
    let result = KeyBind::parse("<C->");
    assert!(
        result.is_err(),
        "Modifier without key should be invalid: {:?}",
        result
    );

    // Should return UnknownKey or similar error
    assert!(
        matches!(
            result,
            Err(ParseError::UnknownKey(_)) | Err(ParseError::Empty)
        ),
        "Should return appropriate error for incomplete binding"
    );
}

#[test]
fn test_matches_lowercase_char_with_shift() {
    // Edge case: Test matcher behavior when terminal sends lowercase char WITH shift modifier
    // This is unusual but possible from some terminals or key remapping tools
    let kb = KeyBind::parse("k").unwrap(); // Plain 'k', no modifiers

    // Unusual event: lowercase 'k' but with SHIFT modifier
    let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::SHIFT);

    // Should NOT match because modifiers differ
    // Plain "k" expects empty modifiers, not SHIFT
    assert!(
        !kb.matches(&event),
        "Plain 'k' binding should not match lowercase 'k' with SHIFT modifier"
    );
}
