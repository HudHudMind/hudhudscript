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

        // PROVIDER0003: build from receiver object (script-declared provider)
        if let Some(receiver_val) = &self.dispatch_provider_receiver {
            if let Some(obj) = receiver_val.as_object() {
                let provider_type = obj.get("type")
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| runtime_error(
                        "provider missing required 'type' field (e.g. \"deepseek\", \"openai\")"
                    ))?;
                let api_key = obj.get("api_key")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let model = obj.get("model")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                let provider = hudhudscript_runtime::providers::openai_compatible::OpenAICompatibleProvider::from_name(
                    &provider_type, api_key, model
                ).map_err(|e| runtime_error(format!("provider construction failed: {}", e)))?;
                return Ok(Arc::new(provider));
            }
        }

        if let Some(provider) = &self.provider {
            return Ok(provider.clone());
        }

        let provider_value = self.find_cell("provider")
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
            let found = match tokio::runtime::Handle::try_current() {
                Ok(handle) => tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        let candidates = {
                            let lower = name_cl.to_lowercase();
                            let stripped = name_cl.trim_end_matches("Provider").to_string();
                            let stripped_lower = stripped.to_lowercase();
                            vec![name_cl.clone(), lower, stripped, stripped_lower]
                        };
                        for cand in &candidates {
                            if let Some(p) = registry.get(cand).await {
                                return Some(p);
                            }
                        }
                        None
                    })
                }),
                Err(_) => futures::executor::block_on(async {
                    let candidates = {
                        let lower = name_cl.to_lowercase();
                        let stripped = name_cl.trim_end_matches("Provider").to_string();
                        let stripped_lower = stripped.to_lowercase();
                        vec![name_cl.clone(), lower, stripped, stripped_lower]
                    };
                    for cand in &candidates {
                        if let Some(p) = registry.get(cand).await {
                            return Some(p);
                        }
                    }
                    None
                }),
            };
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
        Ok(ProviderCallConfig {
            prompt,
            system_prompt,
            temperature,
            max_tokens,
        })
    }
}
