use std::collections::HashMap;
use std::path::Path;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::helpers::{read_file, require_string};

pub fn parse_env_file(args: &[Value16]) -> HudHudResult<Value16> {
    let file_path = require_string(args, 0, "project.parse_env_file")?;
    let content = read_file(Path::new(&file_path), "project.parse_env_file")?;

    let mut obj = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'');
            obj.insert(key.to_string(), Value16::string(val.to_string()));
        }
    }
    Ok(Value16::object(obj))
}
