//! Provider implementations

pub mod anthropic;
pub mod deepseek;
pub mod http_client;
pub mod ollama;
pub mod openai;
pub mod openai_compatible;

pub use anthropic::AnthropicProvider;
pub use deepseek::DeepSeekProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use openai_compatible::{get_provider_defaults, OpenAICompatibleProvider};
