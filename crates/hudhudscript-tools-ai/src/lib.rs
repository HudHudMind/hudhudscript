//! HudHudScript Tools — AI
//!
//! Memory/RAG tooling, context window management, cost tracking,
//! rate limiting, provider fallback, and usage analytics.

pub mod analytics;
pub mod context;
pub mod conversation;
pub mod cost;
pub mod fallback;
pub mod memory;
pub mod rate_limit;

pub use analytics::{Analytics, AnalyticsReport, ModelSummary, ProviderSummary, UsageSummary};
pub use context::{estimate_tokens, ContextWindow, OutputLimiterConfig, ToolOutputLimiter};
pub use conversation::{
    Conversation, ConversationError, Message, Role, StreamAccumulator, ToolCall,
};
pub use cost::{AiTokenUsage, BudgetConfig, CostError, CostTracker, ModelPricing, Provider};
pub use fallback::{
    FallbackChain, FallbackChainBuilder, FallbackEntry, FallbackError, FallbackResult,
};
pub use memory::{InMemoryBackend, MemoryBackend, MemoryEntry, MemoryError, MemoryStore};
pub use rate_limit::{ProviderRateLimit, RateLimitError, RateLimiter};
