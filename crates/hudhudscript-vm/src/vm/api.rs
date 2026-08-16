use crate::vm::VM;
use hudhudscript_bytecode::cache_utils::{enforce_cache_limit, MAX_MCP_CACHE};
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::error::{compile_codes, CompileError};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};
use hudhudscript_governance::Constitution;
use hudhudscript_mcp::McpClient;
use hudhudscript_mcp::Tool as McpToolDefinition;
use hudhudscript_runtime::provider::ProviderRegistry;
use hudhudscript_tools::ToolRegistry;
use std::sync::Arc;

impl VM {
    /// Register a live MCP client under the given server name. Existing
    /// registrations for the same name are replaced. Enforces
    /// `max_mcp_servers` limit by evicting the oldest entry when at capacity.
    pub fn register_mcp_client(&mut self, name: String, client: Arc<McpClient>) {
        let mut clients = self.mcp_clients.lock();
        // If we're at the limit and this is a new name, evict one entry.
        if !clients.contains_key(&name)
            && clients.len() >= self.max_mcp_servers
            && self.max_mcp_servers > 0
        {
            // Evict an arbitrary entry (HashMap iteration order).
            if let Some(evict_key) = clients.keys().next().cloned() {
                clients.remove(&evict_key);
            }
        }
        clients.insert(name, client);
    }

    /// Look up a registered MCP client by server name.
    pub fn get_mcp_client(&self, name: &str) -> Option<Arc<McpClient>> {
        self.mcp_clients.lock().get(name).cloned()
    }

    /// Remove a registered MCP client. Returns `true` if one was present.
    pub fn unregister_mcp_client(&mut self, name: &str) -> bool {
        self.mcp_clients.lock().remove(name).is_some()
    }

    /// Set the maximum number of MCP servers allowed. When the limit is
    /// exceeded, the oldest entries are evicted (arbitrary eviction).
    /// Default is 128.
    pub fn with_max_mcp_servers(&mut self, max: usize) {
        self.max_mcp_servers = max;
    }

    /// MCP-51: Gracefully shut down all registered MCP clients.
    /// Disconnects each client, aborts response handlers, and clears the registry.
    /// Must be called from an async context (tokio runtime).
    pub async fn shutdown_mcp_clients(&self) {
        let clients: Vec<Arc<McpClient>> = self.mcp_clients.lock().values().cloned().collect();
        for c in clients {
            c.shutdown().await;
        }
        self.mcp_clients.lock().clear();
    }

    /// Read a global variable by name (set after `execute()` from main locals).
    /// Returns `None` if the variable doesn't exist.
    pub fn get_global(&self, name: &str) -> Option<Value16> {
        let sym = hudhudscript_bytecode::interner::try_resolve_id(name)?;
        self.globals
            .get(&hudhudscript_bytecode::interner::SymbolId(sym))
            .cloned()
    }

    /// SOP: read the intents declared for a subject template.
    /// Exposed for testability of A3.4.
    pub fn subject_intents(&self, template_name: &str) -> Vec<String> {
        self.subject_templates
            .get(template_name)
            .map(|t| t.intents.clone())
            .unwrap_or_default()
    }

    // ── Provider / LLM registry ─────────────────────────────────────────

    /// Set the maximum recursion / call depth.
    ///
    /// Default is [`hudhudscript_errors::constants::MAX_CALL_DEPTH`] (2000).
    /// Hard ceiling is 4000 — values above this are silently capped to
    /// protect against system stack overflow on the default 8MB Rust
    /// thread stack (~2KB per VM call frame). If you've increased the
    /// thread stack size, you can raise the ceiling proportionally, but
    /// the conservative default protects %99.9 of use cases.
    pub fn with_max_call_depth(&mut self, depth: usize) {
        self.max_call_depth = depth.min(self.max_call_depth_hard_ceiling);
    }

    pub fn with_max_call_depth_ceiling(&mut self, ceiling: usize) {
        self.max_call_depth_hard_ceiling = ceiling;
        self.max_call_depth = self.max_call_depth.min(ceiling);
    }

    /// ENV0004: set provider defaults from hudhud.toml [providers.*]
    pub fn set_toml_providers(
        &mut self,
        providers: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    ) {
        self.toml_providers = providers
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
    }

    /// PYTHON0004: Call a top-level function by name after execute().
    pub fn call_public(
        &mut self,
        func_name: &str,
        args: &[Value16],
        bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<Value16> {
        let chunk = bytecode.get_function(func_name).ok_or_else(|| {
            hudhudscript_bytecode::error::compile_codes::runtime_error(format!(
                "Function '{}' not found",
                func_name
            ))
        })?;
        if chunk.params.len() != args.len() {
            return Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!(
                    "{} expects {} args, got {}",
                    func_name,
                    chunk.params.len(),
                    args.len()
                ),
            ));
        }
        self.call_chunk(
            &chunk,
            &chunk.params,
            args,
            bytecode,
            hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(func_name).0),
        )?;
        Ok(self.registers[255])
    }

    /// Allow network access (for provider calls).
    pub fn allow_network(&mut self) {
        if let Some(ref mut s) = self.sandbox {
            s.allow_network = true;
        }
    }

    /// Grant child-process spawning. Required by stdio MCP servers, which
    /// launch the server as a subprocess; SSE servers need
    /// [`VM::allow_network`] instead. The command allow/deny lists in
    /// `SandboxConfig` still apply on top of this.
    pub fn allow_process(&mut self) {
        if let Some(ref mut s) = self.sandbox {
            s.allow_process = true;
        }
    }

    /// M2: `[runtime] allow_insecure_http` opt-in'i — SSE MCP için `http://`
    /// URL'lerine ve loopback (localhost/127.0.0.1/[::1]) SSRF muafiyetine
    /// izin verir. `allow_network`'ten kasıtlı olarak AYRI izin: biri ağ
    /// erişimi, diğeri şifresiz http. Uzak/özel-ağ hedefleri bayraktan
    /// bağımsız engelli kalır (`validate_sse_url`).
    pub fn allow_insecure_http(&mut self) {
        self.allow_insecure_http = true;
    }

    /// Set the runtime host-access policy (HOST-3).
    pub fn set_host_access_policy(&mut self, policy: crate::vm::host_access::HostAccessPolicy) {
        self.host_access_policy = policy;
    }

    /// Execute swarm/council task sequentially through each agent's provider.
    pub fn dispatch_swarm_run(
        &mut self,
        _swarm_name: &str,
        agent_names: &[String],
        task_val: &Value16,
        bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<Value16> {
        let task_str = self.value_to_string(task_val);
        let mut results = Vec::new();
        for agent_name in agent_names {
            let agent_val = self.get_var_cloned(agent_name);
            if let Some(agent_v) = agent_val {
                // M3 (Kural 7 — tek şerit): receiver AGENT objesidir, provider
                // değil. `Agent.call` ile aynı yol: provider_get_provider
                // endpoint'i agent'ın `provider` alanından (isim VEYA obje)
                // çözer, provider_build_system_context persona/rol'ü
                // ([Agent Role]) aynı receiver'dan kurar. Eski kod receiver'a
                // provider objesini koyduğu için persona council/swarm'da
                // düşüyordu; ayrıca provider'ı yalnız string-isim olarak
                // tanıyordu (obje-provider'lı agent swarm'da hiç çalışmıyordu).
                let is_agent = agent_v
                    .as_object()
                    .map(|o| o.contains_key("provider"))
                    .unwrap_or(false);
                if is_agent {
                    let prev = self.dispatch_provider_receiver.take();
                    self.dispatch_provider_receiver = Some(agent_v);
                    let mut config = hudhudscript_bytecode::ObjMap::default();
                    config.insert(
                        "prompt".to_string(),
                        Value16::string(format!("Task: {}", task_str)),
                    );
                    let result = crate::vm::provider_dispatch::dispatch_provider_call(
                        self,
                        &Value16::object(config),
                    );
                    self.dispatch_provider_receiver = prev;
                    results.push(Value16::string(format!(
                        "{}: {}",
                        agent_name,
                        self.value_to_string(&result.unwrap_or_default())
                    )));
                }
            }
        }
        Ok(Value16::array(results))
    }

    pub fn dispatch_swarm_add_agent(
        &mut self,
        swarm_name: &str,
        agent_val: &Value16,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<Value16> {
        let swarm = self.get_var_cloned(swarm_name);
        if let Some(obj) = swarm.and_then(|v| v.as_object().cloned()) {
            let agent_name = agent_val
                .as_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|| self.value_to_string(agent_val));
            let mut agents: Vec<Value16> = obj
                .get("agents")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            agents.push(Value16::string(agent_name.clone()));
            let mut new_obj = obj;
            new_obj.insert("agents".to_string(), Value16::array(agents));
            self.set_var(swarm_name, Value16::object(new_obj))?;
            return Ok(Value16::bool_(true));
        }
        Ok(Value16::bool_(false))
    }

    pub fn dispatch_swarm_remove_agent(
        &mut self,
        swarm_name: &str,
        agent_name: &str,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<Value16> {
        let swarm = self.get_var_cloned(swarm_name);
        if let Some(obj) = swarm.and_then(|v| v.as_object().cloned()) {
            let agents: Vec<Value16> = obj
                .get("agents")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let filtered: Vec<Value16> = agents
                .into_iter()
                .filter(|a| a.as_string().map(|s| s != agent_name).unwrap_or(true))
                .collect();
            let mut new_obj = obj;
            new_obj.insert("agents".to_string(), Value16::array(filtered));
            self.set_var(swarm_name, Value16::object(new_obj))?;
            return Ok(Value16::bool_(true));
        }
        Ok(Value16::bool_(false))
    }

    pub fn dispatch_council_add_member(
        &mut self,
        council_name: &str,
        member_val: &Value16,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<Value16> {
        let council = self.get_var_cloned(council_name);
        if let Some(obj) = council.and_then(|v| v.as_object().cloned()) {
            let agent_id = member_val
                .as_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|| self.value_to_string(member_val));
            let mut members: Vec<Value16> = obj
                .get("members")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut m = hudhudscript_bytecode::ObjMap::default();
            m.insert("agent_id".to_string(), Value16::string(agent_id));
            members.push(Value16::object(m));
            let mut new_obj = obj;
            new_obj.insert("members".to_string(), Value16::array(members));
            self.set_var(council_name, Value16::object(new_obj))?;
            return Ok(Value16::bool_(true));
        }
        Ok(Value16::bool_(false))
    }

    /// Store full hudhud.toml config as nested Value16 object for config() builtin.
    pub fn set_toml_config_object(&mut self, cfg: Value16) {
        self.toml_config = cfg;
    }

    /// config() builtin: dot-path lookup in toml_config.
    pub fn toml_config_lookup(&self, key: &str) -> Value16 {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &self.toml_config;
        for part in parts {
            if let Some(obj) = current.as_object() {
                if let Some(val) = obj.get(part) {
                    current = val;
                } else {
                    return Value16::null();
                }
            } else {
                return Value16::null();
            }
        }
        current.clone()
    }

    pub fn with_register_arena_kb(&mut self, kb: u32) {
        self.register_arena_size = (kb as usize) * 1024;
    }

    pub fn with_max_builtin_iter(&mut self, limit: usize) {
        self.max_builtin_iter = limit;
    }

    pub fn with_default_stack_bytes(&mut self, bytes: usize) {
        self.default_stack_bytes = bytes;
    }

    pub fn with_provider_timeout_secs(&mut self, secs: u64) {
        self.provider_timeout_secs = secs;
    }

    /// Set the live provider that `this.call()` / `this.stream()` will use.
    ///
    /// Runtime harnesses hand the same `Arc<dyn Provider>` that the
    /// interpreter's agent scope carries, so both executors reach the same
    /// LLM endpoint with the same auth credentials.
    pub fn set_provider(&mut self, provider: Arc<dyn hudhudscript_runtime::provider::Provider>) {
        self.provider = Some(provider);
    }

    /// Install a named provider registry (interpreter-parity API).
    ///
    /// Scripts that set `provider = "name"` in scope and call
    /// `this.call({...})` resolve `name` against this registry.  Tests
    /// and runtime harnesses use this to wire a mock provider without
    /// going through the full declaration machinery.
    pub fn set_provider_registry(&mut self, registry: Arc<ProviderRegistry>) {
        self.provider_registry = Some(registry);
    }

    /// Read-only accessor for the installed provider registry.
    pub fn provider_registry(&self) -> Option<Arc<ProviderRegistry>> {
        self.provider_registry.clone()
    }

    /// Install a tool registry (Issue #23 parity).
    pub fn set_tool_registry(&mut self, registry: Arc<ToolRegistry>) {
        self.tool_registry = Some(registry);
    }

    /// Read-only accessor for the installed tool registry.
    pub fn tool_registry(&self) -> Option<Arc<ToolRegistry>> {
        self.tool_registry.clone()
    }

    // ── Constitution accessors (interpreter-parity API) ──────────────

    /// Returns `true` iff a constitution is currently active.  Thin
    /// public accessor for the internal `active_constitution` field.
    pub fn has_active_constitution(&self) -> bool {
        self.active_constitution.is_some()
    }

    /// Clone the active constitution out (by value) so callers can
    /// inspect its laws without holding a borrow on the VM.  Matches
    /// the interpreter's `get_active_constitution(&self) -> Option<Constitution>`.
    pub fn get_active_constitution(&self) -> Option<Constitution> {
        let id = self.active_constitution.as_ref()?;
        self.constitutions.get(id).cloned()
    }

    /// Evaluate the active constitution against an action context,
    /// returning an error if any mandatory law is violated.  Mirrors
    /// the interpreter's `check_constitution_compliance(&self, ctx)` so
    /// preserved tests stay byte-identical (Kural 1).
    pub fn check_constitution_compliance(&self, context: &EvaluationContext) -> CompileResult<()> {
        let Some(constitution) = self.get_active_constitution() else {
            return Ok(());
        };
        let result = enforce_constitution(&constitution, context, None);
        if !result.allowed {
            let mut err = compile_codes::runtime_error(format!(
                "Governance violation in constitution '{}': {}",
                constitution.id, result.message
            ));
            // Stash each violation as a repeated `violation` context
            // entry so the interpreter-parity assertion pattern
            // (`err.context.iter().filter(|(k, _)| k == "violation")`)
            // keeps finding them.
            for v in &result.violations {
                err = err.with_context("violation", format!("{:?}", v));
            }
            return Err(err);
        }
        Ok(())
    }

    // ── MCP tool definition registry (Issue #436 parity) ─────────────

    /// Store pre-fetched tool definitions for an MCP server.
    pub fn set_mcp_tool_definitions(&self, server_name: String, tools: Vec<McpToolDefinition>) {
        let mut defs = self.mcp_tool_definitions.lock();
        enforce_cache_limit(&mut defs, MAX_MCP_CACHE);
        defs.insert(server_name, tools);
    }

    /// Read-only accessor for an MCP server's tool definitions.
    pub fn get_mcp_tool_definitions(&self, server_name: &str) -> Option<Vec<McpToolDefinition>> {
        self.mcp_tool_definitions.lock().get(server_name).cloned()
    }

    /// Check whether a specific tool exists on an MCP server.
    pub fn has_mcp_tool(&self, server_name: &str, tool_name: &str) -> bool {
        self.mcp_tool_definitions
            .lock()
            .get(server_name)
            .map(|tools| tools.iter().any(|t| t.name == tool_name))
            .unwrap_or(false)
    }

    /// Agent-scope permission enforcement for MCP calls (Issue #449).
    ///
    /// Mirrors the interpreter's `check_agent_permission` — scans the VM's
    /// globals for objects that look like agent definitions (have an
    /// `agent_id` field) and honours their `permission.{deny,dangerous}`
    /// arrays and `"none"` shorthand. Extracted here so the VM's
    /// `McpContext` impl can stay short.
    pub(crate) fn vm_check_agent_permission(&self, server: &str, tool: &str) -> HudHudResult<()> {
        use hudhudscript_bytecode::shared_value::runtime_error;
        let full_tool = format!("{}.{}", server, tool);

        // Scan globals for agent-shaped objects.
        let all_bindings = self.globals.iter();
        for (agent_sym, val) in all_bindings {
            let agent_name = hudhudscript_bytecode::interner::resolve(*agent_sym);
            let obj = match val.as_object() {
                Some(o) => o,
                _ => continue,
            };
            if !obj.contains_key("agent_id") {
                continue;
            }
            let perm = match obj.get("permission").or_else(|| obj.get("izin")) {
                Some(p) => p.clone(),
                None => continue,
            };
            if let Some(perm_obj) = perm.as_object() {
                if let Some(denied) = perm_obj.get("deny").and_then(|v| v.as_array()) {
                    for d in denied {
                        if let Some(s) = d.as_string() {
                            if s == tool || s == full_tool || s == server || s == "*" {
                                return Err(runtime_error(format!(
                                    "Permission denied: agent '{}' has '{}' in its deny list",
                                    agent_name, s
                                )));
                            }
                        }
                    }
                }
                if let Some(dangerous) = perm_obj.get("dangerous").and_then(|v| v.as_array()) {
                    for d in dangerous {
                        if let Some(s) = d.as_string() {
                            if s == tool || s == full_tool || s == server {
                                eprintln!(
                                    "[permission warning] Agent '{}' is calling dangerous tool '{}' (matched '{}')",
                                    agent_name, full_tool, s
                                );
                            }
                        }
                    }
                }
            } else if let Some(s) = perm.as_string() {
                if s == "none" {
                    return Err(runtime_error(format!(
                        "Permission denied: agent '{}' has permission set to 'none'",
                        agent_name
                    )));
                }
            }
        }
        Ok(())
    }

    // ── Promise resolution (#726) ─────────────────────────────────────
    // (intentional break — the `impl VM` block closes here so the
    // standalone `impl McpContext<Value> for VM` can follow. A fresh
    // `impl VM { ... }` reopens after it to keep all remaining methods
    // together.)
}

// ── MCP dispatch context (Kural 7 — shared-builtins/mcp_dispatch) ───────────
//
// The VM and the interpreter are driven through the same
// `crate::vm::mcp_dispatch::dispatch_mcp_tool_call` function
// via this trait. Every hook forwards to existing VM helpers:

impl VM {
    // ── (continued) Promise resolution (#726) ─────────────────────────
}
