//! Unit tests for navigation system
//!
//! Tests the Navigator's stack management, breadcrumb generation, and intent resolution.

use syntropy::tui::navigation::{
    Intent, ItemPayload, Navigator, PluginPayload, Route, StackEntry, TaskPayload,
};

// ============================================================================
// Navigator Initialization Tests
// ============================================================================

#[test]
fn test_navigator_new_single_level() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let nav = Navigator::new(route.clone(), "Plugins".to_string(), " > ".to_string());

    assert_eq!(nav.current(), &route);
    assert_eq!(nav.get_breadcrumbs(), "Plugins");
}

#[test]
fn test_navigator_new_with_custom_separator() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let nav = Navigator::new(route, "Plugins".to_string(), " / ".to_string());

    assert_eq!(nav.get_breadcrumbs(), "Plugins");
}

// ============================================================================
// Breadcrumb Generation Tests
// ============================================================================

#[test]
fn test_breadcrumbs_two_levels() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    let task_route = Route::Task {
        payload: TaskPayload { plugin_idx: 0 },
    };
    nav.push(task_route, "Tasks".to_string());

    assert_eq!(nav.get_breadcrumbs(), "Plugins > Tasks");
}

#[test]
fn test_breadcrumbs_three_levels() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    nav.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "Tasks".to_string(),
    );
    nav.push(
        Route::Item {
            payload: ItemPayload {
                plugin_idx: 0,
                task_key: "test".to_string(),
            },
        },
        "Items".to_string(),
    );

    assert_eq!(nav.get_breadcrumbs(), "Plugins > Tasks > Items");
}

#[test]
fn test_breadcrumbs_custom_separator() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " / ".to_string());

    nav.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "Tasks".to_string(),
    );

    assert_eq!(nav.get_breadcrumbs(), "Plugins / Tasks");
}

#[test]
fn test_breadcrumbs_with_special_characters() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "My-Plugin".to_string(), " → ".to_string());

    nav.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "Export/Backup".to_string(),
    );

    assert_eq!(nav.get_breadcrumbs(), "My-Plugin → Export/Backup");
}

// ============================================================================
// Stack Push/Pop Tests
// ============================================================================

#[test]
fn test_push_updates_current() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    let task_route = Route::Task {
        payload: TaskPayload { plugin_idx: 5 },
    };
    nav.push(task_route.clone(), "Tasks".to_string());

    assert_eq!(nav.current(), &task_route);
}

#[test]
fn test_pop_removes_entry() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route.clone(), "Plugins".to_string(), " > ".to_string());

    nav.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "Tasks".to_string(),
    );

    let popped = nav.pop();
    assert!(popped.is_some());
    assert_eq!(nav.current(), &route);
    assert_eq!(nav.get_breadcrumbs(), "Plugins");
}

#[test]
fn test_pop_updates_breadcrumbs() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    nav.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "Tasks".to_string(),
    );
    nav.push(
        Route::Item {
            payload: ItemPayload {
                plugin_idx: 0,
                task_key: "test".to_string(),
            },
        },
        "Items".to_string(),
    );

    assert_eq!(nav.get_breadcrumbs(), "Plugins > Tasks > Items");

    nav.pop();
    assert_eq!(nav.get_breadcrumbs(), "Plugins > Tasks");

    nav.pop();
    assert_eq!(nav.get_breadcrumbs(), "Plugins");
}

#[test]
fn test_pop_at_root_returns_none() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route.clone(), "Plugins".to_string(), " > ".to_string());

    let result = nav.pop();
    assert_eq!(result, None);
    // Stack should still contain the root
    assert_eq!(nav.current(), &route);
}

// ============================================================================
// Intent Resolution Tests
// ============================================================================

#[test]
fn test_resolve_intent_select_plugin() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    let intent = Intent::SelectPlugin { plugin_idx: 3 };
    let resolved = nav.resolve_intent(intent);

    assert!(resolved.is_some());
    if let Some(Route::Task { payload }) = resolved {
        assert_eq!(payload.plugin_idx, 3);
    } else {
        panic!("Expected Task route");
    }
}

#[test]
fn test_resolve_intent_select_task() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    let intent = Intent::SelectTask {
        plugin_idx: 2,
        task_key: "export".to_string(),
    };
    let resolved = nav.resolve_intent(intent);

    assert!(resolved.is_some());
    if let Some(Route::Item { payload }) = resolved {
        assert_eq!(payload.plugin_idx, 2);
        assert_eq!(payload.task_key, "export");
    } else {
        panic!("Expected Item route");
    }
}

#[test]
fn test_resolve_intent_quit_returns_none() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    let intent = Intent::Quit;
    let resolved = nav.resolve_intent(intent);

    assert_eq!(resolved, None);
}

#[test]
fn test_resolve_intent_none_returns_none() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    let intent = Intent::None;
    let resolved = nav.resolve_intent(intent);

    assert_eq!(resolved, None);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_breadcrumbs_with_empty_name() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route, "".to_string(), " > ".to_string());

    nav.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "Tasks".to_string(),
    );

    assert_eq!(nav.get_breadcrumbs(), " > Tasks");
}

#[test]
fn test_multiple_pops_dont_crash() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let mut nav = Navigator::new(route.clone(), "Plugins".to_string(), " > ".to_string());

    // Try popping multiple times - should all return None
    assert_eq!(nav.pop(), None);
    assert_eq!(nav.pop(), None);
    assert_eq!(nav.pop(), None);

    // Stack should still be valid
    assert_eq!(nav.current(), &route);
}

#[test]
fn test_breadcrumbs_stable_across_reads() {
    let route = Route::Plugin {
        payload: PluginPayload,
    };
    let nav = Navigator::new(route, "Plugins".to_string(), " > ".to_string());

    let breadcrumbs1 = nav.get_breadcrumbs();
    let breadcrumbs2 = nav.get_breadcrumbs();

    // Should return same reference
    assert_eq!(breadcrumbs1, breadcrumbs2);
    assert_eq!(breadcrumbs1, "Plugins");
}

// ============================================================================
// Additional StackEntry Tests
// ============================================================================

#[test]
fn test_stack_entry_with_all_route_variants() {
    // Plugin variant
    let entry1 = StackEntry::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "Plugin".to_string(),
    );
    assert!(matches!(entry1.route, Route::Plugin { .. }));

    // Task variant
    let entry2 = StackEntry::new(
        Route::Task {
            payload: TaskPayload { plugin_idx: 5 },
        },
        "Task".to_string(),
    );
    assert!(matches!(entry2.route, Route::Task { .. }));

    // Item variant
    let entry3 = StackEntry::new(
        Route::Item {
            payload: ItemPayload {
                plugin_idx: 2,
                task_key: "export".to_string(),
            },
        },
        "Item".to_string(),
    );
    assert!(matches!(entry3.route, Route::Item { .. }));
}

// ============================================================================
// Navigator Initialization Edge Cases
// ============================================================================

#[test]
fn test_navigator_with_different_separators() {
    // Test with " > " separator
    let nav1 = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        " > ".to_string(),
    );
    assert_eq!(nav1.get_breadcrumbs(), "A");

    // Test with empty separator
    let nav2 = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        "".to_string(),
    );
    assert_eq!(nav2.get_breadcrumbs(), "A");

    // Test with multi-char separator
    let nav3 = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        " :: ".to_string(),
    );
    assert_eq!(nav3.get_breadcrumbs(), "A");
}

// ============================================================================
// Current Route Tests
// ============================================================================

#[test]
fn test_current_immutable_multiple_calls() {
    let navigator = Navigator::new(
        Route::Task {
            payload: TaskPayload { plugin_idx: 5 },
        },
        "Task".to_string(),
        " > ".to_string(),
    );

    let current1 = navigator.current();
    let current2 = navigator.current();

    assert_eq!(current1, current2);
}

#[test]
fn test_current_reflects_latest_push() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "Plugin".to_string(),
        " > ".to_string(),
    );

    // Before push
    assert!(matches!(navigator.current(), Route::Plugin { .. }));

    // Push new route
    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "Task".to_string(),
    );

    // After push - current reflects new route
    assert!(matches!(navigator.current(), Route::Task { .. }));
}

// ============================================================================
// Push Operation Edge Cases
// ============================================================================

#[test]
fn test_push_with_empty_name() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "Plugins".to_string(),
        " | ".to_string(),
    );

    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "".to_string(),
    );

    assert_eq!(navigator.get_breadcrumbs(), "Plugins | ");
}

#[test]
fn test_push_four_levels_deep() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        "|".to_string(),
    );

    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "B".to_string(),
    );
    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "C".to_string(),
    );
    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "D".to_string(),
    );

    assert_eq!(navigator.get_breadcrumbs(), "A|B|C|D");
}

// ============================================================================
// Pop Operation Edge Cases
// ============================================================================

#[test]
fn test_pop_multiple_times_exhaustive() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        "|".to_string(),
    );

    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "B".to_string(),
    );
    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "C".to_string(),
    );
    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "D".to_string(),
    );

    assert_eq!(navigator.get_breadcrumbs(), "A|B|C|D");

    navigator.pop(); // Remove D
    assert_eq!(navigator.get_breadcrumbs(), "A|B|C");

    navigator.pop(); // Remove C
    assert_eq!(navigator.get_breadcrumbs(), "A|B");

    navigator.pop(); // Remove B
    assert_eq!(navigator.get_breadcrumbs(), "A");

    // Try to pop when only 1 entry remains
    assert!(navigator.pop().is_none());
    assert_eq!(navigator.get_breadcrumbs(), "A");
}

#[test]
fn test_pop_returns_correct_entry() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "Plugins".to_string(),
        " > ".to_string(),
    );

    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 3 },
        },
        "Packages".to_string(),
    );

    let popped = navigator.pop();

    assert!(popped.is_some());
    let entry = popped.unwrap();
    assert!(matches!(entry.route, Route::Task { .. }));
    assert_eq!(entry.name, "Packages");

    if let Route::Task { payload } = entry.route {
        assert_eq!(payload.plugin_idx, 3);
    }
}

// ============================================================================
// Intent Resolution Edge Cases
// ============================================================================

#[test]
fn test_resolve_intent_select_task_with_empty_key() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "Plugins".to_string(),
        " > ".to_string(),
    );

    let intent = Intent::SelectTask {
        plugin_idx: 0,
        task_key: "".to_string(),
    };
    let route = navigator.resolve_intent(intent).unwrap();

    if let Route::Item { payload } = route {
        assert_eq!(payload.task_key, "");
    } else {
        panic!("Expected Item route");
    }
}

#[test]
fn test_resolve_intent_with_high_indices() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "Plugins".to_string(),
        " > ".to_string(),
    );

    let intent = Intent::SelectPlugin { plugin_idx: 999 };
    let route = navigator.resolve_intent(intent);

    assert!(route.is_some());
    if let Some(Route::Task { payload }) = route {
        assert_eq!(payload.plugin_idx, 999);
    }
}

#[test]
fn test_resolve_intent_with_long_task_key() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "Plugins".to_string(),
        " > ".to_string(),
    );

    let long_key = "export_package_list_with_very_long_name".to_string();
    let intent = Intent::SelectTask {
        plugin_idx: 2,
        task_key: long_key.clone(),
    };
    let route = navigator.resolve_intent(intent);

    assert!(route.is_some());
    if let Some(Route::Item { payload }) = route {
        assert_eq!(payload.task_key, long_key);
    }
}

// ============================================================================
// Breadcrumb Generation Edge Cases
// ============================================================================

#[test]
fn test_breadcrumbs_with_unicode() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "プラグイン".to_string(),
        " → ".to_string(),
    );

    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "タスク".to_string(),
    );

    assert_eq!(navigator.get_breadcrumbs(), "プラグイン → タスク");
}

#[test]
fn test_breadcrumbs_with_special_separators() {
    // Test with arrow
    let mut nav1 = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        " → ".to_string(),
    );
    nav1.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "B".to_string(),
    );
    assert_eq!(nav1.get_breadcrumbs(), "A → B");

    // Test with slash
    let mut nav2 = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        " / ".to_string(),
    );
    nav2.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "B".to_string(),
    );
    assert_eq!(nav2.get_breadcrumbs(), "A / B");

    // Test with double colon
    let mut nav3 = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "A".to_string(),
        "::".to_string(),
    );
    nav3.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "B".to_string(),
    );
    assert_eq!(nav3.get_breadcrumbs(), "A::B");
}

#[test]
fn test_breadcrumbs_with_empty_names_multiple() {
    let mut navigator = Navigator::new(
        Route::Plugin {
            payload: PluginPayload,
        },
        "".to_string(),
        " > ".to_string(),
    );

    navigator.push(
        Route::Task {
            payload: TaskPayload { plugin_idx: 0 },
        },
        "".to_string(),
    );
    navigator.push(
        Route::Item {
            payload: ItemPayload {
                plugin_idx: 0,
                task_key: "test".to_string(),
            },
        },
        "".to_string(),
    );

    // Three empty names joined with separators
    assert_eq!(navigator.get_breadcrumbs(), " >  > ");
}

// ============================================================================
// Route Display Tests
// ============================================================================

#[test]
fn test_route_display_all_variants() {
    let plugin_route = Route::Plugin {
        payload: PluginPayload,
    };
    assert_eq!(format!("{}", plugin_route), "Plugin");

    let task_route = Route::Task {
        payload: TaskPayload { plugin_idx: 5 },
    };
    assert_eq!(format!("{}", task_route), "Task");

    let item_route = Route::Item {
        payload: ItemPayload {
            plugin_idx: 2,
            task_key: "export".to_string(),
        },
    };
    assert_eq!(format!("{}", item_route), "Item");
}

// ============================================================================
// Payload Equality Tests
// ============================================================================

#[test]
fn test_task_payload_equality_and_clone() {
    let p1 = TaskPayload { plugin_idx: 5 };
    let p2 = TaskPayload { plugin_idx: 5 };
    let p3 = TaskPayload { plugin_idx: 6 };

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);

    let p4 = p1.clone();
    assert_eq!(p1, p4);
}

#[test]
fn test_item_payload_both_fields_matter() {
    let p1 = ItemPayload {
        plugin_idx: 2,
        task_key: "export".to_string(),
    };
    let p2 = ItemPayload {
        plugin_idx: 2,
        task_key: "export".to_string(),
    };
    let p3 = ItemPayload {
        plugin_idx: 3,
        task_key: "export".to_string(),
    };
    let p4 = ItemPayload {
        plugin_idx: 2,
        task_key: "import".to_string(),
    };

    assert_eq!(p1, p2);
    assert_ne!(p1, p3); // Different plugin_idx
    assert_ne!(p1, p4); // Different task_key
}

#[test]
fn test_plugin_payload_equality() {
    let p1 = PluginPayload;
    let p2 = PluginPayload;

    assert_eq!(p1, p2);
}

// ============================================================================
// Intent Enum Tests
// ============================================================================

#[test]
fn test_intent_variants_equality() {
    let i1 = Intent::SelectPlugin { plugin_idx: 0 };
    let i2 = Intent::SelectPlugin { plugin_idx: 0 };
    let i3 = Intent::SelectPlugin { plugin_idx: 1 };
    let i4 = Intent::Quit;
    let i5 = Intent::None;
    let i6 = Intent::SelectTask {
        plugin_idx: 0,
        task_key: "test".to_string(),
    };

    assert_eq!(i1, i2);
    assert_ne!(i1, i3);
    assert_ne!(i1, i4);
    assert_ne!(i4, i5);
    assert_ne!(i1, i6);
}

#[test]
fn test_intent_select_task_with_different_keys() {
    let i1 = Intent::SelectTask {
        plugin_idx: 0,
        task_key: "export".to_string(),
    };
    let i2 = Intent::SelectTask {
        plugin_idx: 0,
        task_key: "export".to_string(),
    };
    let i3 = Intent::SelectTask {
        plugin_idx: 0,
        task_key: "import".to_string(),
    };

    assert_eq!(i1, i2);
    assert_ne!(i1, i3);
}
