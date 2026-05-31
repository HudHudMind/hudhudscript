//! Rate Limiting Engine (Issue #607)
//!
//! Enforces per-provider rate limits expressed as:
//! - **RPM** — requests per minute
//! - **TPM** — tokens per minute
//!
//! Uses a sliding-window counter backed by a `VecDeque` of timestamped events.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::cost::Provider;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RateLimitError {
    RpmExceeded {
        provider: Provider,
        current: usize,
        limit: usize,
    },
    TpmExceeded {
        provider: Provider,
        current: usize,
        limit: usize,
    },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            RateLimitError::RpmExceeded {
                provider,
                current,
                limit,
            } => write!(
                f,
                "RPM limit exceeded for {}: {}/{} requests in the last minute",
                provider, current, limit
            ),
            RateLimitError::TpmExceeded {
                provider,
                current,
                limit,
            } => write!(
                f,
                "TPM limit exceeded for {}: {}/{} tokens in the last minute",
                provider, current, limit
            ),
        }
    }
}

impl std::error::Error for RateLimitError {}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Rate limits for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRateLimit {
    /// Maximum requests per minute (0 = unlimited).
    pub rpm: usize,
    /// Maximum tokens per minute (0 = unlimited).
    pub tpm: usize,
}

impl Default for ProviderRateLimit {
    fn default() -> Self {
        Self {
            rpm: 60,
            tpm: 90_000,
        }
    }
}

/// Build sensible default rate limits per provider.
pub fn default_rate_limits() -> HashMap<Provider, ProviderRateLimit> {
    let mut m = HashMap::new();
    m.insert(
        Provider::OpenAI,
        ProviderRateLimit {
            rpm: 60,
            tpm: 90_000,
        },
    );
    m.insert(
        Provider::Anthropic,
        ProviderRateLimit {
            rpm: 60,
            tpm: 80_000,
        },
    );
    m.insert(
        Provider::Ollama,
        ProviderRateLimit {
            rpm: 0, // local, unlimited
            tpm: 0,
        },
    );
    m.insert(
        Provider::DeepSeek,
        ProviderRateLimit {
            rpm: 60,
            tpm: 60_000,
        },
    );
    m
}

// ---------------------------------------------------------------------------
// Sliding-window counter
// ---------------------------------------------------------------------------

/// A single recorded event for the sliding window.
#[derive(Debug, Clone)]
struct Event {
    instant: Instant,
    tokens: usize,
}

/// Sliding-window counters for one provider.
#[derive(Debug)]
struct WindowCounter {
    events: VecDeque<Event>,
}

impl WindowCounter {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    /// Remove events older than the window duration.
    fn prune(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        while let Some(front) = self.events.front() {
            if front.instant < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Number of events in the current window.
    fn request_count(&self) -> usize {
        self.events.len()
    }

    /// Sum of tokens across events in the current window.
    fn token_count(&self) -> usize {
        self.events.iter().map(|e| e.tokens).sum()
    }

    /// Push a new event.
    fn push(&mut self, tokens: usize) {
        self.events.push_back(Event {
            instant: Instant::now(),
            tokens,
        });
    }
}

// ---------------------------------------------------------------------------
// RateLimiter
// ---------------------------------------------------------------------------

const WINDOW: Duration = Duration::from_secs(60);

/// Thread-safe rate limiter that tracks per-provider request and token rates.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<RateLimiterInner>>,
}

struct RateLimiterInner {
    limits: HashMap<Provider, ProviderRateLimit>,
    counters: HashMap<Provider, WindowCounter>,
}

impl RateLimiter {
    /// Create a rate limiter with default per-provider limits.
    pub fn new() -> Self {
        Self::with_limits(default_rate_limits())
    }

    /// Create a rate limiter with custom limits.
    pub fn with_limits(limits: HashMap<Provider, ProviderRateLimit>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RateLimiterInner {
                limits,
                counters: HashMap::new(),
            })),
        }
    }

    /// Override limits for a specific provider at runtime.
    pub fn set_provider_limit(&self, provider: Provider, limit: ProviderRateLimit) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.limits.insert(provider, limit);
    }

    /// Check whether a request with the given token count would exceed the
    /// provider's rate limits. Does **not** record the request.
    pub fn check(&self, provider: Provider, tokens: usize) -> Result<(), RateLimitError> {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        // Ensure counter exists, then borrow fields separately.
        inner
            .counters
            .entry(provider)
            .or_insert_with(WindowCounter::new);
        let counter = inner.counters.get_mut(&provider).unwrap();
        counter.prune(WINDOW);
        let req_count = counter.request_count();
        let tok_count = counter.token_count();

        if let Some(limit) = inner.limits.get(&provider) {
            if limit.rpm > 0 && req_count >= limit.rpm {
                return Err(RateLimitError::RpmExceeded {
                    provider,
                    current: req_count,
                    limit: limit.rpm,
                });
            }
            if limit.tpm > 0 && tok_count + tokens > limit.tpm {
                return Err(RateLimitError::TpmExceeded {
                    provider,
                    current: tok_count + tokens,
                    limit: limit.tpm,
                });
            }
        }

        Ok(())
    }

    /// Record a completed request. Call this *after* the request succeeds.
    pub fn record(&self, provider: Provider, tokens: usize) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let counter = inner
            .counters
            .entry(provider)
            .or_insert_with(WindowCounter::new);
        counter.prune(WINDOW);
        counter.push(tokens);
    }

    /// Check **and** record in one atomic step. Returns an error if the
    /// request would violate limits; otherwise records the event and returns
    /// `Ok(())`.
    pub fn acquire(&self, provider: Provider, tokens: usize) -> Result<(), RateLimitError> {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner
            .counters
            .entry(provider)
            .or_insert_with(WindowCounter::new);
        let counter = inner.counters.get_mut(&provider).unwrap();
        counter.prune(WINDOW);
        let req_count = counter.request_count();
        let tok_count = counter.token_count();

        if let Some(limit) = inner.limits.get(&provider) {
            if limit.rpm > 0 && req_count >= limit.rpm {
                return Err(RateLimitError::RpmExceeded {
                    provider,
                    current: req_count,
                    limit: limit.rpm,
                });
            }
            if limit.tpm > 0 && tok_count + tokens > limit.tpm {
                return Err(RateLimitError::TpmExceeded {
                    provider,
                    current: tok_count + tokens,
                    limit: limit.tpm,
                });
            }
        }

        let counter = inner.counters.get_mut(&provider).unwrap();
        counter.push(tokens);
        Ok(())
    }

    /// Current request count in the sliding window for a provider.
    pub fn current_rpm(&self, provider: Provider) -> usize {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let counter = inner
            .counters
            .entry(provider)
            .or_insert_with(WindowCounter::new);
        counter.prune(WINDOW);
        counter.request_count()
    }

    /// Current token count in the sliding window for a provider.
    pub fn current_tpm(&self, provider: Provider) -> usize {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let counter = inner
            .counters
            .entry(provider)
            .or_insert_with(WindowCounter::new);
        counter.prune(WINDOW);
        counter.token_count()
    }

    /// Reset all counters.
    pub fn reset(&self) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        inner.counters.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl RateLimitError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            RateLimitError::RpmExceeded { .. } => {
                hudhudscript_errors::ErrorCode::RateLimitRpmExceeded
            }
            RateLimitError::TpmExceeded { .. } => {
                hudhudscript_errors::ErrorCode::RateLimitTpmExceeded
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<RateLimitError> for hudhudscript_errors::Error {
    fn from(e: RateLimitError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
