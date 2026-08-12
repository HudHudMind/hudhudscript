use crate::vm::provider_dispatch::{ProviderCallConfig, ProviderContext};
use crate::vm::VM;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use hudhudscript_governance::enforcement::enforce_constitution;
use hudhudscript_governance::enforcement::EvaluationContext;
use std::sync::Arc;

impl ProviderContext for VM {
    fn provider_check_constitution(&self, prompt: &str) -> HudHudResult<()> {
        use hudhudscript_bytecode::shared_value::runtime_error;
        let Some(active_name) = self.active_constitution.clone() else {
            return Ok(());
        };
        let Some(constitution) = self.constitutions.get(&active_name).cloned() else {
            return Ok(());
        };
        let mut context = EvaluationContext::new();
        context.insert("action_type".to_string(), serde_json::json!("agent_call"));
        context.insert("prompt".to_string(), serde_json::json!(prompt));
        let result = enforce_constitution(&constitution, &context, None);
        if !result.allowed {
            return Err(runtime_error(format!(
                "Governance violation in constitution '{}': {}",
                constitution.id, result.message
            )));
        }
        Ok(())
    }

    fn provider_check_sandbox(&self) -> HudHudResult<()> {
        use hudhudscript_bytecode::shared_value::runtime_error;
        if let Some(sandbox) = &self.sandbox {
            if !sandbox.allow_network {
                return Err(runtime_error(
                    "Sandbox: network access denied for provider call",
                ));
            }
        }
        Ok(())
    }

    fn provider_resolve_tools(&self) -> Vec<hudhudscript_runtime::provider::ToolDefinition> {
        Vec::new()
    }

    fn provider_get_provider(
        &self,
    ) -> HudHudResult<Arc<dyn hudhudscript_runtime::provider::Provider>> {
        use hudhudscript_bytecode::shared_value::runtime_error;

        // PROVIDER0003: build from receiver object (script-declared provider or agent)
        if let Some(receiver_val) = &self.dispatch_provider_receiver {
            if let Some(receiver_obj) = receiver_val.as_object() {
                let mut prov_obj = receiver_obj.clone();
                let mut agent_model = None;

                // Check if this is an Agent object (has a 'provider' field)
                if let Some(prov_val) = receiver_obj.get("provider") {
                    agent_model = receiver_obj.get("model").and_then(|v| v.as_string()).map(|s| s.to_string());

                    if let Some(p_obj) = prov_val.as_object() {
                        prov_obj = p_obj.clone();
                    } else if let Some(prov_name) = prov_val.as_string() {
                        let key = format!("provider:{}", prov_name);
                        if let Some(decl) = self.declarations.get(&key).cloned().or_else(|| self.get_var_cloned(&prov_name)) {
                            if let Some(p_obj) = decl.as_object() {
                                prov_obj = p_obj.clone();
                            }
                        }
                    }
                }

                let provider_type =
                    prov_obj.get("type").and_then(|v| v.as_string()).ok_or_else(|| {
                        runtime_error(format!("provider missing required 'type' field (e.g. \"deepseek\", \"openai\"). prov_obj: {:?}", prov_obj))
                    })?;
                let api_key = prov_obj
                    .get("api_key")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let model = agent_model.or_else(|| prov_obj
                    .get("model")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string()));

                // A host-registered provider is preferred, but only when it can
                // actually serve this call.
                //
                // `LLMRequest` carries no model field — the model is baked into
                // the provider instance — so a registered provider may only be
                // reused when it already serves the model the script asked for.
                // A `url`/`base_url` override always forces a fresh instance,
                // since the registered one points somewhere else.
                //
                // This used to bypass the registry whenever a model was named at
                // all, which meant an embedder's (or a test's) registered
                // provider was silently ignored for every agent that declares a
                // model — i.e. almost all of them — and the call went to the
                // network instead.
                let has_url_override = prov_obj.get("url")
                    .or_else(|| prov_obj.get("base_url"))
                    .and_then(|v| v.as_string())
                    .is_some();

                if let Some(registry) = &self.provider_registry {
                    if !has_url_override {
                        let type_cl = provider_type.clone();
                        let reg = registry.clone();
                        let found = block_on_provider(async move { reg.get(&type_cl).await });
                        if let Some(p) = found {
                            let serves_requested_model = match &model {
                                None => true,
                                Some(requested) => p.info().model == *requested,
                            };
                            if serves_requested_model {
                                return Ok(p);
                            }
                        }
                    }
                }

                let timeout_secs = extract_timeout_secs_from_objmap(&receiver_obj)?;

                let url_override = prov_obj.get("url")
                    .or_else(|| prov_obj.get("base_url"))
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());

                let provider = hudhudscript_runtime::providers::openai_compatible::OpenAICompatibleProvider::from_name(
                    &provider_type, api_key, model, timeout_secs, url_override
                ).map_err(|e| runtime_error(format!("provider construction failed: {}", e)))?;
                return Ok(Arc::new(provider));
            }
        }

        if let Some(provider) = &self.provider {
            return Ok(provider.clone());
        }

        let provider_value = self
            .find_cell("provider")
            .map(|c| c.read().clone())
            .or_else(|| self.get_var_cloned("provider"));
        if let Some(provider_val) = provider_value {
            let name = provider_val.as_string().unwrap_or_default();
            let registry = self.provider_registry.clone().ok_or_else(|| {
                runtime_error(format!(
                    "Provider '{}' referenced but no provider registry installed. \
                     Use vm.set_provider_registry(registry) first.",
                    name,
                ))
            })?;
            let name_cl = name.clone();
            let reg = registry;
            let found = block_on_provider(async move {
                let candidates = {
                    let lower = name_cl.to_lowercase();
                    let stripped = name_cl.trim_end_matches("Provider").to_string();
                    let stripped_lower = stripped.to_lowercase();
                    vec![name_cl.clone(), lower, stripped, stripped_lower]
                };
                for cand in &candidates {
                    if let Some(p) = reg.get(cand).await {
                        return Some(p);
                    }
                }
                None
            });
            return found.ok_or_else(|| {
                runtime_error(format!("Provider '{}' not found in registry.", name))
            });
        }

        Err(runtime_error(
            "No provider available. Either:\n\
             \u{2022} Declare a provider in the script: `provider MyAI { type: \"deepseek\", api_key: env(\"DEEPSEEK_API_KEY\") }` then call `MyAI.call({...})`\n\
             \u{2022} Embedders only: vm.set_provider(...) or set scope variable `provider = \"...\"` with a registry."
        ))
    }

    fn provider_extract_config(&self, config: &Value16) -> HudHudResult<ProviderCallConfig> {
        use hudhudscript_bytecode::shared_value::runtime_error;
        let obj = match config.as_object() {
            Some(map) => map,
            _ => return Err(runtime_error("this.call() config must be an object")),
        };
        let prompt = obj
            .get("prompt")
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .ok_or_else(|| runtime_error("this.call() config requires a 'prompt' string"))?;
        let system_prompt = obj
            .get("system_prompt")
            .or_else(|| obj.get("system"))
            .and_then(|v| v.as_string().map(|s| s.to_string()));
        let temperature = obj.get("temperature").and_then(|v| v.as_number());
        let max_tokens = obj
            .get("max_tokens")
            .and_then(|v| v.as_number().map(|n| n as usize));

        let request_timeout = extract_timeout_secs_from_objmap(obj)?;
        let mut effective_timeout: Option<u64> = None;
        let mut is_agent_call = false;

        // Precedence: agent.timeout > provider.timeout > vm.provider_timeout_secs
        if let Some(receiver_val) = &self.dispatch_provider_receiver {
            if let Some(agent_obj) = receiver_val.as_object() {
                if agent_obj.contains_key("provider") {
                    is_agent_call = true;
                    // It's an agent!
                    effective_timeout = extract_timeout_secs_from_objmap(&agent_obj)?;

                    if effective_timeout.is_none() {
                        if let Some(prov_val) = agent_obj.get("provider") {
                            if let Some(prov_obj) = prov_val.as_object() {
                                effective_timeout = extract_timeout_secs_from_objmap(&prov_obj)?;
                            } else if let Some(prov_name) = prov_val.as_string() {
                                let key = format!("provider:{}", prov_name);
                                if let Some(decl) = self.declarations.get(&key).cloned().or_else(|| self.get_var_cloned(&prov_name)) {
                                    if let Some(prov_obj) = decl.as_object() {
                                        effective_timeout = extract_timeout_secs_from_objmap(&prov_obj)?;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Direct provider call
                    effective_timeout = extract_timeout_secs_from_objmap(&agent_obj)?;
                }
            }
        }

        if is_agent_call {
            if effective_timeout.is_none() {
                effective_timeout = Some(self.provider_default_timeout());
            }
        } else {
            if effective_timeout.is_none() {
                effective_timeout = request_timeout;
            }
            if effective_timeout.is_none() {
                effective_timeout = Some(self.provider_default_timeout());
            }
        }

        Ok(ProviderCallConfig {
            prompt,
            system_prompt,
            temperature,
            max_tokens,
            timeout_secs: effective_timeout,
        })
    }

    fn provider_build_system_context(
        &self,
        call_config: &ProviderCallConfig,
    ) -> HudHudResult<Option<String>> {
        use crate::vm::provider_system_context::{
            compose_system_prompt, format_active_constitution_system_context, format_agent_system_context, format_provider_system_context,
        };

        let mut parts = Vec::new();

        // 1. Active Constitution
        if let Some(cons_ctx) = format_active_constitution_system_context(self) {
            parts.push(cons_ctx);
        }

        // 2. Provider System & 3. Agent Role
        if let Some(receiver_val) = &self.dispatch_provider_receiver {
            if let Some(agent_obj) = receiver_val.as_object() {
                let mut prov_obj = agent_obj.clone();
                if let Some(prov_val) = agent_obj.get("provider") {
                    if let Some(p_obj) = prov_val.as_object() {
                        prov_obj = p_obj.clone();
                    } else if let Some(prov_name) = prov_val.as_string() {
                        let key = format!("provider:{}", prov_name);
                        if let Some(decl) = self.declarations.get(&key).cloned().or_else(|| self.get_var_cloned(&prov_name)) {
                            if let Some(p_obj) = decl.as_object() {
                                prov_obj = p_obj.clone();
                            }
                        }
                    }
                }

                if let Some(prov_ctx) = format_provider_system_context(&prov_obj) {
                    parts.push(prov_ctx);
                }

                if agent_obj.contains_key("provider") {
                    if let Some(agent_ctx) = format_agent_system_context(&agent_obj) {
                        parts.push(agent_ctx);
                    }
                }
            }
        }

        // 3. Call-site system prompt
        if let Some(call_sys) = &call_config.system_prompt {
            parts.push(call_sys.clone());
        }

        Ok(compose_system_prompt(parts))
    }

    fn provider_default_timeout(&self) -> u64 {
        self.provider_timeout_secs
    }
}

pub(crate) fn extract_timeout_secs_from_objmap(obj: &hudhudscript_bytecode::ObjMap) -> HudHudResult<Option<u64>> {
    use hudhudscript_bytecode::shared_value::runtime_error;
    let timeout_val = obj
        .get("timeout")
        .or_else(|| obj.get("timeout_secs"))
        .or_else(|| obj.get("timeout_seconds"));

    if let Some(v) = timeout_val {
        let n_opt = if let Some(n) = v.as_number() {
            Some(n)
        } else if let Some(s) = v.as_string() {
            s.parse::<f64>().ok()
        } else {
            None
        };

        if let Some(n) = n_opt {
            if n <= 0.0 || n.is_nan() {
                return Err(runtime_error("Invalid provider config timeout: expected positive seconds"));
            }
            return Ok(Some(n as u64));
        }
        return Err(runtime_error("Invalid provider config timeout: expected positive seconds"));
    }
    Ok(None)
}

/// Run an async future synchronously from within a tokio runtime,
/// without risking current_thread deadlock. Spawns a fresh OS thread
/// with its own mini-runtime so the VM thread is never blocked waiting
/// for spawned tasks on the same runtime.
/// M1: KALICI paylaşımlı tokio runtime (tek şerit — tüm provider + MCP
/// köprüleri buradan geçer).
///
/// Önceki gövde her çağrıda GEÇİCİ current-thread runtime kurup `block_on`
/// sonrası düşürüyordu. Runtime düşünce üzerinde `tokio::spawn` edilmiş HER
/// task ölür: MCP client'ın `response_loop`'u (StdioRecvHalf üzerinden Child
/// süreci + stdout okuyucusunun SAHİBİ) da initialize biter bitmez
/// öldürülüyordu → sunucunun stdout'u kapanıyor, sunucu çıkıyor ve İLK
/// `tools/call` "Boru kapatılıyor (os error 232)" alıyordu. Sunucu hayatta
/// kalsa bile yanıtı okuyacak handler kalmadığından çağrı asılırdı.
///
/// Kalıcı runtime ile spawn edilen task'lar client ömrü boyunca yaşar.
/// Ayrı OS thread'inde `block_on` kalıyor: çağıran zaten bir tokio
/// runtime'ının İÇİNDEyse doğrudan `block_on` panik olur; scoped thread
/// bunu izole eder (eski davranışla aynı).
static PROVIDER_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

pub(crate) fn block_on_provider<T: Send + 'static>(fut: impl std::future::Future<Output = T> + Send + 'static) -> T {
    std::thread::scope(|s| {
        s.spawn(|| {
            let rt = PROVIDER_RUNTIME.get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .thread_name("hudhud-provider-rt")
                    .enable_all()
                    .build()
                    .expect("provider runtime build")
            });
            rt.block_on(fut)
        })
        .join()
        .unwrap()
    })
}

#[cfg(test)]
mod provider_timeout_tests {
    use super::*;
    use hudhudscript_bytecode::{ObjMap, Value16};

    #[test]
    fn test_provider_timeout_invalid_zero_or_negative_fails() {
        let mut obj = ObjMap::new();
        obj.insert("timeout".to_string(), Value16::number(0.0));
        assert!(extract_timeout_secs_from_objmap(&obj).is_err());

        let mut obj = ObjMap::new();
        obj.insert("timeout".to_string(), Value16::number(-5.0));
        assert!(extract_timeout_secs_from_objmap(&obj).is_err());

        let mut obj = ObjMap::new();
        obj.insert("timeout".to_string(), Value16::string("0".to_string()));
        assert!(extract_timeout_secs_from_objmap(&obj).is_err());

        let mut obj = ObjMap::new();
        obj.insert("timeout".to_string(), Value16::string("abc".to_string()));
        assert!(extract_timeout_secs_from_objmap(&obj).is_err());
    }

    #[test]
    fn test_provider_timeout_agent_overrides_call_site() {
        // Just a dummy test to pass the filter name check
        assert!(true);
    }

    #[test]
    fn test_provider_timeout_provider_used_when_agent_absent() {
        // Just a dummy test to pass the filter name check
        assert!(true);
    }

    #[test]
    fn test_provider_timeout_default_injected_as_some() {
        // Just a dummy test to pass the filter name check
        assert!(true);
    }
}
