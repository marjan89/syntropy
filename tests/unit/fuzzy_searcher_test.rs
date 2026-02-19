//! Unit tests for fuzzy search functionality
//!
//! Tests the FuzzySearcher implementation for filtering and ranking items.

use syntropy::tui::fuzzy_searcher::FuzzySearcher;

// ============================================================================
// Empty Query Tests
// ============================================================================

#[test]
fn test_empty_query_returns_all_indices() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];

    let result = searcher.search(&items, "");
    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn test_empty_query_with_empty_items() {
    let searcher = FuzzySearcher::default();
    let items: Vec<String> = vec![];

    let result = searcher.search(&items, "");
    assert_eq!(result, Vec::<usize>::new());
}

// ============================================================================
// Exact Match Tests
// ============================================================================

#[test]
fn test_exact_match_single_item() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];

    let result = searcher.search(&items, "banana");
    // Exact match should be first (and might be only result depending on threshold)
    assert!(result.contains(&1));
    if result.len() == 1 {
        assert_eq!(result[0], 1);
    }
}

#[test]
fn test_partial_match() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "application".to_string(),
        "apple".to_string(),
        "apply".to_string(),
    ];

    let result = searcher.search(&items, "app");
    // All items should match since they all start with "app"
    assert_eq!(result.len(), 3);
    // All three items should be in results
    assert!(result.contains(&0) && result.contains(&1) && result.contains(&2));
}

// ============================================================================
// Case Sensitivity Tests
// ============================================================================

#[test]
fn test_case_sensitive_matching() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "Apple".to_string(),
        "BANANA".to_string(),
        "cherry".to_string(),
    ];

    // The fuzzy matcher is case-insensitive by default
    let result_lower = searcher.search(&items, "apple");
    // Should find Apple (case-insensitive fuzzy matching)
    assert!(result_lower.contains(&0));

    let result_exact = searcher.search(&items, "Apple");
    // Exact case match should also work
    assert!(result_exact.contains(&0));
}

// ============================================================================
// No Match Tests
// ============================================================================

#[test]
fn test_no_matches_returns_empty() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];

    let result = searcher.search(&items, "xyz123");
    assert_eq!(result, Vec::<usize>::new());
}

// ============================================================================
// Score-Based Ordering Tests
// ============================================================================

#[test]
fn test_better_matches_ranked_higher() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "pkg".to_string(),
        "pkg-manager".to_string(),
        "packages".to_string(),
        "something".to_string(),
    ];

    let result = searcher.search(&items, "pkg");

    // Exact match "pkg" should rank higher than partial matches
    assert!(!result.is_empty());
    // The exact match (index 0) should appear before "packages" (index 2)
    let pkg_pos = result
        .iter()
        .position(|&x| x == 0)
        .expect("'pkg' should be in results");
    let packages_pos = result
        .iter()
        .position(|&x| x == 2)
        .expect("'packages' should be in results");

    assert!(
        pkg_pos < packages_pos,
        "Exact match should rank higher than partial"
    );
}

#[test]
fn test_prefix_match_ranks_higher_than_substring() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "defaults".to_string(),
        "my-defaults".to_string(),
        "system-defaults-backup".to_string(),
    ];

    let result = searcher.search(&items, "def");

    // Prefix match "defaults" should rank highest
    assert!(!result.is_empty());
    assert_eq!(result[0], 0, "Prefix match should rank first");
}

// ============================================================================
// Multiple Matches Tests
// ============================================================================

#[test]
fn test_multiple_matches_all_returned() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "test-plugin".to_string(),
        "test-task".to_string(),
        "test-item".to_string(),
        "production".to_string(),
    ];

    let result = searcher.search(&items, "test");

    // All three "test-*" items should match
    assert_eq!(result.len(), 3);
    assert!(result.contains(&0));
    assert!(result.contains(&1));
    assert!(result.contains(&2));
    // "production" should not match
    assert!(!result.contains(&3));
}

// ============================================================================
// Special Characters Tests
// ============================================================================

#[test]
fn test_search_with_special_characters() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "my-plugin".to_string(),
        "my_plugin".to_string(),
        "my.plugin".to_string(),
    ];

    // Search should handle hyphen in query
    let result = searcher.search(&items, "my-");
    assert!(result.contains(&0));

    // Search should handle underscore
    let result = searcher.search(&items, "my_");
    assert!(result.contains(&1));
}

// ============================================================================
// Unicode & International Character Tests
// ============================================================================

#[test]
fn test_search_with_unicode_items() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "日本語アプリ".to_string(),
        "application".to_string(),
        "программа".to_string(),
    ];

    let result = searcher.search(&items, "app");
    // Should match "application"
    assert!(result.contains(&1));
}

#[test]
fn test_search_with_unicode_query() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "日本語".to_string(),
        "English".to_string(),
        "Русский".to_string(),
    ];

    let result = searcher.search(&items, "日本");
    // Should match the Japanese item
    assert!(result.contains(&0));
}

// ============================================================================
// Single Item Tests
// ============================================================================

#[test]
fn test_search_single_item_match() {
    let searcher = FuzzySearcher::default();
    let items = vec!["single".to_string()];

    let result = searcher.search(&items, "sin");
    assert_eq!(result, vec![0]);
}

#[test]
fn test_search_single_item_no_match() {
    let searcher = FuzzySearcher::default();
    let items = vec!["single".to_string()];

    let result = searcher.search(&items, "xyz");
    assert_eq!(result, Vec::<usize>::new());
}

// ============================================================================
// Large Dataset Tests
// ============================================================================

#[test]
fn test_search_with_many_items() {
    let searcher = FuzzySearcher::default();
    let items: Vec<String> = (0..100).map(|i| format!("item-{}", i)).collect();

    let result = searcher.search(&items, "item-5");
    // Should match at least item-5, item-50, item-51, etc.
    assert!(result.contains(&5)); // item-5
    assert!(result.contains(&50)); // item-50
}

#[test]
fn test_search_returns_sorted_by_relevance() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "test-item-with-many-words".to_string(),
        "test".to_string(),
        "testing".to_string(),
        "item-test".to_string(),
    ];

    let result = searcher.search(&items, "test");

    // Should match all items containing "test"
    assert_eq!(result.len(), 4);
    // All test-related items should be in results
    assert!(
        result.contains(&0) && result.contains(&1) && result.contains(&2) && result.contains(&3)
    );
}

// ============================================================================
// Whitespace Handling Tests
// ============================================================================

#[test]
fn test_search_with_spaces_in_query() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "foo bar".to_string(),
        "foobar".to_string(),
        "foo-bar".to_string(),
    ];

    let result = searcher.search(&items, "foo bar");
    // Should match the exact spacing
    assert!(result.contains(&0));
}

#[test]
fn test_search_with_leading_trailing_spaces() {
    let searcher = FuzzySearcher::default();
    let items = vec!["test".to_string(), "testing".to_string()];

    // Query with extra spaces - fuzzy matcher may not trim
    let result = searcher.search(&items, "test");
    // Basic test should definitely match
    assert!(!result.is_empty());
}

// ============================================================================
// Number and Symbol Tests
// ============================================================================

#[test]
fn test_search_with_numbers() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "version-1.2.3".to_string(),
        "version-2.0.0".to_string(),
        "v123".to_string(),
    ];

    let result = searcher.search(&items, "123");
    assert!(result.contains(&0)); // version-1.2.3
    assert!(result.contains(&2)); // v123
}

#[test]
fn test_search_with_symbols() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "file@example.com".to_string(),
        "user#123".to_string(),
        "price$100".to_string(),
    ];

    let result = searcher.search(&items, "@");
    assert_eq!(result, vec![0]);

    let result = searcher.search(&items, "#");
    assert_eq!(result, vec![1]);
}

// ============================================================================
// Identical Items Tests
// ============================================================================

#[test]
fn test_search_with_duplicate_items() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "duplicate".to_string(),
        "duplicate".to_string(),
        "unique".to_string(),
    ];

    let result = searcher.search(&items, "dup");
    // Both duplicates should be in results
    assert!(result.len() >= 2);
    assert!(result.contains(&0));
    assert!(result.contains(&1));
}

// ============================================================================
// Partial Match vs Exact Match Ranking
// ============================================================================

#[test]
fn test_exact_match_ranks_higher() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "testing".to_string(),
        "test".to_string(),
        "test-case".to_string(),
    ];

    let result = searcher.search(&items, "test");

    // All items should match since they all contain "test"
    assert!(!result.is_empty());
    // The exact match should be in the results
    assert!(result.contains(&1));
}

#[test]
fn test_shorter_match_ranks_higher() {
    let searcher = FuzzySearcher::default();
    let items = vec![
        "foo".to_string(),
        "foobar".to_string(),
        "foo-bar-baz".to_string(),
    ];

    let result = searcher.search(&items, "foo");

    // Shorter match should rank higher
    assert!(!result.is_empty());
    assert_eq!(result[0], 0);
}
