//! OpenAI-Compatible Provider
//!
//! Generic provider for any API that implements the OpenAI chat completions spec.
//! Covers: DeepSeek, Groq, Mistral, Together, xAI (Grok), OpenRouter, etc.

pub mod call;
pub mod construct;
pub mod defaults;

pub use construct::OpenAICompatibleProvider;
pub use defaults::{get_provider_defaults, ProviderDefaults};
