//! Public API tests for hudhudscript-vcs

use hudhudscript_vcs::branch::{
    AgentDefinition, BranchState, ConfigValue, EntityDefinition, FieldDefinition, StateChange,
};
use hudhudscript_vcs::{
    apply_fuzzy, apply_patch, create_diff, delete_lines, format_colored, format_unified,
    indent_lines, insert_lines, parse_patch, replace_lines, Branch, Conflict, ConflictType,
    DiffLine, MergeResult, MergeStrategy, QuorumType, StateTree, VcsError,
};

// ── StateTree — creation ────────────────────────────────────────────

#[test]
fn state_tree_new_has_root() {
    let tree = StateTree::new();
    let current = tree.current_branch().unwrap();
    assert_eq!(current.name, "level_1");
    assert_eq!(current.parent, None);
}

#[test]
fn state_tree_default_trait() {
    let tree = StateTree::default();
    assert_eq!(tree.current_branch().unwrap().name, "level_1");
}

// ── StateTree — branch operations ───────────────────────────────────

#[test]
fn create_branch_succeeds() {
    let tree = StateTree::new();
    let id = tree.create_branch("feature".to_string(), None).unwrap();
    let branch = tree.get_branch("feature").unwrap();
    assert_eq!(branch.name, "feature");
    assert_eq!(branch.id, id);
}

#[test]
fn create_duplicate_branch_fails() {
    let tree = StateTree::new();
    tree.create_branch("feature".to_string(), None).unwrap();
    let result = tree.create_branch("feature".to_string(), None);
    assert!(matches!(
        result.unwrap_err(),
        VcsError::BranchAlreadyExists(_)
    ));
}

#[test]
fn checkout_switches_current() {
    let tree = StateTree::new();
    tree.create_branch("dev".to_string(), None).unwrap();
    tree.checkout("dev").unwrap();
    assert_eq!(tree.current_branch().unwrap().name, "dev");
}

#[test]
fn checkout_nonexistent_fails() {
    let tree = StateTree::new();
    assert!(matches!(
        tree.checkout("ghost").unwrap_err(),
        VcsError::BranchNotFound(_)
    ));
}

#[test]
fn get_branch_nonexistent_none() {
    let tree = StateTree::new();
    assert!(tree.get_branch("nope").is_none());
}

// ── StateTree — record change ───────────────────────────────────────

#[test]
fn record_entity_added() {
    let tree = StateTree::new();
    tree.record_change(StateChange::EntityAdded {
        name: "User".to_string(),
        definition: EntityDefinition {
            name: "User".to_string(),
            fields: vec![],
        },
    })
    .unwrap();
    assert_eq!(tree.current_branch().unwrap().changes.len(), 1);
}

// ── StateTree — list / delete ───────────────────────────────────────

#[test]
fn list_branches_counts_all() {
    let tree = StateTree::new();
    tree.create_branch("f1".to_string(), None).unwrap();
    tree.create_branch("f2".to_string(), None).unwrap();
    assert_eq!(tree.list_branches().len(), 3);
}

#[test]
fn delete_branch_removes_it() {
    let tree = StateTree::new();
    tree.create_branch("feat".to_string(), None).unwrap();
    tree.delete_branch("feat").unwrap();
    assert!(tree.get_branch("feat").is_none());
}

#[test]
fn delete_root_fails() {
    let tree = StateTree::new();
    assert!(matches!(
        tree.delete_branch("level_1").unwrap_err(),
        VcsError::InvalidOperation(_)
    ));
}

#[test]
fn delete_current_fails() {
    let tree = StateTree::new();
    tree.create_branch("feat".to_string(), None).unwrap();
    tree.checkout("feat").unwrap();
    assert!(matches!(
        tree.delete_branch("feat").unwrap_err(),
        VcsError::InvalidOperation(_)
    ));
}

#[test]
fn delete_nonexistent_fails() {
    let tree = StateTree::new();
    assert!(matches!(
        tree.delete_branch("ghost").unwrap_err(),
        VcsError::BranchNotFound(_)
    ));
}

// ── StateTree — merge ───────────────────────────────────────────────

#[test]
fn merge_no_conflicts() {
    let tree = StateTree::new();
    tree.create_branch("feature".to_string(), None).unwrap();
    tree.checkout("feature").unwrap();
    tree.record_change(StateChange::EntityAdded {
        name: "User".to_string(),
        definition: EntityDefinition {
            name: "User".to_string(),
            fields: vec![],
        },
    })
    .unwrap();
    let result = tree
        .merge("feature", "level_1", MergeStrategy::FastForward)
        .unwrap();
    assert!(result.success);
    assert!(!result.has_conflicts());
}

#[test]
fn merge_entity_conflict() {
    let tree = StateTree::new();
    tree.create_branch("f1".to_string(), None).unwrap();
    tree.create_branch("f2".to_string(), None).unwrap();
    tree.checkout("f1").unwrap();
    tree.record_change(StateChange::EntityAdded {
        name: "User".to_string(),
        definition: EntityDefinition {
            name: "User".to_string(),
            fields: vec![FieldDefinition {
                name: "email".to_string(),
                field_type: "string".to_string(),
            }],
        },
    })
    .unwrap();
    tree.checkout("f2").unwrap();
    tree.record_change(StateChange::EntityAdded {
        name: "User".to_string(),
        definition: EntityDefinition {
            name: "User".to_string(),
            fields: vec![FieldDefinition {
                name: "phone".to_string(),
                field_type: "string".to_string(),
            }],
        },
    })
    .unwrap();
    let result = tree.merge("f1", "f2", MergeStrategy::ThreeWay).unwrap();
    assert!(!result.success);
    assert!(result.has_conflicts());
    assert_eq!(result.conflicts[0].path, "User");
}

#[test]
fn merge_agent_conflict() {
    let tree = StateTree::new();
    tree.create_branch("a1".to_string(), None).unwrap();
    tree.create_branch("a2".to_string(), None).unwrap();
    tree.checkout("a1").unwrap();
    tree.record_change(StateChange::AgentAdded {
        name: "Bot".to_string(),
        definition: AgentDefinition {
            name: "Bot".to_string(),
            role: Some("reader".to_string()),
        },
    })
    .unwrap();
    tree.checkout("a2").unwrap();
    tree.record_change(StateChange::AgentAdded {
        name: "Bot".to_string(),
        definition: AgentDefinition {
            name: "Bot".to_string(),
            role: Some("writer".to_string()),
        },
    })
    .unwrap();
    let result = tree.merge("a1", "a2", MergeStrategy::ThreeWay).unwrap();
    assert!(!result.success);
    assert_eq!(
        result.conflicts[0].conflict_type,
        ConflictType::AgentModified
    );
}

#[test]
fn merge_config_conflict() {
    let tree = StateTree::new();
    tree.create_branch("c1".to_string(), None).unwrap();
    tree.create_branch("c2".to_string(), None).unwrap();
    tree.checkout("c1").unwrap();
    tree.record_change(StateChange::ConfigChanged {
        key: "timeout".to_string(),
        old_value: ConfigValue::Number(30.0),
        new_value: ConfigValue::Number(60.0),
    })
    .unwrap();
    tree.checkout("c2").unwrap();
    tree.record_change(StateChange::ConfigChanged {
        key: "timeout".to_string(),
        old_value: ConfigValue::Number(30.0),
        new_value: ConfigValue::Number(120.0),
    })
    .unwrap();
    let result = tree.merge("c1", "c2", MergeStrategy::ThreeWay).unwrap();
    assert!(!result.success);
    assert_eq!(
        result.conflicts[0].conflict_type,
        ConflictType::ConfigModified
    );
}

#[test]
fn merge_nonexistent_source_fails() {
    let tree = StateTree::new();
    assert!(tree
        .merge("ghost", "level_1", MergeStrategy::FastForward)
        .is_err());
}

#[test]
fn merge_nonexistent_target_fails() {
    let tree = StateTree::new();
    tree.create_branch("feat".to_string(), None).unwrap();
    assert!(tree
        .merge("feat", "ghost", MergeStrategy::FastForward)
        .is_err());
}

#[test]
fn merge_applies_changes_to_target() {
    let tree = StateTree::new();
    tree.create_branch("feat".to_string(), None).unwrap();
    tree.checkout("feat").unwrap();
    tree.record_change(StateChange::EntityAdded {
        name: "Product".to_string(),
        definition: EntityDefinition {
            name: "Product".to_string(),
            fields: vec![],
        },
    })
    .unwrap();
    let result = tree
        .merge("feat", "level_1", MergeStrategy::FastForward)
        .unwrap();
    assert!(result.success);
    let target = tree.get_branch("level_1").unwrap();
    assert!(target.state.entities.contains_key("Product"));
}

#[test]
fn merge_same_entity_same_def_no_conflict() {
    let tree = StateTree::new();
    tree.create_branch("f1".to_string(), None).unwrap();
    tree.create_branch("f2".to_string(), None).unwrap();
    let def = EntityDefinition {
        name: "User".to_string(),
        fields: vec![],
    };
    tree.checkout("f1").unwrap();
    tree.record_change(StateChange::EntityAdded {
        name: "User".to_string(),
        definition: def.clone(),
    })
    .unwrap();
    tree.checkout("f2").unwrap();
    tree.record_change(StateChange::EntityAdded {
        name: "User".to_string(),
        definition: def,
    })
    .unwrap();
    let result = tree.merge("f1", "f2", MergeStrategy::ThreeWay).unwrap();
    assert!(result.success);
}

#[test]
fn branch_with_explicit_parent() {
    let tree = StateTree::new();
    let parent_id = tree.create_branch("parent".to_string(), None).unwrap();
    tree.create_branch("child".to_string(), Some(parent_id))
        .unwrap();
    let child = tree.get_branch("child").unwrap();
    assert_eq!(child.parent, Some(parent_id));
}

// ── Diff — create / format ──────────────────────────────────────────

#[test]
fn diff_identical_no_hunks() {
    assert!(create_diff("aaa\nbbb", "aaa\nbbb").hunks.is_empty());
}

#[test]
fn diff_addition() {
    let diff = create_diff("aaa\nccc", "aaa\nNEW\nccc");
    assert!(diff
        .hunks
        .iter()
        .flat_map(|h| &h.lines)
        .any(|l| matches!(l, DiffLine::Added(s) if s == "NEW")));
}

#[test]
fn diff_removal() {
    let diff = create_diff("aaa\nbbb\nccc", "aaa\nccc");
    assert!(diff
        .hunks
        .iter()
        .flat_map(|h| &h.lines)
        .any(|l| matches!(l, DiffLine::Removed(s) if s == "bbb")));
}

#[test]
fn diff_replacement() {
    let diff = create_diff("aaa\nbbb\nccc", "aaa\nBBB\nccc");
    assert!(diff
        .hunks
        .iter()
        .flat_map(|h| &h.lines)
        .any(|l| matches!(l, DiffLine::Removed(s) if s == "bbb")));
    assert!(diff
        .hunks
        .iter()
        .flat_map(|h| &h.lines)
        .any(|l| matches!(l, DiffLine::Added(s) if s == "BBB")));
}

#[test]
fn diff_empty_old() {
    let diff = create_diff("", "hello\nworld");
    assert_eq!(
        diff.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, DiffLine::Added(_)))
            .count(),
        2
    );
}

#[test]
fn diff_empty_new() {
    let diff = create_diff("hello\nworld", "");
    assert_eq!(
        diff.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, DiffLine::Removed(_)))
            .count(),
        2
    );
}

#[test]
fn diff_both_empty() {
    assert!(create_diff("", "").hunks.is_empty());
}

#[test]
fn diff_default_labels() {
    let diff = create_diff("x", "y");
    assert_eq!(diff.old_file, "a");
    assert_eq!(diff.new_file, "b");
}

#[test]
fn format_unified_headers() {
    let text = format_unified(&create_diff("aaa\nbbb", "aaa\nccc"));
    assert!(text.contains("--- a") && text.contains("+++ b") && text.contains("@@"));
}

#[test]
fn format_unified_empty() {
    assert!(format_unified(&create_diff("same", "same")).is_empty());
}

#[test]
fn format_colored_ansi() {
    let c = format_colored(&create_diff("aaa\nbbb", "aaa\nccc"));
    assert!(c.contains("\x1b[31m") && c.contains("\x1b[32m") && c.contains("\x1b[36m"));
}

#[test]
fn format_colored_empty() {
    assert!(format_colored(&create_diff("same", "same")).is_empty());
}

#[test]
fn diff_line_display() {
    assert_eq!(format!("{}", DiffLine::Context("x".into())), " x");
    assert_eq!(format!("{}", DiffLine::Added("y".into())), "+y");
    assert_eq!(format!("{}", DiffLine::Removed("z".into())), "-z");
}

// ── Patch — parse / apply roundtrip ─────────────────────────────────

#[test]
fn patch_roundtrip_addition() {
    let old = "line1\nline2\nline3\nline4\nline5";
    let new = "line1\nline2\nNEWLINE\nline3\nline4\nline5";
    let patch = parse_patch(&format_unified(&create_diff(old, new))).unwrap();
    assert_eq!(apply_patch(old, &patch).unwrap(), new);
}

#[test]
fn patch_roundtrip_removal() {
    let old = "aaa\nbbb\nccc\nddd";
    let new = "aaa\nddd";
    let patch = parse_patch(&format_unified(&create_diff(old, new))).unwrap();
    assert_eq!(apply_patch(old, &patch).unwrap(), new);
}

#[test]
fn patch_roundtrip_replacement() {
    let old = "aaa\nbbb\nccc";
    let new = "aaa\nBBB\nccc";
    let patch = parse_patch(&format_unified(&create_diff(old, new))).unwrap();
    assert_eq!(apply_patch(old, &patch).unwrap(), new);
}

#[test]
fn parse_patch_basic() {
    let text = "--- a\n+++ b\n@@ -1,3 +1,3 @@\n aaa\n-bbb\n+BBB\n ccc\n";
    let patch = parse_patch(text).unwrap();
    assert_eq!(patch.old_file, "a");
    assert_eq!(patch.new_file, "b");
    assert_eq!(patch.hunks.len(), 1);
    assert_eq!(patch.hunks[0].old_start, 1);
}

#[test]
fn parse_patch_invalid() {
    assert!(parse_patch("not a patch").is_err());
}

#[test]
fn parse_patch_too_short() {
    assert!(parse_patch("one").is_err());
}

#[test]
fn parse_patch_no_hunks() {
    assert!(parse_patch("--- old\n+++ new\nrandom text").is_err());
}

#[test]
fn parse_patch_single_range() {
    let text = "--- a\n+++ b\n@@ -1 +1 @@\n-old\n+new\n";
    let patch = parse_patch(text).unwrap();
    assert_eq!(patch.hunks[0].old_count, 1);
    assert_eq!(patch.hunks[0].new_count, 1);
}

#[test]
fn apply_patch_mismatch_fails() {
    let text = "--- a\n+++ b\n@@ -1,3 +1,3 @@\n aaa\n-bbb\n+BBB\n ccc\n";
    let patch = parse_patch(text).unwrap();
    assert!(apply_patch("xxx\nyyy\nzzz", &patch).is_err());
}

#[test]
fn fuzzy_shifted_down() {
    let old = "aaa\nbbb\nccc";
    let shifted = "EXTRA\naaa\nbbb\nccc";
    let patch = parse_patch(&format_unified(&create_diff(old, "aaa\nBBB\nccc"))).unwrap();
    assert!(apply_patch(shifted, &patch).is_err());
    let fuzzy = apply_fuzzy(shifted, &patch, 1).unwrap();
    assert!(fuzzy.contains("BBB") && !fuzzy.contains("bbb"));
}

// ── Edit — insert / replace / delete / indent ───────────────────────

#[test]
fn edit_insert_beginning() {
    assert_eq!(insert_lines("aaa\nbbb", 1, &["NEW"]), "NEW\naaa\nbbb");
}

#[test]
fn edit_insert_middle() {
    assert_eq!(
        insert_lines("aaa\nbbb\nccc", 2, &["X", "Y"]),
        "aaa\nX\nY\nbbb\nccc"
    );
}

#[test]
fn edit_insert_end() {
    assert_eq!(insert_lines("aaa\nbbb", 100, &["END"]), "aaa\nbbb\nEND");
}

#[test]
fn edit_insert_empty() {
    assert_eq!(insert_lines("aaa\nbbb", 1, &[]), "aaa\nbbb");
}

#[test]
fn edit_replace_single() {
    assert_eq!(
        replace_lines("aaa\nbbb\nccc", 2, 2, &["BBB"]),
        "aaa\nBBB\nccc"
    );
}

#[test]
fn edit_replace_multi() {
    assert_eq!(
        replace_lines("aaa\nbbb\nccc\nddd", 2, 3, &["XXX"]),
        "aaa\nXXX\nddd"
    );
}

#[test]
fn edit_replace_expand() {
    assert_eq!(
        replace_lines("aaa\nbbb\nccc", 2, 2, &["X", "Y", "Z"]),
        "aaa\nX\nY\nZ\nccc"
    );
}

#[test]
fn edit_delete_single() {
    assert_eq!(delete_lines("aaa\nbbb\nccc", 2, 2), "aaa\nccc");
}

#[test]
fn edit_delete_range() {
    assert_eq!(delete_lines("aaa\nbbb\nccc\nddd", 2, 3), "aaa\nddd");
}

#[test]
fn edit_delete_all() {
    assert_eq!(delete_lines("aaa\nbbb\nccc", 1, 3), "");
}

#[test]
fn edit_indent_single() {
    assert_eq!(
        indent_lines("aaa\nbbb\nccc", 2, 2, "    "),
        "aaa\n    bbb\nccc"
    );
}

#[test]
fn edit_indent_range() {
    assert_eq!(
        indent_lines("aaa\nbbb\nccc\nddd", 2, 3, "\t"),
        "aaa\n\tbbb\n\tccc\nddd"
    );
}

#[test]
fn edit_indent_preserves() {
    assert_eq!(
        indent_lines("aaa\n  bbb\nccc", 2, 2, "  "),
        "aaa\n    bbb\nccc"
    );
}

#[test]
fn edit_insert_empty_content() {
    assert_eq!(insert_lines("", 1, &["hello"]), "hello");
}

#[test]
fn edit_delete_beyond_bounds() {
    assert_eq!(delete_lines("aaa\nbbb", 1, 100), "");
}

#[test]
fn edit_indent_beyond_bounds() {
    assert_eq!(indent_lines("aaa\nbbb", 1, 100, "  "), "  aaa\n  bbb");
}

#[test]
fn edit_replace_beyond_bounds() {
    assert_eq!(replace_lines("aaa\nbbb", 1, 100, &["XXX"]), "XXX");
}

// ── Branch ──────────────────────────────────────────────────────────

#[test]
fn branch_new_zero_version() {
    let branch = Branch::new(uuid::Uuid::new_v4(), "test".to_string(), None);
    assert_eq!(branch.version, 0);
    assert!(branch.changes.is_empty());
}

#[test]
fn branch_add_change_increments() {
    let mut branch = Branch::new(uuid::Uuid::new_v4(), "t".to_string(), None);
    branch.add_change(StateChange::EntityAdded {
        name: "E".to_string(),
        definition: EntityDefinition {
            name: "E".to_string(),
            fields: vec![],
        },
    });
    assert_eq!(branch.version, 1);
    assert!(branch.metadata.dirty);
}

#[test]
fn branch_clear_changes() {
    let mut branch = Branch::new(uuid::Uuid::new_v4(), "t".to_string(), None);
    branch.add_change(StateChange::EntityAdded {
        name: "E".to_string(),
        definition: EntityDefinition {
            name: "E".to_string(),
            fields: vec![],
        },
    });
    branch.clear_changes();
    assert!(branch.changes.is_empty());
    assert!(!branch.metadata.dirty);
}

#[test]
fn branch_state_default_empty() {
    let state = BranchState::default();
    assert!(state.entities.is_empty());
    assert!(state.agents.is_empty());
    assert!(state.config.is_empty());
}

// ── Conflict ────────────────────────────────────────────────────────

#[test]
fn conflict_creation() {
    let c = Conflict::new(
        ConflictType::EntityModified,
        "User".to_string(),
        Some("old".into()),
        Some("new".into()),
    );
    assert_eq!(c.conflict_type, ConflictType::EntityModified);
    assert_eq!(c.path, "User");
    assert_eq!(c.source_value, Some("old".to_string()));
}

#[test]
fn conflict_entity_no_auto_resolve() {
    assert!(
        !Conflict::new(ConflictType::EntityModified, "x".into(), None, None).can_auto_resolve()
    );
}

#[test]
fn conflict_agent_no_auto_resolve() {
    assert!(!Conflict::new(ConflictType::AgentModified, "x".into(), None, None).can_auto_resolve());
}

#[test]
fn conflict_config_auto_resolve() {
    assert!(Conflict::new(ConflictType::ConfigModified, "x".into(), None, None).can_auto_resolve());
}

// ── VcsError display ────────────────────────────────────────────────

#[test]
fn vcs_error_display() {
    assert!(
        format!("{}", VcsError::BranchNotFound("main".into())).contains("Branch not found: main")
    );
    assert!(format!("{}", VcsError::BranchAlreadyExists("dev".into()))
        .contains("Branch already exists: dev"));
    assert!(format!("{}", VcsError::MergeConflict("x".into())).contains("Merge conflict: x"));
    assert!(
        format!("{}", VcsError::InvalidOperation("bad".into())).contains("Invalid operation: bad")
    );
}

// ── QuorumType ──────────────────────────────────────────────────────

#[test]
fn quorum_majority() {
    assert!(QuorumType::Majority.meets_quorum(3, 5));
    assert!(!QuorumType::Majority.meets_quorum(2, 5));
}

#[test]
fn quorum_unanimous() {
    assert!(QuorumType::Unanimous.meets_quorum(5, 5));
    assert!(!QuorumType::Unanimous.meets_quorum(4, 5));
}

#[test]
fn quorum_threshold() {
    let q = QuorumType::Threshold(2, 3);
    assert!(q.meets_quorum(2, 3));
    assert!(!q.meets_quorum(1, 3));
}

// ── MergeResult ─────────────────────────────────────────────────────

#[test]
fn merge_result_success() {
    let r = MergeResult::success(vec![]);
    assert!(r.success);
    assert!(!r.has_conflicts());
}

#[test]
fn merge_result_failure() {
    let r = MergeResult::failure(vec![Conflict::new(
        ConflictType::EntityModified,
        "x".into(),
        None,
        None,
    )]);
    assert!(!r.success);
    assert!(r.has_conflicts());
}
