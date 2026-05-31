//! Cache transmission protocol for distributed systems

use crate::cache::CommandCache;
use crate::serialization::{deserialize_definitions, serialize_definitions};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Cache transmission metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionMetadata {
    /// Protocol version
    pub version: String,

    /// Timestamp of transmission
    pub timestamp: SystemTime,

    /// Number of items in transmission
    pub item_count: usize,

    /// Transmission ID (for tracking)
    pub transmission_id: String,

    /// Source agent/node ID
    pub source_id: Option<String>,

    /// Destination agent/node ID
    pub destination_id: Option<String>,

    /// Transmission type (full or incremental)
    pub transmission_type: TransmissionType,
}

/// Type of cache transmission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransmissionType {
    /// Full cache transmission
    Full,

    /// Incremental update (only changed items)
    Incremental,
}

/// Cache transmission package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheTransmission {
    /// Metadata
    pub metadata: TransmissionMetadata,

    /// Serialized cache data
    pub data: String,

    /// Checksum for validation
    pub checksum: Option<String>,
}

/// Transmission result
#[derive(Debug, Clone)]
pub enum TransmissionResult {
    /// Transmission successful
    Success {
        items_transmitted: usize,
        duration: Duration,
    },

    /// Transmission failed
    Failed { error: String, retry_count: usize },

    /// Transmission pending retry
    PendingRetry {
        retry_count: usize,
        next_retry_at: SystemTime,
    },
}

/// Transmission configuration
#[derive(Debug, Clone)]
pub struct TransmissionConfig {
    /// Maximum retry attempts
    pub max_retries: usize,

    /// Initial retry delay
    pub initial_retry_delay: Duration,

    /// Maximum retry delay
    pub max_retry_delay: Duration,

    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,

    /// Enable checksum validation
    pub validate_checksum: bool,
}

impl Default for TransmissionConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            validate_checksum: true,
        }
    }
}

/// Cache transmitter
pub struct CacheTransmitter {
    /// Configuration
    pub config: TransmissionConfig,

    /// Transmission counter (for generating IDs)
    transmission_counter: std::sync::atomic::AtomicUsize,
}

impl CacheTransmitter {
    /// Create new cache transmitter
    pub fn new() -> Self {
        Self {
            config: TransmissionConfig::default(),
            transmission_counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TransmissionConfig) -> Self {
        Self {
            config,
            transmission_counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Prepare full cache transmission
    pub fn prepare_full_transmission(
        &self,
        cache: &CommandCache,
        source_id: Option<String>,
        destination_id: Option<String>,
    ) -> Result<CacheTransmission, String> {
        let transmission_id = self.generate_transmission_id();
        let data =
            serialize_definitions(cache).map_err(|e| format!("Serialization failed: {}", e))?;
        let item_count = cache.constitutions.len() + cache.laws.len() + cache.rules.len();

        let metadata = TransmissionMetadata {
            version: "1.0".to_string(),
            timestamp: SystemTime::now(),
            item_count,
            transmission_id,
            source_id,
            destination_id,
            transmission_type: TransmissionType::Full,
        };

        let checksum = if self.config.validate_checksum {
            Some(self.calculate_checksum(&data))
        } else {
            None
        };

        Ok(CacheTransmission {
            metadata,
            data,
            checksum,
        })
    }

    /// Prepare incremental transmission (only changed items)
    pub fn prepare_incremental_transmission(
        &self,
        cache: &CommandCache,
        changed_ids: Vec<String>,
        source_id: Option<String>,
        destination_id: Option<String>,
    ) -> Result<CacheTransmission, String> {
        let transmission_id = self.generate_transmission_id();

        // Incremental transmission is not yet wired — full serialization is
        // used until the diff-only path lands (tracked separately).
        let data =
            serialize_definitions(cache).map_err(|e| format!("Serialization failed: {}", e))?;

        let metadata = TransmissionMetadata {
            version: "1.0".to_string(),
            timestamp: SystemTime::now(),
            item_count: changed_ids.len(),
            transmission_id,
            source_id,
            destination_id,
            transmission_type: TransmissionType::Incremental,
        };

        let checksum = if self.config.validate_checksum {
            Some(self.calculate_checksum(&data))
        } else {
            None
        };

        Ok(CacheTransmission {
            metadata,
            data,
            checksum,
        })
    }

    /// Receive and validate transmission
    pub fn receive_transmission(
        &self,
        transmission: &CacheTransmission,
        target_cache: &mut CommandCache,
    ) -> Result<TransmissionResult, String> {
        // Validate checksum if enabled
        if self.config.validate_checksum {
            if let Some(checksum) = &transmission.checksum {
                let calculated = self.calculate_checksum(&transmission.data);
                if &calculated != checksum {
                    return Err("Checksum validation failed".to_string());
                }
            }
        }

        // Deserialize into new cache
        let start = SystemTime::now();
        let new_cache = deserialize_definitions(&transmission.data)
            .map_err(|e| format!("Deserialization failed: {}", e))?;

        // Merge into target cache (replace for now)
        *target_cache = new_cache;

        let duration = start.elapsed().unwrap_or(Duration::from_secs(0));

        Ok(TransmissionResult::Success {
            items_transmitted: transmission.metadata.item_count,
            duration,
        })
    }

    /// Transmit with retry logic
    pub fn transmit_with_retry<F>(
        &self,
        transmission: &CacheTransmission,
        mut send_fn: F,
    ) -> TransmissionResult
    where
        F: FnMut(&CacheTransmission) -> Result<(), String>,
    {
        let mut retry_count = 0;
        let mut delay = self.config.initial_retry_delay;
        let start = std::time::Instant::now();

        loop {
            match send_fn(transmission) {
                Ok(_) => {
                    return TransmissionResult::Success {
                        items_transmitted: transmission.metadata.item_count,
                        // v0.4.47.9 — Issue #812: real elapsed time tracking
                        duration: start.elapsed(),
                    };
                }
                Err(error) => {
                    retry_count += 1;

                    if retry_count >= self.config.max_retries {
                        return TransmissionResult::Failed { error, retry_count };
                    }

                    // Exponential backoff
                    std::thread::sleep(delay);
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * self.config.backoff_multiplier)
                            .min(self.config.max_retry_delay.as_secs_f64()),
                    );
                }
            }
        }
    }

    /// Calculate retry delay with exponential backoff
    pub fn calculate_retry_delay(&self, retry_count: usize) -> Duration {
        let delay_secs = self.config.initial_retry_delay.as_secs_f64()
            * self.config.backoff_multiplier.powi(retry_count as i32);

        Duration::from_secs_f64(delay_secs.min(self.config.max_retry_delay.as_secs_f64()))
    }

    /// Generate unique transmission ID
    fn generate_transmission_id(&self) -> String {
        let counter = self
            .transmission_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!(
            "tx-{}-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            counter
        )
    }

    /// Calculate SHA-256 checksum of the given data.
    pub fn calculate_checksum(&self, data: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl Default for CacheTransmitter {
    fn default() -> Self {
        Self::new()
    }
}
