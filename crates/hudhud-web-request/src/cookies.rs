//! Cookie header parser: `Cookie: a=1; b=2` → `{a: "1", b: "2"}`

use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

/// Parse a `Cookie` header value into a Value16 object.
pub fn parse_cookies(cookie_header: &str) -> Value16 {
    let mut map = hudhudscript_bytecode::ObjMap::default();
    if cookie_header.is_empty() {
        return Value16::object(map);
    }
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        match pair.find('=') {
            Some(idx) => {
                let key = pair[..idx].trim().to_string();
                let val = pair[idx + 1..].trim().to_string();
                map.insert(key, Value16::string(val));
            }
            None => {
                map.insert(pair.to_string(), Value16::string(String::new()));
            }
        }
    }
    Value16::object(map)
}
