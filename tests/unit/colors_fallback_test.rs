// Color fallback/cascade tests - TDD tests for desired behavior
// These tests assert CORRECT cascade behavior and will catch bugs in implementation

use ratatui::style::Color;
use syntropy::configs::style::colors::Colors;
use syntropy::tui::views::style::colors::ColorStyle;

// ============================================================================
// Basic Fallback Behavior - Empty String Triggers Cascade
// ============================================================================

#[test]
fn test_empty_string_borders_list_falls_back_to_borders() {
    let colors = Colors {
        borders: "cyan".to_string(),
        borders_list: "".to_string(), // Empty string triggers fallback
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(
        style.borders_list,
        Color::Cyan,
        "Empty borders_list should cascade to borders value"
    );
}

#[test]
fn test_empty_string_text_list_falls_back_to_text() {
    let colors = Colors {
        text: "yellow".to_string(),
        text_list: "".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.text, Color::Yellow);
    assert_eq!(
        style.text_list,
        Color::Yellow,
        "Empty text_list should cascade to text value"
    );
}

#[test]
fn test_empty_string_background_modal_falls_back_to_background() {
    let colors = Colors {
        background: "#1a1a1a".to_string(),
        background_modal: "".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.background, Color::Rgb(26, 26, 26));
    assert_eq!(
        style.background_modal,
        Color::Rgb(26, 26, 26),
        "Empty background_modal should cascade to background value"
    );
}

#[test]
fn test_empty_string_multiple_components_cascade() {
    let colors = Colors {
        borders: "magenta".to_string(),
        borders_list: "".to_string(),
        borders_preview: "".to_string(),
        borders_search: "".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Magenta);
    assert_eq!(style.borders_list, Color::Magenta);
    assert_eq!(style.borders_preview, Color::Magenta);
    assert_eq!(style.borders_search, Color::Magenta);
}

// ============================================================================
// Omitted Fields - Critical Bug Tests
// ============================================================================

// Regression test: Omitted component fields should cascade to global values
#[test]
fn test_omitted_borders_list_falls_back_to_borders() {
    let colors = Colors {
        borders: "cyan".to_string(),
        // borders_list omitted - relies on Default
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(
        style.borders_list,
        Color::Cyan,
        "DESIRED: Omitted borders_list should cascade to borders value"
    );
}

#[test]
fn test_omitted_text_list_falls_back_to_text() {
    let colors = Colors {
        text: "yellow".to_string(),
        // text_list omitted
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.text, Color::Yellow);
    assert_eq!(
        style.text_list,
        Color::Yellow,
        "DESIRED: Omitted text_list should cascade to text value"
    );
}

#[test]
fn test_omitted_background_preview_falls_back_to_background() {
    let colors = Colors {
        background: "#1a1a1a".to_string(),
        // background_preview omitted
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.background, Color::Rgb(26, 26, 26));
    assert_eq!(
        style.background_preview,
        Color::Rgb(26, 26, 26),
        "DESIRED: Omitted background_preview should cascade to background"
    );
}

#[test]
fn test_omitted_all_component_fields_cascade_to_global() {
    let colors = Colors {
        borders: "red".to_string(),
        // All borders_* fields omitted
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Red);
    assert_eq!(
        style.borders_list,
        Color::Red,
        "DESIRED: cascade from borders"
    );
    assert_eq!(
        style.borders_preview,
        Color::Red,
        "DESIRED: cascade from borders"
    );
    assert_eq!(
        style.borders_search,
        Color::Red,
        "DESIRED: cascade from borders"
    );
    assert_eq!(
        style.borders_status,
        Color::Red,
        "DESIRED: cascade from borders"
    );
    assert_eq!(
        style.borders_modal,
        Color::Red,
        "DESIRED: cascade from borders"
    );
}

// ============================================================================
// Explicit Terminal Keyword - Prevents Cascade
// ============================================================================

#[test]
fn test_explicit_terminal_keyword_uses_terminal_default() {
    let colors = Colors {
        borders: "cyan".to_string(),
        borders_list: "terminal".to_string(), // Explicit terminal keyword
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(
        style.borders_list,
        Color::Reset,
        "Explicit 'terminal' should use terminal default, not cascade"
    );
}

#[test]
fn test_explicit_terminal_uppercase() {
    let colors = Colors {
        text: "yellow".to_string(),
        text_list: "TERMINAL".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.text, Color::Yellow);
    assert_eq!(style.text_list, Color::Reset);
}

#[test]
fn test_explicit_terminal_mixed_case() {
    let colors = Colors {
        background: "#000000".to_string(),
        background_modal: "Terminal".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.background, Color::Rgb(0, 0, 0));
    assert_eq!(style.background_modal, Color::Reset);
}

// ============================================================================
// Three-Level Cascade - All Components
// ============================================================================

#[test]
fn test_all_borders_components_cascade_from_global() {
    let colors = Colors {
        borders: "magenta".to_string(),
        // All component fields omitted
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    // All border components should cascade from global
    assert_eq!(style.borders, Color::Magenta);
    assert_eq!(style.borders_list, Color::Magenta);
    assert_eq!(style.borders_preview, Color::Magenta);
    assert_eq!(style.borders_search, Color::Magenta);
    assert_eq!(style.borders_status, Color::Magenta);
    assert_eq!(style.borders_modal, Color::Magenta);
}

#[test]
fn test_all_text_components_cascade_from_global() {
    let colors = Colors {
        text: "yellow".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.text, Color::Yellow);
    assert_eq!(style.text_list, Color::Yellow);
    assert_eq!(style.text_preview, Color::Yellow);
    assert_eq!(style.text_search, Color::Yellow);
    assert_eq!(style.text_status, Color::Yellow);
    assert_eq!(style.text_modal, Color::Yellow);
}

#[test]
fn test_all_background_components_cascade_from_global() {
    let colors = Colors {
        background: "#1a1a1a".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.background, Color::Rgb(26, 26, 26));
    assert_eq!(style.background_list, Color::Rgb(26, 26, 26));
    assert_eq!(style.background_preview, Color::Rgb(26, 26, 26));
    assert_eq!(style.background_search, Color::Rgb(26, 26, 26));
    assert_eq!(style.background_status, Color::Rgb(26, 26, 26));
    assert_eq!(style.background_modal, Color::Rgb(26, 26, 26));
}

#[test]
fn test_cascade_works_with_hex_colors() {
    let colors = Colors {
        borders: "#00ff00".to_string(),
        text: "#ff0000".to_string(),
        background: "#0000ff".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    // Borders cascade
    assert_eq!(style.borders, Color::Rgb(0, 255, 0));
    assert_eq!(style.borders_list, Color::Rgb(0, 255, 0));
    assert_eq!(style.borders_preview, Color::Rgb(0, 255, 0));

    // Text cascade
    assert_eq!(style.text, Color::Rgb(255, 0, 0));
    assert_eq!(style.text_list, Color::Rgb(255, 0, 0));
    assert_eq!(style.text_preview, Color::Rgb(255, 0, 0));

    // Background cascade
    assert_eq!(style.background, Color::Rgb(0, 0, 255));
    assert_eq!(style.background_list, Color::Rgb(0, 0, 255));
    assert_eq!(style.background_preview, Color::Rgb(0, 0, 255));
}

// ============================================================================
// Selective Override Pattern
// ============================================================================

#[test]
fn test_selective_override_one_component() {
    let colors = Colors {
        borders: "cyan".to_string(),
        borders_status: "red".to_string(), // Override just status
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    // Status overridden, others cascade
    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(style.borders_list, Color::Cyan, "Should cascade");
    assert_eq!(style.borders_preview, Color::Cyan, "Should cascade");
    assert_eq!(style.borders_search, Color::Cyan, "Should cascade");
    assert_eq!(style.borders_status, Color::Red, "Explicitly overridden");
    assert_eq!(style.borders_modal, Color::Cyan, "Should cascade");
}

#[test]
fn test_selective_override_multiple_components() {
    let colors = Colors {
        text: "white".to_string(),
        text_status: "green".to_string(),
        text_modal: "yellow".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.text, Color::White);
    assert_eq!(style.text_list, Color::White, "Cascade");
    assert_eq!(style.text_preview, Color::White, "Cascade");
    assert_eq!(style.text_search, Color::White, "Cascade");
    assert_eq!(style.text_status, Color::Green, "Override");
    assert_eq!(style.text_modal, Color::Yellow, "Override");
}

#[test]
fn test_selective_override_with_empty_strings() {
    let colors = Colors {
        borders: "cyan".to_string(),
        borders_list: "".to_string(),      // Empty = cascade
        borders_status: "red".to_string(), // Override
        borders_modal: "".to_string(),     // Empty = cascade
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(style.borders_list, Color::Cyan, "Empty triggers cascade");
    assert_eq!(style.borders_status, Color::Red, "Explicit override");
    assert_eq!(style.borders_modal, Color::Cyan, "Empty triggers cascade");
}

// ============================================================================
// Mixing Formats - Hex, Named, Terminal
// ============================================================================

#[test]
fn test_mixing_hex_and_named_colors_with_empty_fallback() {
    let colors = Colors {
        borders: "#00ff00".to_string(),
        text: "cyan".to_string(),
        background: "terminal".to_string(),
        borders_list: "".to_string(),    // Should fallback to hex
        text_list: "".to_string(),       // Should fallback to named
        background_list: "".to_string(), // Should fallback to terminal
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Rgb(0, 255, 0));
    assert_eq!(style.text, Color::Cyan);
    assert_eq!(style.background, Color::Reset);
    assert_eq!(
        style.borders_list,
        Color::Rgb(0, 255, 0),
        "Hex fallback works"
    );
    assert_eq!(style.text_list, Color::Cyan, "Named fallback works");
    assert_eq!(
        style.background_list,
        Color::Reset,
        "Terminal fallback works"
    );
}

#[test]
fn test_mixing_formats_with_omitted_fields() {
    let colors = Colors {
        borders: "#ff00ff".to_string(),
        text: "yellow".to_string(),
        background: "terminal".to_string(),
        // All component fields omitted
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.borders, Color::Rgb(255, 0, 255));
    assert_eq!(style.borders_list, Color::Rgb(255, 0, 255));

    assert_eq!(style.text, Color::Yellow);
    assert_eq!(style.text_list, Color::Yellow);

    assert_eq!(style.background, Color::Reset);
    assert_eq!(style.background_list, Color::Reset);
}

// ============================================================================
// Standalone Highlights Field
// ============================================================================

#[test]
fn test_highlights_field_standalone_no_fallback() {
    let colors = Colors {
        highlights_text: "blue".to_string(),
        highlights_background: "yellow".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.highlights_text, Color::Blue);
    assert_eq!(style.highlights_background, Color::Yellow);
    // Verify highlights don't affect other fields
    assert_eq!(style.borders, Color::Reset);
    assert_eq!(style.text, Color::Reset);
}

#[test]
fn test_highlights_empty_string_becomes_terminal() {
    let colors = Colors {
        highlights_text: "".to_string(),
        highlights_background: "".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    // Highlights has no fallback chain, so empty → None → unwrap_or(Reset)
    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Reset);
}

// ============================================================================
// Highlights Text and Background Interaction
// ============================================================================

#[test]
fn test_highlights_text_only_without_background() {
    let colors = Colors {
        highlights_text: "blue".to_string(),
        highlights_background: "".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.highlights_text, Color::Blue);
    assert_eq!(style.highlights_background, Color::Reset);
}

#[test]
fn test_highlights_background_only_without_text() {
    let colors = Colors {
        highlights_text: "".to_string(),
        highlights_background: "yellow".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Yellow);
}

#[test]
fn test_highlights_text_and_background_together() {
    let colors = Colors {
        highlights_text: "white".to_string(),
        highlights_background: "blue".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.highlights_text, Color::White);
    assert_eq!(style.highlights_background, Color::Blue);
}

#[test]
fn test_highlights_text_terminal_keyword() {
    let colors = Colors {
        highlights_text: "terminal".to_string(),
        highlights_background: "yellow".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Yellow);
}

#[test]
fn test_highlights_background_terminal_keyword() {
    let colors = Colors {
        highlights_text: "blue".to_string(),
        highlights_background: "terminal".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.highlights_text, Color::Blue);
    assert_eq!(style.highlights_background, Color::Reset);
}

#[test]
fn test_highlights_both_terminal_keyword() {
    let colors = Colors {
        highlights_text: "terminal".to_string(),
        highlights_background: "terminal".to_string(),
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Reset);
}

// ============================================================================
// Error Handling - Invalid Global Field
// ============================================================================

#[test]
fn test_invalid_global_borders_returns_error() {
    let colors = Colors {
        borders: "#xyz".to_string(), // Invalid hex
        ..Default::default()
    };

    let result = ColorStyle::try_from(&colors);

    assert!(result.is_err(), "Invalid global field should fail");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("#xyz"),
        "Error should mention invalid value"
    );
}

#[test]
fn test_invalid_global_text_returns_error() {
    let colors = Colors {
        text: "notacolor".to_string(),
        ..Default::default()
    };

    let result = ColorStyle::try_from(&colors);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("notacolor"));
}

#[test]
fn test_invalid_global_background_returns_error() {
    let colors = Colors {
        background: "#gg0000".to_string(),
        ..Default::default()
    };

    let result = ColorStyle::try_from(&colors);

    assert!(result.is_err());
}

// ============================================================================
// Error Handling - Invalid Component Field
// ============================================================================

#[test]
fn test_invalid_component_field_returns_error() {
    let colors = Colors {
        borders: "cyan".to_string(),
        borders_list: "notacolor".to_string(), // Invalid color name
        ..Default::default()
    };

    let result = ColorStyle::try_from(&colors);

    assert!(
        result.is_err(),
        "Invalid component field should fail (strict validation)"
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("notacolor"));
}

#[test]
fn test_invalid_hex_in_component_field() {
    let colors = Colors {
        text: "white".to_string(),
        text_status: "#12345".to_string(), // Invalid hex (5 chars)
        ..Default::default()
    };

    let result = ColorStyle::try_from(&colors);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("#12345"));
}

// ============================================================================
// Error Handling - Multiple Invalid Fields (Fail Fast)
// ============================================================================

#[test]
fn test_multiple_invalid_fields_fails_on_first() {
    let colors = Colors {
        borders: "#xyz".to_string(), // Invalid (checked first)
        text: "invalid".to_string(), // Also invalid (not reached)
        ..Default::default()
    };

    let result = ColorStyle::try_from(&colors);

    assert!(result.is_err());
    // Fails on first error (borders), doesn't check text
    let err = result.unwrap_err();
    assert!(err.to_string().contains("#xyz"));
}

// ============================================================================
// All Fields Terminal Default
// ============================================================================

#[test]
fn test_all_fields_terminal_keyword() {
    let colors = Colors {
        highlights_text: "terminal".to_string(),
        highlights_background: "terminal".to_string(),
        borders: "terminal".to_string(),
        borders_list: "terminal".to_string(),
        borders_preview: "terminal".to_string(),
        borders_search: "terminal".to_string(),
        borders_status: "terminal".to_string(),
        borders_modal: "terminal".to_string(),
        text: "terminal".to_string(),
        text_list: "terminal".to_string(),
        text_preview: "terminal".to_string(),
        text_search: "terminal".to_string(),
        text_status: "terminal".to_string(),
        text_modal: "terminal".to_string(),
        background: "terminal".to_string(),
        background_list: "terminal".to_string(),
        background_preview: "terminal".to_string(),
        background_search: "terminal".to_string(),
        background_status: "terminal".to_string(),
        background_modal: "terminal".to_string(),
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    // Everything should be Color::Reset
    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Reset);
    assert_eq!(style.borders, Color::Reset);
    assert_eq!(style.borders_list, Color::Reset);
    assert_eq!(style.borders_preview, Color::Reset);
    assert_eq!(style.text, Color::Reset);
    assert_eq!(style.text_list, Color::Reset);
    assert_eq!(style.background, Color::Reset);
    assert_eq!(style.background_list, Color::Reset);
}

// ============================================================================
// Complete Real-World Scenarios
// ============================================================================

#[test]
fn test_minimal_user_config_sets_only_global_fields() {
    // Real-world scenario: User sets only 4 global fields
    let colors = Colors {
        highlights_text: "white".to_string(),
        highlights_background: "blue".to_string(),
        borders: "cyan".to_string(),
        text: "white".to_string(),
        background: "black".to_string(),
        // All component fields omitted - should cascade
        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    // Global fields
    assert_eq!(style.highlights_text, Color::White);
    assert_eq!(style.highlights_background, Color::Blue);
    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(style.text, Color::White);
    assert_eq!(style.background, Color::Black);

    // All borders components should be cyan
    assert_eq!(style.borders_list, Color::Cyan);
    assert_eq!(style.borders_preview, Color::Cyan);
    assert_eq!(style.borders_search, Color::Cyan);
    assert_eq!(style.borders_status, Color::Cyan);
    assert_eq!(style.borders_modal, Color::Cyan);

    // All text components should be white
    assert_eq!(style.text_list, Color::White);
    assert_eq!(style.text_preview, Color::White);
    assert_eq!(style.text_search, Color::White);
    assert_eq!(style.text_status, Color::White);
    assert_eq!(style.text_modal, Color::White);

    // All background components should be black
    assert_eq!(style.background_list, Color::Black);
    assert_eq!(style.background_preview, Color::Black);
    assert_eq!(style.background_search, Color::Black);
    assert_eq!(style.background_status, Color::Black);
    assert_eq!(style.background_modal, Color::Black);
}

#[test]
fn test_power_user_config_with_selective_overrides() {
    let colors = Colors {
        // Global defaults
        borders: "cyan".to_string(),
        text: "white".to_string(),
        background: "black".to_string(),

        // Selective overrides
        borders_status: "yellow".to_string(), // Status bar gets yellow border
        text_status: "green".to_string(),     // Status bar gets green text
        borders_modal: "red".to_string(),     // Modal gets red border
        background_modal: "#1a1a1a".to_string(), // Modal gets dark gray background

        // Rest use empty strings for explicit cascade
        borders_list: "".to_string(),
        borders_preview: "".to_string(),
        borders_search: "".to_string(),
        text_list: "".to_string(),
        text_preview: "".to_string(),
        text_search: "".to_string(),
        text_modal: "".to_string(),
        background_list: "".to_string(),
        background_preview: "".to_string(),
        background_search: "".to_string(),
        background_status: "".to_string(),

        ..Default::default()
    };

    let style = ColorStyle::try_from(&colors).unwrap();

    // Cascaded components
    assert_eq!(style.borders_list, Color::Cyan);
    assert_eq!(style.borders_preview, Color::Cyan);
    assert_eq!(style.borders_search, Color::Cyan);
    assert_eq!(style.text_list, Color::White);
    assert_eq!(style.text_preview, Color::White);

    // Overridden components
    assert_eq!(style.borders_status, Color::Yellow);
    assert_eq!(style.text_status, Color::Green);
    assert_eq!(style.borders_modal, Color::Red);
    assert_eq!(style.background_modal, Color::Rgb(26, 26, 26));
}
