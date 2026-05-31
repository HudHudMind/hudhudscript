//! Tests for tokenomics::thinking — ThinkingBudgetController, TaskComplexity

use hudhudscript_tokenomics::thinking::*;

// ---------------------------------------------------------------------------
// with_defaults
// ---------------------------------------------------------------------------

#[test]
fn test_with_defaults_simple() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Simple), 1024);
}

#[test]
fn test_with_defaults_medium() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Medium), 4096);
}

#[test]
fn test_with_defaults_hard() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Hard), 16384);
}

#[test]
fn test_with_defaults_expert() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Expert), 65536);
}

// ---------------------------------------------------------------------------
// custom tiers
// ---------------------------------------------------------------------------

#[test]
fn test_custom_tiers() {
    let ctrl = ThinkingBudgetController::new(
        2048,
        vec![
            ("low".into(), 512),
            ("mid".into(), 2048),
            ("high".into(), 8192),
            ("max".into(), 32768),
        ],
    );
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Simple), 512);
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Medium), 2048);
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Hard), 8192);
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Expert), 32768);
}

#[test]
fn test_fewer_tiers_fallback_to_default() {
    let ctrl = ThinkingBudgetController::new(9999, vec![("a".into(), 100), ("b".into(), 200)]);
    // Simple -> index 0 = 100, Medium -> index 1 = 200
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Simple), 100);
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Medium), 200);
    // Hard -> index 2 missing -> fallback to default_budget
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Hard), 9999);
    assert_eq!(ctrl.budget_for_complexity(TaskComplexity::Expert), 9999);
}

// ---------------------------------------------------------------------------
// classify_task
// ---------------------------------------------------------------------------

#[test]
fn test_classify_simple_short_no_code() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(
        ctrl.classify_task("What is 2+2?", false, false),
        TaskComplexity::Simple
    );
}

#[test]
fn test_classify_medium_has_code() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(
        ctrl.classify_task("Fix this function", true, false),
        TaskComplexity::Medium
    );
}

#[test]
fn test_classify_medium_long_text() {
    let ctrl = ThinkingBudgetController::with_defaults();
    let long_prompt = "word ".repeat(150);
    assert_eq!(
        ctrl.classify_task(&long_prompt, false, false),
        TaskComplexity::Medium
    );
}

#[test]
fn test_classify_hard_has_math() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(
        ctrl.classify_task("Solve this equation", false, true),
        TaskComplexity::Hard
    );
}

#[test]
fn test_classify_hard_code_and_long() {
    let ctrl = ThinkingBudgetController::with_defaults();
    let long_prompt = "word ".repeat(250);
    assert_eq!(
        ctrl.classify_task(&long_prompt, true, false),
        TaskComplexity::Hard
    );
}

#[test]
fn test_classify_expert_code_and_math() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(
        ctrl.classify_task("Prove this theorem and implement it", true, true),
        TaskComplexity::Expert
    );
}

// ---------------------------------------------------------------------------
// to_anthropic_param
// ---------------------------------------------------------------------------

#[test]
fn test_anthropic_param_structure() {
    let ctrl = ThinkingBudgetController::with_defaults();
    let param = ctrl.to_anthropic_param(4096);
    assert_eq!(param["type"], "enabled");
    assert_eq!(param["budget_tokens"], 4096);
    let obj = param.as_object().unwrap();
    assert_eq!(obj.len(), 2);
}

#[test]
fn test_anthropic_param_zero() {
    let ctrl = ThinkingBudgetController::with_defaults();
    let param = ctrl.to_anthropic_param(0);
    assert_eq!(param["budget_tokens"], 0);
}

#[test]
fn test_anthropic_param_large() {
    let ctrl = ThinkingBudgetController::with_defaults();
    let param = ctrl.to_anthropic_param(65536);
    assert_eq!(param["budget_tokens"], 65536);
}

// ---------------------------------------------------------------------------
// to_openai_param — boundary tests
// ---------------------------------------------------------------------------

#[test]
fn test_openai_param_zero_is_low() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.to_openai_param(0)["reasoning_effort"], "low");
}

#[test]
fn test_openai_param_1024_is_low() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.to_openai_param(1024)["reasoning_effort"], "low");
}

#[test]
fn test_openai_param_1025_is_medium() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.to_openai_param(1025)["reasoning_effort"], "medium");
}

#[test]
fn test_openai_param_8192_is_medium() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.to_openai_param(8192)["reasoning_effort"], "medium");
}

#[test]
fn test_openai_param_8193_is_high() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.to_openai_param(8193)["reasoning_effort"], "high");
}

#[test]
fn test_openai_param_65536_is_high() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.to_openai_param(65536)["reasoning_effort"], "high");
}

// ---------------------------------------------------------------------------
// record_thinking_tokens / total_thinking_tokens
// ---------------------------------------------------------------------------

#[test]
fn test_record_thinking_tokens_initial() {
    let ctrl = ThinkingBudgetController::with_defaults();
    assert_eq!(ctrl.total_thinking_tokens(), 0);
}

#[test]
fn test_record_thinking_tokens_single() {
    let mut ctrl = ThinkingBudgetController::with_defaults();
    ctrl.record_thinking_tokens(1000);
    assert_eq!(ctrl.total_thinking_tokens(), 1000);
}

#[test]
fn test_record_thinking_tokens_accumulates() {
    let mut ctrl = ThinkingBudgetController::with_defaults();
    ctrl.record_thinking_tokens(1000);
    ctrl.record_thinking_tokens(2000);
    ctrl.record_thinking_tokens(500);
    assert_eq!(ctrl.total_thinking_tokens(), 3500);
}
