// ============================================================================
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Governance model defining how rules and laws are enforced
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceModel {
    /// Model type
    pub model_type: GovernanceModelType,

    /// Constitution compliance coefficient (0.0 - 1.0)
    /// 1.0 = must comply, 0.0 = can ignore
    pub constitution_compliance: f64,

    /// Law flexibility coefficient (0.0 - 1.0)
    /// 1.0 = fully flexible, 0.0 = rigid
    pub law_flexibility: f64,

    /// Rule enforcement coefficient (0.0 - 1.0)
    /// 1.0 = strict enforcement, 0.0 = optional
    pub rule_enforcement: f64,

    /// RuleSet usage coefficient (0.0 - 1.0)
    /// 1.0 = must use rulesets, 0.0 = can bypass
    pub ruleset_usage: f64,

    /// RuleChain enforcement coefficient (0.0 - 1.0)
    /// 1.0 = strict chain execution, 0.0 = can skip
    pub rulechain_enforcement: f64,

    /// Special agent roles with privileges
    pub special_roles: HashMap<String, RolePrivileges>,

    /// Decision making strategy
    pub decision_strategy: DecisionStrategy,
}

/// Types of governance models
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceModelType {
    Democracy,
    Monarchy,
    Parliamentary,
    Technocracy,
    Meritocracy,
    Anarchy,
    Chaos,
    Oligarchy,
    Theocracy,
    Autocracy,
    Consensus,
    Hybrid,
}

/// Role privileges in governance model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RolePrivileges {
    /// Can bypass constitution
    pub bypass_constitution: bool,

    /// Can modify laws
    pub modify_laws: bool,

    /// Can override rules
    pub override_rules: bool,

    /// Voting weight (1.0 = normal, >1.0 = more weight)
    pub voting_weight: f64,
}

/// Decision making strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecisionStrategy {
    /// Simple majority (>50%)
    Majority,

    /// Supermajority (>66%)
    Supermajority,

    /// Unanimous (100%)
    Unanimous,

    /// Single authority decides
    SingleAuthority { authority_role: String },

    /// Weighted voting based on role
    WeightedVoting,

    /// Consensus building
    Consensus,

    /// Random selection
    Random,

    /// Data/metric driven
    DataDriven { metric: String },
}

impl GovernanceModel {
    /// Create a new governance model
    pub fn new(
        model_type: GovernanceModelType,
        constitution_compliance: f64,
        law_flexibility: f64,
        rule_enforcement: f64,
        decision_strategy: DecisionStrategy,
    ) -> Self {
        Self {
            model_type,
            constitution_compliance: constitution_compliance.clamp(0.0, 1.0),
            law_flexibility: law_flexibility.clamp(0.0, 1.0),
            rule_enforcement: rule_enforcement.clamp(0.0, 1.0),
            ruleset_usage: 1.0,
            rulechain_enforcement: 1.0,
            special_roles: HashMap::new(),
            decision_strategy,
        }
    }

    /// Create a Democracy model
    pub fn democracy() -> Self {
        Self::new(
            GovernanceModelType::Democracy,
            1.0, // Full constitution compliance
            0.0, // No law flexibility
            1.0, // Full rule enforcement
            DecisionStrategy::Majority,
        )
    }

    /// Create a Monarchy model
    pub fn monarchy(monarch_role: String) -> Self {
        let mut model = Self::new(
            GovernanceModelType::Monarchy,
            0.0, // Monarch doesn't follow constitution
            1.0, // Full law flexibility for monarch
            0.5, // Flexible rule enforcement
            DecisionStrategy::SingleAuthority {
                authority_role: monarch_role.clone(),
            },
        );

        model.special_roles.insert(
            monarch_role,
            RolePrivileges {
                bypass_constitution: true,
                modify_laws: true,
                override_rules: true,
                voting_weight: 10.0,
            },
        );

        model
    }

    /// Create a Parliamentary model
    pub fn parliamentary() -> Self {
        Self::new(
            GovernanceModelType::Parliamentary,
            1.0, // Full constitution compliance
            0.2, // Limited law flexibility
            0.9, // High rule enforcement
            DecisionStrategy::Majority,
        )
    }

    /// Create a Technocracy model
    pub fn technocracy() -> Self {
        Self::new(
            GovernanceModelType::Technocracy,
            0.8,  // High but flexible constitution compliance
            0.4,  // Moderate law flexibility for optimization
            0.95, // Very high rule enforcement
            DecisionStrategy::DataDriven {
                metric: "efficiency".to_string(),
            },
        )
    }

    /// Create an Anarchy model
    pub fn anarchy() -> Self {
        Self::new(
            GovernanceModelType::Anarchy,
            0.0, // No constitution
            1.0, // Full flexibility
            0.0, // No enforcement
            DecisionStrategy::Random,
        )
    }

    /// Create a Chaos model (random coefficients)
    pub fn chaos() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self::new(
            GovernanceModelType::Chaos,
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            DecisionStrategy::Random,
        )
    }

    /// Check if an agent has special privileges
    pub fn has_privileges(&self, role: &str) -> Option<&RolePrivileges> {
        self.special_roles.get(role)
    }

    /// Add special role privileges
    pub fn add_special_role(&mut self, role: String, privileges: RolePrivileges) {
        self.special_roles.insert(role, privileges);
    }
}

impl Default for GovernanceModel {
    fn default() -> Self {
        Self {
            model_type: GovernanceModelType::Democracy,
            constitution_compliance: 1.0,
            law_flexibility: 0.5,
            rule_enforcement: 1.0,
            ruleset_usage: 1.0,
            rulechain_enforcement: 1.0,
            special_roles: HashMap::new(),
            decision_strategy: DecisionStrategy::Consensus,
        }
    }
}
