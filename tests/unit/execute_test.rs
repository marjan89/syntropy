//! Unit tests for execute module functionality
//!
//! Tests the parse_comma_separated_with_escapes function and ItemMatcher
//! used for CLI task execution with item selection.

use syntropy::cli::execute::{ItemMatcher, parse_comma_separated_with_escapes};

// ============================================================================
// parse_comma_separated_with_escapes Tests - Basic Functionality
// ============================================================================

#[test]
fn test_parse_basic_comma_separation() {
    let result = parse_comma_separated_with_escapes("a,b,c");
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_single_item() {
    let result = parse_comma_separated_with_escapes("single");
    assert_eq!(result, vec!["single"]);
}

#[test]
fn test_parse_two_items() {
    let result = parse_comma_separated_with_escapes("first,second");
    assert_eq!(result, vec!["first", "second"]);
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Escaped Commas
// ============================================================================

#[test]
fn test_parse_escaped_comma() {
    let result = parse_comma_separated_with_escapes("a\\,b,c");
    assert_eq!(result, vec!["a,b", "c"]);
}

#[test]
fn test_parse_multiple_escaped_commas() {
    let result = parse_comma_separated_with_escapes("a\\,b\\,c,d");
    assert_eq!(result, vec!["a,b,c", "d"]);
}

#[test]
fn test_parse_escaped_comma_only() {
    let result = parse_comma_separated_with_escapes("item\\,with\\,commas");
    assert_eq!(result, vec!["item,with,commas"]);
}

#[test]
fn test_parse_realistic_backup_filename() {
    let result = parse_comma_separated_with_escapes("backup\\,2024-01-01,other");
    assert_eq!(result, vec!["backup,2024-01-01", "other"]);
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Escaped Backslashes
// ============================================================================

#[test]
fn test_parse_escaped_backslash() {
    let result = parse_comma_separated_with_escapes("a\\\\,b");
    assert_eq!(result, vec!["a\\", "b"]);
}

#[test]
fn test_parse_multiple_escaped_backslashes() {
    let result = parse_comma_separated_with_escapes("a\\\\,b\\\\,c");
    assert_eq!(result, vec!["a\\", "b\\", "c"]);
}

#[test]
fn test_parse_consecutive_escaped_backslashes() {
    let result = parse_comma_separated_with_escapes("a\\\\\\\\,b");
    assert_eq!(result, vec!["a\\\\", "b"]);
}

#[test]
fn test_parse_realistic_windows_path() {
    let result = parse_comma_separated_with_escapes("C:\\\\Users\\\\file,other");
    assert_eq!(result, vec!["C:\\Users\\file", "other"]);
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Mixed Escapes
// ============================================================================

#[test]
fn test_parse_mixed_escapes() {
    let result = parse_comma_separated_with_escapes("a\\,b\\\\,c");
    assert_eq!(result, vec!["a,b\\", "c"]);
}

#[test]
fn test_parse_escaped_backslash_before_comma() {
    // "\\" becomes "\", then followed by unescaped comma separator
    let result = parse_comma_separated_with_escapes("a\\\\,b");
    assert_eq!(result, vec!["a\\", "b"]);
}

#[test]
fn test_parse_escaped_comma_and_backslash() {
    let result = parse_comma_separated_with_escapes("a\\,\\\\b,c");
    assert_eq!(result, vec!["a,\\b", "c"]);
}

#[test]
fn test_parse_complex_mixed_escapes() {
    let result = parse_comma_separated_with_escapes("path\\\\to\\\\file\\,v2,other\\,item");
    assert_eq!(result, vec!["path\\to\\file,v2", "other,item"]);
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Whitespace Trimming
// ============================================================================

#[test]
fn test_parse_whitespace_trimming() {
    let result = parse_comma_separated_with_escapes("a , b , c");
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_leading_whitespace() {
    let result = parse_comma_separated_with_escapes("  a,  b,  c");
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_trailing_whitespace() {
    let result = parse_comma_separated_with_escapes("a  ,b  ,c  ");
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_mixed_whitespace() {
    let result = parse_comma_separated_with_escapes("  a  ,  b  ,  c  ");
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_tabs_and_spaces() {
    let result = parse_comma_separated_with_escapes("\ta\t,\tb\t,\tc\t");
    assert_eq!(result, vec!["a", "b", "c"]);
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Empty Items Filtered
// ============================================================================

#[test]
fn test_parse_consecutive_commas() {
    let result = parse_comma_separated_with_escapes("a,,b");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_multiple_consecutive_commas() {
    let result = parse_comma_separated_with_escapes("a,,,b");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_empty_items_with_whitespace() {
    let result = parse_comma_separated_with_escapes("a, ,b");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_all_empty_items() {
    let result = parse_comma_separated_with_escapes(",,,");
    assert_eq!(result, Vec::<String>::new());
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Leading/Trailing Commas
// ============================================================================

#[test]
fn test_parse_leading_comma() {
    let result = parse_comma_separated_with_escapes(",a,b");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_trailing_comma() {
    let result = parse_comma_separated_with_escapes("a,b,");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_leading_and_trailing_commas() {
    let result = parse_comma_separated_with_escapes(",a,b,");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_multiple_leading_commas() {
    let result = parse_comma_separated_with_escapes(",,,a,b");
    assert_eq!(result, vec!["a", "b"]);
}

#[test]
fn test_parse_multiple_trailing_commas() {
    let result = parse_comma_separated_with_escapes("a,b,,,");
    assert_eq!(result, vec!["a", "b"]);
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Edge Cases
// ============================================================================

#[test]
fn test_parse_empty_string() {
    let result = parse_comma_separated_with_escapes("");
    assert_eq!(result, Vec::<String>::new());
}

#[test]
fn test_parse_only_commas() {
    let result = parse_comma_separated_with_escapes(",,,");
    assert_eq!(result, Vec::<String>::new());
}

#[test]
fn test_parse_only_whitespace() {
    let result = parse_comma_separated_with_escapes("   ");
    assert_eq!(result, Vec::<String>::new());
}

#[test]
fn test_parse_whitespace_with_commas() {
    let result = parse_comma_separated_with_escapes(" , , ");
    assert_eq!(result, Vec::<String>::new());
}

#[test]
fn test_parse_trailing_backslash() {
    // Trailing backslash that doesn't escape anything is preserved
    let result = parse_comma_separated_with_escapes("item\\");
    assert_eq!(result, vec!["item\\"]);
}

#[test]
fn test_parse_trailing_backslash_with_comma() {
    let result = parse_comma_separated_with_escapes("item\\,other");
    assert_eq!(result, vec!["item,other"]);
}

#[test]
fn test_parse_unrecognized_escape() {
    // Backslash followed by character that's not comma or backslash
    let result = parse_comma_separated_with_escapes("item\\x,other");
    assert_eq!(result, vec!["item\\x", "other"]);
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Real-World Scenarios
// ============================================================================

#[test]
fn test_parse_package_names() {
    let result = parse_comma_separated_with_escapes("git,npm,node");
    assert_eq!(result, vec!["git", "npm", "node"]);
}

#[test]
fn test_parse_package_names_with_whitespace() {
    let result = parse_comma_separated_with_escapes("git, npm, node");
    assert_eq!(result, vec!["git", "npm", "node"]);
}

#[test]
fn test_parse_filenames_with_commas() {
    let result = parse_comma_separated_with_escapes("file1\\,backup,file2,file3\\,old");
    assert_eq!(result, vec!["file1,backup", "file2", "file3,old"]);
}

#[test]
fn test_parse_paths_with_backslashes() {
    let result = parse_comma_separated_with_escapes("C:\\\\Windows,D:\\\\Users");
    assert_eq!(result, vec!["C:\\Windows", "D:\\Users"]);
}

#[test]
fn test_parse_complex_real_world() {
    let result =
        parse_comma_separated_with_escapes("backup\\,2024-01-01,path\\\\to\\\\file,normal_item");
    assert_eq!(
        result,
        vec!["backup,2024-01-01", "path\\to\\file", "normal_item"]
    );
}

// ============================================================================
// parse_comma_separated_with_escapes Tests - Unicode & Special Characters
// ============================================================================

#[test]
fn test_parse_unicode_items() {
    let result = parse_comma_separated_with_escapes("日本語,español,français");
    assert_eq!(result, vec!["日本語", "español", "français"]);
}

#[test]
fn test_parse_emoji_items() {
    let result = parse_comma_separated_with_escapes("🔧,🚀,💻");
    assert_eq!(result, vec!["🔧", "🚀", "💻"]);
}

#[test]
fn test_parse_mixed_unicode_and_escapes() {
    let result = parse_comma_separated_with_escapes("日本語\\,test,other");
    assert_eq!(result, vec!["日本語,test", "other"]);
}

// ============================================================================
// ItemMatcher Tests - Test Helper
// ============================================================================

fn create_test_items() -> Vec<String> {
    vec![
        "git".to_string(),
        "npm".to_string(),
        "node".to_string(),
        "Node".to_string(), // For case-sensitivity tests
    ]
}

fn create_tagged_items() -> Vec<String> {
    vec![
        "[pkg] git".to_string(),
        "[pkg] npm".to_string(),
        "[cask] docker".to_string(),
        "[cask] chrome".to_string(),
        "[pkg] node".to_string(),
        "[cask] node".to_string(), // Duplicate name, different tags
    ]
}

// ============================================================================
// ItemMatcher Tests - try_exact_match
// ============================================================================

#[test]
fn test_exact_match_found() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.try_exact_match("git");
    assert_eq!(result, Some("git".to_string()));
}

#[test]
fn test_exact_match_case_sensitive() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // Should match "Node" exactly, not "node"
    let result = matcher.try_exact_match("Node");
    assert_eq!(result, Some("Node".to_string()));
}

#[test]
fn test_exact_match_not_found() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.try_exact_match("nonexistent");
    assert_eq!(result, None);
}

#[test]
fn test_exact_match_case_mismatch() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // "GIT" should not match "git" in exact match
    let result = matcher.try_exact_match("GIT");
    assert_eq!(result, None);
}

#[test]
fn test_exact_match_with_tags() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Exact match should match the full tagged string
    let result = matcher.try_exact_match("[pkg] git");
    assert_eq!(result, Some("[pkg] git".to_string()));
}

// ============================================================================
// ItemMatcher Tests - try_tag_stripped_match
// ============================================================================

#[test]
fn test_tag_stripped_match_single_unambiguous() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    let result = matcher.try_tag_stripped_match("git").unwrap();
    assert_eq!(result, Some("[pkg] git".to_string()));
}

#[test]
fn test_tag_stripped_match_ambiguous() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // "node" exists in both [pkg] and [cask]
    let result = matcher.try_tag_stripped_match("node");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Ambiguous item"));
}

#[test]
fn test_tag_stripped_match_not_found() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    let result = matcher.try_tag_stripped_match("nonexistent").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_tag_stripped_match_single_source_unique() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // "docker" only exists in [cask]
    let result = matcher.try_tag_stripped_match("docker").unwrap();
    assert_eq!(result, Some("[cask] docker".to_string()));
}

#[test]
fn test_tag_stripped_match_untagged_items() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Untagged items should still work
    let result = matcher.try_tag_stripped_match("git").unwrap();
    assert_eq!(result, Some("git".to_string()));
}

// ============================================================================
// ItemMatcher Tests - try_case_insensitive_match
// ============================================================================

#[test]
fn test_case_insensitive_match_found() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // "GIT" should match "git" case-insensitively
    let result = matcher.try_case_insensitive_match("GIT");
    assert_eq!(result, Some("git".to_string()));
}

#[test]
fn test_case_insensitive_match_not_found() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.try_case_insensitive_match("nonexistent");
    assert_eq!(result, None);
}

#[test]
fn test_case_insensitive_match_multiple_matches() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // Both "node" and "Node" exist, so case-insensitive should return None
    let result = matcher.try_case_insensitive_match("NODE");
    assert_eq!(result, None);
}

#[test]
fn test_case_insensitive_match_with_tags() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Should match tag-stripped content case-insensitively
    let result = matcher.try_case_insensitive_match("GIT");
    assert_eq!(result, Some("[pkg] git".to_string()));
}

#[test]
fn test_case_insensitive_match_mixed_case() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.try_case_insensitive_match("GiT");
    assert_eq!(result, Some("git".to_string()));
}

// ============================================================================
// ItemMatcher Tests - match_item Strategy Precedence
// ============================================================================

#[test]
fn test_match_item_exact_match_precedence() {
    let items = vec![
        "git".to_string(),
        "GIT".to_string(),
        "[pkg] git".to_string(),
    ];
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Exact match should take precedence
    let result = matcher.match_item("git").unwrap();
    assert_eq!(result, "git");
}

#[test]
fn test_match_item_tag_stripped_precedence() {
    let items = vec!["[pkg] git".to_string(), "GIT".to_string()];
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Tag-stripped should take precedence over case-insensitive
    let result = matcher.match_item("git").unwrap();
    assert_eq!(result, "[pkg] git");
}

#[test]
fn test_match_item_case_insensitive_fallback() {
    let items = vec!["git".to_string()];
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // Should fall back to case-insensitive
    let result = matcher.match_item("GIT").unwrap();
    assert_eq!(result, "git");
}

#[test]
fn test_match_item_not_found_error() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// ============================================================================
// ItemMatcher Tests - match_item Empty/Whitespace Validation
// ============================================================================

#[test]
fn test_match_item_empty_string() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("empty or whitespace-only")
    );
}

#[test]
fn test_match_item_whitespace_only() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("   ");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("empty or whitespace-only")
    );
}

#[test]
fn test_match_item_tabs_only() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("\t\t");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("empty or whitespace-only")
    );
}

#[test]
fn test_match_item_whitespace_trimmed() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // Leading/trailing whitespace should be trimmed before matching
    let result = matcher.match_item("  git  ").unwrap();
    assert_eq!(result, "git");
}

// ============================================================================
// ItemMatcher Tests - match_all
// ============================================================================

#[test]
fn test_match_all_success() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let requested = vec!["git", "npm"];
    let result = matcher.match_all(&requested).unwrap();
    assert_eq!(result, vec!["git", "npm"]);
}

#[test]
fn test_match_all_with_case_insensitive() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let requested = vec!["GIT", "NPM"];
    let result = matcher.match_all(&requested).unwrap();
    assert_eq!(result, vec!["git", "npm"]);
}

#[test]
fn test_match_all_stops_on_first_error() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let requested = vec!["git", "nonexistent", "npm"];
    let result = matcher.match_all(&requested);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_match_all_empty_request() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let requested: Vec<&str> = vec![];
    let result = matcher.match_all(&requested).unwrap();
    assert_eq!(result, Vec::<String>::new());
}

#[test]
fn test_match_all_with_tags() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    let requested = vec!["git", "docker"];
    let result = matcher.match_all(&requested).unwrap();
    assert_eq!(result, vec!["[pkg] git", "[cask] docker"]);
}

#[test]
fn test_match_all_with_exact_tagged_items() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Using full tagged format for disambiguation
    let requested = vec!["[pkg] node", "[cask] node"];
    let result = matcher.match_all(&requested).unwrap();
    assert_eq!(result, vec!["[pkg] node", "[cask] node"]);
}

// ============================================================================
// ItemMatcher Tests - Multi-Source vs Single-Source Behavior
// ============================================================================

#[test]
fn test_single_source_no_tag_stripping() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // Tag-stripped matching should not be attempted for single-source
    // Should fall back to case-insensitive after exact match fails
    let result = matcher.match_item("git");
    // This will try exact match on "[pkg] git", fail, then case-insensitive on tag-stripped content
    assert!(result.is_ok());
}

#[test]
fn test_multi_source_enables_tag_stripping() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Tag-stripped matching should work for multi-source
    let result = matcher.match_item("git").unwrap();
    assert_eq!(result, "[pkg] git");
}

// ============================================================================
// ItemMatcher Tests - Error Messages
// ============================================================================

#[test]
fn test_error_message_shows_available_items() {
    let items = create_test_items();
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("nonexistent");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("not found"));
    assert!(err_msg.contains("test_task"));
    assert!(err_msg.contains("git"));
    assert!(err_msg.contains("npm"));
}

#[test]
fn test_ambiguous_error_message() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    let result = matcher.match_item("node");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Ambiguous item"));
    assert!(err_msg.contains("[pkg] node"));
    assert!(err_msg.contains("[cask] node"));
}

// ============================================================================
// ItemMatcher Tests - Edge Cases
// ============================================================================

#[test]
fn test_matcher_with_empty_items_list() {
    let items: Vec<String> = vec![];
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("anything");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_matcher_with_duplicate_items() {
    let items = vec!["git".to_string(), "git".to_string()];
    let matcher = ItemMatcher::new(&items, false, "test_task");

    // Should still match (duplicates in list are implementation detail)
    let result = matcher.match_item("git").unwrap();
    assert_eq!(result, "git");
}

#[test]
fn test_matcher_preserves_original_format() {
    let items = create_tagged_items();
    let matcher = ItemMatcher::new(&items, true, "test_task");

    // Should return the exact string from the items list
    let result = matcher.match_item("[pkg] git").unwrap();
    assert_eq!(result, "[pkg] git");
}

#[test]
fn test_matcher_with_unicode_items() {
    let items = vec!["日本語".to_string(), "español".to_string()];
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("日本語").unwrap();
    assert_eq!(result, "日本語");
}

#[test]
fn test_matcher_case_insensitive_unicode() {
    let items = vec!["Café".to_string()];
    let matcher = ItemMatcher::new(&items, false, "test_task");

    let result = matcher.match_item("café").unwrap();
    assert_eq!(result, "Café");
}
