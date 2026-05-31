//! Disk quota monitoring and alerting
//!
//! Tracks the estimated size of cache contents and emits alerts when usage
//! approaches or exceeds configured thresholds. This module is designed to
//! work with in-memory size estimates (serialized byte counts) rather than
//! actual filesystem monitoring, so it can be used in both on-disk and
//! in-memory cache scenarios.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Default quota limit in bytes (100 MB)
pub const DEFAULT_QUOTA_BYTES: u64 = 100 * 1024 * 1024;

/// Default warning threshold (80% of quota)
pub const DEFAULT_WARNING_THRESHOLD: f64 = 0.80;

/// Default critical threshold (95% of quota)
pub const DEFAULT_CRITICAL_THRESHOLD: f64 = 0.95;

/// Alert severity levels for quota violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QuotaAlertLevel {
    /// Usage is within normal bounds
    Normal,

    /// Usage is approaching the quota limit
    Warning,

    /// Usage is near or at the quota limit
    Critical,

    /// Usage has exceeded the quota limit
    Exceeded,
}

impl fmt::Display for QuotaAlertLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
            Self::Exceeded => write!(f, "EXCEEDED"),
        }
    }
}

/// A quota alert emitted when usage crosses a threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaAlert {
    /// Severity of the alert
    pub level: QuotaAlertLevel,

    /// Current usage in bytes
    pub current_bytes: u64,

    /// Configured quota limit in bytes
    pub quota_bytes: u64,

    /// Usage as a percentage (0.0 - 100.0+)
    pub usage_percent: f64,

    /// Human-readable message
    pub message: String,
}

/// Configuration for the quota monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Maximum allowed size in bytes
    pub quota_bytes: u64,

    /// Fraction (0.0..1.0) at which a warning alert is raised
    pub warning_threshold: f64,

    /// Fraction (0.0..1.0) at which a critical alert is raised
    pub critical_threshold: f64,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            quota_bytes: DEFAULT_QUOTA_BYTES,
            warning_threshold: DEFAULT_WARNING_THRESHOLD,
            critical_threshold: DEFAULT_CRITICAL_THRESHOLD,
        }
    }
}

/// Disk quota monitor that tracks cache size and emits alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaMonitor {
    /// Configuration
    config: QuotaConfig,

    /// Current tracked usage in bytes
    current_bytes: u64,

    /// Most recent alert level (for hysteresis / dedup)
    last_alert_level: QuotaAlertLevel,
}

impl QuotaMonitor {
    /// Create a new quota monitor with default configuration
    pub fn new() -> Self {
        Self {
            config: QuotaConfig::default(),
            current_bytes: 0,
            last_alert_level: QuotaAlertLevel::Normal,
        }
    }

    /// Create a quota monitor with custom configuration
    pub fn with_config(config: QuotaConfig) -> Self {
        Self {
            config,
            current_bytes: 0,
            last_alert_level: QuotaAlertLevel::Normal,
        }
    }

    /// Return the current configuration
    pub fn config(&self) -> &QuotaConfig {
        &self.config
    }

    /// Record that `bytes` were added to the cache.
    ///
    /// Returns an alert if the new usage crosses a threshold.
    pub fn record_addition(&mut self, bytes: u64) -> Option<QuotaAlert> {
        self.current_bytes = self.current_bytes.saturating_add(bytes);
        self.check_and_alert()
    }

    /// Record that `bytes` were removed from the cache.
    ///
    /// Returns an alert if the usage dropped back below a threshold
    /// (the level may decrease).
    pub fn record_removal(&mut self, bytes: u64) -> Option<QuotaAlert> {
        self.current_bytes = self.current_bytes.saturating_sub(bytes);
        self.check_and_alert()
    }

    /// Set the current usage to an exact value (e.g. after a full recount).
    pub fn set_current_bytes(&mut self, bytes: u64) {
        self.current_bytes = bytes;
    }

    /// Current usage in bytes
    pub fn current_bytes(&self) -> u64 {
        self.current_bytes
    }

    /// Current usage as a fraction of the quota (0.0..=inf)
    pub fn usage_fraction(&self) -> f64 {
        if self.config.quota_bytes == 0 {
            return if self.current_bytes > 0 {
                f64::INFINITY
            } else {
                0.0
            };
        }
        self.current_bytes as f64 / self.config.quota_bytes as f64
    }

    /// Current usage as a percentage (0.0..=inf)
    pub fn usage_percent(&self) -> f64 {
        self.usage_fraction() * 100.0
    }

    /// Remaining bytes before the quota is reached (0 if already exceeded)
    pub fn remaining_bytes(&self) -> u64 {
        self.config.quota_bytes.saturating_sub(self.current_bytes)
    }

    /// Whether the quota has been exceeded
    pub fn is_exceeded(&self) -> bool {
        self.current_bytes > self.config.quota_bytes
    }

    /// Determine the current alert level without emitting an alert
    pub fn current_alert_level(&self) -> QuotaAlertLevel {
        let fraction = self.usage_fraction();

        if fraction > 1.0 {
            QuotaAlertLevel::Exceeded
        } else if fraction >= self.config.critical_threshold {
            QuotaAlertLevel::Critical
        } else if fraction >= self.config.warning_threshold {
            QuotaAlertLevel::Warning
        } else {
            QuotaAlertLevel::Normal
        }
    }

    /// Check the current usage and emit an alert if the level has changed
    fn check_and_alert(&mut self) -> Option<QuotaAlert> {
        let new_level = self.current_alert_level();

        if new_level == self.last_alert_level {
            return None;
        }

        self.last_alert_level = new_level;

        let percent = self.usage_percent();
        let message = match new_level {
            QuotaAlertLevel::Normal => {
                format!(
                    "Cache usage returned to normal: {:.1}% ({} / {} bytes)",
                    percent, self.current_bytes, self.config.quota_bytes
                )
            }
            QuotaAlertLevel::Warning => {
                format!(
                    "Cache usage warning: {:.1}% ({} / {} bytes)",
                    percent, self.current_bytes, self.config.quota_bytes
                )
            }
            QuotaAlertLevel::Critical => {
                format!(
                    "Cache usage critical: {:.1}% ({} / {} bytes)",
                    percent, self.current_bytes, self.config.quota_bytes
                )
            }
            QuotaAlertLevel::Exceeded => {
                format!(
                    "Cache quota exceeded: {:.1}% ({} / {} bytes)",
                    percent, self.current_bytes, self.config.quota_bytes
                )
            }
        };

        Some(QuotaAlert {
            level: new_level,
            current_bytes: self.current_bytes,
            quota_bytes: self.config.quota_bytes,
            usage_percent: percent,
            message,
        })
    }
}

impl Default for QuotaMonitor {
    fn default() -> Self {
        Self::new()
    }
}
