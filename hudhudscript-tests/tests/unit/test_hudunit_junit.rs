//! Tests for hudhudscript-hudunit JUnit XML report — structure and
//! attribute escaping.

use std::time::Duration;

use hudhudscript_hudunit::report::junit::render;
use hudhudscript_hudunit::runner::{FileOutcome, Outcome, SuiteOutcome, TestOutcome};

#[test]
fn junit_structure_and_escaping() {
    let suite = SuiteOutcome {
        files: vec![FileOutcome {
            path: std::path::PathBuf::from("/t/tests/test_a.hud"),
            group: "math".into(),
            tests: vec![
                TestOutcome {
                    name: "test_ok".into(),
                    outcome: Outcome::Passed,
                    duration: Duration::from_millis(2),
                    groups: vec![],
                    line: 1,
                    output: None,
                },
                TestOutcome {
                    name: "test_bozuk".into(),
                    outcome: Outcome::Failed("expected 2 <&> got 1".into()),
                    duration: Duration::from_millis(1),
                    groups: vec![],
                    line: 5,
                    output: None,
                },
                TestOutcome {
                    name: "test_atla".into(),
                    outcome: Outcome::Skipped,
                    duration: Duration::ZERO,
                    groups: vec![],
                    line: 9,
                    output: None,
                },
            ],
            error: None,
            fail_fast_stopped: false,
        }],
        duration: Duration::from_millis(3),
        ..Default::default()
    };
    let xml = render(&suite);
    assert!(xml.contains("<testsuites name=\"hudunit\" tests=\"3\" failures=\"1\""));
    assert!(xml.contains("<testsuite name=\"/t/tests/test_a.hud\""));
    assert!(xml.contains("<testcase name=\"test_ok\" classname=\"math\""));
    assert!(xml.contains("<skipped/>"));
    assert!(xml.contains("<failure message=\"expected 2 &lt;&amp;&gt; got 1\">"));
    // Valid XML: no raw angle brackets inside attribute values.
    assert!(!xml.contains("message=\"expected 2 <"));
}
