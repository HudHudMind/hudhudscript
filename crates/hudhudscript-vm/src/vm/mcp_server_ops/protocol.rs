use hudhudscript_bytecode::Value16;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;

pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const MAX_SERVERS: usize = 128;

pub struct ToolRecord {
    pub name: String,
    pub description: Option<String>,
    pub input_schema_json: String,
}

pub struct ResourceRecord {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

pub struct ServerRecord {
    pub name: String,
    pub version: String,
    pub transport: String,
    pub port: f64,
    pub running: bool,
    pub tools: Vec<ToolRecord>,
    pub resources: Vec<ResourceRecord>,
}

pub struct McpState {
    pub servers: HashMap<String, ServerRecord>,
}

static MCP_STATE: OnceLock<Mutex<McpState>> = OnceLock::new();

pub fn state() -> &'static Mutex<McpState> {
    MCP_STATE.get_or_init(|| {
        Mutex::new(McpState {
            servers: HashMap::new(),
        })
    })
}

pub fn default_input_schema() -> String {
    "{\"type\":\"object\"}".to_string()
}

pub fn evict_if_full(servers: &mut HashMap<String, ServerRecord>) {
    if servers.len() >= MAX_SERVERS {
        if let Some(oldest_key) = servers.keys().next().cloned() {
            servers.remove(&oldest_key);
        }
    }
}

pub fn tool_to_value(t: &ToolRecord) -> Value16 {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("name".to_string(), Value16::string(t.name.clone()));
    obj.insert(
        "description".to_string(),
        t.description
            .as_ref()
            .map(|s| Value16::string(s.clone()))
            .unwrap_or_else(Value16::null),
    );
    let schema = if t.input_schema_json.is_empty() {
        let mut m = hudhudscript_bytecode::ObjMap::default();
        m.insert("type".to_string(), Value16::string("object".to_string()));
        Value16::object(m)
    } else {
        match serde_json::from_str::<serde_json::Value>(&t.input_schema_json) {
            Ok(j) => crate::json::serde_to_value(&j),
            Err(_) => {
                let mut m = hudhudscript_bytecode::ObjMap::default();
                m.insert("type".to_string(), Value16::string("object".to_string()));
                Value16::object(m)
            }
        }
    };
    obj.insert("inputSchema".to_string(), schema);
    obj.insert("registered".to_string(), Value16::boolean(true));
    Value16::object(obj)
}

pub fn resource_to_value(r: &ResourceRecord) -> Value16 {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("uri".to_string(), Value16::string(r.uri.clone()));
    obj.insert("name".to_string(), Value16::string(r.name.clone()));
    obj.insert(
        "description".to_string(),
        r.description
            .as_ref()
            .map(|s| Value16::string(s.clone()))
            .unwrap_or_else(Value16::null),
    );
    obj.insert(
        "mimeType".to_string(),
        r.mime_type
            .as_ref()
            .map(|s| Value16::string(s.clone()))
            .unwrap_or_else(Value16::null),
    );
    obj.insert("registered".to_string(), Value16::boolean(true));
    Value16::object(obj)
}

pub fn server_to_value(s: &ServerRecord) -> Value16 {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("name".to_string(), Value16::string(s.name.clone()));
    obj.insert("version".to_string(), Value16::string(s.version.clone()));
    obj.insert(
        "transport".to_string(),
        Value16::string(s.transport.clone()),
    );
    obj.insert("port".to_string(), Value16::number(s.port));
    obj.insert(
        "protocol_version".to_string(),
        Value16::string(PROTOCOL_VERSION.to_string()),
    );
    obj.insert("running".to_string(), Value16::boolean(s.running));
    let tools: Vec<Value16> = s.tools.iter().map(|t| tool_to_value(t)).collect();
    obj.insert("tools".to_string(), Value16::array(tools));
    let resources: Vec<Value16> = s.resources.iter().map(|r| resource_to_value(r)).collect();
    obj.insert("resources".to_string(), Value16::array(resources));
    Value16::object(obj)
}
