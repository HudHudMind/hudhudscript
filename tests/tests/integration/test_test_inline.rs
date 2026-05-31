// Extracted from hudhudscript-test inline #[cfg(test)] blocks
use hudhudscript_test::{
    Assertion, AssertionError, CoverageReport, EventMock, FileCoverage, FunctionMock, MockCall,
    MockEvent, ProviderMock, RunResult, SuiteResult, Test, TestFilter, TestFramework, TestResult,
    TestRunner, TestSuite, Verbosity,
};
use std::time::Duration;

// === from assertion.rs ===

#[test]
fn test_equal() {
    assert!(Assertion::equal(1, 1).is_ok());
    assert!(Assertion::equal(1, 2).is_err());
}

#[test]
fn test_not_equal() {
    assert!(Assertion::not_equal(1, 2).is_ok());
    assert!(Assertion::not_equal(1, 1).is_err());
}

#[test]
fn test_is_true() {
    assert!(Assertion::is_true(true).is_ok());
    assert!(Assertion::is_true(false).is_err());
}

#[test]
fn test_is_false() {
    assert!(Assertion::is_false(false).is_ok());
    assert!(Assertion::is_false(true).is_err());
}

#[test]
fn test_is_none_and_is_some() {
    let none: Option<i32> = None;
    let some: Option<i32> = Some(42);
    assert!(Assertion::is_none(&none).is_ok());
    assert!(Assertion::is_none(&some).is_err());
    assert!(Assertion::is_some(&some).is_ok());
    assert!(Assertion::is_some(&none).is_err());
}

#[test]
fn test_ordering() {
    assert!(Assertion::greater_than(3, 2).is_ok());
    assert!(Assertion::greater_than(2, 3).is_err());
    assert!(Assertion::less_than(1, 5).is_ok());
    assert!(Assertion::less_than(5, 1).is_err());
    assert!(Assertion::greater_than_or_equal(3, 3).is_ok());
    assert!(Assertion::less_than_or_equal(3, 3).is_ok());
}

#[test]
fn test_contains() {
    assert!(Assertion::contains("hello world", "world").is_ok());
    assert!(Assertion::contains("hello world", "xyz").is_err());
    assert!(Assertion::not_contains("hello", "xyz").is_ok());
    assert!(Assertion::not_contains("hello", "ell").is_err());
}

#[test]
fn test_starts_ends_with() {
    assert!(Assertion::starts_with("hello world", "hello").is_ok());
    assert!(Assertion::starts_with("hello world", "world").is_err());
    assert!(Assertion::ends_with("hello world", "world").is_ok());
    assert!(Assertion::ends_with("hello world", "hello").is_err());
}

#[test]
fn test_collection_contains() {
    let v = vec![1, 2, 3];
    assert!(Assertion::collection_contains(&v, &2).is_ok());
    assert!(Assertion::collection_contains(&v, &5).is_err());
}

#[test]
fn test_throws() {
    let ok: Result<i32, String> = Ok(42);
    let err: Result<i32, String> = Err("boom".to_string());
    assert!(Assertion::throws(&err).is_ok());
    assert!(Assertion::throws(&ok).is_err());
}

#[test]
fn test_throws_with_message() {
    let err: Result<i32, String> = Err("something went wrong".to_string());
    assert!(Assertion::throws_with_message(&err, "went wrong").is_ok());
    assert!(Assertion::throws_with_message(&err, "not found").is_err());
}

#[test]
fn test_has_length() {
    let v = vec![1, 2, 3];
    assert!(Assertion::has_length(&v, 3).is_ok());
    assert!(Assertion::has_length(&v, 2).is_err());
}

#[test]
fn test_is_empty_and_not_empty() {
    let empty: Vec<i32> = vec![];
    let non_empty = vec![1];
    assert!(Assertion::is_empty(&empty).is_ok());
    assert!(Assertion::is_empty(&non_empty).is_err());
    assert!(Assertion::is_not_empty(&non_empty).is_ok());
    assert!(Assertion::is_not_empty(&empty).is_err());
}

#[test]
fn test_approx_equal() {
    assert!(Assertion::approx_equal(1.0, 1.0001, 0.001).is_ok());
    assert!(Assertion::approx_equal(1.0, 2.0, 0.001).is_err());
}

#[test]
fn test_error_with_label_and_location() {
    let err = AssertionError::new("bad value")
        .with_label("my test")
        .with_location("test.hud:42");
    assert!(err.display_message().contains("my test"));
    assert!(err.display_message().contains("test.hud:42"));
}

#[test]
fn test_matches_pattern() {
    assert!(Assertion::matches_pattern("hello world", "world").is_ok());
    assert!(Assertion::matches_pattern("hello world", "xyz").is_err());
}

#[test]
fn test_error_display_no_label_no_location() {
    let err = AssertionError::new("simple error");
    let msg = err.display_message();
    assert_eq!(msg, "Assertion failed: simple error");
}

#[test]
fn test_error_display_with_location_only() {
    let err = AssertionError::new("err").with_location("file:10");
    let msg = err.display_message();
    assert!(msg.starts_with("at file:10"));
    assert!(msg.contains("Assertion failed"));
}

#[test]
fn test_error_display_with_label_only() {
    let err = AssertionError::new("err").with_label("my label");
    let msg = err.display_message();
    assert!(msg.contains("[my label]"));
}

#[test]
fn test_throws_with_message_ok_returns_err() {
    let ok: Result<i32, String> = Ok(42);
    assert!(Assertion::throws_with_message(&ok, "anything").is_err());
}

#[test]
fn test_greater_than_or_equal_less() {
    assert!(Assertion::greater_than_or_equal(2, 3).is_err());
}

#[test]
fn test_less_than_or_equal_greater() {
    assert!(Assertion::less_than_or_equal(5, 3).is_err());
}

#[test]
fn test_collection_contains_empty() {
    let v: Vec<i32> = vec![];
    assert!(Assertion::collection_contains(&v, &1).is_err());
}

#[test]
fn test_assertion_error_std_display() {
    let err = AssertionError::new("test message");
    assert_eq!(format!("{}", err), "Assertion failed: test message");
}

// === from coverage.rs ===

#[test]
fn test_file_coverage_percent() {
    let mut fc = FileCoverage::new("main.hud");
    fc.total_lines = 100;
    fc.covered_lines = 75;
    assert!((fc.line_coverage_percent() - 75.0).abs() < f64::EPSILON);
}

#[test]
fn test_file_coverage_zero_lines() {
    let fc = FileCoverage::new("empty.hud");
    assert!((fc.line_coverage_percent() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_record_line_hit() {
    let mut fc = FileCoverage::new("test.hud");
    fc.total_lines = 10;
    fc.record_line_hit(1);
    fc.record_line_hit(1);
    fc.record_line_hit(3);
    assert_eq!(fc.covered_lines, 2);
    assert_eq!(fc.line_hits[&1], 2);
    assert_eq!(fc.line_hits[&3], 1);
}

#[test]
fn test_coverage_report_aggregation() {
    let mut report = CoverageReport::new();

    let mut f1 = FileCoverage::new("a.hud");
    f1.total_lines = 50;
    f1.covered_lines = 40;
    f1.total_functions = 5;
    f1.covered_functions = 4;

    let mut f2 = FileCoverage::new("b.hud");
    f2.total_lines = 50;
    f2.covered_lines = 30;
    f2.total_functions = 5;
    f2.covered_functions = 3;

    report.add_file(f1);
    report.add_file(f2);

    assert_eq!(report.total_lines(), 100);
    assert_eq!(report.covered_lines(), 70);
    assert!((report.overall_line_coverage() - 70.0).abs() < f64::EPSILON);
    assert_eq!(report.total_functions(), 10);
    assert_eq!(report.covered_functions(), 7);
}

#[test]
fn test_threshold_pass() {
    let mut report = CoverageReport::new().with_threshold(60.0);
    let mut f = FileCoverage::new("ok.hud");
    f.total_lines = 100;
    f.covered_lines = 80;
    report.add_file(f);
    assert!(report.check_threshold().is_ok());
}

#[test]
fn test_threshold_fail() {
    let mut report = CoverageReport::new().with_threshold(90.0);
    let mut f = FileCoverage::new("low.hud");
    f.total_lines = 100;
    f.covered_lines = 50;
    report.add_file(f);
    assert!(report.check_threshold().is_err());
}

#[test]
fn test_function_coverage_percent() {
    let mut fc = FileCoverage::new("f.hud");
    fc.total_functions = 10;
    fc.covered_functions = 7;
    assert!((fc.function_coverage_percent() - 70.0).abs() < f64::EPSILON);
}

#[test]
fn test_function_coverage_percent_zero() {
    let fc = FileCoverage::new("empty.hud");
    assert!((fc.function_coverage_percent() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_overall_function_coverage() {
    let mut report = CoverageReport::new();
    let mut f1 = FileCoverage::new("a.hud");
    f1.total_functions = 4;
    f1.covered_functions = 2;
    let mut f2 = FileCoverage::new("b.hud");
    f2.total_functions = 6;
    f2.covered_functions = 3;
    report.add_file(f1);
    report.add_file(f2);
    // 5/10 = 50.0%
    assert!((report.overall_function_coverage() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_overall_function_coverage_empty() {
    let report = CoverageReport::new();
    assert!((report.overall_function_coverage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_overall_line_coverage_empty() {
    let report = CoverageReport::new();
    assert!((report.overall_line_coverage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_check_threshold_no_threshold() {
    let report = CoverageReport::new();
    assert!(report.check_threshold().is_ok());
}

#[test]
fn test_summary_with_fail_threshold() {
    let mut report = CoverageReport::new().with_threshold(95.0);
    let mut f = FileCoverage::new("low.hud");
    f.total_lines = 100;
    f.covered_lines = 50;
    f.total_functions = 5;
    f.covered_functions = 2;
    report.add_file(f);
    let s = report.summary();
    assert!(s.contains("FAIL"));
    assert!(s.contains("50.0% lines"));
}

#[test]
fn test_coverage_report_default() {
    let report = CoverageReport::default();
    assert!(report.files.is_empty());
    assert!(report.threshold.is_none());
}

#[test]
fn test_record_line_hit_multiple_lines() {
    let mut fc = FileCoverage::new("test.hud");
    fc.total_lines = 5;
    fc.record_line_hit(1);
    fc.record_line_hit(2);
    fc.record_line_hit(3);
    fc.record_line_hit(1); // duplicate
    assert_eq!(fc.covered_lines, 3);
    assert_eq!(fc.line_hits[&1], 2);
}

#[test]
fn test_summary_format() {
    let mut report = CoverageReport::new().with_threshold(50.0);
    let mut f = FileCoverage::new("x.hud");
    f.total_lines = 10;
    f.covered_lines = 8;
    f.total_functions = 2;
    f.covered_functions = 2;
    report.add_file(f);

    let s = report.summary();
    assert!(s.contains("80.0% lines"));
    assert!(s.contains("PASS"));
}

// === from framework.rs ===

#[test]
fn test_framework() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("test_suite".to_string());
    suite.add_test(Test::new("test1".to_string()));
    framework.add_suite(suite);

    assert_eq!(framework.suite_count(), 1);
    assert_eq!(framework.test_count(), 1);
}

#[test]
fn test_with_body_passes() {
    let test = Test::with_body("passing".to_string(), || Ok(()));
    let mut suite = TestSuite::new("s".to_string());
    suite.add_test(test);
    suite.run();
    assert_eq!(suite.tests[0].result, Some(TestResult::Passed));
    assert!(suite.tests[0].duration.is_some());
}

#[test]
fn test_with_body_fails() {
    let test = Test::with_body("failing".to_string(), || Err("boom".to_string()));
    let mut suite = TestSuite::new("s".to_string());
    suite.add_test(test);
    suite.run();
    assert_eq!(
        suite.tests[0].result,
        Some(TestResult::Failed("boom".to_string()))
    );
}

#[test]
fn test_without_body_becomes_skipped() {
    let test = Test::new("no_body".to_string());
    let mut suite = TestSuite::new("s".to_string());
    suite.add_test(test);
    suite.run();
    assert_eq!(suite.tests[0].result, Some(TestResult::Skipped));
}

#[test]
fn test_run_all_aggregates_counts() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(Test::with_body("pass".to_string(), || Ok(())));
    suite.add_test(Test::with_body("fail".to_string(), || {
        Err("oops".to_string())
    }));
    suite.add_test(Test::new("skip".to_string()));
    framework.add_suite(suite);

    let (passed, failed, skipped) = framework.run_all();
    assert_eq!(passed, 1);
    assert_eq!(failed, 1);
    assert_eq!(skipped, 1);
}

#[test]
fn test_tags() {
    let test = Test::with_body("tagged".to_string(), || Ok(()))
        .with_tags(vec!["slow".to_string(), "integration".to_string()]);
    assert!(test.has_tag("slow"));
    assert!(test.has_tag("integration"));
    assert!(!test.has_tag("fast"));
}

#[test]
fn test_before_each_after_each_hooks() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicUsize::new(0));
    let c1 = counter.clone();
    let c2 = counter.clone();

    let mut suite = TestSuite::new("hooks".to_string())
        .with_before_each(move || {
            c1.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .with_after_each(move || {
            c2.fetch_add(10, Ordering::SeqCst);
            Ok(())
        });

    suite.add_test(Test::with_body("a".to_string(), || Ok(())));
    suite.add_test(Test::with_body("b".to_string(), || Ok(())));
    suite.run();

    // 2 before_each (2*1) + 2 after_each (2*10) = 22
    assert_eq!(counter.load(Ordering::SeqCst), 22);
    assert_eq!(suite.tests[0].result, Some(TestResult::Passed));
    assert_eq!(suite.tests[1].result, Some(TestResult::Passed));
}

#[test]
fn test_before_all_failure_marks_all_tests_failed() {
    let mut suite =
        TestSuite::new("fail_all".to_string()).with_before_all(|| Err("setup failed".to_string()));

    suite.add_test(Test::with_body("a".to_string(), || Ok(())));
    suite.add_test(Test::with_body("b".to_string(), || Ok(())));
    suite.run();

    for t in &suite.tests {
        match &t.result {
            Some(TestResult::Failed(msg)) => {
                assert!(msg.contains("before_all hook failed"));
            }
            other => panic!("expected Failed, got {:?}", other),
        }
    }
}

#[test]
fn test_suite_duration_is_recorded() {
    let mut suite = TestSuite::new("timed".to_string());
    suite.add_test(Test::with_body("a".to_string(), || Ok(())));
    suite.run();
    assert!(suite.duration.is_some());
}

#[test]
fn test_before_each_failure_marks_test_failed() {
    let mut suite = TestSuite::new("before_each_fail".to_string())
        .with_before_each(|| Err("setup error".to_string()));

    suite.add_test(Test::with_body("a".to_string(), || Ok(())));
    suite.run();

    match &suite.tests[0].result {
        Some(TestResult::Failed(msg)) => {
            assert!(msg.contains("before_each hook failed"));
        }
        other => panic!("expected Failed, got {:?}", other),
    }
}

#[test]
fn test_after_each_failure_marks_passed_test_as_failed() {
    let mut suite = TestSuite::new("after_each_fail".to_string())
        .with_after_each(|| Err("teardown error".to_string()));

    suite.add_test(Test::with_body("a".to_string(), || Ok(())));
    suite.run();

    match &suite.tests[0].result {
        Some(TestResult::Failed(msg)) => {
            assert!(msg.contains("after_each hook failed"));
        }
        other => panic!("expected Failed, got {:?}", other),
    }
}

#[test]
fn test_after_each_failure_does_not_override_test_failure() {
    let mut suite = TestSuite::new("both_fail".to_string())
        .with_after_each(|| Err("teardown error".to_string()));

    suite.add_test(Test::with_body("a".to_string(), || {
        Err("test error".to_string())
    }));
    suite.run();

    // Test was already failed, after_each failure should not override
    match &suite.tests[0].result {
        Some(TestResult::Failed(msg)) => {
            assert!(msg.contains("test error"));
        }
        other => panic!("expected Failed with test error, got {:?}", other),
    }
}

#[test]
fn test_preset_result_not_overridden_by_run() {
    let mut test = Test::with_body("pre_skipped".to_string(), || Ok(()));
    test.result = Some(TestResult::Skipped);
    let mut suite = TestSuite::new("s".to_string());
    suite.add_test(test);
    suite.run();
    // The body should not execute since result was pre-set
    assert_eq!(suite.tests[0].result, Some(TestResult::Skipped));
}

#[test]
fn test_after_all_hook_runs() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let mut suite = TestSuite::new("after_all".to_string()).with_after_all(move || {
        called_clone.store(true, Ordering::SeqCst);
        Ok(())
    });

    suite.add_test(Test::with_body("a".to_string(), || Ok(())));
    suite.run();

    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_framework_default() {
    let framework = TestFramework::default();
    assert_eq!(framework.suite_count(), 0);
    assert_eq!(framework.test_count(), 0);
}

#[test]
fn test_debug_output() {
    let test = Test::with_body("dbg_test".to_string(), || Ok(()));
    let debug_str = format!("{:?}", test);
    assert!(debug_str.contains("dbg_test"));

    let suite = TestSuite::new("dbg_suite".to_string());
    let debug_str2 = format!("{:?}", suite);
    assert!(debug_str2.contains("dbg_suite"));
}

// === from mock.rs ===

#[test]
fn test_function_mock_basic() {
    let mock = FunctionMock::new("add");
    mock.returns("42");
    let ret = mock.call(vec!["20".to_string(), "22".to_string()]);
    assert_eq!(ret, Some("42".to_string()));
    assert!(mock.was_called());
    assert!(mock.was_called_times(1));
}

#[test]
fn test_function_mock_no_return() {
    let mock = FunctionMock::new("log");
    let ret = mock.call(vec!["hello".to_string()]);
    assert_eq!(ret, None);
    assert_eq!(mock.call_count(), 1);
}

#[test]
fn test_function_mock_assert_called_with() {
    let mock = FunctionMock::new("greet");
    mock.call(vec!["Alice".to_string()]);
    assert!(mock.assert_called_with(&["Alice"]).is_ok());
    assert!(mock.assert_called_with(&["Bob"]).is_err());
}

#[test]
fn test_function_mock_reset() {
    let mock = FunctionMock::new("f");
    mock.returns("x");
    mock.call(vec![]);
    assert!(mock.was_called());
    mock.reset();
    assert!(!mock.was_called());
}

#[test]
fn test_provider_mock_basic() {
    let provider = ProviderMock::new("db");
    provider.set("user:1", r#"{"name":"Alice"}"#);
    assert_eq!(
        provider.get("user:1"),
        Some(r#"{"name":"Alice"}"#.to_string())
    );
    assert_eq!(provider.get("user:2"), None);
    assert_eq!(provider.access_count(), 2);
}

#[test]
fn test_provider_mock_set_many() {
    let provider = ProviderMock::new("cache");
    provider.set_many(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
    ]);
    assert_eq!(provider.get("a"), Some("1".to_string()));
    assert_eq!(provider.get("b"), Some("2".to_string()));
}

#[test]
fn test_provider_mock_reset() {
    let provider = ProviderMock::new("p");
    provider.set("k", "v");
    provider.get("k");
    provider.reset();
    assert_eq!(provider.get("k"), None);
    // access_log was cleared so only the new access is recorded.
    assert_eq!(provider.access_count(), 1);
}

#[test]
fn test_event_mock_basic() {
    let events = EventMock::new();
    events.emit("click", Some("button_1".to_string()));
    events.emit("click", None);
    events.emit("hover", None);

    assert_eq!(events.count(), 3);
    assert_eq!(events.count_by_name("click"), 2);
    assert!(events.has_event("click"));
    assert!(!events.has_event("scroll"));
}

#[test]
fn test_event_mock_assert_emitted() {
    let events = EventMock::new();
    events.emit("load", None);
    assert!(events.assert_emitted("load").is_ok());
    assert!(events.assert_emitted("unload").is_err());
}

#[test]
fn test_event_mock_assert_emitted_times() {
    let events = EventMock::new();
    events.emit("tick", None);
    events.emit("tick", None);
    assert!(events.assert_emitted_times("tick", 2).is_ok());
    assert!(events.assert_emitted_times("tick", 1).is_err());
}

#[test]
fn test_event_mock_reset() {
    let events = EventMock::new();
    events.emit("x", None);
    events.reset();
    assert_eq!(events.count(), 0);
}

#[test]
fn test_function_mock_debug() {
    let mock = FunctionMock::new("my_func");
    mock.call(vec!["a".to_string()]);
    let debug_str = format!("{:?}", mock);
    assert!(debug_str.contains("my_func"));
    assert!(debug_str.contains("1")); // call_count
}

#[test]
fn test_function_mock_name() {
    let mock = FunctionMock::new("greet");
    assert_eq!(mock.name(), "greet");
}

#[test]
fn test_function_mock_calls_snapshot() {
    let mock = FunctionMock::new("f");
    mock.call(vec!["x".to_string()]);
    mock.call(vec!["y".to_string()]);
    let calls = mock.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].args, vec!["x"]);
    assert_eq!(calls[1].args, vec!["y"]);
}

#[test]
fn test_function_mock_was_called_times() {
    let mock = FunctionMock::new("f");
    assert!(mock.was_called_times(0));
    mock.call(vec![]);
    mock.call(vec![]);
    assert!(mock.was_called_times(2));
    assert!(!mock.was_called_times(1));
}

#[test]
fn test_provider_mock_name() {
    let provider = ProviderMock::new("my_provider");
    assert_eq!(provider.name(), "my_provider");
}

#[test]
fn test_provider_mock_access_log() {
    let provider = ProviderMock::new("db");
    provider.set("k1", "v1");
    provider.get("k1");
    provider.get("k2");
    let log = provider.access_log();
    assert_eq!(log, vec!["k1", "k2"]);
}

#[test]
fn test_event_mock_default() {
    let events = EventMock::default();
    assert_eq!(events.count(), 0);
}

#[test]
fn test_event_mock_events_snapshot() {
    let events = EventMock::new();
    events.emit("click", Some("btn".to_string()));
    events.emit("hover", None);
    let snapshot = events.events();
    assert_eq!(snapshot.len(), 2);
    assert_eq!(snapshot[0].name, "click");
    assert_eq!(snapshot[0].payload, Some("btn".to_string()));
    assert_eq!(snapshot[1].name, "hover");
    assert_eq!(snapshot[1].payload, None);
}

#[test]
fn test_function_mock_clone() {
    let mock = FunctionMock::new("f");
    mock.returns("42");
    mock.call(vec!["a".to_string()]);
    let clone = mock.clone();
    // Clone shares the same Arc, so calls are shared
    assert_eq!(clone.call_count(), 1);
    assert_eq!(clone.name(), "f");
}

// === from runner.rs ===

#[test]
fn test_runner_counts_preset_results() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("test".to_string());
    let mut test = Test::new("test1".to_string());
    test.result = Some(TestResult::Passed);
    suite.add_test(test);
    framework.add_suite(suite);

    let mut runner = TestRunner::new(framework);
    let result = runner.run();
    assert_eq!(result.passed, 1);
    assert_eq!(result.failed, 0);
    assert_eq!(result.skipped, 0);
}

#[test]
fn test_runner_executes_bodies() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(Test::with_body("pass".to_string(), || Ok(())));
    suite.add_test(Test::with_body("fail".to_string(), || {
        Err("err".to_string())
    }));
    suite.add_test(Test::new("skip".to_string()));
    framework.add_suite(suite);

    let mut runner = TestRunner::new(framework);
    let result = runner.run();
    assert_eq!(result.passed, 1);
    assert_eq!(result.failed, 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn test_filter_by_name() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(Test::with_body("alpha_test".to_string(), || Ok(())));
    suite.add_test(Test::with_body("beta_test".to_string(), || Ok(())));
    framework.add_suite(suite);

    let filter = TestFilter::new().with_name("alpha");
    let mut runner = TestRunner::new(framework).with_filter(filter);
    let result = runner.run();
    assert_eq!(result.passed, 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn test_filter_by_tag() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(
        Test::with_body("fast".to_string(), || Ok(())).with_tags(vec!["fast".to_string()]),
    );
    suite.add_test(
        Test::with_body("slow".to_string(), || Ok(())).with_tags(vec!["slow".to_string()]),
    );
    framework.add_suite(suite);

    let filter = TestFilter::new().with_exclude_tags(vec!["slow".to_string()]);
    let mut runner = TestRunner::new(framework).with_filter(filter);
    let result = runner.run();
    assert_eq!(result.passed, 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn test_run_result_summary() {
    let result = RunResult {
        passed: 5,
        failed: 1,
        skipped: 2,
        total_duration: Some(Duration::from_millis(123)),
        suite_results: Vec::new(),
    };
    let s = result.summary();
    assert!(s.contains("8 tests"));
    assert!(s.contains("5 passed"));
    assert!(s.contains("1 failed"));
    assert!(s.contains("2 skipped"));
}

#[test]
fn test_verbose_output() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(Test::with_body("ok".to_string(), || Ok(())));
    framework.add_suite(suite);

    let mut runner = TestRunner::new(framework).with_verbosity(Verbosity::Verbose);
    runner.run();
    let output = runner.output().join("\n");
    assert!(output.contains("PASS ok"));
}

#[test]
fn test_quiet_output() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(Test::with_body("ok".to_string(), || Ok(())));
    framework.add_suite(suite);

    let mut runner = TestRunner::new(framework).with_verbosity(Verbosity::Quiet);
    runner.run();
    assert!(runner.output().is_empty());
}

#[test]
fn test_suite_filter() {
    let mut framework = TestFramework::new();
    let mut s1 = TestSuite::new("auth_suite".to_string());
    s1.add_test(Test::with_body("t1".to_string(), || Ok(())));
    let mut s2 = TestSuite::new("math_suite".to_string());
    s2.add_test(Test::with_body("t2".to_string(), || Ok(())));
    framework.add_suite(s1);
    framework.add_suite(s2);

    let filter = TestFilter::new().with_suite("auth");
    let mut runner = TestRunner::new(framework).with_filter(filter);
    let result = runner.run();
    assert_eq!(result.passed, 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn test_suite_results_populated() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("s".to_string());
    suite.add_test(Test::with_body("a".to_string(), || Ok(())));
    suite.add_test(Test::with_body("b".to_string(), || Err("x".to_string())));
    framework.add_suite(suite);

    let mut runner = TestRunner::new(framework);
    let result = runner.run();
    assert_eq!(result.suite_results.len(), 1);
    assert_eq!(result.suite_results[0].passed, 1);
    assert_eq!(result.suite_results[0].failed, 1);
    assert_eq!(result.suite_results[0].failures.len(), 1);
}

#[test]
fn test_count_results() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("s".to_string());
    let mut t1 = Test::new("t1".to_string());
    t1.result = Some(TestResult::Passed);
    let mut t2 = Test::new("t2".to_string());
    t2.result = Some(TestResult::Failed("err".to_string()));
    let t3 = Test::new("t3".to_string()); // no result → skipped
    suite.add_test(t1);
    suite.add_test(t2);
    suite.add_test(t3);
    framework.add_suite(suite);

    let runner = TestRunner::new(framework);
    let result = runner.count_results();
    assert_eq!(result.passed, 1);
    assert_eq!(result.failed, 1);
    assert_eq!(result.skipped, 1);
    assert_eq!(result.total(), 3);
    assert!(result.total_duration.is_none());
}

#[test]
fn test_filter_by_include_tags() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(
        Test::with_body("integration".to_string(), || Ok(()))
            .with_tags(vec!["integration".to_string()]),
    );
    suite.add_test(
        Test::with_body("unit".to_string(), || Ok(())).with_tags(vec!["unit".to_string()]),
    );
    framework.add_suite(suite);

    let filter = TestFilter::new().with_include_tags(vec!["integration".to_string()]);
    let mut runner = TestRunner::new(framework).with_filter(filter);
    let result = runner.run();
    assert_eq!(result.passed, 1);
    assert_eq!(result.skipped, 1);
}

#[test]
fn test_run_result_summary_no_duration() {
    let result = RunResult {
        passed: 3,
        failed: 0,
        skipped: 0,
        total_duration: None,
        suite_results: Vec::new(),
    };
    let s = result.summary();
    assert!(s.contains("3 tests"));
    // No duration portion
    assert!(!s.contains("in "));
}

#[test]
fn test_verbosity_default() {
    let v = Verbosity::default();
    assert_eq!(v, Verbosity::Normal);
}

#[test]
fn test_normal_output_shows_fail() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("s".to_string());
    suite.add_test(Test::with_body("bad".to_string(), || {
        Err("broken".to_string())
    }));
    framework.add_suite(suite);

    let mut runner = TestRunner::new(framework).with_verbosity(Verbosity::Normal);
    runner.run();
    let output = runner.output().join("\n");
    assert!(output.contains("FAIL bad: broken"));
}

#[test]
fn test_verbose_shows_skip() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("s".to_string());
    suite.add_test(Test::new("skipped_test".to_string()));
    framework.add_suite(suite);

    let mut runner = TestRunner::new(framework).with_verbosity(Verbosity::Verbose);
    runner.run();
    let output = runner.output().join("\n");
    assert!(output.contains("SKIP skipped_test"));
}

#[test]
fn test_is_success() {
    let ok = RunResult {
        passed: 3,
        failed: 0,
        skipped: 1,
        total_duration: None,
        suite_results: Vec::new(),
    };
    assert!(ok.is_success());

    let bad = RunResult {
        passed: 2,
        failed: 1,
        skipped: 0,
        total_duration: None,
        suite_results: Vec::new(),
    };
    assert!(!bad.is_success());
}

// === from lib.rs ===

#[test]
fn test_framework_creation() {
    let framework = TestFramework::new();
    assert_eq!(framework.suite_count(), 0);
}

#[test]
fn test_assertion_lib() {
    let result = Assertion::equal(42, 42);
    assert!(result.is_ok());
}

#[test]
fn test_framework_add_and_run() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("s1".to_string());
    suite.add_test(Test::with_body("pass".to_string(), || Ok(())));
    suite.add_test(Test::with_body("fail".to_string(), || {
        Err("boom".to_string())
    }));
    suite.add_test(Test::new("skip".to_string()));
    framework.add_suite(suite);

    let (p, f, s) = framework.run_all();
    assert_eq!(p, 1);
    assert_eq!(f, 1);
    assert_eq!(s, 1);
}

#[test]
fn test_runner_with_filter_and_verbosity() {
    let mut framework = TestFramework::new();
    let mut suite = TestSuite::new("suite".to_string());
    suite.add_test(
        Test::with_body("alpha".to_string(), || Ok(())).with_tags(vec!["fast".to_string()]),
    );
    suite.add_test(
        Test::with_body("beta".to_string(), || Ok(())).with_tags(vec!["slow".to_string()]),
    );
    framework.add_suite(suite);

    let filter = TestFilter::new().with_exclude_tags(vec!["slow".to_string()]);
    let mut runner = TestRunner::new(framework)
        .with_filter(filter)
        .with_verbosity(Verbosity::Verbose);
    let result = runner.run();
    assert_eq!(result.passed, 1);
    assert_eq!(result.skipped, 1);
    assert!(result.is_success());
    let output = runner.output().join("\n");
    assert!(output.contains("PASS alpha"));
    assert!(output.contains("SKIP beta"));
}

#[test]
fn test_coverage_report_basic() {
    let mut report = CoverageReport::new().with_threshold(50.0);
    let mut fc = FileCoverage::new("test.hud");
    fc.total_lines = 100;
    fc.covered_lines = 80;
    fc.total_functions = 10;
    fc.covered_functions = 8;
    report.add_file(fc);

    assert!((report.overall_line_coverage() - 80.0).abs() < f64::EPSILON);
    assert!(report.check_threshold().is_ok());
    let summary = report.summary();
    assert!(summary.contains("80.0% lines"));
    assert!(summary.contains("PASS"));
}

#[test]
fn test_mock_store_and_recall() {
    let func_mock = FunctionMock::new("test_func");
    func_mock.returns("42");
    let ret = func_mock.call(vec!["arg1".to_string()]);
    assert_eq!(ret, Some("42".to_string()));
    assert!(func_mock.was_called());

    let provider = ProviderMock::new("db");
    provider.set("key", "value");
    assert_eq!(provider.get("key"), Some("value".to_string()));

    let events = EventMock::new();
    events.emit("click", None);
    assert!(events.has_event("click"));
    assert!(events.assert_emitted("click").is_ok());
}

#[test]
fn test_run_result_total_and_success() {
    let result = RunResult {
        passed: 5,
        failed: 0,
        skipped: 2,
        total_duration: None,
        suite_results: Vec::new(),
    };
    assert_eq!(result.total(), 7);
    assert!(result.is_success());
}
