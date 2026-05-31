//! Tests for tokenomics::cascade
//! Extracted from inline #[cfg(test)] module

use hudhudscript_tokenomics::cascade::*;

#[test]
fn test_classify_simple() {
    assert_eq!(
        ComplexityClassifier::classify("What is 2+2?", None),
        Complexity::Simple
    );
    assert_eq!(
        ComplexityClassifier::classify("Hi there", None),
        Complexity::Simple
    );
}

#[test]
fn test_classify_medium() {
    assert_eq!(
        ComplexityClassifier::classify("Explain the differences between TCP and UDP", None),
        Complexity::Medium
    );
}

#[test]
fn test_classify_hard() {
    let prompt =
        "```rust\nfn broken() { }\n```\nDebug this function and fix the architecture issue. "
            .repeat(20);
    assert_eq!(
        ComplexityClassifier::classify(&prompt, None),
        Complexity::Hard
    );
}

#[test]
fn test_classify_math_hard() {
    assert_eq!(
        ComplexityClassifier::classify("Prove the fundamental theorem of calculus", None),
        Complexity::Hard
    );
}

#[test]
fn test_route_simple() {
    let router = CascadeRouter::with_defaults();
    let decision = router.route("What color is the sky?", None);
    assert_eq!(decision.tier_name, "fast");
    assert_eq!(decision.complexity, Complexity::Simple);
}

#[test]
fn test_route_hard() {
    let router = CascadeRouter::with_defaults();
    let decision = router.route("Prove the Riemann hypothesis", None);
    assert_eq!(decision.tier_name, "powerful");
}

#[test]
fn test_budget_downgrade() {
    let mut router = CascadeRouter::with_defaults();
    router.set_budget_remaining(0.05);
    let decision = router.route("Prove the Riemann hypothesis", None);
    assert_eq!(decision.tier_name, "fast"); // forced to cheapest
    assert!(decision.reason.contains("Downgraded"));
}

#[test]
fn test_quality_gate_pass() {
    let gate = QualityGate::new(0.7);
    let assessment = gate.assess(
        "This is a clear and definitive answer with good detail.",
        10,
    );
    assert!(assessment.passed);
}

#[test]
fn test_quality_gate_fail_hedging() {
    let gate = QualityGate::new(0.9);
    let response = "I'm not sure, but maybe it's possible that I think this could perhaps be the answer, arguably.";
    let assessment = gate.assess(response, 10);
    assert!(!assessment.passed);
}

#[test]
fn test_quality_gate_fail_short() {
    let gate = QualityGate::new(0.7);
    let assessment = gate.assess("OK", 0);
    assert!(!assessment.passed);
}

#[test]
fn test_escalation() {
    let router = CascadeRouter::with_defaults();
    let escalation = router.should_escalate("Um", "fast", 50);
    assert!(escalation.is_some());
    assert_eq!(escalation.unwrap().tier_name, "balanced");
}

#[test]
fn test_no_escalation_good_response() {
    let router = CascadeRouter::with_defaults();
    let escalation = router.should_escalate(
        "The answer is clearly 42, based on the following reasoning which I will detail at length.",
        "fast",
        10,
    );
    assert!(escalation.is_none());
}

#[test]
fn test_no_escalation_from_top_tier() {
    let router = CascadeRouter::with_defaults();
    let escalation = router.should_escalate("short", "powerful", 100);
    assert!(escalation.is_none()); // no higher tier to escalate to
}

#[test]
fn test_classify_with_system_prompt() {
    // Line 25: the Some(sp) branch in classify
    let complexity = ComplexityClassifier::classify(
        "What is 2+2?",
        Some("You are an expert mathematician. Prove every theorem rigorously."),
    );
    // system_prompt contains "prove" and "theorem" => Hard
    assert_eq!(complexity, Complexity::Hard);
}

#[test]
fn test_quality_gate_refusal_pattern() {
    // Lines 133-134: refusal pattern detection
    let gate = QualityGate::new(0.9);
    let assessment = gate.assess(
        "I cannot provide that information as an AI assistant, but here is what I know about the topic at hand.",
        10,
    );
    // "i cannot" and "as an ai" are both refusal patterns => -0.3 each? No, has_refusal is bool.
    // confidence = 1.0 - 0.3 = 0.7, which is < 0.9 min_confidence
    assert!(!assessment.passed);
    assert_eq!(assessment.confidence, 0.7);
    assert_eq!(assessment.reasons.len(), 1);
    assert_eq!(assessment.reasons[0], "Response contains refusal pattern");
}

#[test]
fn test_budget_downgrade_hard_to_medium() {
    // Lines 219-221: Hard -> Medium when budget between 0.1 and 0.3
    let mut router = CascadeRouter::with_defaults();
    router.set_budget_remaining(0.2); // between 0.1 and 0.3
    let decision = router.route("Prove the Riemann hypothesis", None);
    // Hard downgraded to Medium => routed to "balanced" tier (cheapest handling Medium)
    assert_eq!(decision.tier_name, "balanced");
    assert_eq!(decision.complexity, Complexity::Hard);
    assert!(decision.reason.contains("Downgraded"));
    assert!(decision.reason.contains("Hard"));
    assert!(decision.reason.contains("Medium"));
}

#[test]
fn test_budget_low_simple_passthrough() {
    // Line 221: `other => other` — non-Hard complexity passes through unchanged
    // when budget is between 0.1 and 0.3
    let mut router = CascadeRouter::with_defaults();
    router.set_budget_remaining(0.2); // between 0.1 and 0.3
                                      // Simple task should stay Simple (not downgraded)
    let decision = router.route("What color is the sky?", None);
    assert_eq!(decision.complexity, Complexity::Simple);
    assert_eq!(decision.tier_name, "fast");
    // No "Downgraded" in reason since effective == original
    assert!(!decision.reason.contains("Downgraded"));
}

#[test]
fn test_route_no_tier_found_fallback() {
    // Lines 247, 249-251, 253: fallback when no tier handles the complexity
    let tiers = vec![CascadeTier {
        name: "basic".into(),
        model: "small-model".into(),
        provider: "test".into(),
        handles: vec![Complexity::Simple], // only handles Simple
        cost_per_1k: 0.001,
    }];
    let router = CascadeRouter::new(tiers, 0.7);
    // "Prove theorem" => Hard, no tier handles Hard => fallback to last tier
    let decision = router.route("Prove the fundamental theorem", None);
    assert_eq!(decision.tier_name, "basic");
    assert_eq!(decision.model, "small-model");
    assert_eq!(decision.provider, "test");
    assert_eq!(decision.reason, "Fallback to most capable tier");
}
