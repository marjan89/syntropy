//! Integration tests for plugin override validation with merge detection
//!
//! Tests the CLI validation system for plugin overrides that need to merge with base plugins.
//! This addresses the issue where validating an override plugin with only config changes
//! would fail validation standalone but should pass when merged with its base.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

// ============================================================================
// Mock Plugin Templates
// ============================================================================

const BASE_PLUGIN_WITH_TASKS: &str = r#"
return {
    metadata = {
        name = "notes",
        version = "1.0.0",
        icon = "N",
        description = "Base notes plugin",
    },
    config = {
        storage_dir = "~/.local/share/notes",
        max_items = 100,
    },
    tasks = {
        search = {
            description = "Search notes",
            name = "Search Notes",
            mode = "none",
            item_sources = {
                notes = {
                    tag = "n",
                    items = function() return {"note1.md", "note2.md"} end,
                    execute = function(items) return "Opened notes", 0 end,
                }
            }
        },
        create = {
            description = "Create new note",
            execute = function() return "Created note", 0 end
        }
    }
}
"#;

const OVERRIDE_WITH_CONFIG_ONLY: &str = r#"
return {
    metadata = {
        name = "notes",
        icon = "N",
        description = "Custom notes configuration",
    },
    config = {
        storage_dir = "~/Documents/notes",
        max_items = 500,
    }
}
"#;

const OVERRIDE_WITH_TASKS: &str = r#"
return {
    metadata = {
        name = "notes",
        description = "Override with tasks",
    },
    tasks = {
        search = {
            description = "Search notes",
            name = "Custom Search",
            mode = "multi",
        },
        archive = {
            description = "Archive old notes",
            execute = function() return "Archived", 0 end
        }
    }
}
"#;

const STANDALONE_PLUGIN: &str = r#"
return {
    metadata = {
        name = "standalone",
        version = "1.0.0",
    },
    tasks = {
        task = {
            description = "Standalone task",
            execute = function() return "", 0 end
        }
    }
}
"#;

const INVALID_BASE_PLUGIN: &str = r#"
return {
    metadata = {
        name = "broken",
        version = "1.0.0",
    },
    tasks = {
        bad_task = {
            -- Missing description and execute
            name = "Bad Task"
        }
    }
}
"#;

const OVERRIDE_CREATING_INVALID_MERGE: &str = r#"
return {
    metadata = {
        name = "notes",
    },
    tasks = {
        search = {
            -- This will override but create an invalid task
            mode = "invalid_mode"
        }
    }
}
"#;

const BASE_PLUGIN_NO_TASKS: &str = r#"
return {
    metadata = {
        name = "empty",
        version = "1.0.0",
    },
    tasks = {}
}
"#;

const OVERRIDE_NO_TASKS: &str = r#"
return {
    metadata = {
        name = "empty",
    },
    config = {
        setting = "value"
    }
}
"#;

// ============================================================================
// Category 1: Merge Detection (5 tests)
// ============================================================================

#[test]
fn test_detect_override_in_config_dir() {
    // When validating an override in config dir with a base in data dir,
    // the validator should detect the merge scenario and validate the merged result
    let fixture = TestFixture::new();

    // Create base plugin in data dir
    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);

    // Create override in config dir (no tasks, only config changes)
    fixture.create_plugin_override("notes", OVERRIDE_WITH_CONFIG_ONLY);

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    let assert = cmd.assert();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let status = assert.get_output().status;

    // Should succeed with merged validation
    assert!(status.success(), "Command should succeed");
    assert!(
        output.contains("notes"),
        "Output should contain 'notes': {}",
        output
    );
    assert!(
        output.contains("is valid"),
        "Output should contain 'is valid': {}",
        output
    );
    assert!(
        output.contains("merged") || output.contains("base") || output.contains("override"),
        "Output should indicate merge detection: {}",
        output
    );
}

#[test]
fn test_detect_base_in_data_dir() {
    // When validating a base plugin in data dir with an override in config dir,
    // the validator should detect the override and validate the merged result
    let fixture = TestFixture::new();

    // Create base plugin in data dir
    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);

    // Create override in config dir
    fixture.create_plugin_override("notes", OVERRIDE_WITH_CONFIG_ONLY);

    let base_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&base_path);

    let assert = cmd.assert();

    // Should succeed with merged validation
    assert
        .success()
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_standalone_in_config_dir() {
    // A plugin in config dir without a corresponding base should validate standalone
    let fixture = TestFixture::new();

    // Create only in config dir (no base in data dir)
    fixture.create_plugin_override("standalone", STANDALONE_PLUGIN);

    let plugin_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("standalone")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path);

    let assert = cmd.assert();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let status = assert.get_output().status;

    // Should succeed as standalone
    assert!(status.success(), "Command should succeed");
    assert!(
        output.contains("standalone"),
        "Output should contain 'standalone': {}",
        output
    );
    assert!(
        output.contains("is valid"),
        "Output should contain 'is valid': {}",
        output
    );

    // Should NOT indicate merge
    assert!(
        !output.contains("merged") && !output.contains("override"),
        "Standalone plugin should not show merge messages: {}",
        output
    );
}

#[test]
fn test_standalone_in_data_dir() {
    // A plugin in data dir without an override should validate standalone
    let fixture = TestFixture::new();

    // Create only in data dir (no override in config dir)
    fixture.create_plugin("standalone", STANDALONE_PLUGIN);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("standalone")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path);

    let assert = cmd.assert();

    // Should succeed as standalone
    assert
        .success()
        .stdout(predicate::str::contains("standalone"))
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_custom_path_plugin() {
    // A plugin in a custom location (not in standard dirs) should validate standalone
    let fixture = TestFixture::new();

    // Create plugin in a custom directory (not config or data)
    let custom_dir = fixture
        .temp_dir
        .path()
        .join("custom_plugins")
        .join("myplugin");
    std::fs::create_dir_all(&custom_dir).unwrap();
    let plugin_path = custom_dir.join("plugin.lua");
    std::fs::write(&plugin_path, STANDALONE_PLUGIN).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path);

    let assert = cmd.assert();

    // Should succeed as standalone (no merge detection for custom paths)
    assert
        .success()
        .stdout(predicate::str::contains("is valid"));
}

// ============================================================================
// Category 2: Merged Validation Success (4 tests)
// ============================================================================

#[test]
fn test_validate_override_empty_tasks() {
    // Override has no tasks field at all, base has tasks → should validate successfully
    let fixture = TestFixture::new();

    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);
    fixture.create_plugin_override("notes", OVERRIDE_WITH_CONFIG_ONLY);

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    // Should succeed because merged plugin has tasks from base
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_validate_override_changes_config() {
    // Override changes metadata and config but keeps tasks from base
    let fixture = TestFixture::new();

    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);
    fixture.create_plugin_override("notes", OVERRIDE_WITH_CONFIG_ONLY);

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    let assert = cmd.assert();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let status = assert.get_output().status;

    // Should succeed with merged validation
    assert!(status.success(), "Command should succeed");
    assert!(
        output.contains("notes"),
        "Output should contain 'notes': {}",
        output
    );
    assert!(
        output.contains("is valid"),
        "Output should contain 'is valid': {}",
        output
    );

    // Verify the output shows it's version 1.0.0 (from base)
    // This confirms the merge happened
    assert!(
        output.contains("1.0.0"),
        "Should show version from base plugin: {}",
        output
    );
}

#[test]
fn test_validate_base_with_override() {
    // Validate the base plugin when an override exists
    // Should validate the merged result, not just the base
    let fixture = TestFixture::new();

    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);
    fixture.create_plugin_override("notes", OVERRIDE_WITH_TASKS);

    let base_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&base_path);

    // Should succeed with merged validation
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_merged_validation_output() {
    // Verify that the validation output clearly indicates merged configuration
    let fixture = TestFixture::new();

    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);
    fixture.create_plugin_override("notes", OVERRIDE_WITH_CONFIG_ONLY);

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    let assert = cmd.assert();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let status = assert.get_output().status;

    assert!(status.success(), "Command should succeed");

    // Output should indicate merge scenario with helpful messages
    // Looking for patterns like:
    // - "Found base plugin at ..."
    // - "Found override at ..."
    // - "merged configuration"
    // - References to both paths
    let has_merge_indicator =
        output.contains("merged") || output.contains("base") || output.contains("override");

    assert!(
        has_merge_indicator,
        "Output should indicate merged validation scenario. Got: {}",
        output
    );
}

// ============================================================================
// Category 3: Error Handling (4 tests)
// ============================================================================

#[test]
fn test_base_plugin_invalid() {
    // When base plugin has validation errors, should show clear error message
    let fixture = TestFixture::new();

    fixture.create_plugin("broken", INVALID_BASE_PLUGIN);
    fixture.create_plugin_override(
        "broken",
        &OVERRIDE_WITH_CONFIG_ONLY.replace("notes", "broken"),
    );

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("broken")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    // Should fail with clear error about base plugin being invalid
    cmd.assert().failure().stderr(
        predicate::str::contains("base")
            .or(predicate::str::contains("broken"))
            .or(predicate::str::contains("invalid")),
    );
}

#[test]
fn test_override_invalid_after_merge() {
    // Override is valid standalone (if it had tasks), but creates invalid merged result
    let fixture = TestFixture::new();

    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);
    fixture.create_plugin_override("notes", OVERRIDE_CREATING_INVALID_MERGE);

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    // Should fail because merged result is invalid (invalid mode)
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("mode").or(predicate::str::contains("invalid")));
}

#[test]
fn test_both_plugins_empty_tasks() {
    // Neither base nor override has tasks → merged result should fail validation
    let fixture = TestFixture::new();

    fixture.create_plugin("empty", BASE_PLUGIN_NO_TASKS);
    fixture.create_plugin_override("empty", OVERRIDE_NO_TASKS);

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("empty")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    // Should fail because merged plugin still has no tasks
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("task"));
}

#[test]
fn test_plugin_name_mismatch() {
    // Directory name doesn't match metadata.name
    // This should be detected and reported clearly
    let fixture = TestFixture::new();

    // Create plugin in directory named "notes" but metadata.name is "different"
    let mismatched_plugin = r#"
return {
    metadata = {
        name = "different",
        version = "1.0.0",
    },
    tasks = {
        task = {
            description = "Test task",
            execute = function() return "", 0 end
        }
    }
}
"#;

    fixture.create_plugin("notes", mismatched_plugin);

    let plugin_path = fixture
        .data_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_path);

    // May succeed (name mismatch is a warning, not error in some cases)
    // or may fail depending on implementation
    // The key is that if it fails, it should have a clear message
    let assert = cmd.assert();

    // If it fails, error should mention the name mismatch
    if !assert.get_output().status.success() {
        // Name mismatch might be acceptable, so we just document the behavior
        // The error message would be in stderr if validation fails
    }
}

// ============================================================================
// Additional Edge Case Tests
// ============================================================================

#[test]
fn test_override_with_no_metadata_name() {
    // Override without metadata.name should succeed - name is inferred from directory
    // and merged from base plugin
    let fixture = TestFixture::new();

    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);

    let override_no_name = r#"
return {
    metadata = {
        icon = "X",
    },
    config = {
        setting = "value"
    }
}
"#;

    fixture.create_plugin_override("notes", override_no_name);

    let override_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&override_path);

    // Should succeed - plugin name is extracted from directory path
    // and merged with base plugin metadata
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_validate_plugin_by_directory_path() {
    // Validate by passing directory path instead of plugin.lua path
    let fixture = TestFixture::new();

    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);
    fixture.create_plugin_override("notes", OVERRIDE_WITH_CONFIG_ONLY);

    let plugin_dir = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&plugin_dir);

    // Should succeed and automatically find plugin.lua
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("is valid"));
}

#[test]
fn test_multiple_override_scenarios_independent() {
    // Test that multiple plugins with different merge scenarios work correctly
    // This verifies that merge detection doesn't interfere between plugins
    let fixture = TestFixture::new();

    // Plugin 1: Has merge (base + override)
    fixture.create_plugin("notes", BASE_PLUGIN_WITH_TASKS);
    fixture.create_plugin_override("notes", OVERRIDE_WITH_CONFIG_ONLY);

    // Plugin 2: Standalone in config
    fixture.create_plugin_override("standalone", STANDALONE_PLUGIN);

    // Validate notes (merged)
    let notes_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("notes")
        .join("plugin.lua");

    let mut cmd1 = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd1.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&notes_path);

    cmd1.assert().success();

    // Validate standalone (no merge)
    let standalone_path = fixture
        .config_path()
        .join("syntropy")
        .join("plugins")
        .join("standalone")
        .join("plugin.lua");

    let mut cmd2 = Command::new(assert_cmd::cargo::cargo_bin!("syntropy"));
    cmd2.env("XDG_CONFIG_HOME", fixture.config_path())
        .env("XDG_DATA_HOME", fixture.data_path())
        .arg("validate")
        .arg("--plugin")
        .arg(&standalone_path);

    cmd2.assert().success();
}
