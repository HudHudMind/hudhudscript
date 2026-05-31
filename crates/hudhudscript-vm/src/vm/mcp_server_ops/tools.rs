use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

use super::protocol::{
    default_input_schema, resource_to_value, state, tool_to_value, ResourceRecord, ToolRecord,
};

pub fn mcp_server_add_tool(args: &[Value16]) -> SharedResult<Value16> {
    let tool_def = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| runtime_error("McpServer.add_tool: expected tool definition object"))?;

    let name = tool_def
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| runtime_error("McpServer.add_tool: tool definition must include 'name'"))?;

    let description = tool_def
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let input_schema_json = tool_def
        .get("input_schema")
        .or_else(|| tool_def.get("inputSchema"))
        .map(|v| crate::json::value_to_json_string(v))
        .unwrap_or_else(default_input_schema);

    let server_name = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut st = state().lock();
    let target_name = match server_name {
        Some(n) => n,
        None => match st.servers.keys().next().cloned() {
            Some(n) => n,
            None => {
                return Err(runtime_error(
                    "McpServer.add_tool: no MCP server created. Call McpServer.create() first",
                ));
            }
        },
    };

    let server = st.servers.get_mut(&target_name).ok_or_else(|| {
        runtime_error(format!(
            "McpServer.add_tool: server '{}' not found",
            target_name
        ))
    })?;

    let record = ToolRecord {
        name,
        description,
        input_schema_json,
    };
    server.tools.push(record);
    let added = server.tools.last().unwrap();
    Ok(tool_to_value(added))
}

pub fn mcp_server_add_resource(args: &[Value16]) -> SharedResult<Value16> {
    let res_def = args.first().and_then(|v| v.as_object()).ok_or_else(|| {
        runtime_error("McpServer.add_resource: expected resource definition object")
    })?;

    let uri = res_def
        .get("uri")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| runtime_error("McpServer.add_resource: resource must include 'uri'"))?;

    let name = res_def
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uri.clone());

    let description = res_def
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mime_type = res_def
        .get("mime_type")
        .or_else(|| res_def.get("mimeType"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let server_name = args.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut st = state().lock();
    let target_name = match server_name {
        Some(n) => n,
        None => match st.servers.keys().next().cloned() {
            Some(n) => n,
            None => {
                return Err(runtime_error(
                    "McpServer.add_resource: no MCP server created. Call McpServer.create() first",
                ));
            }
        },
    };

    let server = st.servers.get_mut(&target_name).ok_or_else(|| {
        runtime_error(format!(
            "McpServer.add_resource: server '{}' not found",
            target_name
        ))
    })?;

    let record = ResourceRecord {
        uri,
        name,
        description,
        mime_type,
    };
    server.resources.push(record);
    let added = server.resources.last().unwrap();
    Ok(resource_to_value(added))
}

pub fn mcp_server_tools(_args: &[Value16]) -> SharedResult<Value16> {
    let st = state().lock();
    let tools: Vec<Value16> = st
        .servers
        .values()
        .flat_map(|s| s.tools.iter().map(|t| tool_to_value(t)))
        .collect();
    Ok(Value16::array(tools))
}

pub fn mcp_server_resources(_args: &[Value16]) -> SharedResult<Value16> {
    let st = state().lock();
    let resources: Vec<Value16> = st
        .servers
        .values()
        .flat_map(|s| s.resources.iter().map(|r| resource_to_value(r)))
        .collect();
    Ok(Value16::array(resources))
}
