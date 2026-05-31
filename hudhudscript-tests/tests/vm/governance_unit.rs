#[cfg(test)]
mod tests {
    //! GOV-001 regression tests — unparseable / unknown-format rules must
    //! fail-closed (deny) rather than silently allow the action through.

    use chrono::Utc;
    use hudhudscript_governance::{
        enforcement::{enforce_constitution, EvaluationContext},
        Condition, Constitution, EnforcementLevel, Law,
    };
    use hudhudscript_vm::vm::governance_ops::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn unparseable_rule_string_is_fail_closed() {
        // Build a law whose rule string is not in the supported grammar.
        // Previously this was silently dropped -> constitution allowed the
        // action (fail-open). Now it must push an `__always_fail` condition
        // so the enforcement engine denies the action (fail-closed).
        let mut conditions = Vec::new();
        let rule_str = "totally garbage rule format xyz";
        if let Some(cond) = parse_rule_to_condition(rule_str) {
            conditions.push(cond);
        } else {
            conditions.push(Condition::Equals {
                field: "__always_fail".to_string(),
                value: json!(true),
            });
        }
        assert!(
            !conditions.is_empty(),
            "unparseable rule must produce a fail-closed condition, got empty"
        );

        let mut laws = HashMap::new();
        laws.insert(
            "c.law0".to_string(),
            Law {
                id: "c.law0".to_string(),
                constitution_id: "c".to_string(),
                name: "garbage".to_string(),
                description: String::new(),
                enforcement_level: EnforcementLevel::Mandatory,
                conditions,
            },
        );
        let constitution = Constitution {
            id: "c".to_string(),
            name: "c".to_string(),
            description: None,
            laws,
            created_at: Utc::now(),
            version: 1,
        };

        let ctx = EvaluationContext::new();
        let result = enforce_constitution(&constitution, &ctx, None);
        assert!(
            !result.allowed,
            "unknown-format mandatory rule must deny, got allowed"
        );
    }
}
