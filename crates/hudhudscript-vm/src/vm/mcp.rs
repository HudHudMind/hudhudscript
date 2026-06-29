use crate::vm::mcp_dispatch::McpContext;
use crate::vm::VM;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};
use std::sync::Arc;

impl McpContext for VM {
    fn mcp_check_permission(&self, server: &str, tool: &str) -> HudHudResult<()> {
        self.vm_check_agent_permission(server, tool)
    }

    fn mcp_check_constitution(&self, server: &str, tool: &str) -> HudHudResult<()> {
        use hudhudscript_bytecode::shared_value::runtime_error;
        let Some(active_name) = self.active_constitution.clone() else {
            return Ok(());
        };
        let Some(constitution) = self.constitutions.get(&active_name).cloned() else {
            return Ok(());
        };
        let mut context = EvaluationContext::new();
        context.insert("action_type".to_string(), serde_json::json!("mcp_call"));
        context.insert("server_name".to_string(), serde_json::json!(server));
        context.insert("tool_name".to_string(), serde_json::json!(tool));
        let result = enforce_constitution(&constitution, &context, None);
        if !result.allowed {
            return Err(runtime_error(format!(
                "Governance violation in constitution '{}': {} ({} violations)",
                constitution.id,
                result.message,
                result.violations.len()
            )));
        }
        Ok(())
    }

    fn mcp_check_sandbox(&self, server: &str) -> HudHudResult<()> {
        use hudhudscript_bytecode::shared_value::runtime_error;
        if let Some(sandbox) = &self.sandbox {
            // MCP-40/41: require network OR process capability.
            let can_access = sandbox.allow_network || sandbox.allow_process;
            if !can_access {
                return Err(runtime_error(format!(
                    "Sandbox: MCP access denied for '{}'. Enable allow_network or allow_process in sandbox config.",
                    server
                )));
            }
            if !sandbox.allowed_hosts.is_empty() {
                let mcp_host = format!("mcp.{}", server);
                let allowed = sandbox
                    .allowed_hosts
                    .iter()
                    .any(|h| h == &mcp_host || h == server);
                if !allowed {
                    return Err(runtime_error(format!(
                        "Sandbox: mcp server '{}' is not in the allowed host list",
                        server
                    )));
                }
            }
        }
        Ok(())
    }

    fn mcp_get_client(&self, server: &str) -> Option<Arc<hudhudscript_mcp::McpClient>> {
        VM::get_mcp_client(self, server)
    }
}
