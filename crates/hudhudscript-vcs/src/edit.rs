//! Code editing primitives
//!
//! Line-based editing operations with indentation preservation.
//! All line indices are **1-based** (matching what users see in editors).

/// Insert new lines **before** the given 1-based line number.
///
/// If `line` is greater than the number of lines, the new lines are appended
/// at the end.
pub fn insert_lines(content: &str, line: usize, new_lines: &[&str]) -> String {
    let mut lines: Vec<&str> = content.lines().collect();
    let idx = (line.saturating_sub(1)).min(lines.len());
    for (i, nl) in new_lines.iter().enumerate() {
        lines.insert(idx + i, nl);
    }
    lines.join("\n")
}

/// Replace lines in the range `[start, end]` (1-based, inclusive) with
/// `new_lines`.
pub fn replace_lines(content: &str, start: usize, end: usize, new_lines: &[&str]) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let s = start.saturating_sub(1).min(lines.len());
    let e = end.min(lines.len());

    let mut result: Vec<&str> = Vec::with_capacity(lines.len());
    result.extend_from_slice(&lines[..s]);
    result.extend_from_slice(new_lines);
    if e < lines.len() {
        result.extend_from_slice(&lines[e..]);
    }
    result.join("\n")
}

/// Delete lines in the range `[start, end]` (1-based, inclusive).
pub fn delete_lines(content: &str, start: usize, end: usize) -> String {
    replace_lines(content, start, end, &[])
}

/// Prepend `indent` to every line in the range `[start, end]` (1-based,
/// inclusive).
pub fn indent_lines(content: &str, start: usize, end: usize, indent: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let s = start.saturating_sub(1).min(lines.len());
    let e = end.min(lines.len());

    for line in &mut lines[s..e] {
        *line = format!("{}{}", indent, line);
    }

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
