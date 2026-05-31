/// Check whether a line is a horizontal rule (---, ***, ___).
pub fn is_horizontal_rule(line: &str) -> bool {
    let s = line.replace(' ', "");
    (s.starts_with("---") && s.chars().all(|c| c == '-'))
        || (s.starts_with("***") && s.chars().all(|c| c == '*'))
        || (s.starts_with("___") && s.chars().all(|c| c == '_'))
}

/// Check whether a line is a Markdown table separator (`|---|---|`).
pub fn is_table_separator(line: &str) -> bool {
    line.contains('|')
        && line
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
}

/// Parse a single table row into cells.
pub fn parse_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim().trim_matches('|');
    trimmed.split('|').map(|c| c.trim().to_string()).collect()
}

pub(crate) fn is_ordered_list_start(line: &str) -> bool {
    strip_ordered_prefix(line).is_some()
}

/// Strip the leading "N. " prefix from an ordered list item.
pub fn strip_ordered_prefix(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i >= bytes.len() {
        return None;
    }
    if bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1] == b' ' {
        Some(&line[i + 2..])
    } else {
        None
    }
}
