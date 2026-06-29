//! Multipart/form-data parser.
//!
//! Parses `multipart/form-data` request bodies, extracting file name,
//! content type, and content for each part. Returns a Value16 object
//! `{ fieldname: {filename, content_type, data} }`.

use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

/// Parsed multipart part.
struct MultipartPart {
    name: String,
    filename: Option<String>,
    content_type: Option<String>,
    data: Vec<u8>,
}

/// Parse `multipart/form-data` body.
/// Returns a Value16 object where keys are form field names and values
/// are objects with `filename`, `content_type`, and `data` (string) fields.
pub fn parse_multipart(body: &[u8], boundary: &str) -> Value16 {
    let parts = split_parts(body, boundary);
    let mut map = hudhudscript_bytecode::ObjMap::default();

    for part in parts {
        let parsed = parse_part(&part);
        if parsed.name.is_empty() {
            continue;
        }
        let mut file_obj = hudhudscript_bytecode::ObjMap::default();
        file_obj.insert(
            "filename".to_string(),
            match &parsed.filename {
                Some(f) => Value16::string(f.clone()),
                None => Value16::null(),
            },
        );
        file_obj.insert(
            "content_type".to_string(),
            match &parsed.content_type {
                Some(ct) => Value16::string(ct.clone()),
                None => Value16::null(),
            },
        );
        // Store data as string (binary safe — caller can base64 if needed)
        file_obj.insert(
            "data".to_string(),
            Value16::string(String::from_utf8_lossy(&parsed.data).to_string()),
        );
        map.insert(parsed.name, Value16::object(file_obj));
    }

    Value16::object(map)
}

/// Split body by boundary delimiter.
fn split_parts(body: &[u8], boundary: &str) -> Vec<Vec<u8>> {
    let delimiter = format!("--{}", boundary);
    let delim_bytes = delimiter.as_bytes();
    let end_delim = format!("--{}--", boundary);
    let end_delim_bytes = end_delim.as_bytes();

    let mut parts = Vec::new();
    let mut start = 0;

    // Skip past the first boundary
    if let Some(pos) = find_bytes(body, delim_bytes, start) {
        start = pos + delim_bytes.len();
        // Skip CRLF after boundary
        if start + 2 <= body.len() && &body[start..start + 2] == b"\r\n" {
            start += 2;
        } else if start < body.len() && body[start] == b'\n' {
            start += 1;
        }
    }

    while start < body.len() {
        // Check for end delimiter
        if start + end_delim_bytes.len() <= body.len()
            && &body[start..start + end_delim_bytes.len()] == end_delim_bytes
        {
            break;
        }

        match find_bytes(body, delim_bytes, start) {
            Some(next_boundary) => {
                // Part ends before the CRLF preceding next boundary
                let mut end = next_boundary;
                if end >= 2 && &body[end - 2..end] == b"\r\n" {
                    end -= 2;
                } else if end >= 1 && body[end - 1] == b'\n' {
                    end -= 1;
                }
                if end > start {
                    parts.push(body[start..end].to_vec());
                }
                start = next_boundary + delim_bytes.len();
                // Skip CRLF after boundary
                if start + 2 <= body.len() && &body[start..start + 2] == b"\r\n" {
                    start += 2;
                } else if start < body.len() && body[start] == b'\n' {
                    start += 1;
                }
            }
            None => {
                // Last part (trim trailing end delimiter)
                let mut end = body.len();
                if end >= end_delim_bytes.len()
                    && &body[end - end_delim_bytes.len()..end] == end_delim_bytes
                {
                    end -= end_delim_bytes.len();
                }
                if end >= 2 && &body[end - 2..end] == b"\r\n" {
                    end -= 2;
                } else if end >= 1 && body[end - 1] == b'\n' {
                    end -= 1;
                }
                if end > start {
                    parts.push(body[start..end].to_vec());
                }
                break;
            }
        }
    }

    parts
}

/// Find first occurrence of needle in haystack starting from `from`.
fn find_bytes(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from >= haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| from + p)
}

/// Parse a single multipart part (headers + body).
fn parse_part(part: &[u8]) -> MultipartPart {
    let mut name = String::new();
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut body_start = 0;

    // Parse headers
    let mut pos = 0;
    while pos < part.len() {
        // Find end of header line
        let line_end = match find_bytes(part, b"\r\n", pos) {
            Some(p) => p,
            None => break,
        };

        if line_end == pos {
            // Empty line = end of headers
            body_start = line_end + 2;
            break;
        }

        let header_line = &part[pos..line_end];
        if let Some(colon) = header_line.iter().position(|&b| b == b':') {
            let key = String::from_utf8_lossy(&header_line[..colon])
                .trim()
                .to_lowercase();
            let val = String::from_utf8_lossy(&header_line[colon + 1..])
                .trim()
                .to_string();

            match key.as_str() {
                "content-disposition" => {
                    // Extract name="..." and filename="..."
                    name = extract_attr(&val, "name");
                    filename = extract_attr_opt(&val, "filename");
                }
                "content-type" => {
                    content_type = Some(val);
                }
                _ => {}
            }
        }

        pos = line_end + 2;
    }

    let data = part[body_start..].to_vec();

    MultipartPart {
        name,
        filename,
        content_type,
        data,
    }
}

/// Extract a quoted attribute value from a header like `form-data; name="foo"`.
fn extract_attr(header: &str, attr: &str) -> String {
    extract_attr_opt(header, attr).unwrap_or_default()
}

fn extract_attr_opt(header: &str, attr: &str) -> Option<String> {
    let prefix = format!("{}=", attr);
    let rest = header.split(';').find(|s| s.trim().starts_with(&prefix))?;
    let val_part = rest.trim()[prefix.len()..].trim();
    // Strip quotes if present
    if val_part.starts_with('"') && val_part.ends_with('"') && val_part.len() >= 2 {
        Some(val_part[1..val_part.len() - 1].to_string())
    } else {
        Some(val_part.to_string())
    }
}

