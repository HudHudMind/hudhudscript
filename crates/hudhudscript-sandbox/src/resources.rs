//! Resource limits

use crate::{ResourceConfig, ResourceUsage, Result, SandboxError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Bandwidth / IO limit types (Issue #603)
// ---------------------------------------------------------------------------

/// Network bandwidth limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BandwidthLimit {
    /// Maximum bytes per second (0 = unlimited).
    pub bytes_per_second: u64,
}

impl BandwidthLimit {
    /// No bandwidth limit.
    pub fn unlimited() -> Self {
        Self {
            bytes_per_second: 0,
        }
    }

    /// Create a limit of `bps` bytes per second.
    pub fn new(bps: u64) -> Self {
        Self {
            bytes_per_second: bps,
        }
    }

    /// Return `true` if this limit is effectively unlimited.
    pub fn is_unlimited(&self) -> bool {
        self.bytes_per_second == 0
    }

    /// Check whether `bytes` transferred in `elapsed` would exceed the limit.
    pub fn check(&self, bytes: u64, elapsed: Duration) -> bool {
        if self.is_unlimited() {
            return true;
        }
        let secs = elapsed.as_secs_f64();
        if secs <= 0.0 {
            return bytes == 0;
        }
        let rate = bytes as f64 / secs;
        rate <= self.bytes_per_second as f64
    }
}

/// Disk I/O rate limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoLimit {
    /// Maximum read bytes per second (0 = unlimited).
    pub read_bps: u64,
    /// Maximum write bytes per second (0 = unlimited).
    pub write_bps: u64,
}

impl IoLimit {
    /// No I/O limit.
    pub fn unlimited() -> Self {
        Self {
            read_bps: 0,
            write_bps: 0,
        }
    }

    /// Create limits with explicit read/write rates.
    pub fn new(read_bps: u64, write_bps: u64) -> Self {
        Self {
            read_bps,
            write_bps,
        }
    }

    /// Return `true` if both directions are unlimited.
    pub fn is_unlimited(&self) -> bool {
        self.read_bps == 0 && self.write_bps == 0
    }
}

// ---------------------------------------------------------------------------
// ResourceLimits
// ---------------------------------------------------------------------------

pub struct ResourceLimits {
    config: ResourceConfig,
    start_time: Instant,
    memory_used: Arc<Mutex<usize>>,
    cpu_percent: Arc<Mutex<f64>>,
    bandwidth_limit: BandwidthLimit,
    io_limit: IoLimit,
    /// Maximum disk usage in bytes (0 = unlimited).
    max_disk_bytes: u64,
}

impl ResourceLimits {
    pub fn new(config: ResourceConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            memory_used: Arc::new(Mutex::new(0)),
            cpu_percent: Arc::new(Mutex::new(0.0)),
            bandwidth_limit: BandwidthLimit::unlimited(),
            io_limit: IoLimit::unlimited(),
            max_disk_bytes: 0,
        }
    }

    /// Set the network bandwidth limit.
    pub fn set_bandwidth_limit(&mut self, limit: BandwidthLimit) {
        self.bandwidth_limit = limit;
    }

    /// Get the current bandwidth limit.
    pub fn bandwidth_limit(&self) -> &BandwidthLimit {
        &self.bandwidth_limit
    }

    /// Set the disk I/O rate limit.
    pub fn set_io_limit(&mut self, limit: IoLimit) {
        self.io_limit = limit;
    }

    /// Get the current I/O limit.
    pub fn io_limit(&self) -> &IoLimit {
        &self.io_limit
    }

    /// Set the maximum disk usage in bytes (0 = unlimited).
    pub fn set_max_disk_bytes(&mut self, bytes: u64) {
        self.max_disk_bytes = bytes;
    }

    /// Get the maximum disk usage in bytes.
    pub fn max_disk_bytes(&self) -> u64 {
        self.max_disk_bytes
    }

    /// Check whether `bytes_used` disk usage is within the configured limit.
    pub fn check_disk_usage(&self, bytes_used: u64) -> Result<()> {
        if self.max_disk_bytes > 0 && bytes_used > self.max_disk_bytes {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "Disk limit exceeded: {} bytes (limit: {} bytes)",
                bytes_used, self.max_disk_bytes
            )));
        }
        Ok(())
    }

    /// Check whether network transfer of `bytes` over `elapsed` is within the
    /// bandwidth limit.
    pub fn check_bandwidth(&self, bytes: u64, elapsed: Duration) -> Result<()> {
        if !self.bandwidth_limit.check(bytes, elapsed) {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "Bandwidth limit exceeded: {} bytes/sec limit",
                self.bandwidth_limit.bytes_per_second
            )));
        }
        Ok(())
    }

    /// Check if resource usage is within limits
    pub fn check_usage(&self, memory_bytes: usize, cpu_percent: f64) -> Result<()> {
        // Update current usage
        *self.memory_used.lock().unwrap() = memory_bytes;
        *self.cpu_percent.lock().unwrap() = cpu_percent;

        // Check memory limit
        if memory_bytes > self.config.max_memory_bytes {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "Memory limit exceeded: {} bytes (limit: {} bytes)",
                memory_bytes, self.config.max_memory_bytes
            )));
        }

        // Check CPU limit
        if cpu_percent > self.config.max_cpu_percent {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "CPU limit exceeded: {:.1}% (limit: {:.1}%)",
                cpu_percent, self.config.max_cpu_percent
            )));
        }

        // Check execution time limit
        let elapsed = self.start_time.elapsed();
        let max_duration = Duration::from_millis(self.config.max_execution_time_ms);

        if elapsed > max_duration {
            return Err(SandboxError::ResourceLimitExceeded(format!(
                "Execution time limit exceeded: {:?} (limit: {:?})",
                elapsed, max_duration
            )));
        }

        Ok(())
    }

    /// Get current resource usage
    pub fn get_usage(&self) -> ResourceUsage {
        ResourceUsage {
            memory_used: *self.memory_used.lock().unwrap(),
            memory_limit: self.config.max_memory_bytes,
            cpu_percent: *self.cpu_percent.lock().unwrap(),
            cpu_limit: self.config.max_cpu_percent,
        }
    }

    /// Reset execution timer
    pub fn reset_timer(&mut self) {
        self.start_time = Instant::now();
    }
}
