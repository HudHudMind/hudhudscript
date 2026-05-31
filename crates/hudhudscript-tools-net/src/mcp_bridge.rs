//! MCP ↔ ToolRegistry bridge (Issue #116)
//!
//! Connects a running `McpClient` to the `ToolRegistry` so that all tools
//! exposed by an MCP server are automatically available for dispatch.

use hudhudscript_mcp::client::McpClient;
use hudhudscript_tools_schema::registry::{RegistryError, ToolRegistry};
use hudhudscript_tools_schema::schema::{JsonSchema, ToolMetadata, ToolSchema};
use std::sync::Arc;
use tracing::{info, warn};

/// Wire one MCP client into a `ToolRegistry`.
///
/// This discovers all tools from the server and registers them under the
/// `server_name` namespace so that `registry.call_tool(name, args)` routes
/// the call through the MCP protocol automatically.
///
/// # Example
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use hudhudscript_mcp::{McpClient, TransportConfig};
/// use hudhudscript_tools_schema::ToolRegistry;
/// use hudhudscript_tools_net::mcp_bridge;
/// use std::sync::Arc;
///
/// let config = TransportConfig::stdio("uvx", vec!["mcp-server-sqlite".to_string()]);
/// let client = Arc::new(McpClient::new(config).await?);
/// client.initialize().await?;
///
/// let registry = ToolRegistry::new();
/// let discovered = mcp_bridge::wire_client(&registry, "sqlite", client).await?;
/// println!("Registered {discovered} tools from sqlite server");
/// # Ok(())
/// # }
/// ```
pub async fn wire_client(
    registry: &ToolRegistry,
    server_name: &str,
    client: Arc<McpClient>,
) -> Result<usize, RegistryError> {
    // Store the client so the registry can route calls later
    registry.register_client(server_name.to_string(), client.clone());

    // Discover tools from the server
    let tools_response = client
        .list_tools(None)
        .await
        .map_err(|e| RegistryError::DiscoveryFailed(e.to_string()))?;

    let mut count = 0;

    for tool in tools_response.tools {
        // Convert the raw JSON schema value into our typed JsonSchema
        let input_schema: JsonSchema = match serde_json::from_value(tool.input_schema.clone()) {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    "Skipping tool '{}' from '{}': invalid schema — {}",
                    tool.name, server_name, e
                );
                continue;
            }
        };

        let schema = ToolSchema {
            name: tool.name.clone(),
            description: tool.description.clone(),
            input_schema,
            server: server_name.to_string(),
        };

        let metadata =
            ToolMetadata::new(tool.name.clone(), server_name.to_string(), tool.description);

        registry.register_tool(schema, metadata)?;
        count += 1;
    }

    info!(
        "Wired {} tool(s) from MCP server '{}' into the registry",
        count, server_name
    );

    Ok(count)
}

/// Wire multiple MCP clients at once.
///
/// Returns the total number of tools registered across all servers.  Errors
/// from individual servers are logged as warnings rather than aborting the
/// entire registration.
pub async fn wire_clients(registry: &ToolRegistry, clients: Vec<(&str, Arc<McpClient>)>) -> usize {
    let mut total = 0;
    for (name, client) in clients {
        match wire_client(registry, name, client).await {
            Ok(n) => total += n,
            Err(e) => warn!("Failed to wire MCP server '{}': {}", name, e),
        }
    }
    total
}
