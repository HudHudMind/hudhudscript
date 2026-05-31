//! Cascade/waterfall model routing with task complexity classification
//!
//! Routes queries to the cheapest model likely to succeed, with quality gates
//! that escalate to more expensive models when needed.

use serde::{Deserialize, Serialize};

// ── Task Complexity ──────────────────────────────────────────────────────────

/// Task complexity level determined by the classifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Medium,
    Hard,
}

/// Heuristic task complexity classifier
pub struct ComplexityClassifier;

impl ComplexityClassifier {
    /// Classify prompt complexity using heuristics
    pub fn classify(prompt: &str, system_prompt: Option<&str>) -> Complexity {
        let combined = if let Some(sp) = system_prompt {
            format!("{} {}", sp, prompt)
        } else {
            prompt.to_string()
        };
        let lower = combined.to_lowercase();
        let word_count = combined.split_whitespace().count();

        // Hard indicators
        let has_code_fence = combined.contains("```");
        let has_math = lower.contains("prove")
            || lower.contains("theorem")
            || lower.contains("integral")
            || lower.contains("equation")
            || lower.contains("∫")
            || lower.contains("∑");
        let has_multi_step = lower.contains("step by step")
            || lower.contains("first,")
            || lower.contains("then,")
            || lower.contains("finally,");
        let has_debug = lower.contains("debug")
            || lower.contains("fix this")
            || lower.contains("what's wrong")
            || lower.contains("error");
        let has_complex_code = has_code_fence
            && (lower.contains("refactor")
                || lower.contains("optimize")
                || lower.contains("review")
                || lower.contains("architecture"));

        if has_math || has_complex_code || (has_code_fence && has_debug && word_count > 200) {
            return Complexity::Hard;
        }

        // Medium indicators
        let has_code = has_code_fence
            || lower.contains("function")
            || lower.contains("class ")
            || lower.contains("impl ")
            || lower.contains("def ");
        let has_analysis = lower.contains("explain")
            || lower.contains("compare")
            || lower.contains("analyze")
            || lower.contains("why");

        if has_code || has_multi_step || has_analysis || word_count > 150 {
            return Complexity::Medium;
        }

        Complexity::Simple
    }
}

// ── Cascade Tier ─────────────────────────────────────────────────────────────

/// A model tier in the cascade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeTier {
    /// Tier name (e.g., "fast", "balanced", "powerful")
    pub name: String,
    /// Model identifier
    pub model: String,
    /// Provider name
    pub provider: String,
    /// Complexity levels this tier handles
    pub handles: Vec<Complexity>,
    /// Approximate cost per 1K tokens (input + output average)
    pub cost_per_1k: f64,
}

// ── Quality Gate ─────────────────────────────────────────────────────────────

/// Quality assessment of a model response
#[derive(Debug, Clone)]
pub struct QualityAssessment {
    pub passed: bool,
    pub confidence: f64,
    pub reasons: Vec<String>,
}

/// Quality gate that decides whether to accept a response or escalate
pub struct QualityGate {
    /// Minimum acceptable confidence
    min_confidence: f64,
    /// Maximum hedging phrases allowed
    max_hedging: usize,
}

impl QualityGate {
    pub fn new(min_confidence: f64) -> Self {
        Self {
            min_confidence,
            max_hedging: 3,
        }
    }

    /// Assess response quality
    pub fn assess(&self, response: &str, expected_min_length: usize) -> QualityAssessment {
        let mut confidence = 1.0;
        let mut reasons = Vec::new();

        // Check hedging language
        let hedging_phrases = [
            "i'm not sure",
            "it's possible",
            "i think",
            "maybe",
            "i'm not certain",
            "it might",
            "perhaps",
            "arguably",
        ];
        let lower = response.to_lowercase();
        let hedging_count = hedging_phrases
            .iter()
            .filter(|p| lower.contains(*p))
            .count();

        if hedging_count > self.max_hedging {
            confidence -= 0.2 * (hedging_count - self.max_hedging) as f64;
            reasons.push(format!("{} hedging phrases detected", hedging_count));
        }

        // Check response length
        if response.len() < expected_min_length && expected_min_length > 0 {
            confidence -= 0.15;
            reasons.push("Response shorter than expected".into());
        }

        // Check for error/refusal patterns
        let refusal_patterns = ["i cannot", "i can't", "as an ai", "i don't have access"];
        let has_refusal = refusal_patterns.iter().any(|p| lower.contains(p));
        if has_refusal {
            confidence -= 0.3;
            reasons.push("Response contains refusal pattern".into());
        }

        // Check for empty or trivially short response
        if response.trim().len() < 10 {
            confidence = 0.0;
            reasons.push("Response too short".into());
        }

        confidence = confidence.clamp(0.0, 1.0);

        QualityAssessment {
            passed: confidence >= self.min_confidence,
            confidence,
            reasons,
        }
    }
}

// ── Cascade Router ───────────────────────────────────────────────────────────

/// Routing decision
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub model: String,
    pub provider: String,
    pub tier_name: String,
    pub complexity: Complexity,
    pub reason: String,
}

/// Budget-aware cascade router
pub struct CascadeRouter {
    tiers: Vec<CascadeTier>,
    quality_gate: QualityGate,
    budget_remaining_pct: f64,
}

impl CascadeRouter {
    pub fn new(tiers: Vec<CascadeTier>, min_quality_confidence: f64) -> Self {
        Self {
            tiers,
            quality_gate: QualityGate::new(min_quality_confidence),
            budget_remaining_pct: 1.0,
        }
    }

    /// Create with default tiers
    pub fn with_defaults() -> Self {
        let tiers = vec![
            CascadeTier {
                name: "fast".into(),
                model: "claude-haiku-3.5".into(),
                provider: "anthropic".into(),
                handles: vec![Complexity::Simple],
                cost_per_1k: 0.0024,
            },
            CascadeTier {
                name: "balanced".into(),
                model: "claude-sonnet-4".into(),
                provider: "anthropic".into(),
                handles: vec![Complexity::Simple, Complexity::Medium],
                cost_per_1k: 0.009,
            },
            CascadeTier {
                name: "powerful".into(),
                model: "claude-opus-4".into(),
                provider: "anthropic".into(),
                handles: vec![Complexity::Simple, Complexity::Medium, Complexity::Hard],
                cost_per_1k: 0.045,
            },
        ];
        Self::new(tiers, 0.7)
    }

    /// Update budget status for fallback decisions
    pub fn set_budget_remaining(&mut self, pct: f64) {
        self.budget_remaining_pct = pct.clamp(0.0, 1.0);
    }

    /// Route a query to the best model
    pub fn route(&self, prompt: &str, system_prompt: Option<&str>) -> RoutingDecision {
        let complexity = ComplexityClassifier::classify(prompt, system_prompt);

        // Budget-aware downgrade
        let effective_complexity = if self.budget_remaining_pct < 0.1 {
            Complexity::Simple // Force cheapest
        } else if self.budget_remaining_pct < 0.3 {
            match complexity {
                Complexity::Hard => Complexity::Medium,
                other => other,
            }
        } else {
            complexity
        };

        // Find cheapest tier that handles this complexity
        let tier = self
            .tiers
            .iter()
            .filter(|t| t.handles.contains(&effective_complexity))
            .min_by(|a, b| a.cost_per_1k.partial_cmp(&b.cost_per_1k).unwrap());

        match tier {
            Some(t) => RoutingDecision {
                model: t.model.clone(),
                provider: t.provider.clone(),
                tier_name: t.name.clone(),
                complexity,
                reason: if effective_complexity != complexity {
                    format!(
                        "Downgraded from {:?} to {:?} due to budget ({:.0}%)",
                        complexity,
                        effective_complexity,
                        self.budget_remaining_pct * 100.0
                    )
                } else {
                    format!("Routed {:?} task to {} tier", complexity, t.name)
                },
            },
            None => {
                // Fallback to last (most capable) tier
                let last = self.tiers.last().unwrap();
                RoutingDecision {
                    model: last.model.clone(),
                    provider: last.provider.clone(),
                    tier_name: last.name.clone(),
                    complexity,
                    reason: "Fallback to most capable tier".into(),
                }
            }
        }
    }

    /// Decide whether to escalate based on quality assessment
    pub fn should_escalate(
        &self,
        response: &str,
        current_tier: &str,
        expected_min_length: usize,
    ) -> Option<RoutingDecision> {
        let assessment = self.quality_gate.assess(response, expected_min_length);
        if assessment.passed {
            return None;
        }

        // Find next tier up
        let current_idx = self.tiers.iter().position(|t| t.name == current_tier);
        if let Some(idx) = current_idx {
            if idx + 1 < self.tiers.len() {
                let next = &self.tiers[idx + 1];
                return Some(RoutingDecision {
                    model: next.model.clone(),
                    provider: next.provider.clone(),
                    tier_name: next.name.clone(),
                    complexity: Complexity::Hard,
                    reason: format!(
                        "Escalated from {}: confidence {:.2}, {:?}",
                        current_tier, assessment.confidence, assessment.reasons
                    ),
                });
            }
        }
        None
    }
}
