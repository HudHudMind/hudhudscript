use crate::common::{CliError, HudHudConfig};
use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_with_recovery};
use hudhudscript_runtime::{
    AnthropicProvider, OllamaProvider, OpenAICompatibleProvider, OpenAIProvider, ProviderConfig,
    ProviderRegistry, ProviderType, TokenBudget,
};
use hudhudscript_vm::{OutputLocale, VM};
use std::fs;
use std::sync::Arc;

pub fn setup_provider_registry(
    debug: bool,
    tokenomics_budget: Option<hudhudscript_runtime::provider::TokenBudget>,
) -> Result<Arc<ProviderRegistry>, String> {
    let registry = ProviderRegistry::new();
    let budget = tokenomics_budget;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;

    rt.block_on(async {
        // ── OpenAI ──────────────────────────────────────────────────────────
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                let config = ProviderConfig {
                    provider_type: ProviderType::OpenAI,
                    model: "gpt-4o".to_string(),
                    api_key: Some(api_key),
                    endpoint: None,
                    temperature: Some(0.7),
                    max_tokens: Some(4096),
                    budget: None,
                    timeout_secs: None,
                    extra: std::collections::HashMap::new(),
                };
                match OpenAIProvider::new(config) {
                    Ok(p) => {
                        let p = Arc::new(p);
                        if let Some(ref b) = budget {
                            registry
                                .register_with_tokenomics(
                                    "openai".to_string(),
                                    p.clone(),
                                    Some(b.max_tokens_per_call),
                                    Some(b.max_tokens_per_day),
                                    None,
                                )
                                .await;
                        } else {
                            registry.register("openai".to_string(), p.clone()).await;
                        }
                        registry.register("OpenAI".to_string(), p.clone()).await;
                        registry.register("OpenAIProvider".to_string(), p).await;
                        if debug {
                            println!("✓ OpenAI provider registered");
                        }
                    }
                    Err(e) => {
                        if debug {
                            println!("⚠ OpenAI: {}", e);
                        }
                    }
                }
            }
        } else if debug {
            println!("⚠ OPENAI_API_KEY not set");
        }

        // ── Anthropic ───────────────────────────────────────────────────────
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            if !api_key.is_empty() {
                let config = ProviderConfig {
                    provider_type: ProviderType::Anthropic,
                    model: "claude-3-5-sonnet-20241022".to_string(),
                    api_key: Some(api_key),
                    endpoint: None,
                    temperature: Some(0.7),
                    max_tokens: Some(4096),
                    budget: None,
                    timeout_secs: None,
                    extra: std::collections::HashMap::new(),
                };
                match AnthropicProvider::new(config) {
                    Ok(p) => {
                        let p = Arc::new(p);
                        if let Some(ref b) = budget {
                            registry
                                .register_with_tokenomics(
                                    "anthropic".to_string(),
                                    p.clone(),
                                    Some(b.max_tokens_per_call),
                                    Some(b.max_tokens_per_day),
                                    None,
                                )
                                .await;
                        } else {
                            registry.register("anthropic".to_string(), p.clone()).await;
                        }
                        registry.register("Anthropic".to_string(), p.clone()).await;
                        registry.register("AnthropicProvider".to_string(), p).await;
                        if debug {
                            println!("✓ Anthropic provider registered");
                        }
                    }
                    Err(e) => {
                        if debug {
                            println!("⚠ Anthropic: {}", e);
                        }
                    }
                }
            }
        } else if debug {
            println!("⚠ ANTHROPIC_API_KEY not set");
        }

        // ── Ollama (local, no key needed) ───────────────────────────────────
        let ollama_base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let ollama_config = ProviderConfig {
            provider_type: ProviderType::Ollama,
            model: "llama3.2".to_string(),
            api_key: None,
            endpoint: Some(format!("{}/api/generate", ollama_base_url)),
            temperature: Some(0.7),
            max_tokens: Some(2048),
            budget: None,
            timeout_secs: None,
            extra: std::collections::HashMap::new(),
        };
        match OllamaProvider::new(ollama_config) {
            Ok(p) => {
                let p = Arc::new(p);
                if let Some(ref b) = budget {
                    registry
                        .register_with_tokenomics(
                            "ollama".to_string(),
                            p.clone(),
                            Some(b.max_tokens_per_call),
                            Some(b.max_tokens_per_day),
                            None,
                        )
                        .await;
                } else {
                    registry.register("ollama".to_string(), p.clone()).await;
                }
                registry.register("Ollama".to_string(), p.clone()).await;
                registry.register("OllamaProvider".to_string(), p).await;
                if debug {
                    println!("✓ Ollama provider registered ({})", ollama_base_url);
                }
            }
            Err(e) => {
                if debug {
                    println!("⚠ Ollama: {}", e);
                }
            }
        }

        // ── OpenAI-compatible providers ─────────────────────────────────────
        let compat_providers = [
            ("deepseek", "DEEPSEEK_API_KEY", "DeepSeek"),
            ("groq", "GROQ_API_KEY", "Groq"),
            ("mistral", "MISTRAL_API_KEY", "Mistral"),
            ("together", "TOGETHER_API_KEY", "Together"),
            ("xai", "XAI_API_KEY", "XAI"),
            ("openrouter", "OPENROUTER_API_KEY", "OpenRouter"),
            ("cohere", "COHERE_API_KEY", "Cohere"),
        ];

        for (name, env_var, display) in &compat_providers {
            if let Ok(api_key) = std::env::var(env_var) {
                if !api_key.is_empty() {
                    match OpenAICompatibleProvider::from_name(name, api_key, None, None, None) {
                        Ok(p) => {
                            let p = Arc::new(p);
                            if let Some(ref b) = budget {
                                registry
                                    .register_with_tokenomics(
                                        name.to_string(),
                                        p.clone(),
                                        Some(b.max_tokens_per_call),
                                        Some(b.max_tokens_per_day),
                                        None,
                                    )
                                    .await;
                            } else {
                                registry.register(name.to_string(), p.clone()).await;
                            }
                            registry.register(display.to_string(), p.clone()).await;
                            registry.register(format!("{}Provider", display), p).await;
                            if debug {
                                println!("✓ {} provider registered", display);
                            }
                        }
                        Err(e) => {
                            if debug {
                                println!("⚠ {}: {}", display, e);
                            }
                        }
                    }
                }
            } else if debug {
                println!("⚠ {} not set ({})", env_var, display);
            }
        }

        let providers = registry.list().await;
        if debug {
            println!(
                "✓ Provider registry ready: {} aliases registered",
                providers.len()
            );
        }
    });

    Ok(Arc::new(registry))
}

/// Setup MCP clients from hudhud.toml [mcp.servers] configuration.
/// No-op when `mcp` feature is disabled.
#[cfg(not(feature = "mcp"))]
pub async fn setup_mcp_clients(
    _servers: &std::collections::HashMap<String, crate::common::McpServerConfig>,
    _debug: bool,
) -> Result<std::collections::HashMap<String, Arc<()>>, String> {
    Ok(std::collections::HashMap::new())
}

/// Setup MCP clients from hudhud.toml [mcp.servers] configuration.
#[cfg(feature = "mcp")]
pub async fn setup_mcp_clients(
    servers: &std::collections::HashMap<String, crate::common::McpServerConfig>,
    debug: bool,
) -> Result<std::collections::HashMap<String, Arc<McpClient>>, String> {
    let mut clients = std::collections::HashMap::new();

    if servers.is_empty() {
        if debug {
            println!("⚠ No MCP servers in hudhud.toml [mcp.servers]");
        }
        return Ok(clients);
    }

    let max_servers = 128usize;
    for (name, config) in servers {
        if name.trim().is_empty() {
            continue;
        }
        if clients.len() >= max_servers {
            break;
        }
        if debug {
            println!("🔌 MCP server '{}'...", name);
        }

        let transport = TransportConfig::stdio(config.command.clone(), config.args.clone());
        match McpClient::connect_initialized(transport, std::time::Duration::from_secs(5)).await {
            Ok(client) => {
                if debug {
                    println!("  ✓ '{}' connected", name);
                }
                clients.insert(name.clone(), client);
            }
            Err(e) => {
                if debug {
                    eprintln!("  ⚠ '{}': {}", name, e);
                }
            }
        }
    }
    if debug && !clients.is_empty() {
        println!("✓ {} MCP server(s)", clients.len());
    }
    Ok(clients)
}
