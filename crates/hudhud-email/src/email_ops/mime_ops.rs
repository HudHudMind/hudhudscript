//! MIME parsing and Maildir listing operations.

use std::collections::HashMap;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::helpers::{runtime_error, type_error};

pub fn email_parse_mime(args: &[Value16]) -> HudHudResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(
            "email.parse_mime() requires 1 argument: raw_text",
        ));
    }
    let raw = args[0]
        .as_str()
        .ok_or_else(|| type_error("string", args[0].type_name_str(), "email.parse_mime raw_text"))?;

    let (header_section, body) = match raw.find("\r\n\r\n") {
        Some(pos) => (&raw[..pos], raw[pos + 4..].to_string()),
        None => match raw.find("\n\n") {
            Some(pos) => (&raw[..pos], raw[pos + 2..].to_string()),
            None => (raw, String::new()),
        },
    };

    let mut from = String::new();
    let mut to = String::new();
    let mut subject = String::new();
    let mut date = String::new();
    let mut headers: HashMap<String, Value16> = HashMap::new();

    let mut unfolded = String::new();
    for line in header_section.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            unfolded.push(' ');
            unfolded.push_str(line.trim());
        } else {
            if !unfolded.is_empty() {
                unfolded.push('\n');
            }
            unfolded.push_str(line);
        }
    }

    for line in unfolded.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key_trimmed = key.trim();
            let value_trimmed = value.trim().to_string();
            let key_lower = key_trimmed.to_lowercase();
            headers.insert(key_trimmed.to_string(), Value16::string(value_trimmed.clone()));
            match key_lower.as_str() {
                "from" => from = value_trimmed,
                "to" => to = value_trimmed,
                "subject" => subject = value_trimmed,
                "date" => date = value_trimmed,
                _ => {}
            }
        }
    }

    let mut result = HashMap::new();
    result.insert("from".to_string(), Value16::string(from));
    result.insert("to".to_string(), Value16::string(to));
    result.insert("subject".to_string(), Value16::string(subject));
    result.insert("date".to_string(), Value16::string(date));
    result.insert("body".to_string(), Value16::string(body));
    result.insert("headers".to_string(), Value16::object(headers));
    Ok(Value16::object(result))
}

pub fn email_list_maildir(args: &[Value16]) -> HudHudResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(
            "email.list_maildir() requires 1 argument: path",
        ));
    }
    let dir_path = args[0]
        .as_str()
        .ok_or_else(|| type_error("string", args[0].type_name_str(), "email.list_maildir path"))?
        .to_string();
    let path = std::path::Path::new(&dir_path);

    if !path.is_dir() {
        return Err(runtime_error(format!(
            "Maildir path does not exist or is not a directory: {}",
            dir_path
        )));
    }

    let mut results: Vec<Value16> = Vec::new();

    let entries = std::fs::read_dir(path)
        .map_err(|e| runtime_error(format!("Failed to read Maildir: {}", e)))?;

    for entry in entries.flatten() {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }
        let filename = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let header_end = content
            .find("\r\n\r\n")
            .or_else(|| content.find("\n\n"))
            .unwrap_or(content.len());
        let header_section = &content[..header_end];

        let mut from = String::new();
        let mut subject = String::new();
        let mut date = String::new();

        for line in header_section.lines() {
            if let Some(val) = line
                .strip_prefix("From:")
                .or_else(|| line.strip_prefix("from:"))
            {
                from = val.trim().to_string();
            } else if let Some(val) = line
                .strip_prefix("Subject:")
                .or_else(|| line.strip_prefix("subject:"))
            {
                subject = val.trim().to_string();
            } else if let Some(val) = line
                .strip_prefix("Date:")
                .or_else(|| line.strip_prefix("date:"))
            {
                date = val.trim().to_string();
            }
        }

        let mut item = HashMap::new();
        item.insert("filename".to_string(), Value16::string(filename));
        item.insert("from".to_string(), Value16::string(from));
        item.insert("subject".to_string(), Value16::string(subject));
        item.insert("date".to_string(), Value16::string(date));
        results.push(Value16::object(item));
    }
    Ok(Value16::array(results))
}
