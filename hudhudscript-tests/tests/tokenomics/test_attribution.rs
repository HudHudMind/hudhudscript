//! Public API tests for tokenomics::attribution

use chrono::Utc;
use hudhudscript_tokenomics::attribution::*;
use uuid::Uuid;

fn make_event(feature: &str, user: &str, model: &str, cost: f64) -> CostEvent {
    CostEvent {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        user_id: Some(user.into()),
        session_id: Some("sess1".into()),
        feature_tag: Some(feature.into()),
        environment: None,
        prompt_version: None,
        model: model.into(),
        provider: "anthropic".into(),
        input_tokens: 1000,
        output_tokens: 500,
        thinking_tokens: 0,
        cached_tokens: 0,
        total_cost_usd: cost,
    }
}

#[test]
fn test_new_attributor() {
    let attr = CostAttributor::new();
    assert_eq!(attr.total_events(), 0);
    assert_eq!(attr.total_cost(), 0.0);
}

#[test]
fn test_default_impl() {
    let attr = CostAttributor::default();
    assert_eq!(attr.total_events(), 0);
}

#[test]
fn test_record_event() {
    let mut attr = CostAttributor::new();
    attr.record_event(make_event("chat", "u1", "claude", 0.05));
    assert_eq!(attr.total_events(), 1);
}

#[test]
fn test_cost_by_feature() {
    let mut attr = CostAttributor::new();
    attr.record_event(make_event("chat", "u1", "claude", 0.05));
    attr.record_event(make_event("chat", "u2", "claude", 0.03));
    attr.record_event(make_event("search", "u1", "gpt-4o", 0.02));
    let by_feat = attr.cost_by_feature();
    assert!((by_feat["chat"] - 0.08).abs() < 0.001);
    assert!((by_feat["search"] - 0.02).abs() < 0.001);
}

#[test]
fn test_cost_by_user() {
    let mut attr = CostAttributor::new();
    attr.record_event(make_event("chat", "alice", "claude", 0.10));
    attr.record_event(make_event("chat", "bob", "claude", 0.05));
    let by_user = attr.cost_by_user();
    assert!((by_user["alice"] - 0.10).abs() < 0.001);
    assert!((by_user["bob"] - 0.05).abs() < 0.001);
}

#[test]
fn test_cost_by_model() {
    let mut attr = CostAttributor::new();
    attr.record_event(make_event("chat", "u1", "claude-haiku", 0.01));
    attr.record_event(make_event("chat", "u1", "gpt-4o", 0.05));
    let by_model = attr.cost_by_model();
    assert_eq!(by_model.len(), 2);
    assert!((by_model["claude-haiku"] - 0.01).abs() < 0.001);
}

#[test]
fn test_cost_by_session() {
    let mut attr = CostAttributor::new();
    attr.record_event(make_event("chat", "u1", "claude", 0.05));
    let by_session = attr.cost_by_session();
    assert!(by_session.contains_key("sess1"));
}

#[test]
fn test_total_cost() {
    let mut attr = CostAttributor::new();
    attr.record_event(make_event("a", "u1", "m", 0.10));
    attr.record_event(make_event("b", "u2", "m", 0.20));
    assert!((attr.total_cost() - 0.30).abs() < 0.001);
}

#[test]
fn test_total_cost_in_range() {
    use chrono::Duration;
    let mut attr = CostAttributor::new();
    let now = Utc::now();
    let yesterday = now - Duration::hours(24);

    let mut event_now = make_event("chat", "u1", "m", 0.10);
    event_now.timestamp = now;
    attr.record_event(event_now);

    let mut event_old = make_event("chat", "u1", "m", 0.50);
    event_old.timestamp = yesterday;
    attr.record_event(event_old);

    let total = attr.total_cost_in_range(now - Duration::minutes(1), now + Duration::minutes(1));
    assert!((total - 0.10).abs() < 0.001);
}

#[test]
fn test_feature_budget() {
    let mut attr = CostAttributor::new();
    attr.set_feature_budget("chat".into(), 1.00);
    attr.record_event(make_event("chat", "u1", "claude", 0.75));
    let status = attr.check_feature_budget("chat").unwrap();
    assert!(!status.exceeded);
    assert!((status.remaining - 0.25).abs() < 0.001);
    assert_eq!(status.feature, "chat");
    assert!((status.budget - 1.00).abs() < 0.001);
}

#[test]
fn test_feature_budget_exceeded() {
    let mut attr = CostAttributor::new();
    attr.set_feature_budget("chat".into(), 0.50);
    attr.record_event(make_event("chat", "u1", "claude", 0.60));
    let status = attr.check_feature_budget("chat").unwrap();
    assert!(status.exceeded);
}

#[test]
fn test_feature_budget_not_set() {
    let attr = CostAttributor::new();
    assert!(attr.check_feature_budget("unknown").is_none());
}

#[test]
fn test_feature_budget_no_events_today() {
    use chrono::Duration;
    let mut attr = CostAttributor::new();
    attr.set_feature_budget("chat".into(), 1.00);

    // Record an event from yesterday — should NOT count toward today's budget
    let yesterday = Utc::now() - Duration::hours(25);
    let old_event = CostEvent {
        id: uuid::Uuid::new_v4(),
        timestamp: yesterday,
        user_id: Some("u1".into()),
        session_id: Some("s1".into()),
        feature_tag: Some("chat".into()),
        environment: None,
        prompt_version: None,
        model: "claude".into(),
        provider: "anthropic".into(),
        input_tokens: 1000,
        output_tokens: 500,
        thinking_tokens: 0,
        cached_tokens: 0,
        total_cost_usd: 0.80,
    };
    attr.record_event(old_event);

    let status = attr.check_feature_budget("chat").unwrap();
    assert_eq!(
        status.spent_today, 0.0,
        "Yesterday's events should not count toward today's budget"
    );
    assert!((status.remaining - 1.00).abs() < 0.001);
    assert!(!status.exceeded);
}
