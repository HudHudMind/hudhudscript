//! HudHudScript Runtime
//!
//! This crate provides the agent runtime for managing agent lifecycle and execution.

pub mod agent;
pub mod persistence;
pub mod perspective;
pub mod provider;
pub mod providers;
pub mod raii;
pub mod response_cache;
pub mod router;
pub mod runtime;
pub mod tokenomics_provider;

pub use agent::{
    Agent, AgentConfig, AgentId, AgentMetadata, AgentState, ExecutionRecord, ExecutionStatus,
    StateValue, Task, TaskId, TaskImplementation, TaskParameter,
};
pub use persistence::{FilePersistence, PersistenceError, StateSnapshot};
pub use perspective::{FieldAccess, Perspective, PerspectiveError, PerspectiveHolder};
pub use provider::{
    estimate_tokens, FunctionCall, FunctionCallResult, FunctionCallResultType, LLMRequest,
    LLMResponse, LLMToolCall, Provider, ProviderConfig, ProviderError, ProviderInfo,
    ProviderRegistry, ProviderType, TokenBudget, TokenTracker, TokenUsage, TokenUsageStats,
    ToolCallResult, ToolDefinition,
};
pub use providers::{
    get_provider_defaults, AnthropicProvider, OllamaProvider, OpenAICompatibleProvider,
    OpenAIProvider,
};
pub use raii::{AgentSession, ConnectionHandle, Disposable, ScopedResource};
pub use response_cache::{CacheConfig, CacheKey, CacheStats, CachedResponse, ResponseCache};
pub use router::{ProviderHealth, ProviderRouter, RouterConfig, RoutingStrategy};
pub use runtime::{AgentRuntime, RuntimeConfig, RuntimeError, RuntimeStatistics};
