//! Real unit tests for hudhudscript-ids — ID generation, validation, sanitization

use hudhudscript_ids::*;

// ── IdGenerator ─────────────────────────────────────────────────────────

#[test]
fn generator_new_starts_at_one() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_constitution_id(), "cons.1");
    assert_eq!(gen.next_rule_id(), "rule.1");
    assert_eq!(gen.next_council_id(), "council_1");
    assert_eq!(gen.next_swarm_id(), "swarm_1");
    assert_eq!(gen.next_community_id(), "community_1");
}

#[test]
fn generator_increments_correctly() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_constitution_id(), "cons.1");
    assert_eq!(gen.next_constitution_id(), "cons.2");
    assert_eq!(gen.next_constitution_id(), "cons.3");
}

#[test]
fn generator_law_id_includes_parent_constitution() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next_law_id("cons.1"), "cons1.law1");
    assert_eq!(gen.next_law_id("cons.2"), "cons2.law2");
    assert_eq!(gen.next_law_id("cons.1"), "cons1.law3");
}

#[test]
fn generator_law_counter_is_global() {
    let gen = IdGenerator::new();
    // Law counter increments regardless of which constitution
    assert_eq!(gen.next_law_id("cons.5"), "cons5.law1");
    assert_eq!(gen.next_law_id("cons.99"), "cons99.law2");
}

#[test]
fn generator_with_start_values() {
    let gen = IdGenerator::with_start_values(100, 200, 300, 400, 500, 600);
    assert_eq!(gen.next_constitution_id(), "cons.100");
    assert_eq!(gen.next_rule_id(), "rule.300");
    assert_eq!(gen.next_law_id("cons.1"), "cons1.law200");
    assert_eq!(gen.next_council_id(), "council_400");
    assert_eq!(gen.next_swarm_id(), "swarm_500");
    assert_eq!(gen.next_community_id(), "community_600");
}

#[test]
fn generator_each_type_uses_separate_counter() {
    let gen = IdGenerator::new();
    // Each ID type has its own atomic counter
    let c1 = gen.next_constitution_id();
    let r1 = gen.next_rule_id();
    let co1 = gen.next_council_id();
    assert_eq!(c1, "cons.1");
    assert_eq!(r1, "rule.1");
    assert_eq!(co1, "council_1");
    // Verify counters haven't crossed
    assert_eq!(gen.next_constitution_id(), "cons.2");
}

#[test]
fn generator_current_count_does_not_increment() {
    let gen = IdGenerator::new();
    assert_eq!(gen.current_constitution_count(), 1);
    assert_eq!(gen.current_rule_count(), 1);
    gen.next_constitution_id();
    assert_eq!(gen.current_constitution_count(), 2);
    assert_eq!(gen.current_rule_count(), 1); // unchanged
}

#[test]
fn generator_default_impl() {
    let gen = IdGenerator::default();
    assert_eq!(gen.next_constitution_id(), "cons.1");
}

// ── Validators ───────────────────────────────────────────────────────────

use hudhudscript_ids::validator::*;

#[test]
fn validate_constitution_id_valid_cases() {
    assert!(validate_constitution_id("cons.1"));
    assert!(validate_constitution_id("cons.999"));
    assert!(validate_constitution_id("cons.123456789"));
}

#[test]
fn validate_constitution_id_invalid_cases() {
    assert!(!validate_constitution_id("cons1"));
    assert!(!validate_constitution_id("cons."));
    assert!(!validate_constitution_id("constitution.1"));
    assert!(!validate_constitution_id(""));
    assert!(!validate_constitution_id("cons.-1"));
    assert!(!validate_constitution_id("cons.1."));
}

#[test]
fn validate_law_id_valid_cases() {
    assert!(validate_law_id("cons1.law1"));
    assert!(validate_law_id("cons999.law123"));
    assert!(validate_law_id("cons5.law9999"));
}

#[test]
fn validate_law_id_invalid_cases() {
    assert!(!validate_law_id("cons.1.law1"));
    assert!(!validate_law_id("cons1.law"));
    assert!(!validate_law_id("law1"));
    assert!(!validate_law_id(""));
    assert!(!validate_law_id("cons1.law.1"));
}

#[test]
fn validate_rule_id_valid_cases() {
    assert!(validate_rule_id("rule.1"));
    assert!(validate_rule_id("rule.999"));
    assert!(validate_rule_id("rule.18446744073709551615"));
}

#[test]
fn validate_rule_id_invalid_cases() {
    assert!(!validate_rule_id("rule1"));
    assert!(!validate_rule_id("rule."));
    assert!(!validate_rule_id("rules.1"));
    assert!(!validate_rule_id(""));
    assert!(!validate_rule_id("rule.abc"));
}

// ── Sanitization ─────────────────────────────────────────────────────────

#[test]
fn sanitize_id_preserves_valid() {
    assert_eq!(sanitize_id("cons.1"), "cons.1");
    assert_eq!(sanitize_id("rule.42"), "rule.42");
}

#[test]
fn sanitize_id_trims_whitespace() {
    assert_eq!(sanitize_id("  cons.1  "), "cons.1");
    assert_eq!(sanitize_id("\trule.5\n"), "rule.5");
}

#[test]
fn sanitize_id_removes_null_bytes() {
    assert_eq!(sanitize_id("cons\x00.1"), "cons.1");
    assert_eq!(sanitize_id("\x00rule\x00.\x001\x00"), "rule.1");
}

#[test]
fn sanitize_id_removes_control_characters() {
    assert_eq!(sanitize_id("cons\n.1"), "cons.1");
    assert_eq!(sanitize_id("rule\t.\r1"), "rule.1");
}

#[test]
fn sanitize_id_truncates_to_256_chars() {
    let long = "x".repeat(500);
    let result = sanitize_id(&long);
    assert_eq!(result.len(), 256);
}

#[test]
fn sanitize_and_validate_combined() {
    assert_eq!(sanitize_and_validate_constitution_id("cons.1"), Some("cons.1".to_string()));
    assert_eq!(sanitize_and_validate_constitution_id("  cons.42  "), Some("cons.42".to_string()));
    assert_eq!(sanitize_and_validate_constitution_id("invalid"), None);

    assert_eq!(sanitize_and_validate_law_id("cons1.law1"), Some("cons1.law1".to_string()));
    assert_eq!(sanitize_and_validate_law_id("invalid"), None);

    assert_eq!(sanitize_and_validate_rule_id("rule.1"), Some("rule.1".to_string()));
    assert_eq!(sanitize_and_validate_rule_id("  rule.99\n"), Some("rule.99".to_string()));
    assert_eq!(sanitize_and_validate_rule_id("nope"), None);
}
