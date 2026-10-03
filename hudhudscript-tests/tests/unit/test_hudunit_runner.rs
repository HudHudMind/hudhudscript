//! Tests for hudhudscript-hudunit runner — full run pipeline (pass/fail/skip,
//! filter, setup/teardown, coverage marks, assert_throws, fail-fast, timeout,
//! stdout capture).

use hudhudscript_hudunit::config::HudunitConfig;
use hudhudscript_hudunit::runner::{run, Outcome, RunOptions, SuiteOutcome};

// ── run pipeline ──────────────────────────────────────────────────

fn run_source(name: &str, source: &str, opts: &RunOptions) -> SuiteOutcome {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, source).unwrap();
    let file = hudhudscript_hudunit::discover::discover_file(&path, None).unwrap();
    let cfg = HudunitConfig::default();
    let outcome = run(&[file], &cfg, opts);
    std::fs::remove_file(&path).ok();
    outcome
}

#[test]
fn passing_assertions() {
    let outcome = run_source(
        "hudunit_selftest_pass.hud",
        "fn test_ok() {\n    assert_eq(1 + 1, 2);\n    assert_true(true);\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(outcome.passed(), 1);
    assert_eq!(outcome.failed_count(), 0);
    assert!(outcome.exit_ok());
}

#[test]
fn failing_assertions() {
    let outcome = run_source(
        "hudunit_selftest_fail.hud",
        "fn test_bad() {\n    assert_eq(1, 2);\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(outcome.failed_count(), 1);
    assert!(!outcome.exit_ok());
    let message = match outcome
        .tests()
        .find(|t| t.name == "test_bad")
        .map(|t| &t.outcome)
    {
        Some(Outcome::Failed(message)) => message.clone(),
        _ => panic!("expected failure"),
    };
    assert!(
        message.contains("assert_eq failed"),
        "message: {}",
        message
    );
}

#[test]
fn skipped_tests_reported() {
    let outcome = run_source(
        "hudunit_selftest_skip.hud",
        "fn ignore_test_x() {\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(outcome.skipped(), 1);
    assert_eq!(outcome.failed_count(), 0);
    assert!(outcome.exit_ok());
}

#[test]
fn filter_selects_subset() {
    let outcome = run_source(
        "hudunit_selftest_filter.hud",
        "fn test_a() {\n    assert_eq(1, 1);\n}\nfn test_b() {\n    assert_eq(1, 2);\n}\n",
        &RunOptions {
            filter: Some("test_a".into()),
            ..Default::default()
        },
    );
    assert_eq!(outcome.passed(), 1);
    assert_eq!(outcome.failed_count(), 0);
}

#[test]
fn setup_teardown_run_around_tests() {
    let outcome = run_source(
        "hudunit_selftest_setup.hud",
        "let counter = 0;\n\
         fn setup() {\n    counter = 1;\n}\n\
         fn teardown() {\n    counter = 0;\n}\n\
         fn test_with_setup() {\n    assert_eq(counter, 1);\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(outcome.passed(), 1, "setup should have set counter=1");
}

#[test]
fn coverage_marks_collected() {
    let outcome = run_source(
        "hudunit_selftest_cov.hud",
        "fn helper(x) {\n    return x * 2;\n}\n\
         fn test_helper() {\n    assert_eq(helper(2), 4);\n}\n",
        &RunOptions {
            coverage: true,
            ..Default::default()
        },
    );
    assert_eq!(outcome.passed(), 1);
    let coverage = outcome.coverage.expect("coverage data");
    assert!(coverage.overall_line_percent > 0.0);
    assert!(coverage
        .functions
        .iter()
        .any(|f| f.name == "helper" && f.covered));
}

#[test]
fn assert_throws_passes_on_exception() {
    let outcome = run_source(
        "hudunit_selftest_throws.hud",
        "fn patlayan() {\n    throw \"boom\";\n}\n\
         fn test_patlar() {\n    assert_throws(patlayan);\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(outcome.passed(), 1);
}

#[test]
fn assert_throws_fails_without_exception() {
    let outcome = run_source(
        "hudunit_selftest_throws2.hud",
        "fn sessiz() {\n    return 1;\n}\n\
         fn test_patlamali() {\n    assert_throws(sessiz);\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(outcome.failed_count(), 1);
    let message = match outcome
        .tests()
        .find(|t| t.name == "test_patlamali")
        .map(|t| &t.outcome)
    {
        Some(Outcome::Failed(message)) => message.clone(),
        _ => panic!("expected failure"),
    };
    assert!(message.contains("assert_throws failed"), "message: {}", message);
}

#[test]
fn optional_assertion_message_included() {
    let outcome = run_source(
        "hudunit_selftest_msg.hud",
        "fn test_mesajli() {\n    assert_eq(1, 2, \"toplama kontrolu\");\n}\n",
        &RunOptions::default(),
    );
    let message = match outcome
        .tests()
        .find(|t| t.name == "test_mesajli")
        .map(|t| &t.outcome)
    {
        Some(Outcome::Failed(message)) => message.clone(),
        _ => panic!("expected failure"),
    };
    assert!(
        message.contains("toplama kontrolu") && message.contains("assert_eq failed"),
        "message: {}",
        message
    );
}

#[test]
fn fail_fast_stops_at_first_failure() {
    let outcome = run_source(
        "hudunit_selftest_failfast.hud",
        "fn test_a() {\n    assert_eq(1, 2);\n}\n\
         fn test_b() {\n    assert_eq(1, 1);\n}\n\
         fn test_c() {\n    assert_eq(1, 1);\n}\n",
        &RunOptions {
            fail_fast: true,
            ..Default::default()
        },
    );
    assert_eq!(outcome.tests().count(), 1, "only the failing test runs");
    assert_eq!(outcome.failed_count(), 1);
    assert!(outcome.aborted);
    assert!(!outcome.exit_ok());

    // Without the flag the same file reports all three.
    let full = run_source(
        "hudunit_selftest_failfast2.hud",
        "fn test_a() {\n    assert_eq(1, 2);\n}\n\
         fn test_b() {\n    assert_eq(1, 1);\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(full.tests().count(), 2);
    assert!(!full.aborted);
}

#[test]
fn assertions_still_work_without_message() {
    // Backwards compatibility: the old two-argument form must keep the
    // plain message (missing arg arrives as null and is skipped).
    let outcome = run_source(
        "hudunit_selftest_nomsg.hud",
        "fn test_mesajsiz() {\n    assert_eq(1, 2);\n}\n",
        &RunOptions::default(),
    );
    let message = match outcome
        .tests()
        .find(|t| t.name == "test_mesajsiz")
        .map(|t| &t.outcome)
    {
        Some(Outcome::Failed(message)) => message.clone(),
        _ => panic!("expected failure"),
    };
    // The VM wraps thrown exceptions in its runtime-error envelope; the
    // plain assertion text must be present (message-prefix variants are
    // covered by optional_assertion_message_included).
    assert!(message.contains("assert_eq failed: expected 2, got 1"), "message: {}", message);
}

#[test]
fn slow_test_flagged_as_timeout() {
    let path = std::env::temp_dir().join("hudunit_selftest_timeout.hud");
    // Busy loop: deterministic slowness that safely exceeds the 1 ms
    // budget on any machine.
    std::fs::write(
        &path,
        "fn test_yavas() {\n    sleep(50);\n    assert_eq(1, 1);\n}\n",
    )
    .unwrap();
    let file = hudhudscript_hudunit::discover::discover_file(&path, None).unwrap();
    let cfg = HudunitConfig {
        timeout_ms: 1,
        ..HudunitConfig::default()
    };
    let outcome = run(&[file], &cfg, &RunOptions::default());
    std::fs::remove_file(&path).ok();
    let message = match outcome
        .tests()
        .find(|t| t.name == "test_yavas")
        .map(|t| &t.outcome)
    {
        Some(Outcome::Failed(message)) => message.clone(),
        _ => panic!("expected timeout failure"),
    };
    assert!(message.contains("timeout"), "message: {}", message);
}

#[test]
fn test_stdout_is_captured() {
    let outcome = run_source(
        "hudunit_selftest_print.hud",
        "fn test_yazdirir() {\n    print(\"hudunit-print-probe\");\n    assert_eq(1, 1);\n}\n",
        &RunOptions::default(),
    );
    assert_eq!(outcome.passed(), 1);
    let test = outcome
        .tests()
        .find(|t| t.name == "test_yazdirir")
        .unwrap();
    assert!(
        test.output.as_deref().unwrap_or("").contains("hudunit-print-probe"),
        "captured: {:?}",
        test.output
    );
}
