//! Query string parser: `?a=1&b=2` → `{a: "1", b: "2"}`

use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

/// Parse `application/x-www-form-urlencoded` query string or body.
pub fn parse_query_string(input: &str) -> Value16 {
    let mut map = hudhudscript_bytecode::ObjMap::default();
    if input.is_empty() {
        return Value16::object(map);
    }
    for pair in input.split('&') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        match pair.find('=') {
            Some(idx) => {
                let key = url_decode(&pair[..idx]);
                let val = url_decode(&pair[idx + 1..]);
                map.insert(key, Value16::string(val));
            }
            None => {
                let key = url_decode(pair);
                map.insert(key, Value16::string(String::new()));
            }
        }
    }
    Value16::object(map)
}

/// Percent-decode a URL-encoded string.
fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h1), Some(h2)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h1 << 4) | h2);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
