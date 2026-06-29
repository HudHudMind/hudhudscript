use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

use super::protocol::{evict_if_full, server_to_value, state, ServerRecord, PROTOCOL_VERSION};

pub fn mcp_server_create(args: &[Value16]) -> SharedResult<Value16> {
    let (name, version, transport, port) = match args.first() {
        Some(v) => {
            if let Some(obj) = v.as_object() {
                let name = obj
                    .get("name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("hudhud-mcp-server")
                    .to_string();
                let version = obj
                    .get("version")
                    .and_then(|x| x.as_str())
                    .unwrap_or("0.1.0")
                    .to_string();
                let transport = obj
                    .get("transport")
                    .and_then(|x| x.as_str())
                    .unwrap_or("stdio")
                    .to_string();
                let port = obj.get("port").and_then(|x| x.as_number()).unwrap_or(0.0);
                (name, version, transport, port)
            } else if let Some(s) = v.as_str() {
                (s.to_string(), "0.1.0".to_string(), "stdio".to_string(), 0.0)
            } else {
                return Err(runtime_error(
                    "McpServer.create: expected options object or name string",
                ));
            }
        }
        None => (
            "hudhud-mcp-server".to_string(),
            "0.1.0".to_string(),
            "stdio".to_string(),
            0.0,
        ),
    };

    let mut st = state().lock();
    evict_if_full(&mut st.servers);
    let record = ServerRecord {
        name: name.clone(),
        version,
        transport,
        port,
        running: false,
        tools: Vec::new(),
        resources: Vec::new(),
    };
    st.servers.insert(name.clone(), record);
    let server = st.servers.get(&name).unwrap();
    Ok(server_to_value(server))
}

pub fn mcp_server_start(args: &[Value16]) -> SharedResult<Value16> {
    let transport = args
        .first()
        .map(|v| {
            if let Some(obj) = v.as_object() {
                obj.get("transport")
                    .and_then(|x| x.as_str())
                    .unwrap_or("stdio")
                    .to_string()
            } else if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                "stdio".to_string()
            }
        })
        .unwrap_or_else(|| "stdio".to_string());

    let port = args
        .first()
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("port").and_then(|x| x.as_number()))
        .unwrap_or(0.0);

    let mut st = state().lock();
    for server in st.servers.values_mut() {
        server.running = true;
        server.transport = transport.clone();
    }

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("running".to_string(), Value16::boolean(true));
    result.insert("transport".to_string(), Value16::string(transport));
    if port > 0.0 {
        result.insert("port".to_string(), Value16::number(port));
    }
    Ok(Value16::object(result))
}

pub fn mcp_server_stop(_args: &[Value16]) -> SharedResult<Value16> {
    let mut st = state().lock();
    for server in st.servers.values_mut() {
        server.running = false;
    }

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("running".to_string(), Value16::boolean(false));
    result.insert("stopped".to_string(), Value16::boolean(true));
    Ok(Value16::object(result))
}

pub fn mcp_server_status(args: &[Value16]) -> SharedResult<Value16> {
    let st = state().lock();
    let any_running = st.servers.values().any(|s| s.running);
    let running_hint = args
        .first()
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("running"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let running = any_running || running_hint;

    let tools_count: usize = st.servers.values().map(|s| s.tools.len()).sum();
    let resources_count: usize = st.servers.values().map(|s| s.resources.len()).sum();

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("running".to_string(), Value16::boolean(running));
    result.insert(
        "tools_count".to_string(),
        Value16::number(tools_count as f64),
    );
    result.insert(
        "resources_count".to_string(),
        Value16::number(resources_count as f64),
    );
    result.insert(
        "protocol_version".to_string(),
        Value16::string(PROTOCOL_VERSION.to_string()),
    );
    Ok(Value16::object(result))
}
