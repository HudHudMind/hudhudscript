//! Tests for hudhudscript-hudunit HTML report — escaping, epoch math,
//! document rendering.

use std::time::Duration;

use hudhudscript_hudunit::report::html::{civil_from_days, escape, render};
use hudhudscript_hudunit::runner::{FileOutcome, Outcome, SuiteOutcome, TestOutcome};

// ── escaping / dates ──────────────────────────────────────────────

#[test]
fn escapes_html() {
    assert_eq!(escape("<script>&'\""), "&lt;script&gt;&amp;&#39;&quot;");
}

#[test]
fn civil_from_days_epoch() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(civil_from_days(19_723), (2024, 1, 1));
}

// ── rendering ─────────────────────────────────────────────────────

#[test]
fn html_report_renders_cards() {
    let suite = SuiteOutcome {
        files: vec![FileOutcome {
            path: std::path::PathBuf::from("/t/tests/test_a.hud"),
            group: "test_a".into(),
            tests: vec![TestOutcome {
                name: "test_x".into(),
                outcome: Outcome::Passed,
                duration: Duration::from_millis(2),
                groups: vec![],
                line: 1,
                output: None,
            }],
            error: None,
            fail_fast_stopped: false,
        }],
        duration: Duration::from_millis(4),
        ..Default::default()
    };
    let html = render(&suite);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("test_x"));
    assert!(html.contains("1 test") || html.contains("Geçti"));
}
