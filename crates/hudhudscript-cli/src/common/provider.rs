use crate::common::{CliError, HudHudConfig};
use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_with_recovery};
use hudhudscript_runtime::{
    AnthropicProvider, OllamaProvider, OpenAICompatibleProvider, OpenAIProvider, ProviderConfig,
    ProviderRegistry, ProviderType,
};
use hudhudscript_vm::{OutputLocale, VM};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn setup_provider_registry(debug: bool) -> Result<Arc<ProviderRegistry>, String> {
    let registry = ProviderRegistry::new();

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

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
                    extra: std::collections::HashMap::new(),
                };
                match OpenAIProvider::new(config) {
                    Ok(p) => {
                        let p = Arc::new(p);
                        registry.register("openai".to_string(), p.clone()).await;
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
                    extra: std::collections::HashMap::new(),
                };
                match AnthropicProvider::new(config) {
                    Ok(p) => {
                        let p = Arc::new(p);
                        registry.register("anthropic".to_string(), p.clone()).await;
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
            extra: std::collections::HashMap::new(),
        };
        match OllamaProvider::new(ollama_config) {
            Ok(p) => {
                let p = Arc::new(p);
                registry.register("ollama".to_string(), p.clone()).await;
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
                    match OpenAICompatibleProvider::from_name(name, api_key, None) {
                        Ok(p) => {
                            let p = Arc::new(p);
                            registry.register(name.to_string(), p.clone()).await;
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

/// MCP Server Configuration
#[cfg(feature = "mcp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// MCP Configuration File
#[cfg(feature = "mcp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: std::collections::HashMap<String, McpServerConfig>,
}

/// Load MCP configuration from file
#[cfg(feature = "mcp")]
pub fn load_mcp_config(path: &PathBuf) -> Result<McpConfig, String> {
    if !path.exists() {
        return Ok(McpConfig {
            mcp_servers: std::collections::HashMap::new(),
        });
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read MCP config: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse MCP config: {}", e))
}

/// Setup MCP clients from configuration (no-op when `mcp` feature is disabled)
#[cfg(not(feature = "mcp"))]
pub async fn setup_mcp_clients(
    _debug: bool,
) -> Result<std::collections::HashMap<String, Arc<()>>, String> {
    Ok(std::collections::HashMap::new())
}

/// Setup MCP clients from configuration
#[cfg(feature = "mcp")]
pub async fn setup_mcp_clients(
    debug: bool,
) -> Result<std::collections::HashMap<String, Arc<McpClient>>, String> {
    let mut clients = std::collections::HashMap::new();

    // Try to load user-level config
    let user_config_path = dirs::home_dir().map(|mut p| {
        p.push(".kiro/settings/mcp.json");
        p
    });

    // Try to load workspace-level config
    let workspace_config_path = PathBuf::from(".kiro/settings/mcp.json");

    // Merge configs (workspace overrides user)
    let mut merged_servers = std::collections::HashMap::new();

    if let Some(user_path) = user_config_path {
        if let Ok(user_config) = load_mcp_config(&user_path) {
            if debug && !user_config.mcp_servers.is_empty() {
                println!(
                    "📖 Loaded user MCP config: {} servers",
                    user_config.mcp_servers.len()
                );
            }
            merged_servers.extend(user_config.mcp_servers);
        }
    }

    if let Ok(workspace_config) = load_mcp_config(&workspace_config_path) {
        if debug && !workspace_config.mcp_servers.is_empty() {
            println!(
                "📖 Loaded workspace MCP config: {} servers",
                workspace_config.mcp_servers.len()
            );
        }
        merged_servers.extend(workspace_config.mcp_servers);
    }

    if merged_servers.is_empty() {
        if debug {
            println!("⚠ No MCP servers configured");
            println!("  Create ~/.kiro/settings/mcp.json or .kiro/settings/mcp.json to configure MCP servers");
        }
        return Ok(clients);
    }

    // Create clients for each server
    for (name, config) in merged_servers {
        if debug {
            println!("🔌 Connecting to MCP server '{}'...", name);
        }

        // Create transport config
        let transport_config = TransportConfig::stdio(config.command.clone(), config.args.clone());

        // Create client with a timeout to avoid hanging when MCP servers are unavailable
        let connect_result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let client = McpClient::new(transport_config).await?;
            let client = Arc::new(client);
            let init_response = client.initialize().await?;
            client.start_response_handler().await;
            Ok::<(Arc<McpClient>, hudhudscript_mcp::InitializeResponse), anyhow::Error>((
                client,
                init_response,
            ))
        })
        .await;

        match connect_result {
            Ok(Ok((client, init_response))) => {
                if debug {
                    println!(
                        "  ✓ Connected to '{}' ({})",
                        name, init_response.server_info.name
                    );
                }
                clients.insert(name.clone(), client);
            }
            Ok(Err(e)) => {
                if debug {
                    println!("  ⚠ Failed to connect to '{}': {}", name, e);
                }
            }
            Err(_) => {
                eprintln!("  ⚠ Timeout connecting to MCP server '{}' (skipped)", name);
            }
        }
    }

    if debug && !clients.is_empty() {
        println!("✓ {} MCP server(s) ready", clients.len());
    }

    Ok(clients)
}
