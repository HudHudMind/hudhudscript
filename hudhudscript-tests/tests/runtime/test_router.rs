use hudhudscript_runtime::router::{ProviderRouter, RouterConfig, RoutingStrategy};
use std::time::Duration;

#[tokio::test]
async fn test_round_robin_routing() {
    let router = ProviderRouter::new(RouterConfig {
        strategy: RoutingStrategy::RoundRobin,
        ..Default::default()
    });

    router.add_provider("openai", 0.03).await;
    router.add_provider("anthropic", 0.015).await;

    let first = router.route().await.unwrap();
    let second = router.route().await.unwrap();
    // Should alternate between providers.
    assert_ne!(first, second);
}

#[tokio::test]
async fn test_least_latency_routing() {
    let router = ProviderRouter::new(RouterConfig {
        strategy: RoutingStrategy::LeastLatency,
        ..Default::default()
    });

    router.add_provider("slow", 0.03).await;
    router.add_provider("fast", 0.03).await;

    router.record_success("slow", 500.0).await;
    router.record_success("fast", 50.0).await;

    let chosen = router.route().await.unwrap();
    assert_eq!(chosen, "fast");
}

#[tokio::test]
async fn test_cost_optimized_routing() {
    let router = ProviderRouter::new(RouterConfig {
        strategy: RoutingStrategy::CostOptimized,
        ..Default::default()
    });

    router.add_provider("expensive", 0.06).await;
    router.add_provider("cheap", 0.001).await;

    let chosen = router.route().await.unwrap();
    assert_eq!(chosen, "cheap");
}

#[tokio::test]
async fn test_manual_routing() {
    let router = ProviderRouter::new(RouterConfig {
        strategy: RoutingStrategy::Manual("anthropic".to_string()),
        ..Default::default()
    });

    router.add_provider("openai", 0.03).await;
    router.add_provider("anthropic", 0.015).await;

    let chosen = router.route().await.unwrap();
    assert_eq!(chosen, "anthropic");
}

#[tokio::test]
async fn test_mark_unhealthy_after_failures() {
    let router = ProviderRouter::new(RouterConfig {
        max_failures: 2,
        strategy: RoutingStrategy::Manual("flaky".to_string()),
        ..Default::default()
    });

    router.add_provider("flaky", 0.03).await;

    // First failure — still healthy.
    router.record_failure("flaky").await;
    assert!(router.route().await.is_some());

    // Second failure — threshold reached, now unhealthy.
    router.record_failure("flaky").await;
    assert!(router.route().await.is_none());
}

#[tokio::test]
async fn test_success_resets_error_count() {
    let router = ProviderRouter::new(RouterConfig {
        max_failures: 3,
        strategy: RoutingStrategy::Manual("provider".to_string()),
        ..Default::default()
    });

    router.add_provider("provider", 0.03).await;

    router.record_failure("provider").await;
    router.record_failure("provider").await;
    // A success resets the counter.
    router.record_success("provider", 100.0).await;
    router.record_failure("provider").await;
    router.record_failure("provider").await;
    // Still below threshold (2 < 3).
    assert!(router.route().await.is_some());
}

#[tokio::test]
async fn test_auto_recovery() {
    let router = ProviderRouter::new(RouterConfig {
        max_failures: 1,
        recovery_timeout: Duration::from_millis(50),
        strategy: RoutingStrategy::Manual("recoverable".to_string()),
    });

    router.add_provider("recoverable", 0.03).await;

    router.record_failure("recoverable").await;
    assert!(router.route().await.is_none());

    // Wait for recovery timeout.
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(router.route().await.is_some());
}

#[tokio::test]
async fn test_manual_mark_unhealthy_and_healthy() {
    let router = ProviderRouter::new(RouterConfig {
        strategy: RoutingStrategy::Manual("target".to_string()),
        ..Default::default()
    });

    router.add_provider("target", 0.03).await;
    assert!(router.route().await.is_some());

    router.mark_unhealthy("target").await;
    assert!(router.route().await.is_none());

    router.mark_healthy("target").await;
    assert!(router.route().await.is_some());
}

#[tokio::test]
async fn test_health_snapshot() {
    let router = ProviderRouter::default();
    router.add_provider("a", 0.01).await;
    router.add_provider("b", 0.02).await;

    let snap = router.health_snapshot().await;
    assert_eq!(snap.len(), 2);
    assert!(snap.iter().all(|h| h.is_healthy));
}

#[tokio::test]
async fn test_no_healthy_providers_returns_none() {
    let router = ProviderRouter::new(RouterConfig {
        max_failures: 1,
        strategy: RoutingStrategy::RoundRobin,
        ..Default::default()
    });

    router.add_provider("only", 0.03).await;
    router.record_failure("only").await;

    assert!(router.route().await.is_none());
}

#[tokio::test]
async fn test_remove_provider() {
    let router = ProviderRouter::default();
    router.add_provider("temp", 0.03).await;
    assert_eq!(router.health_snapshot().await.len(), 1);

    router.remove_provider("temp").await;
    assert_eq!(router.health_snapshot().await.len(), 0);
}
