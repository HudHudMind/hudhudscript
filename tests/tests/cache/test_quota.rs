use hudhudscript_cache::{
    QuotaAlertLevel, QuotaConfig, QuotaMonitor, DEFAULT_CRITICAL_THRESHOLD, DEFAULT_QUOTA_BYTES,
    DEFAULT_WARNING_THRESHOLD,
};

fn monitor_with_quota(quota_bytes: u64) -> QuotaMonitor {
    QuotaMonitor::with_config(QuotaConfig {
        quota_bytes,
        warning_threshold: 0.80,
        critical_threshold: 0.95,
    })
}

#[test]
fn test_normal_usage_no_alert() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(100);
    assert!(alert.is_none());
}

#[test]
fn test_warning_alert() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(850);

    assert!(alert.is_some());
    let alert = alert.unwrap();
    assert_eq!(alert.level, QuotaAlertLevel::Warning);
    assert_eq!(alert.current_bytes, 850);
}

#[test]
fn test_critical_alert() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(960);

    assert!(alert.is_some());
    assert_eq!(alert.unwrap().level, QuotaAlertLevel::Critical);
}

#[test]
fn test_exceeded_alert() {
    let mut monitor = monitor_with_quota(1000);
    let alert = monitor.record_addition(1100);

    assert!(alert.is_some());
    assert_eq!(alert.unwrap().level, QuotaAlertLevel::Exceeded);
    assert!(monitor.is_exceeded());
}

#[test]
fn test_no_duplicate_alert_at_same_level() {
    let mut monitor = monitor_with_quota(1000);

    let a1 = monitor.record_addition(850);
    assert!(a1.is_some());

    let a2 = monitor.record_addition(10);
    assert!(a2.is_none());
}

#[test]
fn test_recovery_alert() {
    let mut monitor = monitor_with_quota(1000);

    monitor.record_addition(900);
    let alert = monitor.record_removal(800);

    assert!(alert.is_some());
    assert_eq!(alert.unwrap().level, QuotaAlertLevel::Normal);
}

#[test]
fn test_remaining_bytes() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(300);
    assert_eq!(monitor.remaining_bytes(), 700);
}

#[test]
fn test_remaining_bytes_when_exceeded() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(1500);
    assert_eq!(monitor.remaining_bytes(), 0);
}

#[test]
fn test_usage_percent() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(500);
    assert!((monitor.usage_percent() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_set_current_bytes() {
    let mut monitor = monitor_with_quota(1000);
    monitor.set_current_bytes(750);
    assert_eq!(monitor.current_bytes(), 750);
}

#[test]
fn test_default_config() {
    let config = QuotaConfig::default();
    assert_eq!(config.quota_bytes, DEFAULT_QUOTA_BYTES);
    assert!((config.warning_threshold - DEFAULT_WARNING_THRESHOLD).abs() < f64::EPSILON);
    assert!((config.critical_threshold - DEFAULT_CRITICAL_THRESHOLD).abs() < f64::EPSILON);
}

#[test]
fn test_zero_quota_with_usage_returns_infinity() {
    let mut monitor = QuotaMonitor::with_config(QuotaConfig {
        quota_bytes: 0,
        warning_threshold: 0.80,
        critical_threshold: 0.95,
    });
    monitor.set_current_bytes(100);
    assert!(monitor.usage_fraction().is_infinite());
    assert!(monitor.usage_percent().is_infinite());
}

#[test]
fn test_zero_quota_with_zero_usage_returns_zero() {
    let monitor = QuotaMonitor::with_config(QuotaConfig {
        quota_bytes: 0,
        warning_threshold: 0.80,
        critical_threshold: 0.95,
    });
    assert!((monitor.usage_fraction() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_quota_monitor_default_trait() {
    let monitor = QuotaMonitor::default();
    assert_eq!(monitor.current_bytes(), 0);
    assert_eq!(monitor.config().quota_bytes, DEFAULT_QUOTA_BYTES);
}

#[test]
fn test_quota_alert_level_display() {
    assert_eq!(QuotaAlertLevel::Normal.to_string(), "NORMAL");
    assert_eq!(QuotaAlertLevel::Warning.to_string(), "WARNING");
    assert_eq!(QuotaAlertLevel::Critical.to_string(), "CRITICAL");
    assert_eq!(QuotaAlertLevel::Exceeded.to_string(), "EXCEEDED");
}

#[test]
fn test_quota_alert_level_ordering() {
    assert!(QuotaAlertLevel::Normal < QuotaAlertLevel::Warning);
    assert!(QuotaAlertLevel::Warning < QuotaAlertLevel::Critical);
    assert!(QuotaAlertLevel::Critical < QuotaAlertLevel::Exceeded);
}

#[test]
fn test_current_alert_level_at_boundary() {
    let mut monitor = monitor_with_quota(1000);
    monitor.set_current_bytes(800);
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Warning);

    monitor.set_current_bytes(950);
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Critical);

    monitor.set_current_bytes(1000);
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Critical);

    monitor.set_current_bytes(1001);
    assert_eq!(monitor.current_alert_level(), QuotaAlertLevel::Exceeded);
}

#[test]
fn test_saturating_addition() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(u64::MAX - 10);
    monitor.record_addition(100);
    assert_eq!(monitor.current_bytes(), u64::MAX);
}

#[test]
fn test_saturating_removal() {
    let mut monitor = monitor_with_quota(1000);
    monitor.record_addition(50);
    monitor.record_removal(100);
    assert_eq!(monitor.current_bytes(), 0);
}
