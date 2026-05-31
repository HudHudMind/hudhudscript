//! Tests extracted from hudhudscript-sandbox/src/resources.rs

use hudhudscript_sandbox::{
    resources::{BandwidthLimit, IoLimit, ResourceLimits},
    ResourceConfig,
};
use std::time::Duration;

#[test]
fn test_memory_limit() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024, // 100 MB
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };

    let limits = ResourceLimits::new(config);

    // Should allow within limit
    assert!(limits.check_usage(50 * 1024 * 1024, 50.0).is_ok());

    // Should deny over limit
    assert!(limits.check_usage(150 * 1024 * 1024, 50.0).is_err());
}

#[test]
fn test_cpu_limit() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };

    let limits = ResourceLimits::new(config);

    // Should allow within limit
    assert!(limits.check_usage(50 * 1024 * 1024, 70.0).is_ok());

    // Should deny over limit
    assert!(limits.check_usage(50 * 1024 * 1024, 90.0).is_err());
}

#[test]
fn test_get_usage() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };

    let limits = ResourceLimits::new(config);

    // Update usage
    let _ = limits.check_usage(50 * 1024 * 1024, 60.0);

    // Get usage
    let usage = limits.get_usage();
    assert_eq!(usage.memory_used, 50 * 1024 * 1024);
    assert_eq!(usage.memory_limit, 100 * 1024 * 1024);
    assert_eq!(usage.cpu_percent, 60.0);
    assert_eq!(usage.cpu_limit, 80.0);
}

#[test]
fn test_bandwidth_limit_unlimited() {
    let bw = BandwidthLimit::unlimited();
    assert!(bw.is_unlimited());
    assert!(bw.check(1_000_000, Duration::from_secs(1)));
}

#[test]
fn test_bandwidth_limit_within() {
    let bw = BandwidthLimit::new(1_000_000); // 1 MB/s
    assert!(!bw.is_unlimited());
    // 500 KB in 1 second = 500 KB/s — within limit
    assert!(bw.check(500_000, Duration::from_secs(1)));
}

#[test]
fn test_bandwidth_limit_exceeded() {
    let bw = BandwidthLimit::new(1_000_000); // 1 MB/s
                                             // 2 MB in 1 second — exceeds limit
    assert!(!bw.check(2_000_000, Duration::from_secs(1)));
}

#[test]
fn test_io_limit() {
    let io = IoLimit::new(10_000_000, 5_000_000);
    assert!(!io.is_unlimited());
    assert_eq!(io.read_bps, 10_000_000);
    assert_eq!(io.write_bps, 5_000_000);

    let io2 = IoLimit::unlimited();
    assert!(io2.is_unlimited());
}

#[test]
fn test_disk_usage_within() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let mut limits = ResourceLimits::new(config);
    limits.set_max_disk_bytes(1_000_000);
    assert!(limits.check_disk_usage(500_000).is_ok());
}

#[test]
fn test_disk_usage_exceeded() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let mut limits = ResourceLimits::new(config);
    limits.set_max_disk_bytes(1_000_000);
    assert!(limits.check_disk_usage(2_000_000).is_err());
}

#[test]
fn test_check_bandwidth_ok() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let mut limits = ResourceLimits::new(config);
    limits.set_bandwidth_limit(BandwidthLimit::new(1_000_000));
    assert!(limits
        .check_bandwidth(500_000, Duration::from_secs(1))
        .is_ok());
}

#[test]
fn test_reset_timer() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let mut limits = ResourceLimits::new(config);
    limits.reset_timer();
    // After reset, time check should pass
    assert!(limits.check_usage(0, 0.0).is_ok());
}

#[test]
fn test_bandwidth_zero_elapsed() {
    let bw = BandwidthLimit::new(1_000_000);
    // 0 bytes in 0 seconds → ok
    assert!(bw.check(0, Duration::from_secs(0)));
    // nonzero bytes in 0 seconds → not ok
    assert!(!bw.check(100, Duration::from_secs(0)));
}

#[test]
fn test_disk_usage_unlimited() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let limits = ResourceLimits::new(config);
    // max_disk_bytes is 0 (unlimited by default)
    assert_eq!(limits.max_disk_bytes(), 0);
    assert!(limits.check_disk_usage(999_999_999).is_ok());
}

#[test]
fn test_set_and_get_io_limit() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let mut limits = ResourceLimits::new(config);
    let io = IoLimit::new(500_000, 250_000);
    limits.set_io_limit(io);
    assert_eq!(limits.io_limit().read_bps, 500_000);
    assert_eq!(limits.io_limit().write_bps, 250_000);
    assert!(!limits.io_limit().is_unlimited());
}

#[test]
fn test_set_and_get_bandwidth_limit() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let mut limits = ResourceLimits::new(config);
    limits.set_bandwidth_limit(BandwidthLimit::new(5_000_000));
    assert_eq!(limits.bandwidth_limit().bytes_per_second, 5_000_000);
    assert!(!limits.bandwidth_limit().is_unlimited());
}

#[test]
fn test_check_bandwidth_exceeded() {
    let config = ResourceConfig {
        max_memory_bytes: 100 * 1024 * 1024,
        max_cpu_percent: 80.0,
        max_execution_time_ms: 30_000,
    };
    let mut limits = ResourceLimits::new(config);
    limits.set_bandwidth_limit(BandwidthLimit::new(1_000_000));
    assert!(limits
        .check_bandwidth(5_000_000, Duration::from_secs(1))
        .is_err());
}
