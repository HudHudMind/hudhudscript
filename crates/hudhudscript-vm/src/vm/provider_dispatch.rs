//! Shared provider / LLM call dispatch (Kural 7).
//!
//! Both the interpreter and the VM route `this.call({prompt: …})` and
//! `this.stream({prompt: …})` through the functions in this module:
//!
//! - [`dispatch_provider_call`] — single LLM call, returns Value
//! - [`dispatch_provider_call_with_tools`] — LLM call + tool-call
//!   follow-up loop (max N iterations), returns Value
//!
//! The tool-call loop was previously interpreter-local (~85 LOC in
//! `hudhudscript-interpreter/src/integration/provider.rs:500-584`).
//! Now it lives here so both runtimes can execute agent scripts that
//! use tool calling through the exact same iteration logic. Each
//! runtime provides its own `ProviderContext::call_tool_handler` to
//! bridge the script-function dispatch (interpreter walks AST closures,
//! VM invokes bytecode chunks via CallbackInvoker).

use std::collections::HashMap;
use std::sync::Arc;

use hudhudscript_runtime::provider::{
    LLMRequest, LLMResponse, LLMToolCall, Provider, ToolCallResult, ToolDefinition,
};

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

// ── Provider context trait ──────────────────────────────────────────────────

/// Runtime-provided hooks for the provider dispatch pipeline.
///
/// Each runtime (interpreter, VM) implements this trait to wire in its
/// own governance, sandbox, tool resolution, provider construction, and
/// tool-handler invocation.
pub trait ProviderContext {
    /// Constitution compliance check for agent calls.
    fn provider_check_constitution(&self, prompt: &str) -> SharedResult<()>;

    /// Sandbox network policy check — provider calls are outbound HTTP.
    fn provider_check_sandbox(&self) -> SharedResult<()>;

    /// Resolve tool definitions available to this agent.
    fn provider_resolve_tools(&self) -> Vec<ToolDefinition>;

    /// Returns the system default timeout for provider calls.
    fn provider_default_timeout(&self) -> u64;

    /// Get or construct the live provider instance.
    fn provider_get_provider(&self) -> SharedResult<Arc<dyn Provider>>;

    /// Extract the LLM config overrides from the call-site config object.
    fn provider_extract_config(&self, config: &Value16) -> SharedResult<ProviderCallConfig>;

    /// Execute a script-defined tool handler.
    ///
    /// `tool_name` is the tool the LLM requested; `handler` is the script
    /// function value (closure / bytecode chunk) that implements it;
    /// `args_json` is the LLM-supplied JSON arguments. The return value is
    /// the handler's output formatted as a string for the next LLM turn.
    ///
    /// Default: returns an error (runtime hasn't implemented tool dispatch).
    fn call_tool_handler(
        &mut self,
        _tool_name: &str,
        _handler: &Value16,
        _args_json: &serde_json::Value,
    ) -> SharedResult<String> {
        Err(runtime_error(
            "Tool handler dispatch not implemented in this runtime",
        ))
    }

    /// Find the handler Value for a named tool among the resolved tools.
    ///
    /// Default: returns None (no tools available). Override if the runtime
    /// stores tool handlers alongside their definitions.
    fn find_tool_handler(&self, _tool_name: &str) -> Option<Value16> {
        None
    }

    /// Build the full system context (constitution + laws + agent role + call-site system prompt).
    fn provider_build_system_context(
        &self,
        call_config: &ProviderCallConfig,
    ) -> SharedResult<Option<String>> {
        Ok(call_config.system_prompt.clone())
    }
}

/// Flattened config for a single provider call.
#[derive(Debug, Clone)]
pub struct ProviderCallConfig {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    pub timeout_secs: Option<u64>,
}

// ── Single-call dispatch ────────────────────────────────────────────────────

/// Execute a single provider / LLM call (no tool follow-ups).
///
/// Returns a `Value::Object` with `{content, model, tokens_used,
/// finish_reason, tool_calls?}`.
pub fn dispatch_provider_call<C>(context: &C, config: &Value16) -> SharedResult<Value16>
where
    C: ProviderContext,
{
    let response = provider_call_raw(context, config)?;
    Ok(llm_response_to_value(&response))
}

/// Execute a provider call with tool-call follow-up loop.
///
/// If the LLM responds with `finish_reason: "tool_calls"`, the loop:
/// 1. Extracts the tool calls from the response
/// 2. For each call, looks up the handler via `context.find_tool_handler`
///    and invokes it via `context.call_tool_handler`
/// 3. Builds a follow-up prompt with the tool results
/// 4. Re-calls the provider
/// 5. Repeats up to `max_iterations` times
///
/// Both the interpreter and the VM call this function. The interpreter
/// passes its own `ProviderContext` impl that walks AST closures; the VM
/// passes its impl that invokes bytecode chunks. The tool-loop logic
/// itself is shared — Kural 7.
pub fn dispatch_provider_call_with_tools<C>(
    context: &mut C,
    config: &Value16,
    max_iterations: usize,
) -> SharedResult<Value16>
where
    C: ProviderContext,
{
    // Initial call
    let mut response = provider_call_raw(context, config)?;
    let tools = context.provider_resolve_tools();

    let mut iteration = 0;
    while response.finish_reason == "tool_calls" && iteration < max_iterations {
        let tool_calls = match response.tool_calls.take() {
            Some(calls) if !calls.is_empty() => calls,
            _ => break,
        };

        let tool_results = execute_tool_calls(context, &tool_calls)?;

        // Build follow-up prompt with tool results appended
        let tool_results_text: String = tool_results
            .iter()
            .map(|r| format!("[Tool: {}]\n{}", r.name, r.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let follow_up_prompt = format!(
            "{}\n\n[Tool Results]\n{}\n\nPlease continue based on the tool results above.",
            response.content, tool_results_text
        );

        // Re-derive provider config for the follow-up (keep original temp/max_tokens)
        let orig_config = context.provider_extract_config(config)?;
        let follow_up_system_prompt = context.provider_build_system_context(&orig_config)?;
        let follow_up = build_follow_up_request(&orig_config, &follow_up_prompt, &tools, follow_up_system_prompt);

        let provider = context.provider_get_provider()?;
        response = call_provider_async(provider, follow_up, context.provider_default_timeout())?;
        iteration += 1;
    }

    Ok(llm_response_to_value(&response))
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Execute a single provider call, returning the raw `LLMResponse`.
fn provider_call_raw<C>(context: &C, config: &Value16) -> SharedResult<LLMResponse>
where
    C: ProviderContext,
{
    let call_config = context.provider_extract_config(config)?;
    context.provider_check_constitution(&call_config.prompt)?;
    context.provider_check_sandbox()?;

    let tools = context.provider_resolve_tools();
    let provider = context.provider_get_provider()?;

    let system_prompt = context.provider_build_system_context(&call_config)?;

    let request = LLMRequest {
        prompt: call_config.prompt,
        system_prompt,
        temperature: call_config.temperature,
        max_tokens: call_config.max_tokens,
        mnemonics: None,
        optimize: false,
        tools: if tools.is_empty() { None } else { Some(tools) },
        timeout_secs: call_config.timeout_secs,
    };

    call_provider_async(provider, request, context.provider_default_timeout())
}

fn call_provider_async(
    provider: Arc<dyn Provider>,
    request: LLMRequest,
    default_timeout: u64,
) -> SharedResult<LLMResponse> {
    let timeout_secs = request.timeout_secs.unwrap_or(default_timeout);
    let timeout_duration = std::time::Duration::from_secs(timeout_secs);
    let call_fut = async move {
        match tokio::time::timeout(timeout_duration, provider.call(request)).await {
            Ok(result) => result.map_err(|e| format!("{}", e)),
            Err(_) => Err(format!(
                "Provider call timed out after {}s",
                timeout_secs
            )),
        }
    };
    let result = crate::vm::provider::block_on_provider(call_fut);
    result.map_err(|e| runtime_error(format!("Provider call failed: {}", e)))
}

/// Execute a batch of tool calls through the context's handler dispatch.
fn execute_tool_calls<C>(
    context: &mut C,
    tool_calls: &[LLMToolCall],
) -> SharedResult<Vec<ToolCallResult>>
where
    C: ProviderContext,
{
    let mut results = Vec::with_capacity(tool_calls.len());

    for tc in tool_calls {
        let content = if let Some(handler) = context.find_tool_handler(&tc.name) {
            match context.call_tool_handler(&tc.name, &handler, &tc.arguments) {
                Ok(output) => output,
                Err(e) => format!("Tool error: {}", e),
            }
        } else {
            format!("Tool '{}' not found in agent scope", tc.name)
        };

        results.push(ToolCallResult {
            tool_call_id: tc.id.clone(),
            name: tc.name.clone(),
            content,
        });
    }

    Ok(results)
}

/// Build a follow-up LLM request after tool execution.
fn build_follow_up_request(
    orig_config: &ProviderCallConfig,
    follow_up_prompt: &str,
    tools: &[ToolDefinition],
    composed_system_prompt: Option<String>,
) -> LLMRequest {
    LLMRequest {
        prompt: follow_up_prompt.to_string(),
        system_prompt: composed_system_prompt,
        temperature: orig_config.temperature,
        max_tokens: orig_config.max_tokens,
        mnemonics: None,
        optimize: false,
        tools: if tools.is_empty() {
            None
        } else {
            Some(tools.to_vec())
        },
        timeout_secs: orig_config.timeout_secs,
    }
}

// ── Response conversion ─────────────────────────────────────────────────────

/// Convert an [`LLMResponse`] to a script-visible `Value::Object`.
///
/// Public so both the shared dispatcher AND any runtime-specific wrapper
/// can use the same conversion — Kural 7.
pub fn llm_response_to_value(response: &LLMResponse) -> Value16 {
    let mut obj: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "content".to_string(),
        Value16::string(response.content.clone()),
    );
    obj.insert("model".to_string(), Value16::string(response.model.clone()));
    obj.insert(
        "finish_reason".to_string(),
        Value16::string(response.finish_reason.clone()),
    );

    let mut tokens = hudhudscript_bytecode::ObjMap::default();
    tokens.insert(
        "prompt_tokens".to_string(),
        Value16::number(response.tokens_used.prompt_tokens as f64),
    );
    tokens.insert(
        "completion_tokens".to_string(),
        Value16::number(response.tokens_used.completion_tokens as f64),
    );
    tokens.insert(
        "total_tokens".to_string(),
        Value16::number(response.tokens_used.total_tokens as f64),
    );
    obj.insert("tokens_used".to_string(), Value16::object(tokens));

    if let Some(tool_calls) = &response.tool_calls {
        let calls: Vec<Value16> = tool_calls
            .iter()
            .map(|tc| {
                let mut call_obj: hudhudscript_bytecode::ObjMap =
                    hudhudscript_bytecode::ObjMap::default();
                call_obj.insert("id".to_string(), Value16::string(tc.id.clone()));
                call_obj.insert("name".to_string(), Value16::string(tc.name.clone()));
                call_obj.insert(
                    "arguments".to_string(),
                    Value16::string(tc.arguments.to_string()),
                );
                Value16::object(call_obj)
            })
            .collect();
        obj.insert("tool_calls".to_string(), Value16::array(calls));
    }

    Value16::object(obj)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_follow_up_request() {
        let orig_config = ProviderCallConfig {
            prompt: "original prompt".to_string(),
            system_prompt: Some("original system".to_string()),
            temperature: Some(0.5),
            max_tokens: Some(100),
            timeout_secs: Some(300),
        };

        let follow_up = build_follow_up_request(
            &orig_config,
            "new prompt",
            &[],
            Some("Role text".to_string()),
        );

        assert_eq!(follow_up.prompt, "new prompt");
        assert_eq!(follow_up.system_prompt, Some("Role text".to_string()));
        assert_eq!(follow_up.temperature, Some(0.5));
        assert_eq!(follow_up.max_tokens, Some(100));
        assert_eq!(follow_up.timeout_secs, Some(300));
    }
}
