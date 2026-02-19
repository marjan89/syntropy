// Color parsing tests - TDD tests for desired behavior
// These tests assert CORRECT behavior and will catch bugs in implementation

use ratatui::style::Color;
use syntropy::tui::views::style::colors::parse_color;

// ============================================================================
// Named Colors - Basic Tests
// ============================================================================

#[test]
fn test_parse_color_black() {
    let result = parse_color("black").unwrap();
    assert_eq!(result, Some(Color::Black));
}

#[test]
fn test_parse_color_red() {
    let result = parse_color("red").unwrap();
    assert_eq!(result, Some(Color::Red));
}

#[test]
fn test_parse_color_green() {
    let result = parse_color("green").unwrap();
    assert_eq!(result, Some(Color::Green));
}

#[test]
fn test_parse_color_yellow() {
    let result = parse_color("yellow").unwrap();
    assert_eq!(result, Some(Color::Yellow));
}

#[test]
fn test_parse_color_blue() {
    let result = parse_color("blue").unwrap();
    assert_eq!(result, Some(Color::Blue));
}

#[test]
fn test_parse_color_magenta() {
    let result = parse_color("magenta").unwrap();
    assert_eq!(result, Some(Color::Magenta));
}

#[test]
fn test_parse_color_cyan() {
    let result = parse_color("cyan").unwrap();
    assert_eq!(result, Some(Color::Cyan));
}

#[test]
fn test_parse_color_white() {
    let result = parse_color("white").unwrap();
    assert_eq!(result, Some(Color::White));
}

#[test]
fn test_parse_color_gray() {
    let result = parse_color("gray").unwrap();
    assert_eq!(result, Some(Color::Gray));
}

#[test]
fn test_parse_color_grey() {
    let result = parse_color("grey").unwrap();
    assert_eq!(result, Some(Color::Gray));
}

#[test]
fn test_parse_color_darkgray() {
    let result = parse_color("darkgray").unwrap();
    assert_eq!(result, Some(Color::DarkGray));
}

#[test]
fn test_parse_color_darkgrey() {
    let result = parse_color("darkgrey").unwrap();
    assert_eq!(result, Some(Color::DarkGray));
}

// ============================================================================
// Named Colors - Light Variants
// ============================================================================

#[test]
fn test_parse_color_lightred() {
    let result = parse_color("lightred").unwrap();
    assert_eq!(result, Some(Color::LightRed));
}

#[test]
fn test_parse_color_lightgreen() {
    let result = parse_color("lightgreen").unwrap();
    assert_eq!(result, Some(Color::LightGreen));
}

#[test]
fn test_parse_color_lightyellow() {
    let result = parse_color("lightyellow").unwrap();
    assert_eq!(result, Some(Color::LightYellow));
}

#[test]
fn test_parse_color_lightblue() {
    let result = parse_color("lightblue").unwrap();
    assert_eq!(result, Some(Color::LightBlue));
}

#[test]
fn test_parse_color_lightmagenta() {
    let result = parse_color("lightmagenta").unwrap();
    assert_eq!(result, Some(Color::LightMagenta));
}

#[test]
fn test_parse_color_lightcyan() {
    let result = parse_color("lightcyan").unwrap();
    assert_eq!(result, Some(Color::LightCyan));
}

// ============================================================================
// Terminal Keyword
// ============================================================================

#[test]
fn test_parse_color_terminal_lowercase() {
    let result = parse_color("terminal").unwrap();
    assert_eq!(result, Some(Color::Reset));
}

#[test]
fn test_parse_color_terminal_uppercase() {
    let result = parse_color("TERMINAL").unwrap();
    assert_eq!(result, Some(Color::Reset));
}

#[test]
fn test_parse_color_terminal_mixed_case() {
    let result = parse_color("Terminal").unwrap();
    assert_eq!(result, Some(Color::Reset));
}

#[test]
fn test_parse_color_terminal_with_whitespace() {
    let result = parse_color(" terminal ").unwrap();
    assert_eq!(result, Some(Color::Reset));
}

// ============================================================================
// Case Insensitivity
// ============================================================================

#[test]
fn test_parse_color_uppercase_red() {
    let result = parse_color("RED").unwrap();
    assert_eq!(result, Some(Color::Red));
}

#[test]
fn test_parse_color_uppercase_blue() {
    let result = parse_color("BLUE").unwrap();
    assert_eq!(result, Some(Color::Blue));
}

#[test]
fn test_parse_color_mixed_case_red() {
    let result = parse_color("Red").unwrap();
    assert_eq!(result, Some(Color::Red));
}

#[test]
fn test_parse_color_mixed_case_lightred() {
    let result = parse_color("LightRed").unwrap();
    assert_eq!(result, Some(Color::LightRed));
}

#[test]
fn test_parse_color_mixed_case_darkgray() {
    let result = parse_color("DarkGray").unwrap();
    assert_eq!(result, Some(Color::DarkGray));
}

// ============================================================================
// Whitespace Handling
// ============================================================================

#[test]
fn test_parse_color_leading_whitespace() {
    let result = parse_color("  blue").unwrap();
    assert_eq!(result, Some(Color::Blue));
}

#[test]
fn test_parse_color_trailing_whitespace() {
    let result = parse_color("blue  ").unwrap();
    assert_eq!(result, Some(Color::Blue));
}

#[test]
fn test_parse_color_surrounding_whitespace() {
    let result = parse_color("  blue  ").unwrap();
    assert_eq!(result, Some(Color::Blue));
}

#[test]
fn test_parse_color_tab_whitespace() {
    let result = parse_color("\tblue\t").unwrap();
    assert_eq!(result, Some(Color::Blue));
}

#[test]
fn test_parse_color_uppercase_with_whitespace() {
    let result = parse_color(" RED ").unwrap();
    assert_eq!(result, Some(Color::Red));
}

// ============================================================================
// Empty/Whitespace Strings
// ============================================================================

#[test]
fn test_parse_color_empty_string() {
    let result = parse_color("").unwrap();
    assert_eq!(result, None);
}

// Regression test: Whitespace-only strings should be treated as empty
#[test]
fn test_parse_color_whitespace_only_single_space() {
    let result = parse_color(" ").unwrap();
    assert_eq!(result, None, "Single space should be treated as empty");
}

#[test]
fn test_parse_color_whitespace_only_multiple_spaces() {
    let result = parse_color("   ").unwrap();
    assert_eq!(result, None, "Multiple spaces should be treated as empty");
}

#[test]
fn test_parse_color_whitespace_only_tab() {
    let result = parse_color("\t").unwrap();
    assert_eq!(result, None, "Tab should be treated as empty");
}

// ============================================================================
// Hex Colors - Valid Input
// ============================================================================

#[test]
fn test_parse_color_hex_white() {
    let result = parse_color("#ffffff").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn test_parse_color_hex_black() {
    let result = parse_color("#000000").unwrap();
    assert_eq!(result, Some(Color::Rgb(0, 0, 0)));
}

#[test]
fn test_parse_color_hex_red() {
    let result = parse_color("#ff0000").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 0, 0)));
}

#[test]
fn test_parse_color_hex_green() {
    let result = parse_color("#00ff00").unwrap();
    assert_eq!(result, Some(Color::Rgb(0, 255, 0)));
}

#[test]
fn test_parse_color_hex_blue() {
    let result = parse_color("#0000ff").unwrap();
    assert_eq!(result, Some(Color::Rgb(0, 0, 255)));
}

#[test]
fn test_parse_color_hex_magenta() {
    let result = parse_color("#ff00ff").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 0, 255)));
}

#[test]
fn test_parse_color_hex_cyan() {
    let result = parse_color("#00ffff").unwrap();
    assert_eq!(result, Some(Color::Rgb(0, 255, 255)));
}

#[test]
fn test_parse_color_hex_yellow() {
    let result = parse_color("#ffff00").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 0)));
}

#[test]
fn test_parse_color_hex_gray() {
    let result = parse_color("#777777").unwrap();
    assert_eq!(result, Some(Color::Rgb(119, 119, 119)));
}

#[test]
fn test_parse_color_hex_dark_gray() {
    let result = parse_color("#1a1a1a").unwrap();
    assert_eq!(result, Some(Color::Rgb(26, 26, 26)));
}

// ============================================================================
// Hex Colors - Case Insensitivity
// ============================================================================

#[test]
fn test_parse_color_hex_uppercase() {
    let result = parse_color("#FFFFFF").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn test_parse_color_hex_uppercase_magenta() {
    let result = parse_color("#FF00FF").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 0, 255)));
}

#[test]
fn test_parse_color_hex_mixed_case() {
    let result = parse_color("#FfFfFf").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn test_parse_color_hex_mixed_case_rgb() {
    let result = parse_color("#aAbBcC").unwrap();
    assert_eq!(result, Some(Color::Rgb(170, 187, 204)));
}

// ============================================================================
// Hex Colors - With Whitespace
// ============================================================================

#[test]
fn test_parse_color_hex_with_leading_whitespace() {
    let result = parse_color(" #ffffff").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn test_parse_color_hex_with_trailing_whitespace() {
    let result = parse_color("#ffffff ").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn test_parse_color_hex_with_surrounding_whitespace() {
    let result = parse_color("  #ffffff  ").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

// ============================================================================
// Hex Colors - Boundary Values
// ============================================================================

#[test]
fn test_parse_color_hex_min_value() {
    let result = parse_color("#000000").unwrap();
    assert_eq!(result, Some(Color::Rgb(0, 0, 0)));
}

#[test]
fn test_parse_color_hex_max_value() {
    let result = parse_color("#ffffff").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn test_parse_color_hex_leading_zeros() {
    let result = parse_color("#00ff00").unwrap();
    assert_eq!(result, Some(Color::Rgb(0, 255, 0)));
}

// ============================================================================
// Error Cases - Unknown Color Names
// ============================================================================

#[test]
fn test_parse_color_unknown_color_chartreuse() {
    let result = parse_color("chartreuse");
    assert!(result.is_err(), "chartreuse is not a valid color name");
}

#[test]
fn test_parse_color_unknown_color_olive() {
    let result = parse_color("olive");
    assert!(result.is_err(), "olive is not a valid color name");
}

#[test]
fn test_parse_color_unknown_color_purple() {
    let result = parse_color("purple");
    assert!(
        result.is_err(),
        "purple is not a valid color name (use magenta)"
    );
}

// ============================================================================
// Error Cases - Typos in Color Names
// ============================================================================

#[test]
fn test_parse_color_typo_blu() {
    let result = parse_color("blu");
    assert!(result.is_err(), "blu is a typo for blue");
}

#[test]
fn test_parse_color_typo_reed() {
    let result = parse_color("reed");
    assert!(result.is_err(), "reed is a typo for red");
}

#[test]
fn test_parse_color_typo_grean() {
    let result = parse_color("grean");
    assert!(result.is_err(), "grean is a typo for green");
}

// ============================================================================
// Error Cases - Invalid Hex Length
// ============================================================================

#[test]
fn test_parse_color_hex_too_short_3_chars() {
    let result = parse_color("#fff");
    assert!(result.is_err(), "#fff is too short (need 6 hex chars)");
}

#[test]
fn test_parse_color_hex_too_short_4_chars() {
    let result = parse_color("#ffff");
    assert!(result.is_err(), "#ffff is too short");
}

#[test]
fn test_parse_color_hex_too_short_5_chars() {
    let result = parse_color("#fffff");
    assert!(result.is_err(), "#fffff is too short");
}

#[test]
fn test_parse_color_hex_too_long_7_chars() {
    let result = parse_color("#fffffff");
    assert!(result.is_err(), "#fffffff is too long");
}

#[test]
fn test_parse_color_hex_too_long_8_chars() {
    let result = parse_color("#ffffffff");
    assert!(result.is_err(), "#ffffffff is too long");
}

// ============================================================================
// Error Cases - Missing # Prefix
// ============================================================================

#[test]
fn test_parse_color_hex_missing_prefix() {
    let result = parse_color("ffffff");
    assert!(result.is_err(), "ffffff is missing # prefix");
}

#[test]
fn test_parse_color_hex_missing_prefix_123456() {
    let result = parse_color("123456");
    assert!(result.is_err(), "123456 is missing # prefix");
}

#[test]
fn test_parse_color_hex_missing_prefix_00ff00() {
    let result = parse_color("00ff00");
    assert!(result.is_err(), "00ff00 is missing # prefix");
}

// ============================================================================
// Error Cases - Invalid Hex Characters
// ============================================================================

#[test]
fn test_parse_color_hex_invalid_chars_g() {
    let result = parse_color("#gggggg");
    assert!(result.is_err(), "g is not a valid hex character");
}

#[test]
fn test_parse_color_hex_invalid_chars_xyz() {
    let result = parse_color("#xyz123");
    assert!(result.is_err(), "xyz are not valid hex characters");
}

#[test]
fn test_parse_color_hex_invalid_chars_trailing() {
    let result = parse_color("#12345g");
    assert!(result.is_err(), "g is not a valid hex character");
}

// ============================================================================
// Error Cases - Special Characters in Hex
// ============================================================================

#[test]
fn test_parse_color_hex_with_spaces() {
    let result = parse_color("#ff ff ff");
    assert!(result.is_err(), "spaces not allowed within hex string");
}

#[test]
fn test_parse_color_hex_with_dashes() {
    let result = parse_color("#ff-00-ff");
    assert!(result.is_err(), "dashes not allowed in hex string");
}

#[test]
fn test_parse_color_hex_with_underscores() {
    let result = parse_color("#ff_00_ff");
    assert!(result.is_err(), "underscores not allowed in hex string");
}

// ============================================================================
// Error Cases - Unicode and Special Characters
// ============================================================================

#[test]
fn test_parse_color_unicode_emoji() {
    let result = parse_color("🎨");
    assert!(result.is_err(), "emoji is not a valid color");
}

#[test]
fn test_parse_color_unicode_japanese() {
    let result = parse_color("日本語");
    assert!(result.is_err(), "Japanese characters are not valid colors");
}

// ============================================================================
// Error Cases - Very Long Strings (DoS Prevention)
// ============================================================================

#[test]
fn test_parse_color_very_long_string() {
    let long_string = "a".repeat(1000);
    let result = parse_color(&long_string);
    // Should fail (either by length check or by invalid color name)
    assert!(result.is_err(), "Very long strings should be rejected");
}

// ============================================================================
// Edge Cases - RGB Boundary Values
// ============================================================================

#[test]
fn test_parse_color_hex_all_zeros() {
    let result = parse_color("#000000").unwrap();
    assert_eq!(result, Some(Color::Rgb(0, 0, 0)));
}

#[test]
fn test_parse_color_hex_all_max() {
    let result = parse_color("#ffffff").unwrap();
    assert_eq!(result, Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn test_parse_color_hex_same_values() {
    let result = parse_color("#aaaaaa").unwrap();
    assert_eq!(result, Some(Color::Rgb(170, 170, 170)));
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_parse_color_error_includes_input() {
    let result = parse_color("chartreuse");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("chartreuse"),
        "Error message should include the invalid input"
    );
}

#[test]
fn test_parse_color_error_hex_mentions_format() {
    let result = parse_color("#xyz");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("hex") || err_msg.contains("format") || err_msg.contains("invalid"),
        "Error message should mention hex/format/invalid"
    );
}
