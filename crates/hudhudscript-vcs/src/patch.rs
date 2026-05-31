//! Patch parsing and application
//!
//! Parses unified diff text into a `Patch` struct, then applies it to file
//! content with exact or fuzzy (shifted-context) matching.

use crate::diff::DiffLine;
use crate::state_tree::VcsError;

/// A parsed patch (one or more hunks from a unified diff).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// Old file label
    pub old_file: String,
    /// New file label
    pub new_file: String,
    /// Hunks in order
    pub hunks: Vec<PatchHunk>,
}

/// A single hunk inside a patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchHunk {
    /// Starting line in old file (1-based)
    pub old_start: usize,
    /// Number of old-file lines covered
    pub old_count: usize,
    /// Starting line in new file (1-based)
    pub new_start: usize,
    /// Number of new-file lines covered
    pub new_count: usize,
    /// Diff lines
    pub lines: Vec<DiffLine>,
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse a unified diff text into a `Patch`.
pub fn parse_patch(text: &str) -> Result<Patch, VcsError> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 2 {
        return Err(VcsError::InvalidOperation(
            "Patch text too short".to_string(),
        ));
    }

    let old_file = parse_file_header(&lines, "---")?;
    let new_file = parse_file_header(&lines, "+++")?;

    let mut hunks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("@@") {
            let (hunk, next) = parse_hunk(&lines, i)?;
            hunks.push(hunk);
            i = next;
        } else {
            i += 1;
        }
    }

    if hunks.is_empty() {
        return Err(VcsError::InvalidOperation(
            "No hunks found in patch".to_string(),
        ));
    }

    Ok(Patch {
        old_file,
        new_file,
        hunks,
    })
}

fn parse_file_header(lines: &[&str], prefix: &str) -> Result<String, VcsError> {
    for line in lines {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Ok(rest.trim().to_string());
        }
    }
    Err(VcsError::InvalidOperation(format!(
        "Missing '{}' header in patch",
        prefix
    )))
}

fn parse_hunk(lines: &[&str], start: usize) -> Result<(PatchHunk, usize), VcsError> {
    let header = lines[start];
    let (old_start, old_count, new_start, new_count) = parse_hunk_header(header)?;

    let mut diff_lines = Vec::new();
    let mut i = start + 1;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("@@") || line.starts_with("---") || line.starts_with("+++") {
            break;
        }
        if let Some(rest) = line.strip_prefix('+') {
            diff_lines.push(DiffLine::Added(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix('-') {
            diff_lines.push(DiffLine::Removed(rest.to_string()));
        } else if let Some(rest) = line.strip_prefix(' ') {
            diff_lines.push(DiffLine::Context(rest.to_string()));
        } else if line.is_empty() {
            // Empty context line (trailing whitespace stripped)
            diff_lines.push(DiffLine::Context(String::new()));
        } else {
            // Treat unrecognised lines as context
            diff_lines.push(DiffLine::Context(line.to_string()));
        }
        i += 1;
    }

    Ok((
        PatchHunk {
            old_start,
            old_count,
            new_start,
            new_count,
            lines: diff_lines,
        },
        i,
    ))
}

fn parse_hunk_header(header: &str) -> Result<(usize, usize, usize, usize), VcsError> {
    // Expected: @@ -old_start,old_count +new_start,new_count @@
    let err = || VcsError::InvalidOperation(format!("Invalid hunk header: {}", header));

    let header = header.trim();
    let inner = header
        .strip_prefix("@@")
        .and_then(|s| {
            // Strip trailing @@ (and anything after it)
            s.find("@@").map(|idx| &s[..idx])
        })
        .ok_or_else(err)?
        .trim();

    let parts: Vec<&str> = inner.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(err());
    }

    let (old_start, old_count) = parse_range(parts[0].strip_prefix('-').ok_or_else(err)?)?;
    let (new_start, new_count) = parse_range(parts[1].strip_prefix('+').ok_or_else(err)?)?;

    Ok((old_start, old_count, new_start, new_count))
}

fn parse_range(s: &str) -> Result<(usize, usize), VcsError> {
    let err = || VcsError::InvalidOperation(format!("Invalid range: {}", s));

    if let Some((a, b)) = s.split_once(',') {
        Ok((
            a.parse::<usize>().map_err(|_| err())?,
            b.parse::<usize>().map_err(|_| err())?,
        ))
    } else {
        // Single number means count = 1
        Ok((s.parse::<usize>().map_err(|_| err())?, 1))
    }
}

// ---------------------------------------------------------------------------
// Application — exact
// ---------------------------------------------------------------------------

/// Apply a patch to `content` using exact context matching.
pub fn apply_patch(content: &str, patch: &Patch) -> Result<String, VcsError> {
    apply_fuzzy(content, patch, 0)
}

// ---------------------------------------------------------------------------
// Application — fuzzy
// ---------------------------------------------------------------------------

/// Apply a patch with fuzzy matching: context lines may be shifted by up to
/// `fuzz` lines in either direction from the expected position.
pub fn apply_fuzzy(content: &str, patch: &Patch, fuzz: usize) -> Result<String, VcsError> {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    // We apply hunks in reverse order so earlier line-number adjustments
    // don't invalidate later hunks.
    let mut hunks: Vec<&PatchHunk> = patch.hunks.iter().collect();
    hunks.sort_by(|a, b| b.old_start.cmp(&a.old_start));

    for hunk in hunks {
        lines = apply_single_hunk(lines, hunk, fuzz)?;
    }

    Ok(lines.join("\n"))
}

fn apply_single_hunk(
    mut lines: Vec<String>,
    hunk: &PatchHunk,
    fuzz: usize,
) -> Result<Vec<String>, VcsError> {
    // Collect the old-side lines from the hunk (context + removed).
    let old_lines: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|l| match l {
            DiffLine::Context(s) => Some(s.as_str()),
            DiffLine::Removed(s) => Some(s.as_str()),
            DiffLine::Added(_) => None,
        })
        .collect();

    // Collect the new-side lines (context + added).
    let new_lines: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|l| match l {
            DiffLine::Context(s) => Some(s.as_str()),
            DiffLine::Added(s) => Some(s.as_str()),
            DiffLine::Removed(_) => None,
        })
        .collect();

    // Try to find a match starting from the expected position, then fan out.
    let expected = if hunk.old_start > 0 {
        hunk.old_start - 1
    } else {
        0
    };

    let match_pos = find_match(&lines, &old_lines, expected, fuzz)?;

    // Replace matched range with new lines.
    let replacement: Vec<String> = new_lines.iter().map(|s| s.to_string()).collect();
    let end = match_pos + old_lines.len();
    lines.splice(match_pos..end, replacement);

    Ok(lines)
}

fn find_match(
    lines: &[String],
    pattern: &[&str],
    expected: usize,
    fuzz: usize,
) -> Result<usize, VcsError> {
    // Handle empty pattern (pure insertion)
    if pattern.is_empty() {
        let pos = expected.min(lines.len());
        return Ok(pos);
    }

    // Try the expected position first, then fan out by offset 1..=fuzz
    // checking both directions at each step.
    for offset in 0..=fuzz {
        // Try expected + offset
        let pos_down = expected.checked_add(offset);
        if let Some(pos) = pos_down {
            if pos + pattern.len() <= lines.len() && matches_at(lines, pattern, pos) {
                return Ok(pos);
            }
        }

        // Try expected - offset (skip duplicate when offset == 0)
        if offset > 0 {
            let pos_up = expected.checked_sub(offset);
            if let Some(pos) = pos_up {
                if pos + pattern.len() <= lines.len() && matches_at(lines, pattern, pos) {
                    return Ok(pos);
                }
            }
        }
    }

    // If not found within fuzz range, do a full scan as a last resort
    // within a wider window of fuzz * 2 lines in both directions.
    let scan_lo = expected.saturating_sub(fuzz * 2);
    let scan_hi = (expected + fuzz * 2 + 1).min(lines.len());
    for pos in scan_lo..scan_hi {
        if pos + pattern.len() <= lines.len() && matches_at(lines, pattern, pos) {
            return Ok(pos);
        }
    }

    Err(VcsError::InvalidOperation(format!(
        "Patch hunk does not match at expected position {} (fuzz={})",
        expected + 1,
        fuzz,
    )))
}

fn matches_at(lines: &[String], pattern: &[&str], pos: usize) -> bool {
    for (i, p) in pattern.iter().enumerate() {
        if lines[pos + i] != *p {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
