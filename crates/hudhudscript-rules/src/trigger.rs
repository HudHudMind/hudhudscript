//! Trigger evaluation for skill event pattern matching
//!
//! Provides the `TriggerEvaluator` which checks whether a bus event matches
//! a skill trigger using glob-style patterns or exact event name matching.

use crate::skill::SkillTrigger;

/// A bus event that can be matched against triggers
#[derive(Debug, Clone, PartialEq)]
pub struct BusEvent {
    /// Event type name, e.g. "file.changed"
    pub event_type: String,
    /// Optional payload path or identifier for pattern matching
    pub payload: Option<String>,
}

/// Evaluates whether a bus event matches a skill trigger
pub struct TriggerEvaluator;

impl TriggerEvaluator {
    /// Check if `event` satisfies `trigger`.
    ///
    /// - For `EventTrigger`: event type must match exactly, and if a pattern
    ///   is specified the event payload must match the glob pattern.
    /// - For `CronTrigger`: always returns false (cron is time-based, not event-based).
    /// - For `ManualTrigger`: always returns false (requires explicit invocation).
    pub fn matches(event: &BusEvent, trigger: &SkillTrigger) -> bool {
        match trigger {
            SkillTrigger::Event { event: ev, pattern } => {
                if event.event_type != *ev {
                    return false;
                }
                match (pattern, &event.payload) {
                    (Some(pat), Some(payload)) => glob_match(pat, payload),
                    (Some(_), None) => false,
                    (None, _) => true,
                }
            }
            SkillTrigger::Cron { .. } => false,
            SkillTrigger::Manual { .. } => false,
        }
    }
}

/// Simple glob pattern matching supporting `*` (any segment) and `**` is not
/// required here — we convert `*` to regex `[^/]*` for single-segment wildcards
/// and use a basic approach.
pub fn glob_match(pattern: &str, input: &str) -> bool {
    // Convert glob pattern to regex
    let mut regex_str = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume second *
                                  // optional path separator after **
                    if chars.peek() == Some(&'/') {
                        chars.next();
                    }
                    regex_str.push_str(".*");
                } else {
                    regex_str.push_str("[^/]*");
                }
            }
            '?' => regex_str.push('.'),
            '.' | '+' | '(' | ')' | '{' | '}' | '[' | ']' | '^' | '$' | '|' | '\\' => {
                regex_str.push('\\');
                regex_str.push(ch);
            }
            _ => regex_str.push(ch),
        }
    }
    regex_str.push('$');

    // We avoid pulling in the regex crate at runtime by using a simple
    // char-by-char matcher for the common case. However, for correctness
    // with complex patterns we fall back to basic matching.
    simple_regex_match(&regex_str, input)
}

/// Minimal regex-like matcher that handles literal chars, `.`, `[^/]*`, and `.*`.
fn simple_regex_match(pattern: &str, input: &str) -> bool {
    // Strip anchors
    let pat = pattern
        .strip_prefix('^')
        .unwrap_or(pattern)
        .strip_suffix('$')
        .unwrap_or(pattern);
    do_match(pat.as_bytes(), input.as_bytes())
}

fn do_match(pat: &[u8], inp: &[u8]) -> bool {
    if pat.is_empty() {
        return inp.is_empty();
    }

    // Handle `[^/]*` — match any non-slash sequence
    if pat.starts_with(b"[^/]*") {
        let rest = &pat[5..];
        // Try consuming 0..n non-slash chars
        for i in 0..=inp.len() {
            if i > 0 && inp[i - 1] == b'/' {
                break;
            }
            if do_match(rest, &inp[i..]) {
                return true;
            }
        }
        return false;
    }

    // Handle `.*` — match anything
    if pat.starts_with(b".*") {
        let rest = &pat[2..];
        for i in 0..=inp.len() {
            if do_match(rest, &inp[i..]) {
                return true;
            }
        }
        return false;
    }

    // Handle `.` — match single char
    if pat[0] == b'.' && (pat.len() < 2 || (pat[1] != b'*')) {
        if inp.is_empty() {
            return false;
        }
        return do_match(&pat[1..], &inp[1..]);
    }

    // Handle escaped chars `\X`
    if pat[0] == b'\\' && pat.len() >= 2 {
        if inp.is_empty() || inp[0] != pat[1] {
            return false;
        }
        return do_match(&pat[2..], &inp[1..]);
    }

    // Literal match
    if inp.is_empty() || pat[0] != inp[0] {
        return false;
    }
    do_match(&pat[1..], &inp[1..])
}
