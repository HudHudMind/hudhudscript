//! Diff engine - unified diff generation and formatting
//!
//! Provides Myers-like LCS-based diff computation, unified diff formatting,
//! and ANSI-colored terminal output.

use std::fmt;

/// A single line in a diff hunk
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLine {
    /// Unchanged context line
    Context(String),
    /// Line added in the new file
    Added(String),
    /// Line removed from the old file
    Removed(String),
}

impl fmt::Display for DiffLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffLine::Context(s) => write!(f, " {}", s),
            DiffLine::Added(s) => write!(f, "+{}", s),
            DiffLine::Removed(s) => write!(f, "-{}", s),
        }
    }
}

/// A contiguous hunk of changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    /// Starting line in the old file (1-based)
    pub old_start: usize,
    /// Number of lines from the old file
    pub old_count: usize,
    /// Starting line in the new file (1-based)
    pub new_start: usize,
    /// Number of lines from the new file
    pub new_count: usize,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
}

/// Complete diff result between two files
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    /// Old file name / label
    pub old_file: String,
    /// New file name / label
    pub new_file: String,
    /// Hunks of changes
    pub hunks: Vec<DiffHunk>,
}

// ---------------------------------------------------------------------------
// LCS-based diff algorithm
// ---------------------------------------------------------------------------

/// Compute the longest common subsequence table.
/// Returns a 2D table where `table[i][j]` is the LCS length of
/// `old[0..i]` and `new[0..j]`.
fn lcs_table(old: &[&str], new: &[&str]) -> Vec<Vec<usize>> {
    let m = old.len();
    let n = new.len();
    let mut table = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = table[i - 1][j].max(table[i][j - 1]);
            }
        }
    }
    table
}

/// Raw edit operation used internally before grouping into hunks.
#[derive(Debug, Clone)]
enum EditOp {
    Equal(String),
    Insert(String),
    Delete(String),
}

/// Back-trace through the LCS table to produce an edit script.
fn backtrack(old: &[&str], new: &[&str], table: &[Vec<usize>]) -> Vec<EditOp> {
    let mut ops: Vec<EditOp> = Vec::new();
    let mut i = old.len();
    let mut j = new.len();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            ops.push(EditOp::Equal(old[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || table[i][j - 1] >= table[i - 1][j]) {
            ops.push(EditOp::Insert(new[j - 1].to_string()));
            j -= 1;
        } else {
            ops.push(EditOp::Delete(old[i - 1].to_string()));
            i -= 1;
        }
    }

    ops.reverse();
    ops
}

/// Number of context lines surrounding each change in a hunk.
const CONTEXT: usize = 3;

/// Group edit operations into hunks with surrounding context.
fn build_hunks(ops: &[EditOp]) -> Vec<DiffHunk> {
    // Mark which ops are "interesting" (non-equal) and expand context.
    let len = ops.len();
    let mut dominated = vec![false; len];
    for (idx, op) in ops.iter().enumerate() {
        if !matches!(op, EditOp::Equal(_)) {
            let lo = idx.saturating_sub(CONTEXT);
            let hi = (idx + CONTEXT + 1).min(len);
            for d in &mut dominated[lo..hi] {
                *d = true;
            }
        }
    }

    // Split into contiguous runs of dominated ops -> hunks.
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut old_line: usize = 0; // 0-based cursor into old
    let mut new_line: usize = 0;

    let mut i = 0;
    while i < len {
        if !dominated[i] {
            // Skip non-dominated equal line
            match &ops[i] {
                EditOp::Equal(_) => {
                    old_line += 1;
                    new_line += 1;
                }
                _ => unreachable!(),
            }
            i += 1;
            continue;
        }

        // Start a new hunk
        let hunk_old_start = old_line + 1; // 1-based
        let hunk_new_start = new_line + 1;
        let mut lines: Vec<DiffLine> = Vec::new();
        let mut hunk_old_count = 0usize;
        let mut hunk_new_count = 0usize;

        while i < len && dominated[i] {
            match &ops[i] {
                EditOp::Equal(s) => {
                    lines.push(DiffLine::Context(s.clone()));
                    hunk_old_count += 1;
                    hunk_new_count += 1;
                    old_line += 1;
                    new_line += 1;
                }
                EditOp::Delete(s) => {
                    lines.push(DiffLine::Removed(s.clone()));
                    hunk_old_count += 1;
                    old_line += 1;
                }
                EditOp::Insert(s) => {
                    lines.push(DiffLine::Added(s.clone()));
                    hunk_new_count += 1;
                    new_line += 1;
                }
            }
            i += 1;
        }

        hunks.push(DiffHunk {
            old_start: hunk_old_start,
            old_count: hunk_old_count,
            new_start: hunk_new_start,
            new_count: hunk_new_count,
            lines,
        });
    }

    hunks
}

/// Create a diff between two text strings.
///
/// Returns a `DiffResult` with hunks describing the differences. File labels
/// default to `"a"` and `"b"`.
pub fn create_diff(old: &str, new: &str) -> DiffResult {
    create_diff_named(old, new, "a", "b")
}

/// Create a diff with custom file labels.
pub fn create_diff_named(old: &str, new: &str, old_name: &str, new_name: &str) -> DiffResult {
    let old_lines: Vec<&str> = if old.is_empty() {
        Vec::new()
    } else {
        old.lines().collect()
    };
    let new_lines: Vec<&str> = if new.is_empty() {
        Vec::new()
    } else {
        new.lines().collect()
    };

    let table = lcs_table(&old_lines, &new_lines);
    let ops = backtrack(&old_lines, &new_lines, &table);
    let hunks = build_hunks(&ops);

    DiffResult {
        old_file: old_name.to_string(),
        new_file: new_name.to_string(),
        hunks,
    }
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Format a diff result as a standard unified diff string.
pub fn format_unified(diff: &DiffResult) -> String {
    let mut out = String::new();
    if diff.hunks.is_empty() {
        return out;
    }

    out.push_str(&format!("--- {}\n", diff.old_file));
    out.push_str(&format!("+++ {}\n", diff.new_file));

    for hunk in &diff.hunks {
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
        ));
        for line in &hunk.lines {
            out.push_str(&format!("{}\n", line));
        }
    }
    out
}

/// Format a diff result with ANSI color codes for terminal display.
///
/// - Removed lines: red (`\x1b[31m`)
/// - Added lines: green (`\x1b[32m`)
/// - Hunk headers: cyan (`\x1b[36m`)
/// - Context lines: default
pub fn format_colored(diff: &DiffResult) -> String {
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const CYAN: &str = "\x1b[36m";
    const RESET: &str = "\x1b[0m";

    let mut out = String::new();
    if diff.hunks.is_empty() {
        return out;
    }

    out.push_str(&format!("{RED}--- {}{RESET}\n", diff.old_file));
    out.push_str(&format!("{GREEN}+++ {}{RESET}\n", diff.new_file));

    for hunk in &diff.hunks {
        out.push_str(&format!(
            "{CYAN}@@ -{},{} +{},{} @@{RESET}\n",
            hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
        ));
        for line in &hunk.lines {
            match line {
                DiffLine::Context(s) => out.push_str(&format!(" {}\n", s)),
                DiffLine::Added(s) => out.push_str(&format!("{GREEN}+{}{RESET}\n", s)),
                DiffLine::Removed(s) => out.push_str(&format!("{RED}-{}{RESET}\n", s)),
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
