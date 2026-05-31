//! Tests for tokenomics::reinforcement — ReinforcementAgent

use chrono::Utc;
use hudhudscript_tokenomics::reinforcement::ReinforcementAgent;
use hudhudscript_tokenomics::types::Reward;

// ---------------------------------------------------------------------------
// ReinforcementAgent::new
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_new_does_not_panic() {
    let _agent = ReinforcementAgent::new(true);
}

#[tokio::test]
async fn test_new_false_enabled() {
    let _agent = ReinforcementAgent::new(false);
}

// ---------------------------------------------------------------------------
// ReinforcementAgent::with_params
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_with_params_custom() {
    let agent = ReinforcementAgent::with_params(true, 0.5, 0.8, 0.2);
    let ctx = serde_json::json!({"s": "test"});
    let id = agent.record_action("a1", ctx).await.unwrap();
    agent
        .submit_reward(Reward {
            action_id: id,
            reward_value: 3.0,
            feedback: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();
    agent.update_policy().await.unwrap();
}

#[tokio::test]
async fn test_with_params_extreme_alpha() {
    let agent = ReinforcementAgent::with_params(true, 10.0, 0.5, 0.1);
    // alpha should be clamped to 1.0
    let ctx = serde_json::json!({"x": 1});
    let id = agent.record_action("a", ctx).await.unwrap();
    agent
        .submit_reward(Reward {
            action_id: id,
            reward_value: 1.0,
            feedback: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_with_params_extreme_epsilon() {
    let agent = ReinforcementAgent::with_params(true, 0.1, 0.9, 5.0);
    // epsilon clamped to 1.0 => always explore
    let ctx = serde_json::json!({"x": 1});
    let actions = vec!["a".to_string(), "b".to_string()];
    let chosen = agent.select_action(&ctx, &actions).await;
    assert!(chosen.is_some());
}

// ---------------------------------------------------------------------------
// record_action
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_record_action_returns_uuid() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"operation": "compile", "size": "large"});
    let id = agent.record_action("cache", ctx).await.unwrap();
    assert!(!id.is_nil());
}

#[tokio::test]
async fn test_record_action_unique_ids() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"x": 1});
    let id1 = agent.record_action("a", ctx.clone()).await.unwrap();
    let id2 = agent.record_action("a", ctx).await.unwrap();
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn test_record_multiple_actions() {
    let agent = ReinforcementAgent::new(true);
    for i in 0..10 {
        let ctx = serde_json::json!({"i": i});
        let id = agent
            .record_action(&format!("action_{}", i), ctx)
            .await
            .unwrap();
        assert!(!id.is_nil());
    }
}

// ---------------------------------------------------------------------------
// submit_reward
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_submit_reward_success() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"op": "compile"});
    let id = agent.record_action("cache", ctx).await.unwrap();
    let reward = Reward {
        action_id: id,
        reward_value: 1.0,
        feedback: None,
        timestamp: Utc::now(),
    };
    assert!(agent.submit_reward(reward).await.is_ok());
}

#[tokio::test]
async fn test_submit_reward_with_feedback() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"op": "test"});
    let id = agent.record_action("action", ctx).await.unwrap();
    let reward = Reward {
        action_id: id,
        reward_value: 5.0,
        feedback: Some("excellent".into()),
        timestamp: Utc::now(),
    };
    assert!(agent.submit_reward(reward).await.is_ok());
}

#[tokio::test]
async fn test_submit_reward_negative_value() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"op": "test"});
    let id = agent.record_action("bad_action", ctx).await.unwrap();
    let reward = Reward {
        action_id: id,
        reward_value: -5.0,
        feedback: None,
        timestamp: Utc::now(),
    };
    assert!(agent.submit_reward(reward).await.is_ok());
}

#[tokio::test]
async fn test_submit_reward_unknown_action_id() {
    let agent = ReinforcementAgent::new(true);
    let reward = Reward {
        action_id: uuid::Uuid::new_v4(),
        reward_value: 1.0,
        feedback: None,
        timestamp: Utc::now(),
    };
    // Unknown action_id: no record to update, but no error
    assert!(agent.submit_reward(reward).await.is_ok());
}

// ---------------------------------------------------------------------------
// select_action
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_select_action_empty_actions() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"x": 1});
    assert_eq!(agent.select_action(&ctx, &[]).await, None);
}

#[tokio::test]
async fn test_select_action_single_choice() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"x": 1});
    let actions = vec!["only_option".to_string()];
    let chosen = agent.select_action(&ctx, &actions).await;
    assert_eq!(chosen, Some("only_option".to_string()));
}

#[tokio::test]
async fn test_select_action_greedy_chooses_best() {
    let agent = ReinforcementAgent::with_params(true, 0.1, 0.9, 0.0);
    let ctx = serde_json::json!({"x": 1});
    let id = agent.record_action("good", ctx.clone()).await.unwrap();
    agent
        .submit_reward(Reward {
            action_id: id,
            reward_value: 10.0,
            feedback: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();
    let actions = vec!["good".to_string(), "bad".to_string()];
    let chosen = agent.select_action(&ctx, &actions).await;
    assert_eq!(chosen, Some("good".to_string()));
}

#[tokio::test]
async fn test_select_action_exploration() {
    let agent = ReinforcementAgent::with_params(true, 0.1, 0.9, 1.0);
    let ctx = serde_json::json!({"x": 1});
    let actions = vec!["a".to_string(), "b".to_string()];
    let chosen = agent.select_action(&ctx, &actions).await;
    assert!(chosen.is_some());
    assert!(actions.contains(&chosen.unwrap()));
}

// ---------------------------------------------------------------------------
// update_policy
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_update_policy_success() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"mode": "test"});
    let id = agent.record_action("action_a", ctx).await.unwrap();
    agent
        .submit_reward(Reward {
            action_id: id,
            reward_value: 5.0,
            feedback: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();
    assert!(agent.update_policy().await.is_ok());
}

#[tokio::test]
async fn test_update_policy_no_rewards() {
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!({"x": 1});
    agent.record_action("a", ctx).await.unwrap();
    // No reward submitted; policy update should still succeed
    assert!(agent.update_policy().await.is_ok());
}

#[tokio::test]
async fn test_update_policy_multiple_rewards() {
    let agent = ReinforcementAgent::with_params(true, 0.1, 0.9, 0.0);
    let ctx = serde_json::json!({"s": "test"});
    let id1 = agent.record_action("a1", ctx.clone()).await.unwrap();
    agent
        .submit_reward(Reward {
            action_id: id1,
            reward_value: 5.0,
            feedback: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();
    let id2 = agent.record_action("a2", ctx.clone()).await.unwrap();
    agent
        .submit_reward(Reward {
            action_id: id2,
            reward_value: 3.0,
            feedback: None,
            timestamp: Utc::now(),
        })
        .await
        .unwrap();
    agent.update_policy().await.unwrap();

    // After update, greedy selection should prefer a1 (higher reward)
    let actions = vec!["a1".to_string(), "a2".to_string()];
    let chosen = agent.select_action(&ctx, &actions).await;
    assert_eq!(chosen, Some("a1".to_string()));
}

// ---------------------------------------------------------------------------
// Integration: record + reward + select
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_full_cycle() {
    let agent = ReinforcementAgent::with_params(true, 0.1, 0.9, 0.0);
    let ctx = serde_json::json!({"task": "compile"});

    // Train on two actions
    for _ in 0..5 {
        let id = agent
            .record_action("fast_compile", ctx.clone())
            .await
            .unwrap();
        agent
            .submit_reward(Reward {
                action_id: id,
                reward_value: 8.0,
                feedback: None,
                timestamp: Utc::now(),
            })
            .await
            .unwrap();

        let id2 = agent
            .record_action("slow_compile", ctx.clone())
            .await
            .unwrap();
        agent
            .submit_reward(Reward {
                action_id: id2,
                reward_value: 2.0,
                feedback: None,
                timestamp: Utc::now(),
            })
            .await
            .unwrap();
    }

    agent.update_policy().await.unwrap();

    let actions = vec!["fast_compile".to_string(), "slow_compile".to_string()];
    let chosen = agent.select_action(&ctx, &actions).await;
    assert_eq!(chosen, Some("fast_compile".to_string()));
}

#[tokio::test]
async fn test_context_as_non_object() {
    // Context is not a JSON object, but a string
    let agent = ReinforcementAgent::new(true);
    let ctx = serde_json::json!("simple_string_context");
    let id = agent.record_action("a", ctx).await.unwrap();
    assert!(!id.is_nil());
}
