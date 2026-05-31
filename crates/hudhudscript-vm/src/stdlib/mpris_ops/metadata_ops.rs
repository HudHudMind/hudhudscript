pub(crate) fn extract_string_from_property(raw: &str) -> String {
    if let Some(start) = raw.find('\'') {
        if let Some(end) = raw[start + 1..].find('\'') {
            return raw[start + 1..start + 1 + end].to_string();
        }
    }
    for line in raw.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with("variant") || trimmed.contains("string") {
            if let Some(s) = trimmed.split('"').nth(1) {
                return s.to_string();
            }
        }
    }
    raw.trim().to_string()
}

pub(crate) fn extract_double_from_property(raw: &str) -> f64 {
    for token in raw.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-');
        if !cleaned.is_empty() {
            if let Ok(n) = cleaned.parse::<f64>() {
                return n;
            }
        }
    }
    0.0
}

pub(crate) fn extract_int64_from_property(raw: &str) -> i64 {
    for token in raw.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '-');
        if !cleaned.is_empty() {
            if let Ok(n) = cleaned.parse::<i64>() {
                return n;
            }
        }
    }
    0
}

pub(crate) fn extract_string_array_from_property(raw: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_array = false;
    for segment in raw.split('\'') {
        let s = segment.trim();
        if s.contains('[') {
            in_array = true;
            continue;
        }
        if in_array && !s.is_empty() && !s.starts_with(',') && !s.starts_with(']') {
            result.push(s.to_string());
        }
        if s.contains(']') {
            in_array = false;
        }
    }
    result
}

pub(crate) fn extract_metadata_string(raw: &str, key: &str) -> String {
    if let Some(idx) = raw.find(key) {
        let after = &raw[idx + key.len()..];
        if key.contains("artist") {
            let artists = extract_string_array_from_property(after);
            if !artists.is_empty() {
                return artists.join(", ");
            }
        }
        return extract_string_from_property(after);
    }
    String::new()
}

pub(crate) fn extract_metadata_int64(raw: &str, key: &str) -> i64 {
    if let Some(idx) = raw.find(key) {
        let after = &raw[idx + key.len()..];
        return extract_int64_from_property(after);
    }
    0
}
