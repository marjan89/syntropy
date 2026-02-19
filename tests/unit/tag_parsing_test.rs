//! Unit tests for tag parsing functionality
//!
//! Tests the parse_tag and strip_tag functions used for multi-source task routing.

use syntropy::execution::runner::{parse_tag, strip_tag};

// ============================================================================
// parse_tag Tests - Valid Tags
// ============================================================================

#[test]
fn test_parse_tag_with_valid_tag() {
    let (tag, content) = parse_tag("[pkg] PackageName");
    assert_eq!(tag, Some("pkg"));
    assert_eq!(content, "PackageName");
}

#[test]
fn test_parse_tag_with_single_char_tag() {
    let (tag, content) = parse_tag("[w] Safari - Google");
    assert_eq!(tag, Some("w"));
    assert_eq!(content, "Safari - Google");
}

#[test]
fn test_parse_tag_with_numeric_tag() {
    let (tag, content) = parse_tag("[123] Item");
    assert_eq!(tag, Some("123"));
    assert_eq!(content, "Item");
}

#[test]
fn test_parse_tag_with_hyphenated_tag() {
    let (tag, content) = parse_tag("[my-source] Content");
    assert_eq!(tag, Some("my-source"));
    assert_eq!(content, "Content");
}

#[test]
fn test_parse_tag_with_underscore_tag() {
    let (tag, content) = parse_tag("[item_source] Content");
    assert_eq!(tag, Some("item_source"));
    assert_eq!(content, "Content");
}

// ============================================================================
// parse_tag Tests - No Tag Cases
// ============================================================================

#[test]
fn test_parse_tag_without_tag() {
    let (tag, content) = parse_tag("Spotify");
    assert_eq!(tag, None);
    assert_eq!(content, "Spotify");
}

#[test]
fn test_parse_tag_with_empty_brackets() {
    let (tag, content) = parse_tag("[] Empty");
    // Empty brackets are parsed as an empty tag
    assert_eq!(tag, Some(""));
    assert_eq!(content, "Empty");
}

#[test]
fn test_parse_tag_with_malformed_tag_no_closing_bracket() {
    let (tag, content) = parse_tag("[incomplete Item");
    assert_eq!(tag, None);
    assert_eq!(content, "[incomplete Item");
}

#[test]
fn test_parse_tag_with_closing_bracket_only() {
    let (tag, content) = parse_tag("test] Content");
    assert_eq!(tag, None);
    assert_eq!(content, "test] Content");
}

// ============================================================================
// parse_tag Tests - Edge Cases
// ============================================================================

#[test]
fn test_parse_tag_with_multiple_brackets() {
    let (tag, content) = parse_tag("[first] [second] Content");
    assert_eq!(tag, Some("first"));
    assert_eq!(content, "[second] Content");
}

#[test]
fn test_parse_tag_with_extra_whitespace() {
    let (tag, content) = parse_tag("[w]   Safari");
    assert_eq!(tag, Some("w"));
    // Only the first space after ] is stripped, remaining whitespace preserved
    assert_eq!(content, "  Safari");
}

// Regression test: Tags without spaces after ] should parse correctly
#[test]
fn test_parse_tag_with_no_space_after_bracket() {
    let (tag, content) = parse_tag("[pkg]PackageName");
    assert_eq!(tag, Some("pkg"));
    assert_eq!(content, "PackageName");
}

#[test]
fn test_parse_tag_empty_string() {
    let (tag, content) = parse_tag("");
    assert_eq!(tag, None);
    assert_eq!(content, "");
}

#[test]
fn test_parse_tag_only_brackets() {
    // Edge case: just "[]" with nothing after
    // Implementation now handles this with bounds checking
    let (tag, content) = parse_tag("[]");
    assert_eq!(tag, Some(""));
    assert_eq!(content, "");
}

#[test]
fn test_parse_tag_brackets_with_one_char_after() {
    // Edge case: "[]x" - only one char after closing bracket
    // Content should be extracted correctly even with no space
    let (tag, content) = parse_tag("[]x");
    assert_eq!(tag, Some(""));
    assert_eq!(content, "x");
}

#[test]
fn test_parse_tag_with_special_characters_in_tag() {
    let (tag, content) = parse_tag("[src@v1] Content");
    assert_eq!(tag, Some("src@v1"));
    assert_eq!(content, "Content");
}

// ============================================================================
// strip_tag Tests
// ============================================================================

#[test]
fn test_strip_tag_with_tag() {
    assert_eq!(strip_tag("[pkg] PackageName"), "PackageName");
}

#[test]
fn test_strip_tag_without_tag() {
    assert_eq!(strip_tag("Spotify"), "Spotify");
}

#[test]
fn test_strip_tag_with_whitespace() {
    // Only the first space after ] is stripped, remaining whitespace preserved
    assert_eq!(strip_tag("[w]   Safari"), "  Safari");
}

#[test]
fn test_strip_tag_empty_string() {
    assert_eq!(strip_tag(""), "");
}

#[test]
fn test_strip_tag_with_multiple_brackets() {
    assert_eq!(strip_tag("[first] [second] Content"), "[second] Content");
}

// ============================================================================
// Real-World Use Cases
// ============================================================================

#[test]
fn test_parse_tag_realistic_package_item() {
    let (tag, content) = parse_tag("[pkg] git");
    assert_eq!(tag, Some("pkg"));
    assert_eq!(content, "git");
}

#[test]
fn test_parse_tag_realistic_cask_item() {
    let (tag, content) = parse_tag("[cask] google-chrome");
    assert_eq!(tag, Some("cask"));
    assert_eq!(content, "google-chrome");
}

#[test]
fn test_parse_tag_realistic_window_item() {
    let (tag, content) = parse_tag("[w] Safari - www.apple.com");
    assert_eq!(tag, Some("w"));
    assert_eq!(content, "Safari - www.apple.com");
}

#[test]
fn test_strip_tag_batch_processing() {
    let items = ["[pkg] git", "[pkg] node", "[cask] iterm2", "Spotify"];

    let stripped: Vec<&str> = items.iter().map(|s| strip_tag(s)).collect();

    assert_eq!(stripped, ["git", "node", "iterm2", "Spotify"]);
}

// ============================================================================
// parse_tag Additional Edge Cases - Unicode & Special Characters
// ============================================================================

#[test]
fn test_parse_tag_with_unicode_in_tag() {
    let (tag, content) = parse_tag("[日本語] Content");
    assert_eq!(tag, Some("日本語"));
    assert_eq!(content, "Content");
}

#[test]
fn test_parse_tag_with_unicode_in_content() {
    let (tag, content) = parse_tag("[tag] 日本語コンテンツ");
    assert_eq!(tag, Some("tag"));
    assert_eq!(content, "日本語コンテンツ");
}

#[test]
fn test_parse_tag_with_emoji_in_tag() {
    let (tag, content) = parse_tag("[🔧] Tool");
    assert_eq!(tag, Some("🔧"));
    assert_eq!(content, "Tool");
}

#[test]
fn test_parse_tag_with_dots_in_tag() {
    let (tag, content) = parse_tag("[v1.2.3] Version");
    assert_eq!(tag, Some("v1.2.3"));
    assert_eq!(content, "Version");
}

#[test]
fn test_parse_tag_with_colon_in_tag() {
    let (tag, content) = parse_tag("[prefix:name] Item");
    assert_eq!(tag, Some("prefix:name"));
    assert_eq!(content, "Item");
}

#[test]
fn test_parse_tag_with_slash_in_tag() {
    let (tag, content) = parse_tag("[path/to/source] File");
    assert_eq!(tag, Some("path/to/source"));
    assert_eq!(content, "File");
}

// ============================================================================
// parse_tag Content Edge Cases
// ============================================================================

#[test]
fn test_parse_tag_content_with_brackets() {
    let (tag, content) = parse_tag("[tag] Content [v2]");
    assert_eq!(tag, Some("tag"));
    assert_eq!(content, "Content [v2]");
}

#[test]
fn test_parse_tag_single_char_content() {
    let (tag, content) = parse_tag("[t] x");
    assert_eq!(tag, Some("t"));
    assert_eq!(content, "x");
}

#[test]
fn test_parse_tag_content_with_leading_whitespace() {
    let (tag, content) = parse_tag("[tag]    Content");
    assert_eq!(tag, Some("tag"));
    // Only the first space after ] is stripped, remaining whitespace preserved
    assert_eq!(content, "   Content");
}

#[test]
fn test_parse_tag_very_long_tag() {
    let long_tag = "a".repeat(100);
    let input = format!("[{}] Content", long_tag);
    let (tag, content) = parse_tag(&input);
    assert_eq!(tag, Some(long_tag.as_str()));
    assert_eq!(content, "Content");
}

#[test]
fn test_parse_tag_very_long_content() {
    let long_content = "x".repeat(1000);
    let input = format!("[tag] {}", long_content);
    let (tag, content) = parse_tag(&input);
    assert_eq!(tag, Some("tag"));
    assert_eq!(content, long_content.as_str());
}

// ============================================================================
// parse_tag Malformed Input Tests
// ============================================================================

#[test]
fn test_parse_tag_only_opening_bracket() {
    let (tag, content) = parse_tag("[");
    assert_eq!(tag, None);
    assert_eq!(content, "[");
}

#[test]
fn test_parse_tag_only_closing_bracket() {
    let (tag, content) = parse_tag("]");
    assert_eq!(tag, None);
    assert_eq!(content, "]");
}

#[test]
fn test_parse_tag_reversed_brackets() {
    let (tag, content) = parse_tag("]tag[ Content");
    assert_eq!(tag, None);
    assert_eq!(content, "]tag[ Content");
}

#[test]
fn test_parse_tag_nested_brackets() {
    let (tag, content) = parse_tag("[[inner]] Content");
    // Nested brackets should be rejected as malformed
    assert_eq!(tag, None);
    assert_eq!(content, "[[inner]] Content");
}

#[test]
fn test_parse_tag_with_newline_in_content() {
    let (tag, content) = parse_tag("[tag] Line1\nLine2");
    assert_eq!(tag, Some("tag"));
    assert_eq!(content, "Line1\nLine2");
}

// ============================================================================
// strip_tag Additional Edge Cases
// ============================================================================

#[test]
fn test_strip_tag_with_unicode() {
    assert_eq!(strip_tag("[日本語] コンテンツ"), "コンテンツ");
}

#[test]
fn test_strip_tag_with_emoji() {
    assert_eq!(strip_tag("[🔧] Tool"), "Tool");
}

#[test]
fn test_strip_tag_content_with_brackets() {
    assert_eq!(strip_tag("[tag] Content [v2]"), "Content [v2]");
}

#[test]
fn test_strip_tag_single_char_content() {
    assert_eq!(strip_tag("[t] x"), "x");
}

#[test]
fn test_strip_tag_very_long_content() {
    let long_content = "x".repeat(1000);
    let input = format!("[tag] {}", long_content);
    assert_eq!(strip_tag(&input), long_content.as_str());
}

#[test]
fn test_strip_tag_malformed_no_closing() {
    assert_eq!(strip_tag("[incomplete Item"), "[incomplete Item");
}

#[test]
fn test_strip_tag_malformed_no_opening() {
    assert_eq!(strip_tag("tag] Content"), "tag] Content");
}

#[test]
fn test_strip_tag_nested_brackets() {
    assert_eq!(strip_tag("[[inner]] Content"), "[[inner]] Content");
}

#[test]
fn test_strip_tag_with_newline() {
    assert_eq!(strip_tag("[tag] Line1\nLine2"), "Line1\nLine2");
}

// ============================================================================
// parse_tag and strip_tag Consistency Tests
// ============================================================================

#[test]
fn test_strip_tag_matches_parse_tag_content() {
    let test_cases = vec![
        "[pkg] PackageName",
        "[w] Safari",
        "Spotify",
        "",
        "[tag] Content [v2]",
        "[🔧] Tool",
    ];

    for case in test_cases {
        let (_, parse_content) = parse_tag(case);
        let strip_content = strip_tag(case);
        assert_eq!(parse_content, strip_content, "Mismatch for input: {}", case);
    }
}

// ============================================================================
// Real-World Multi-Source Scenarios
// ============================================================================

#[test]
fn test_parse_tag_packages_apps_mixed() {
    let items = vec![
        "[pkg] git",
        "[pkg] node",
        "[cask] google-chrome",
        "[cask] iterm2",
        "Manual Item",
    ];

    let mut package_items = vec![];
    let mut cask_items = vec![];
    let mut other_items = vec![];

    for item in items {
        let (tag, content) = parse_tag(item);
        match tag {
            Some("pkg") => package_items.push(content),
            Some("cask") => cask_items.push(content),
            _ => other_items.push(content),
        }
    }

    assert_eq!(package_items, vec!["git", "node"]);
    assert_eq!(cask_items, vec!["google-chrome", "iterm2"]);
    assert_eq!(other_items, vec!["Manual Item"]);
}

#[test]
fn test_strip_tag_preserves_content_integrity() {
    // Ensure stripping doesn't modify the actual content
    let items = [
        ("[tag] Content-with-dashes", "Content-with-dashes"),
        ("[tag] Content_with_underscores", "Content_with_underscores"),
        ("[tag] Content.with.dots", "Content.with.dots"),
        ("[tag] Content with spaces", "Content with spaces"),
    ];

    for (input, expected) in items {
        assert_eq!(strip_tag(input), expected);
    }
}

// ============================================================================
// Edge Case Tests - Agent Audit Recommendations
// ============================================================================

#[test]
fn test_parse_tag_with_replacement_character() {
    // Test handling of Unicode replacement character (U+FFFD)
    // This character appears when invalid UTF-8 is decoded
    let input = "[tag] Content\u{FFFD}Invalid";
    let (tag, content) = parse_tag(input);
    assert_eq!(tag, Some("tag"));
    assert!(
        content.contains("Invalid"),
        "Content should preserve text after replacement character"
    );
    assert!(
        content.contains('\u{FFFD}'),
        "Replacement character should be preserved"
    );
}

// Regression test: Malformed tags with brackets inside should be rejected
#[test]
fn test_parse_tag_with_bracket_in_tag() {
    let input = "[a]b] content";
    let (tag, content) = parse_tag(input);

    // Malformed tag should be rejected
    assert_eq!(tag, None, "Tag containing ] should be treated as invalid");
    assert_eq!(
        content, input,
        "Malformed tag should return entire input as content"
    );
}
