//! Integration tests for colors loading and config integration
//!
//! Tests the TOML → Colors → ColorStyle pipeline end-to-end.

use ratatui::style::Color;
use syntropy::configs::style::colors::Colors;
use syntropy::tui::views::style::colors::ColorStyle;

// ============================================================================
// Config Templates
// ============================================================================

const MINIMAL_COLORS_CONFIG: &str = r##"
# Empty colors config
"##;

const BASIC_COLORS_CONFIG: &str = r##"
highlights_text = "blue"
highlights_background = "cyan"
borders = "cyan"
text = "white"
background = "black"
"##;

const FULL_COLORS_CONFIG: &str = r##"
highlights_text = "black"
highlights_background = "yellow"
borders = "#00ff00"
borders_list = "red"
borders_preview = "#ff00ff"
borders_search = "blue"
borders_status = "green"
borders_modal = "cyan"
text = "white"
text_list = "#ffffff"
text_preview = "lightblue"
text_search = "lightyellow"
text_status = "lightgreen"
text_modal = "lightred"
background = "black"
background_list = "#000000"
background_preview = "darkgray"
background_search = "gray"
background_status = "#1a1a1a"
background_modal = "darkgrey"
"##;

const INVALID_HEX_CONFIG: &str = r##"
borders = "#xyz"
"##;

const INVALID_COLOR_NAME_CONFIG: &str = r##"
text = "chartreuse"
"##;

const MIXED_VALID_INVALID_CONFIG: &str = r##"
borders = "cyan"
text = "notacolor"
"##;

// ============================================================================
// Basic Config Loading
// ============================================================================

#[test]
fn test_load_empty_colors_section() {
    let config_toml = MINIMAL_COLORS_CONFIG;
    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");

    // Empty section should use defaults (all "terminal")
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    // All fields should be Color::Reset (terminal default)
    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Reset);
    assert_eq!(style.borders, Color::Reset);
    assert_eq!(style.text, Color::Reset);
    assert_eq!(style.background, Color::Reset);
}

#[test]
fn test_load_basic_colors_config() {
    let config_toml = BASIC_COLORS_CONFIG;
    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");

    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    assert_eq!(style.highlights_text, Color::Blue);
    assert_eq!(style.highlights_background, Color::Cyan);
    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(style.text, Color::White);
    assert_eq!(style.background, Color::Black);
}

#[test]
fn test_load_full_colors_config() {
    let config_toml = FULL_COLORS_CONFIG;
    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");

    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    // Verify a sample of fields
    assert_eq!(style.highlights_text, Color::Black);
    assert_eq!(style.highlights_background, Color::Yellow);
    assert_eq!(style.borders, Color::Rgb(0, 255, 0));
    assert_eq!(style.borders_list, Color::Red);
    assert_eq!(style.borders_preview, Color::Rgb(255, 0, 255));
    assert_eq!(style.text, Color::White);
    assert_eq!(style.text_list, Color::Rgb(255, 255, 255));
    assert_eq!(style.background, Color::Black);
    assert_eq!(style.background_list, Color::Rgb(0, 0, 0));
}

// ============================================================================
// Partial Configuration (Common Use Case)
// ============================================================================

// Description: Omitted component fields should cascade to global values
#[test]
fn test_partial_config_with_only_global_fields() {
    let config_toml = r##"
borders = "cyan"
text = "yellow"
background = "#1a1a1a"
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    // Global fields
    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(style.text, Color::Yellow);
    assert_eq!(style.background, Color::Rgb(26, 26, 26));

    // Component fields should cascade (DESIRED behavior)
    assert_eq!(
        style.borders_list,
        Color::Cyan,
        "DESIRED: Component should cascade to global"
    );
    assert_eq!(
        style.text_list,
        Color::Yellow,
        "DESIRED: Component should cascade to global"
    );
    assert_eq!(
        style.background_list,
        Color::Rgb(26, 26, 26),
        "DESIRED: Component should cascade to global"
    );
}

#[test]
fn test_partial_config_with_selective_overrides() {
    let config_toml = r##"
borders = "cyan"
borders_status = "red"
text = "white"
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(
        style.borders_list,
        Color::Cyan,
        "DESIRED: Omitted field should cascade"
    );
    assert_eq!(
        style.borders_status,
        Color::Red,
        "Explicit override should work"
    );
    assert_eq!(
        style.borders_modal,
        Color::Cyan,
        "DESIRED: Omitted field should cascade"
    );
}

// ============================================================================
// Error Propagation
// ============================================================================

#[test]
fn test_invalid_hex_color_fails_conversion() {
    let config_toml = INVALID_HEX_CONFIG;
    let colors: Colors = toml::from_str(config_toml).expect("TOML parsing should succeed");

    let result = ColorStyle::try_from(&colors);

    assert!(
        result.is_err(),
        "Invalid hex color should fail ColorStyle conversion"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("#xyz") || err.to_string().contains("format"),
        "Error message should mention invalid hex"
    );
}

#[test]
fn test_invalid_color_name_fails_conversion() {
    let config_toml = INVALID_COLOR_NAME_CONFIG;
    let colors: Colors = toml::from_str(config_toml).expect("TOML parsing should succeed");

    let result = ColorStyle::try_from(&colors);

    assert!(
        result.is_err(),
        "Invalid color name should fail ColorStyle conversion"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("chartreuse") || err.to_string().contains("format"),
        "Error message should mention invalid color"
    );
}

#[test]
fn test_mixed_valid_invalid_fails_fast() {
    let config_toml = MIXED_VALID_INVALID_CONFIG;
    let colors: Colors = toml::from_str(config_toml).expect("TOML parsing should succeed");

    let result = ColorStyle::try_from(&colors);

    assert!(result.is_err(), "Should fail on first invalid color");
    // Note: Exact error depends on parse order, but should mention invalid color
}

// ============================================================================
// Backwards Compatibility
// ============================================================================

#[test]
fn test_config_without_colors_section() {
    // Config completely omits [styles.colors]
    let _config_toml = r##"
[keybindings]
quit = "q"
"##;

    // When deserializing a full Config struct, colors should use default
    // For this test, we'll just test Colors directly with default
    let colors = Colors::default();
    let style = ColorStyle::try_from(&colors).expect("Default colors should convert successfully");

    // All fields should be terminal default
    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Reset);
    assert_eq!(style.borders, Color::Reset);
    assert_eq!(style.text, Color::Reset);
    assert_eq!(style.background, Color::Reset);
}

#[test]
fn test_empty_colors_section_uses_defaults() {
    let config_toml = r##"
# Empty section
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    // Should use all defaults (terminal)
    assert_eq!(style.highlights_text, Color::Reset);
    assert_eq!(style.highlights_background, Color::Reset);
    assert_eq!(style.borders, Color::Reset);
    assert_eq!(style.borders_list, Color::Reset);
    assert_eq!(style.text, Color::Reset);
    assert_eq!(style.text_list, Color::Reset);
    assert_eq!(style.background, Color::Reset);
    assert_eq!(style.background_list, Color::Reset);
}

#[test]
fn test_explicit_terminal_keyword_in_config() {
    let config_toml = r##"
borders = "terminal"
text = "terminal"
background = "terminal"
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    assert_eq!(style.borders, Color::Reset);
    assert_eq!(style.text, Color::Reset);
    assert_eq!(style.background, Color::Reset);
}

// ============================================================================
// Case Sensitivity and Formatting
// ============================================================================

#[test]
fn test_case_insensitive_color_names_in_config() {
    let config_toml = r##"
borders = "CYAN"
text = "Red"
background = "lightBLUE"
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(style.text, Color::Red);
    assert_eq!(style.background, Color::LightBlue);
}

#[test]
fn test_hex_colors_case_insensitive_in_config() {
    let config_toml = r##"
borders = "#FFFFFF"
text = "#ff00ff"
background = "#FfFfFf"
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    assert_eq!(style.borders, Color::Rgb(255, 255, 255));
    assert_eq!(style.text, Color::Rgb(255, 0, 255));
    assert_eq!(style.background, Color::Rgb(255, 255, 255));
}

#[test]
fn test_grey_gray_spelling_variants_in_config() {
    let config_toml = r##"
borders = "grey"
text = "gray"
background = "darkgrey"
highlights_text = "gray"
highlights_background = "darkgray"
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    assert_eq!(style.borders, Color::Gray);
    assert_eq!(style.text, Color::Gray);
    assert_eq!(style.background, Color::DarkGray);
    assert_eq!(style.highlights_text, Color::Gray);
    assert_eq!(style.highlights_background, Color::DarkGray);
}

// ============================================================================
// Complex Real-World Configs
// ============================================================================

#[test]
fn test_real_world_minimal_user_config() {
    // Typical user: sets 4 global fields, expects cascade
    let config_toml = r##"
highlights_text = "white"
highlights_background = "blue"
borders = "cyan"
text = "white"
background = "black"
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    // Global fields
    assert_eq!(style.highlights_text, Color::White);
    assert_eq!(style.highlights_background, Color::Blue);
    assert_eq!(style.borders, Color::Cyan);
    assert_eq!(style.text, Color::White);
    assert_eq!(style.background, Color::Black);

    // All components should inherit (DESIRED behavior)
    assert_eq!(style.borders_list, Color::Cyan, "DESIRED: cascade");
    assert_eq!(style.borders_preview, Color::Cyan, "DESIRED: cascade");
    assert_eq!(style.text_list, Color::White, "DESIRED: cascade");
    assert_eq!(style.text_preview, Color::White, "DESIRED: cascade");
    assert_eq!(style.background_list, Color::Black, "DESIRED: cascade");
    assert_eq!(style.background_preview, Color::Black, "DESIRED: cascade");
}

#[test]
fn test_real_world_advanced_user_config() {
    // Advanced user: hex colors + selective overrides + explicit empty strings
    let config_toml = r##"
highlights_text = "white"
highlights_background = "blue"
borders = "#336699"
text = "white"
background = "#000000"
borders_status = "yellow"
text_status = "green"
borders_list = ""
text_list = ""
background_list = ""
"##;

    let colors: Colors = toml::from_str(config_toml).expect("Failed to parse TOML");
    let style = ColorStyle::try_from(&colors).expect("Failed to convert to ColorStyle");

    // Global hex colors
    assert_eq!(style.borders, Color::Rgb(51, 102, 153));
    assert_eq!(style.background, Color::Rgb(0, 0, 0));

    // Explicit overrides
    assert_eq!(style.borders_status, Color::Yellow);
    assert_eq!(style.text_status, Color::Green);

    // Empty strings trigger cascade
    assert_eq!(
        style.borders_list,
        Color::Rgb(51, 102, 153),
        "Empty string should cascade to borders"
    );
    assert_eq!(
        style.text_list,
        Color::White,
        "Empty string should cascade to text"
    );
    assert_eq!(
        style.background_list,
        Color::Rgb(0, 0, 0),
        "Empty string should cascade to background"
    );
}

// ============================================================================
// Integration with TestFixture
// ============================================================================

// Note: TestFixture integration is already tested in other integration test files.
// Colors are parsed via TOML deserialization which is thoroughly tested above.
