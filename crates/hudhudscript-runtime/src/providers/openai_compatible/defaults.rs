//! Provider default configurations for OpenAI-compatible endpoints.

use crate::provider::ProviderType;

/// Known OpenAI-compatible providers with their default base URLs
pub struct ProviderDefaults {
    pub base_url: &'static str,
    pub default_model: &'static str,
    pub env_var: &'static str,
    pub provider_type: ProviderType,
}

pub fn get_provider_defaults(name: &str) -> Option<ProviderDefaults> {
    match name.to_lowercase().as_str() {
        "openai" | "gpt" => Some(ProviderDefaults {
            base_url: "https://api.openai.com/v1",
            default_model: "gpt-4o",
            env_var: "OPENAI_API_KEY",
            provider_type: ProviderType::OpenAI,
        }),
        "deepseek" => Some(ProviderDefaults {
            base_url: "https://api.deepseek.com/v1",
            default_model: "deepseek-chat",
            env_var: "DEEPSEEK_API_KEY",
            provider_type: ProviderType::DeepSeek,
        }),
        "groq" => Some(ProviderDefaults {
            base_url: "https://api.groq.com/openai/v1",
            default_model: "llama-3.3-70b-versatile",
            env_var: "GROQ_API_KEY",
            provider_type: ProviderType::Groq,
        }),
        "mistral" => Some(ProviderDefaults {
            base_url: "https://api.mistral.ai/v1",
            default_model: "mistral-large-latest",
            env_var: "MISTRAL_API_KEY",
            provider_type: ProviderType::Mistral,
        }),
        "together" => Some(ProviderDefaults {
            base_url: "https://api.together.xyz/v1",
            default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
            env_var: "TOGETHER_API_KEY",
            provider_type: ProviderType::Together,
        }),
        "xai" | "grok" => Some(ProviderDefaults {
            base_url: "https://api.x.ai/v1",
            default_model: "grok-2",
            env_var: "XAI_API_KEY",
            provider_type: ProviderType::XAI,
        }),
        "openrouter" => Some(ProviderDefaults {
            base_url: "https://openrouter.ai/api/v1",
            default_model: "openai/gpt-4o",
            env_var: "OPENROUTER_API_KEY",
            provider_type: ProviderType::OpenRouter,
        }),
        "cohere" => Some(ProviderDefaults {
            base_url: "https://api.cohere.ai/v1",
            default_model: "command-r-plus",
            env_var: "COHERE_API_KEY",
            provider_type: ProviderType::Cohere,
        }),
        // Kimi (Moonshot AI) — OpenAI-compatible
        "kimi" | "moonshot" => Some(ProviderDefaults {
            base_url: "https://api.moonshot.cn/v1",
            default_model: "moonshot-v1-8k",
            env_var: "KIMI_API_KEY",
            provider_type: ProviderType::OpenRouter, // reuse Http-compatible type
        }),
        // Gemini via OpenAI-compatible endpoint
        "gemini" | "google" => Some(ProviderDefaults {
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
            default_model: "gemini-2.0-flash",
            env_var: "GEMINI_API_KEY",
            provider_type: ProviderType::Gemini,
        }),
        // Cerebras
        "cerebras" => Some(ProviderDefaults {
            base_url: "https://api.cerebras.ai/v1",
            default_model: "llama3.1-70b",
            env_var: "CEREBRAS_API_KEY",
            provider_type: ProviderType::OpenRouter,
        }),
        // Perplexity
        "perplexity" => Some(ProviderDefaults {
            base_url: "https://api.perplexity.ai",
            default_model: "llama-3.1-sonar-large-128k-online",
            env_var: "PERPLEXITY_API_KEY",
            provider_type: ProviderType::OpenRouter,
        }),
        // Ollama — local, no API key needed, OpenAI-compatible
        "ollama" => Some(ProviderDefaults {
            base_url: "http://localhost:11434/v1",
            default_model: "llama3.2",
            env_var: "OLLAMA_API_KEY",
            provider_type: ProviderType::Ollama,
        }),
        // Ollama Cloud — hosted at ollama.com, requires API key, OpenAI-compatible
        "ollamacloud" | "ollama_cloud" | "ollama-cloud" => Some(ProviderDefaults {
            base_url: "https://ollama.com/v1",
            default_model: "llama3.3:70b",
            env_var: "OLLAMA_API_KEY",
            provider_type: ProviderType::Ollama,
        }),
        // LM Studio — local OpenAI-compatible server
        "lmstudio" | "lm_studio" | "lm-studio" => Some(ProviderDefaults {
            base_url: "http://localhost:1234/v1",
            default_model: "local-model",
            env_var: "LMSTUDIO_API_KEY",
            provider_type: ProviderType::Http,
        }),
        _ => None,
    }
}
