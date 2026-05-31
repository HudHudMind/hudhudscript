//! Extended thinking / reasoning token budget controller

/// Task complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskComplexity {
    Simple,
    Medium,
    Hard,
    Expert,
}

/// Thinking budget controller — maps task complexity to token budgets
pub struct ThinkingBudgetController {
    default_budget: usize,
    tiers: Vec<(String, usize)>,
    thinking_tokens_used: usize,
}

impl ThinkingBudgetController {
    pub fn new(default_budget: usize, tiers: Vec<(String, usize)>) -> Self {
        Self {
            default_budget,
            tiers,
            thinking_tokens_used: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(
            4096,
            vec![
                ("minimal".into(), 1024),
                ("standard".into(), 4096),
                ("deep".into(), 16384),
                ("maximum".into(), 65536),
            ],
        )
    }

    /// Get thinking budget for a complexity level
    pub fn budget_for_complexity(&self, complexity: TaskComplexity) -> usize {
        let idx = match complexity {
            TaskComplexity::Simple => 0,
            TaskComplexity::Medium => 1,
            TaskComplexity::Hard => 2,
            TaskComplexity::Expert => 3,
        };
        self.tiers
            .get(idx)
            .map(|(_, t)| *t)
            .unwrap_or(self.default_budget)
    }

    /// Simple heuristic to classify task complexity
    pub fn classify_task(&self, prompt: &str, has_code: bool, has_math: bool) -> TaskComplexity {
        let word_count = prompt.split_whitespace().count();

        if has_math && has_code {
            return TaskComplexity::Expert;
        }
        if has_math || (has_code && word_count > 200) {
            return TaskComplexity::Hard;
        }
        if has_code || word_count > 100 {
            return TaskComplexity::Medium;
        }
        TaskComplexity::Simple
    }

    /// Anthropic thinking parameter
    pub fn to_anthropic_param(&self, budget: usize) -> serde_json::Value {
        serde_json::json!({
            "type": "enabled",
            "budget_tokens": budget
        })
    }

    /// OpenAI reasoning_effort parameter
    pub fn to_openai_param(&self, budget: usize) -> serde_json::Value {
        let effort = if budget <= 1024 {
            "low"
        } else if budget <= 8192 {
            "medium"
        } else {
            "high"
        };
        serde_json::json!({ "reasoning_effort": effort })
    }

    /// Record thinking tokens used
    pub fn record_thinking_tokens(&mut self, tokens: usize) {
        self.thinking_tokens_used += tokens;
    }

    pub fn total_thinking_tokens(&self) -> usize {
        self.thinking_tokens_used
    }
}
