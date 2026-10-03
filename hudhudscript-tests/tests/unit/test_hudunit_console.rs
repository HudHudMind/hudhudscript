//! Tests for hudhudscript-hudunit console report — plain (no ANSI) rendering.

use std::time::Duration;

use hudhudscript_hudunit::report::console::ConsoleReporter;
use hudhudscript_hudunit::runner::{FileOutcome, Outcome, SuiteOutcome, TestOutcome};

#[test]
fn plain_rendering_avoids_ansi_codes() {
    let suite = SuiteOutcome {
        files: vec![FileOutcome {
            path: std::path::PathBuf::from("/t/tests/math/test_a.hud"),
            group: "math".into(),
            tests: vec![TestOutcome {
                name: "test_x".into(),
                outcome: Outcome::Passed,
                duration: Duration::from_micros(12),
                groups: vec!["math".into()],
                line: 3,
                output: None,
            }],
            error: None,
            fail_fast_stopped: false,
        }],
        duration: Duration::from_millis(5),
        ..Default::default()
    };
    let reporter = ConsoleReporter::new(false, false, false);
    let mut buf: Vec<u8> = Vec::new();
    reporter.render(&mut buf, &suite).unwrap();
    let text = String::from_utf8(buf).unwrap();
    assert!(!text.contains("\x1b["));
    assert!(text.contains("test_x"));
    assert!(text.contains("1 test: 1 geçti"));
}
