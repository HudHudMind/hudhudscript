//! Tests extracted from hudhudscript-sandbox/src/tool_dispatch.rs

use hudhudscript_sandbox::{ToolDispatcher, ToolHandler, ToolPermissions};
use serde_json::json;

struct EchoTool;

impl ToolHandler for EchoTool {
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default() // no special permissions
    }

    fn call(&self, arguments: &serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(arguments.clone())
    }
}

struct FileTool;

impl ToolHandler for FileTool {
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            file_reads: vec!["/etc/passwd".to_string()],
            ..Default::default()
        }
    }

    fn call(&self, _arguments: &serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(json!("file contents"))
    }
}

struct NetworkTool;

impl ToolHandler for NetworkTool {
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            network_hosts: vec![("localhost".to_string(), 8080)],
            ..Default::default()
        }
    }

    fn call(&self, _arguments: &serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(json!("network result"))
    }
}

struct SpawnTool;

impl ToolHandler for SpawnTool {
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            spawn_commands: vec!["rm".to_string()],
            ..Default::default()
        }
    }

    fn call(&self, _arguments: &serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(json!("spawned"))
    }
}

struct FailingTool;

impl ToolHandler for FailingTool {
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }

    fn call(&self, _arguments: &serde_json::Value) -> Result<serde_json::Value, String> {
        Err("tool execution failed".to_string())
    }
}

struct WriteFileTool;

impl ToolHandler for WriteFileTool {
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            file_writes: vec!["/etc/shadow".to_string()],
            ..Default::default()
        }
    }

    fn call(&self, _arguments: &serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(json!("written"))
    }
}

#[test]
fn test_echo_tool_allowed() {
    let mut dispatcher = ToolDispatcher::restrictive();
    dispatcher.register("echo", Box::new(EchoTool));

    let result = dispatcher.dispatch("echo", &json!({"msg": "hello"}));
    assert!(result.executed);
    assert_eq!(result.output.unwrap(), json!({"msg": "hello"}));
}

#[test]
fn test_unknown_tool_denied() {
    let dispatcher = ToolDispatcher::restrictive();
    let result = dispatcher.dispatch("nonexistent", &json!({}));
    assert!(!result.executed);
    assert!(result.error.unwrap().contains("unknown tool"));
}

#[test]
fn test_file_tool_denied_by_restrictive_sandbox() {
    let mut dispatcher = ToolDispatcher::restrictive();
    dispatcher.register("file_tool", Box::new(FileTool));

    let result = dispatcher.dispatch("file_tool", &json!({}));
    // /etc/passwd is in the deny list of the restrictive sandbox
    assert!(!result.executed);
    assert!(result.error.unwrap().contains("sandbox denied"));
}

#[test]
fn test_network_tool_denied_by_restrictive() {
    let mut dispatcher = ToolDispatcher::restrictive();
    dispatcher.register("net_tool", Box::new(NetworkTool));
    let result = dispatcher.dispatch("net_tool", &json!({}));
    assert!(!result.executed);
    assert!(result.error.unwrap().contains("sandbox denied"));
}

#[test]
fn test_spawn_tool_denied_by_restrictive() {
    let mut dispatcher = ToolDispatcher::restrictive();
    dispatcher.register("spawn_tool", Box::new(SpawnTool));
    let result = dispatcher.dispatch("spawn_tool", &json!({}));
    assert!(!result.executed);
    assert!(result.error.unwrap().contains("sandbox denied"));
}

#[test]
fn test_failing_tool_returns_error() {
    let mut dispatcher = ToolDispatcher::permissive();
    dispatcher.register("fail_tool", Box::new(FailingTool));
    let result = dispatcher.dispatch("fail_tool", &json!({}));
    // Executed is true (sandbox passed), but there's an error from the handler
    assert!(result.executed);
    assert!(result.output.is_none());
    assert_eq!(result.error.unwrap(), "tool execution failed");
}

#[test]
fn test_write_file_denied_by_restrictive() {
    let mut dispatcher = ToolDispatcher::restrictive();
    dispatcher.register("write_tool", Box::new(WriteFileTool));
    let result = dispatcher.dispatch("write_tool", &json!({}));
    assert!(!result.executed);
    assert!(result.error.unwrap().contains("sandbox denied"));
}

#[test]
fn test_permissive_dispatcher() {
    let mut dispatcher = ToolDispatcher::permissive();
    dispatcher.register("echo", Box::new(EchoTool));
    let result = dispatcher.dispatch("echo", &json!({"a": 1}));
    assert!(result.executed);
    assert_eq!(result.output.unwrap(), json!({"a": 1}));
}

#[test]
fn test_dispatch_result_tool_name() {
    let dispatcher = ToolDispatcher::restrictive();
    let result = dispatcher.dispatch("nonexistent", &json!({}));
    assert_eq!(result.tool_name, "nonexistent");
}

#[test]
fn test_registered_tools_list() {
    let mut dispatcher = ToolDispatcher::restrictive();
    dispatcher.register("echo", Box::new(EchoTool));

    let tools = dispatcher.registered_tools();
    assert!(tools.contains(&"echo"));
}
