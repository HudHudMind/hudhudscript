//! MCP-60: Optional real `mcp-server-git` integration test.
//!
//! Only runs when `HUDHUD_MCP_GIT_TEST=1` is set.
//! Uses `uvx mcp-server-git` if available; skips gracefully if not.

use hudhudscript_mcp::{McpClient, TransportConfig};
use std::process::Command;

/// Check if `uvx` is available. Returns the path.
fn find_uvx() -> Option<String> {
    let output = Command::new("uvx").args(["--version"]).output().ok()?;
    if output.status.success() {
        Some("uvx".to_string())
    } else {
        None
    }
}

fn git_repo_path() -> String {
    std::env::var("HUDHUD_MCP_GIT_REPO").unwrap_or_else(|_| ".".to_string())
}

fn is_git_test_enabled() -> bool {
    std::env::var("HUDHUD_MCP_GIT_TEST").unwrap_or_default() == "1"
}

async fn try_connect() -> Option<McpClient> {
    if !is_git_test_enabled() {
        return None;
    }
    let _runner = find_uvx()?;
    let repo = git_repo_path();
    let config = TransportConfig::stdio(
        "uvx".to_string(),
        vec![
            "mcp-server-git".to_string(),
            "--repository".to_string(),
            repo,
        ],
    );
    McpClient::new(config).await.ok()
}

// ── Tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_real_git_server_initialize() {
    let Some(client) = try_connect().await else {
        return;
    };
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(15), client.initialize()).await;
    if let Ok(Ok(init)) = result {
        assert_eq!(init.protocol_version, "2024-11-05");
        assert!(!init.server_info.name.is_empty());
    }
}

#[tokio::test]
async fn test_real_git_server_list_tools() {
    let Some(client) = try_connect().await else {
        return;
    };
    if client.initialize().await.is_err() {
        return;
    }
    if let Ok(Ok(tools)) =
        tokio::time::timeout(std::time::Duration::from_secs(10), client.list_tools(None)).await
    {
        assert!(!tools.tools.is_empty(), "Git server should have tools");
    }
}
