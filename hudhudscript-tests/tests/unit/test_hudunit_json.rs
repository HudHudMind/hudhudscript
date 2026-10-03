//! Tests for hudhudscript-hudunit JSON report — status fields.

use std::time::Duration;

use hudhudscript_hudunit::report::json::render;
use hudhudscript_hudunit::runner::{FileOutcome, Outcome, SuiteOutcome, TestOutcome};

#[test]
fn json_contains_status_fields() {
    let suite = SuiteOutcome {
        files: vec![FileOutcome {
            path: std::path::PathBuf::from("/t/tests/test_a.hud"),
            group: "test_a".into(),
            tests: vec![TestOutcome {
                name: "test_x".into(),
                outcome: Outcome::Failed("assert_eq failed".into()),
                duration: Duration::from_millis(1),
                groups: vec![],
                line: 2,
                output: None,
            }],
            error: None,
            fail_fast_stopped: false,
        }],
        duration: Duration::from_millis(3),
        ..Default::default()
    };
    let text = render(&suite);
    assert!(text.contains("\"status\": \"failed\""));
    assert!(text.contains("\"message\": \"assert_eq failed\""));
    assert!(text.contains("\"exit_ok\": false"));
}
