use hudhudscript_ids::generator::IdGenerator;
use hudhudscript_ids::validator::*;
use hudhudscript_ids::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

// ============================================================================
// IdGenerator — construction
// ============================================================================

#[test]
fn generator_new_starts_all_counters_at_1() {
    let gen = IdGenerator::new();
    assert_eq!(gen.current_constitution_count(), 1);
    assert_eq!(gen.current_law_count(), 1);
    assert_eq!(gen.current_rule_count(), 1);
    assert_eq!(gen.current_council_count(), 1);
    assert_eq!(gen.current_swarm_count(), 1);
    assert_eq!(gen.current_community_count(), 1);
}

#[test]
fn generator_with_start_values_custom() {
    let gen = IdGenerator::with_start_values(10, 20, 30, 40, 50, 60);
    assert_eq!(gen.current_constitution_count(), 10);
    assert_eq!(gen.current_law_count(), 20);
    assert_eq!(gen.current_rule_count(), 30);
    assert_eq!(gen.current_council_count(), 40);
    assert_eq!(gen.current_swarm_count(), 50);
    assert_eq!(gen.current_community_count(), 60);
}

#[test]
fn generator_default_trait_matches_new() {
    let gen = IdGenerator::default();
    assert_eq!(gen.current_constitution_count(), 1);
    assert_eq!(gen.next_constitution_id(), "cons.1");
}

// ============================================================================
// IdGenerator — sequential generation
// ============================================================================

#[test]
fn next_constitution_id_sequential() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_constitution_id(), "cons.1");
    assert_eq!(gen.next_constitution_id(), "cons.2");
    assert_eq!(gen.next_constitution_id(), "cons.3");
    assert_eq!(gen.current_constitution_count(), 4);
}

#[test]
fn next_law_id_sequential_with_constitution() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_law_id("cons.1"), "cons1.law1");
    assert_eq!(gen.next_law_id("cons.1"), "cons1.law2");
    assert_eq!(gen.next_law_id("cons.2"), "cons2.law3");
    assert_eq!(gen.current_law_count(), 4);
}

#[test]
fn next_rule_id_sequential() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_rule_id(), "rule.1");
    assert_eq!(gen.next_rule_id(), "rule.2");
    assert_eq!(gen.next_rule_id(), "rule.3");
    assert_eq!(gen.current_rule_count(), 4);
}

#[test]
fn next_council_id_sequential() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_council_id(), "council_1");
    assert_eq!(gen.next_council_id(), "council_2");
    assert_eq!(gen.current_council_count(), 3);
}

#[test]
fn next_swarm_id_sequential() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_swarm_id(), "swarm_1");
    assert_eq!(gen.next_swarm_id(), "swarm_2");
    assert_eq!(gen.current_swarm_count(), 3);
}

#[test]
fn next_community_id_sequential() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_community_id(), "community_1");
    assert_eq!(gen.next_community_id(), "community_2");
    assert_eq!(gen.current_community_count(), 3);
}

#[test]
fn law_id_with_different_constitutions() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_law_id("cons.10"), "cons10.law1");
    assert_eq!(gen.next_law_id("cons.999"), "cons999.law2");
}

#[test]
fn law_id_with_non_standard_prefix_uses_zero() {
    let gen = IdGenerator::new();
    // When the prefix is not "cons.", strip_prefix returns None and falls back to "0"
    assert_eq!(gen.next_law_id("xyz"), "cons0.law1");
}

#[test]
fn generator_with_start_values_generates_from_custom_start() {
    let gen = IdGenerator::with_start_values(100, 200, 300, 400, 500, 600);
    assert_eq!(gen.next_constitution_id(), "cons.100");
    assert_eq!(gen.next_rule_id(), "rule.300");
    assert_eq!(gen.next_council_id(), "council_400");
    assert_eq!(gen.next_swarm_id(), "swarm_500");
    assert_eq!(gen.next_community_id(), "community_600");
}

// ============================================================================
// IdGenerator — format validation
// ============================================================================

#[test]
fn generated_ids_match_expected_prefixes() {
    let gen = IdGenerator::new();
    assert!(gen.next_constitution_id().starts_with("cons."));
    assert!(gen.next_law_id("cons.1").starts_with("cons"));
    assert!(gen.next_law_id("cons.1").contains(".law"));
    assert!(gen.next_rule_id().starts_with("rule."));
    assert!(gen.next_council_id().starts_with("council_"));
    assert!(gen.next_swarm_id().starts_with("swarm_"));
    assert!(gen.next_community_id().starts_with("community_"));
}

// ============================================================================
// IdGenerator — concurrency
// ============================================================================

#[test]
fn concurrent_constitution_ids_are_unique() {
    let gen = Arc::new(IdGenerator::new());
    let mut handles = vec![];
    for _ in 0..10 {
        let g = Arc::clone(&gen);
        handles.push(thread::spawn(move || {
            (0..100)
                .map(|_| g.next_constitution_id())
                .collect::<Vec<_>>()
        }));
    }
    let mut all = HashSet::new();
    for h in handles {
        for id in h.join().unwrap() {
            assert!(all.insert(id), "duplicate ID");
        }
    }
    assert_eq!(all.len(), 1000);
}

#[test]
fn concurrent_mixed_ids_count_correctly() {
    let gen = Arc::new(IdGenerator::new());
    let mut handles = vec![];
    for _ in 0..5 {
        let g = Arc::clone(&gen);
        handles.push(thread::spawn(move || {
            for _ in 0..50 {
                g.next_constitution_id();
                g.next_law_id("cons.1");
                g.next_rule_id();
                g.next_council_id();
                g.next_swarm_id();
                g.next_community_id();
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(gen.current_constitution_count(), 251);
    assert_eq!(gen.current_law_count(), 251);
    assert_eq!(gen.current_rule_count(), 251);
    assert_eq!(gen.current_council_count(), 251);
    assert_eq!(gen.current_swarm_count(), 251);
    assert_eq!(gen.current_community_count(), 251);
}

// ============================================================================
// Validator — validate_constitution_id
// ============================================================================

#[test]
fn validate_constitution_id_valid_cases() {
    assert!(validate_constitution_id("cons.1"));
    assert!(validate_constitution_id("cons.999"));
    assert!(validate_constitution_id("cons.123456"));
}

#[test]
fn validate_constitution_id_invalid_cases() {
    assert!(!validate_constitution_id("cons1"));
    assert!(!validate_constitution_id("cons."));
    assert!(!validate_constitution_id("constitution.1"));
    assert!(!validate_constitution_id("cons.1.extra"));
    assert!(!validate_constitution_id("cons.abc"));
    assert!(!validate_constitution_id(""));
    assert!(!validate_constitution_id("cons.-1"));
    assert!(!validate_constitution_id(".cons.1"));
}

// ============================================================================
// Validator — validate_law_id
// ============================================================================

#[test]
fn validate_law_id_valid_cases() {
    assert!(validate_law_id("cons1.law1"));
    assert!(validate_law_id("cons999.law123"));
}

#[test]
fn validate_law_id_invalid_cases() {
    assert!(!validate_law_id("cons.1.law1"));
    assert!(!validate_law_id("cons1.law"));
    assert!(!validate_law_id("law1"));
    assert!(!validate_law_id(""));
    assert!(!validate_law_id("cons-1.law1"));
}

// ============================================================================
// Validator — validate_rule_id
// ============================================================================

#[test]
fn validate_rule_id_valid_cases() {
    assert!(validate_rule_id("rule.1"));
    assert!(validate_rule_id("rule.999"));
}

#[test]
fn validate_rule_id_invalid_cases() {
    assert!(!validate_rule_id("rule1"));
    assert!(!validate_rule_id("rule."));
    assert!(!validate_rule_id("rules.1"));
    assert!(!validate_rule_id(""));
    assert!(!validate_rule_id("rule.abc"));
}

// ============================================================================
// Validator — sanitize_id
// ============================================================================

#[test]
fn sanitize_id_preserves_normal_ids() {
    assert_eq!(sanitize_id("cons.1"), "cons.1");
    assert_eq!(sanitize_id("cons1.law1"), "cons1.law1");
    assert_eq!(sanitize_id("rule.1"), "rule.1");
}

#[test]
fn sanitize_id_trims_whitespace() {
    assert_eq!(sanitize_id("  cons.1  "), "cons.1");
    assert_eq!(sanitize_id("\tcons.1\t"), "cons.1");
}

#[test]
fn sanitize_id_removes_null_bytes() {
    assert_eq!(sanitize_id("cons\x00.1"), "cons.1");
}

#[test]
fn sanitize_id_removes_control_characters() {
    assert_eq!(sanitize_id("cons\x01.1"), "cons.1");
    assert_eq!(sanitize_id("cons\x1F.1"), "cons.1");
    assert_eq!(sanitize_id("cons.\x7F1"), "cons.1");
}

#[test]
fn sanitize_id_removes_unicode_control_characters() {
    assert_eq!(sanitize_id("cons\u{0080}.1"), "cons.1");
    assert_eq!(sanitize_id("cons\u{009F}.1"), "cons.1");
}

#[test]
fn sanitize_id_truncates_at_256_chars() {
    let long = "a".repeat(300);
    assert_eq!(sanitize_id(&long).len(), 256);
}

#[test]
fn sanitize_id_empty_and_whitespace() {
    assert_eq!(sanitize_id(""), "");
    assert_eq!(sanitize_id("   "), "");
    assert_eq!(sanitize_id("\t\n\r"), "");
}

// ============================================================================
// Validator — sanitize_and_validate combined
// ============================================================================

#[test]
fn sanitize_and_validate_constitution_id_valid() {
    assert_eq!(
        sanitize_and_validate_constitution_id("cons.1"),
        Some("cons.1".to_string())
    );
    assert_eq!(
        sanitize_and_validate_constitution_id("  cons.1  "),
        Some("cons.1".to_string())
    );
    assert_eq!(
        sanitize_and_validate_constitution_id("cons\x00.1"),
        Some("cons.1".to_string())
    );
}

#[test]
fn sanitize_and_validate_constitution_id_invalid() {
    assert_eq!(sanitize_and_validate_constitution_id("invalid"), None);
    assert_eq!(sanitize_and_validate_constitution_id("cons1"), None);
}

#[test]
fn sanitize_and_validate_law_id_valid() {
    assert_eq!(
        sanitize_and_validate_law_id("cons1.law1"),
        Some("cons1.law1".to_string())
    );
    assert_eq!(
        sanitize_and_validate_law_id("  cons1.law1  "),
        Some("cons1.law1".to_string())
    );
}

#[test]
fn sanitize_and_validate_law_id_invalid() {
    assert_eq!(sanitize_and_validate_law_id("invalid"), None);
}

#[test]
fn sanitize_and_validate_rule_id_valid() {
    assert_eq!(
        sanitize_and_validate_rule_id("rule.1"),
        Some("rule.1".to_string())
    );
    assert_eq!(
        sanitize_and_validate_rule_id("  rule.1  "),
        Some("rule.1".to_string())
    );
}

#[test]
fn sanitize_and_validate_rule_id_invalid() {
    assert_eq!(sanitize_and_validate_rule_id("invalid"), None);
    assert_eq!(sanitize_and_validate_rule_id("rule1"), None);
}

// ============================================================================
// Validator — injection attempts
// ============================================================================

#[test]
fn injection_attempts_do_not_validate() {
    assert!(!validate_constitution_id(&sanitize_id(
        "cons.1'; DROP TABLE--"
    )));
    assert!(!validate_constitution_id(&sanitize_id("cons.1<script>")));
    assert!(!validate_constitution_id(&sanitize_id("../cons.1")));
}

// ============================================================================
// Cross-module: generated IDs pass validation
// ============================================================================

#[test]
fn generated_constitution_ids_pass_validation() {
    let gen = IdGenerator::new();
    for _ in 0..10 {
        assert!(validate_constitution_id(&gen.next_constitution_id()));
    }
}

#[test]
fn generated_law_ids_pass_validation() {
    let gen = IdGenerator::new();
    for i in 1..=10 {
        let cons_id = format!("cons.{}", i);
        assert!(validate_law_id(&gen.next_law_id(&cons_id)));
    }
}

#[test]
fn generated_rule_ids_pass_validation() {
    let gen = IdGenerator::new();
    for _ in 0..10 {
        assert!(validate_rule_id(&gen.next_rule_id()));
    }
}
