//! Integration tests for tag stripping in multi-source execute pipeline
//!
//! These tests verify that tags added during the items pipeline are correctly
//! stripped before items reach execute functions in multi-source tasks.
//!
//! **Critical Gap Addressed**: While tag parsing has extensive unit tests (56 tests),
//! NO tests previously verified the end-to-end flow of tags being added, then stripped
//! before reaching execute functions. This suite fills that gap.

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::TestFixture;

const MINIMAL_CONFIG: &str = r#"
default_plugin_icon = "⚒"

[keybindings]
back = "<esc>"
select_previous = "<up>"
select_next = "<down>"
scroll_preview_up = "["
scroll_preview_down = "]"
toggle_preview = "<C-p>"
select = "<tab>"
confirm = "<enter>"
"#;

// ============================================================================
// Plugin Constants
// ============================================================================

const TWO_SOURCE_BASIC: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        dual = {
            description = "Test task",
            name = "Dual Source Task",
            mode = "multi",
            item_sources = {
                packages = {
                    tag = "p",
                    items = function() return {"git", "node", "vim"} end,
                    execute = function(items)
                        -- CRITICAL: Verify no tags leaked
                        for _, item in ipairs(items) do
                            if string.match(item, "^%[") then
                                error("Tag leaked: " .. item)
                            end
                        end
                        return "packages:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                cask = {
                    tag = "c",
                    items = function() return {"iterm2", "chrome"} end,
                    execute = function(items)
                        for _, item in ipairs(items) do
                            if string.match(item, "^%[") then
                                error("Tag leaked: " .. item)
                            end
                        end
                        return "cask:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

const THREE_SOURCE_MIXED: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        triple = {
            description = "Test task",
            name = "Triple Source Task",
            mode = "multi",
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"alpha", "beta"} end,
                    execute = function(items)
                        return "s1:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                src2 = {
                    tag = "s2",
                    items = function() return {"gamma", "delta", "epsilon"} end,
                    execute = function(items)
                        return "s2:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                src3 = {
                    tag = "s3",
                    items = function() return {"zeta"} end,
                    execute = function(items)
                        return "s3:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

const FOUR_SOURCE_COMPLEX: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        quad = {
            description = "Test task",
            name = "Quad Source Task",
            mode = "multi",
            item_sources = {
                a = {tag = "a", items = function() return {"a1", "a2"} end, execute = function(items) return "a:[" .. table.concat(items, "|") .. "]", 0 end},
                b = {tag = "b", items = function() return {"b1"} end, execute = function(items) return "b:[" .. table.concat(items, "|") .. "]", 0 end},
                c = {tag = "c", items = function() return {"c1", "c2", "c3"} end, execute = function(items) return "c:[" .. table.concat(items, "|") .. "]", 0 end},
                d = {tag = "d", items = function() return {"d1", "d2"} end, execute = function(items) return "d:[" .. table.concat(items, "|") .. "]", 0 end},
            },
        },
    },
}
"#;

const NO_SPACE_AFTER_BRACKET: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        nospace = {
            description = "Test task",
            name = "No Space Test",
            mode = "multi",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"PackageA", "PackageB"} end,
                    execute = function(items)
                        return "received:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

const BRACKETS_IN_CONTENT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        brackets = {
            description = "Test task",
            name = "Brackets in Content",
            mode = "multi",
            item_sources = {
                pkg = {
                    tag = "p",
                    items = function()
                        return {
                            "item[v1.0]",
                            "package[optional]",
                            "[standalone]"
                        }
                    end,
                    execute = function(items)
                        return "pkg:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

const UNICODE_CONTENT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        unicode = {
            description = "Test task",
            name = "Unicode Content",
            mode = "multi",
            item_sources = {
                jp = {
                    tag = "日",
                    items = function() return {"こんにちは", "さようなら"} end,
                    execute = function(items)
                        return "jp:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                emoji = {
                    tag = "🔧",
                    items = function() return {"tool1🔨", "tool2🔩"} end,
                    execute = function(items)
                        return "emoji:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

const EMPTY_AND_WHITESPACE: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        edge = {
            description = "Test task",
            name = "Edge Cases",
            mode = "multi",
            item_sources = {
                ws = {
                    tag = "ws",
                    items = function()
                        return {
                            "  leading",
                            "trailing  ",
                            "  both  "
                        }
                    end,
                    execute = function(items)
                        return "ws:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

const LONG_ITEM_NAMES: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        long = {
            description = "Test task",
            name = "Long Items",
            mode = "multi",
            item_sources = {
                big = {
                    tag = "big",
                    items = function()
                        local long_item = string.rep("x", 100)
                        return {long_item .. "1", long_item .. "2"}
                    end,
                    execute = function(items)
                        -- Verify lengths are correct (101 chars each)
                        local lengths = {}
                        for _, item in ipairs(items) do
                            table.insert(lengths, tostring(#item))
                        end
                        return "lengths:[" .. table.concat(lengths, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

// ============================================================================
// Category 1: Basic Multi-Source Tag Stripping (3 tests)
// ============================================================================

#[test]
fn test_two_sources_both_strip_tags() {
    // Verify that 2-source task strips tags correctly from both sources
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", TWO_SOURCE_BASIC);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("dual")
        .assert()
        .success()
        .stdout(predicate::str::contains("packages:[git|node|vim]"))
        .stdout(predicate::str::contains("cask:[iterm2|chrome]"));
}

#[test]
fn test_three_sources_all_strip_tags() {
    // Verify that 3-source task strips tags correctly from all sources
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", THREE_SOURCE_MIXED);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("triple")
        .assert()
        .success()
        .stdout(predicate::str::contains("s1:[alpha|beta]"))
        .stdout(predicate::str::contains("s2:[gamma|delta|epsilon]"))
        .stdout(predicate::str::contains("s3:[zeta]"));
}

#[test]
fn test_four_sources_all_strip_tags() {
    // Verify scalability to 4+ sources
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", FOUR_SOURCE_COMPLEX);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("quad")
        .assert()
        .success()
        .stdout(predicate::str::contains("a:[a1|a2]"))
        .stdout(predicate::str::contains("b:[b1]"))
        .stdout(predicate::str::contains("c:[c1|c2|c3]"))
        .stdout(predicate::str::contains("d:[d1|d2]"));
}

// ============================================================================
// Category 2: Edge Cases in Content (6 tests)
// ============================================================================

#[test]
fn test_no_space_after_bracket_strips_correctly() {
    // Verify that [tag]item format works (our bug fix should handle this)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", NO_SPACE_AFTER_BRACKET);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("nospace")
        .assert()
        .success()
        .stdout(predicate::str::contains("received:[PackageA|PackageB]"));
}

#[test]
fn test_brackets_in_content_preserved() {
    // Verify items containing [] in their content don't break tag stripping
    // Items starting with '[' are now handled correctly (single-source doesn't strip)
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", BRACKETS_IN_CONTENT);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("brackets")
        .assert()
        .success()
        // All items including "[standalone]" should be preserved
        .stdout(predicate::str::contains(
            "pkg:[item[v1.0]|package[optional]|[standalone]]",
        ));
}

#[test]
fn test_unicode_content_preserved() {
    // Verify Unicode content (Japanese, emoji) preserved through tag stripping
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", UNICODE_CONTENT);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("unicode")
        .assert()
        .success()
        .stdout(predicate::str::contains("jp:[こんにちは|さようなら]"))
        .stdout(predicate::str::contains("emoji:[tool1🔨|tool2🔩]"));
}

#[test]
fn test_empty_items_handled() {
    // Verify empty item sources don't crash and their execute is NOT called
    // This is the CORRECT behavior after the fix - empty sources should skip execution
    const EMPTY_ITEMS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        empty = {
            description = "Test task",
            name = "Empty Items",
            mode = "multi",
            item_sources = {
                empty = {
                    tag = "e",
                    items = function() return {} end,
                    execute = function(items)
                        return "empty:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                nonempty = {
                    tag = "n",
                    items = function() return {"item"} end,
                    execute = function(items)
                        return "nonempty:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", EMPTY_ITEMS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("empty")
        .assert()
        .success()
        // After the fix, execute is NOT called for empty source, so only nonempty appears
        .stdout(
            predicate::str::contains("nonempty:[item]")
                .and(predicate::str::contains("empty:[]").not()),
        );
}

#[test]
fn test_long_item_names_not_truncated() {
    // Verify long item names (100+ chars) are not truncated during tag stripping
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", LONG_ITEM_NAMES);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("long")
        .assert()
        .success()
        .stdout(predicate::str::contains("lengths:[101|101]"));
}

#[test]
fn test_whitespace_trimmed_correctly() {
    // Verify whitespace handling in tag stripping
    // NOTE: parse_tag only trims the space AFTER the tag bracket, not item content
    // Items like "  leading" remain "  leading" after tag stripping
    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", EMPTY_AND_WHITESPACE);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("edge")
        .assert()
        .success()
        // Whitespace in items is preserved (parse_tag only trims after bracket)
        .stdout(predicate::str::contains(
            "ws:[  leading|trailing  |  both  ]",
        ));
}

// ============================================================================
// Category 3: Round-Trip Integrity (3 tests)
// ============================================================================

#[test]
fn test_items_received_match_items_returned() {
    // Verify exact match: items() output → tag → strip → execute() input
    const EXACT_MATCH: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        exact = {
            description = "Test task",
            name = "Exact Match",
            mode = "multi",
            item_sources = {
                src = {
                    tag = "s",
                    items = function() return {"abc", "def", "ghi"} end,
                    execute = function(items)
                        -- Verify exact items received
                        local expected = {"abc", "def", "ghi"}
                        for i, item in ipairs(items) do
                            if item ~= expected[i] then
                                error("Mismatch at " .. i .. ": " .. item .. " vs " .. expected[i])
                            end
                        end
                        return "exact:match", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", EXACT_MATCH);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("exact")
        .assert()
        .success()
        .stdout(predicate::str::contains("exact:match"));
}

#[test]
fn test_special_characters_preserved() {
    // Verify special characters in items are preserved exactly
    const SPECIAL_CHARS: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        special = {
            description = "Test task",
            name = "Special Characters",
            mode = "multi",
            item_sources = {
                sc = {
                    tag = "sc",
                    items = function()
                        return {
                            "item-with-dashes",
                            "item_with_underscores",
                            "item.with.dots",
                            "item@version:1.0/path"
                        }
                    end,
                    execute = function(items)
                        return "sc:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", SPECIAL_CHARS);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("special")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "sc:[item-with-dashes|item_with_underscores|item.with.dots|item@version:1.0/path]",
        ));
}

#[test]
fn test_mixed_content_types() {
    // Verify numbers, symbols, and letters all work
    const MIXED: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        mixed = {
            description = "Test task",
            name = "Mixed Content",
            mode = "multi",
            item_sources = {
                m = {
                    tag = "m",
                    items = function() return {"123", "abc", "!@#", "mix123abc"} end,
                    execute = function(items)
                        return "mixed:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", MIXED);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("mixed")
        .assert()
        .success()
        .stdout(predicate::str::contains("mixed:[123|abc|!@#|mix123abc]"));
}

// ============================================================================
// Category 4: Multi-Source Behavior (3 tests)
// ============================================================================

#[test]
fn test_sources_execute_independently() {
    // Verify each source's execute only receives its own items
    const INDEPENDENT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        independent = {
            description = "Test task",
            name = "Independent Sources",
            mode = "multi",
            item_sources = {
                alpha = {
                    tag = "a",
                    items = function() return {"a1", "a2", "a3"} end,
                    execute = function(items)
                        -- Verify we ONLY received alpha items
                        for _, item in ipairs(items) do
                            if not string.match(item, "^a%d+$") then
                                error("Wrong item in alpha: " .. item)
                            end
                        end
                        return "alpha:count=" .. #items, 0
                    end,
                },
                beta = {
                    tag = "b",
                    items = function() return {"b1", "b2"} end,
                    execute = function(items)
                        -- Verify we ONLY received beta items
                        for _, item in ipairs(items) do
                            if not string.match(item, "^b%d+$") then
                                error("Wrong item in beta: " .. item)
                            end
                        end
                        return "beta:count=" .. #items, 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", INDEPENDENT);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("independent")
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha:count=3"))
        .stdout(predicate::str::contains("beta:count=2"));
}

#[test]
fn test_source_routing_by_tag() {
    // Verify tags route items to correct execute functions
    const ROUTING: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        routing = {
            description = "Test task",
            name = "Tag Routing",
            mode = "multi",
            item_sources = {
                packages = {
                    tag = "pkg",
                    items = function() return {"formulae1", "formulae2"} end,
                    execute = function(items)
                        return "packages_execute:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                cask = {
                    tag = "cask",
                    items = function() return {"app1", "app2", "app3"} end,
                    execute = function(items)
                        return "cask_execute:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", ROUTING);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("routing")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "packages_execute:[formulae1|formulae2]",
        ))
        .stdout(predicate::str::contains("cask_execute:[app1|app2|app3]"));
}

#[test]
fn test_mixed_preselection_strips_tags() {
    // Verify preselection works correctly with tag stripping
    const PRESELECT: &str = r#"
return {
    metadata = {name = "test", version = "1.0.0", icon = "T", platforms = {"macos"}},
    tasks = {
        preselect = {
            description = "Test task",
            name = "Preselection",
            mode = "multi",
            item_sources = {
                src1 = {
                    tag = "s1",
                    items = function() return {"a", "b", "c"} end,
                    preselected_items = function() return {"a", "c"} end,
                    execute = function(items)
                        return "s1:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
                src2 = {
                    tag = "s2",
                    items = function() return {"x", "y", "z"} end,
                    preselected_items = function() return {"y"} end,
                    execute = function(items)
                        return "s2:[" .. table.concat(items, "|") .. "]", 0
                    end,
                },
            },
        },
    },
}
"#;

    let fixture = TestFixture::new();
    fixture.create_config("syntropy.toml", MINIMAL_CONFIG);
    fixture.create_plugin("test", PRESELECT);

    Command::new(assert_cmd::cargo::cargo_bin!("syntropy"))
        .env("XDG_DATA_HOME", fixture.data_path())
        .env("XDG_CONFIG_HOME", fixture.config_path())
        .arg("execute")
        .arg("--plugin")
        .arg("test")
        .arg("--task")
        .arg("preselect")
        .assert()
        .success()
        .stdout(predicate::str::contains("s1:[a|c]"))
        .stdout(predicate::str::contains("s2:[y]"));
}
