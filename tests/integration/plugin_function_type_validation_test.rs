//! Integration tests for plugin function type validation
//!
//! These tests define the desired behavior for type validation of plugin functions.
//! Functions with incorrect return types should be rejected during validation,
//! providing clear error messages to plugin authors.
//!
//! Note: execute() functions are NOT validated at runtime because they have side effects.
//! Only items(), preview(), and preselected_items() functions are validated.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

// ============================================================================
// Category 1: items() Return Type Validation
// ============================================================================

#[test]
fn test_items_wrong_return_type_map_rejected() {
    // Issue 2: items() must return array, not a map/hash table
    // DESIRED BEHAVIOR: Validation should reject non-array tables
    const ITEMS_MAP: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        bad = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {a = "1", b = "2"} end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", ITEMS_MAP);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    // DESIRED: Validation should catch non-array return type
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("items").and(predicate::str::contains("array")));
}

#[test]
fn test_items_wrong_return_type_number_rejected() {
    // Issue 2: items() must return array, not a number
    // DESIRED BEHAVIOR: Validation should reject primitive types
    const ITEMS_NUMBER: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        bad = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return 123 end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", ITEMS_NUMBER);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    // DESIRED: Validation should reject wrong type
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("items").and(predicate::str::contains("return")));
}

// ============================================================================
// Category 3: preview() Return Type Validation (Issue 3)
// ============================================================================

#[test]
fn test_preview_wrong_return_type_number_rejected() {
    // Issue 3: preview() must return string, not a number
    // DESIRED BEHAVIOR: Validation should reject non-string return types
    const PREVIEW_NUMBER: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        bad = {
            description = "Test task",
            preview = function() return 999 end,
            execute = function() return "ok", 0 end
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", PREVIEW_NUMBER);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    // DESIRED: Validation should catch type mismatch
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("preview").and(predicate::str::contains("string")));
}

#[test]
fn test_item_source_preview_wrong_return_type_rejected() {
    // Issue 3: Item source preview() must also return string
    // DESIRED BEHAVIOR: All preview functions must return strings
    const ITEMS_PREVIEW_NUMBER: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        bad = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b"} end,
                    preview = function(item) return 999 end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", ITEMS_PREVIEW_NUMBER);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    // DESIRED: Validation should catch preview type mismatch
    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("preview").and(predicate::str::contains("return")));
}

// ============================================================================
// Category 4: Preselected Items Cross-Validation (Issue 3)
// ============================================================================

#[test]
fn test_preselected_items_not_in_items_list_rejected() {
    // Preselected items must be subset of items returned by items()
    // DESIRED BEHAVIOR: Should be rejected at validation with clear error
    const MISMATCHED_PRESELECTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        bad = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c"} end,
                    preselected_items = function() return {"x", "y"} end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", MISMATCHED_PRESELECTION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("preselected items"))
        .stderr(predicate::str::contains("not found in items"));
}

#[test]
fn test_partial_preselected_items_mismatch_rejected() {
    // Some preselected items valid, some not - should reject
    const PARTIAL_MISMATCH: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        bad = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c"} end,
                    preselected_items = function() return {"a", "x"} end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", PARTIAL_MISMATCH);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("x"))
        .stderr(predicate::str::contains("not found in items"));
}

#[test]
fn test_valid_preselected_items_accepted() {
    // All preselected items in items list - should pass
    const VALID_PRESELECTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        good = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c"} end,
                    preselected_items = function() return {"a", "c"} end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", VALID_PRESELECTION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .success();
}

#[test]
fn test_empty_preselected_items_accepted() {
    // Empty preselection is valid (no items preselected)
    const EMPTY_PRESELECTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        good = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c"} end,
                    preselected_items = function() return {} end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", EMPTY_PRESELECTION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .success();
}

#[test]
fn test_no_preselected_items_function_accepted() {
    // Preselected_items function is optional
    const NO_PRESELECTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        good = {
            description = "Test task",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"a", "b", "c"} end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", NO_PRESELECTION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .success();
}

#[test]
fn test_multi_source_preselected_items_validation() {
    // Each source validated independently
    const MULTI_SOURCE_PRESELECTION: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0"},
    tasks = {
        multi = {
            description = "Test task",
            mode = "multi",
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"a", "b"} end,
                    preselected_items = function() return {"a"} end,
                    execute = function(items) return "ok", 0 end
                },
                src2 = {
                    tag = "s2",
                    items = function() return {"x", "y"} end,
                    preselected_items = function() return {"z"} end,
                    execute = function(items) return "ok", 0 end
                }
            }
        }
    }
}
"#;

    let fixture = TestFixture::new();
    fixture.create_plugin("test", MULTI_SOURCE_PRESELECTION);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("test")
        .join("plugin.lua");

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("z"))
        .stderr(predicate::str::contains("not found in items"));
}
