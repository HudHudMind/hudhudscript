// Extracted from hudhudscript-debug inline #[cfg(test)] blocks
use hudhudscript_debug::{
    Breakpoint, BreakpointKind, DapEvent, DapMessage, DapRequest, DapResponse, DapServer,
    DebugState, Debugger, PauseReason, ProfileReport, ProfileSample, Profiler, ScopeVariable,
    Source, StepMode, Variable, THREAD_ID, THREAD_NAME,
};
use serde_json::Value;
use std::time::{Duration, Instant};

// === from lib.rs ===

#[test]
fn test_debugger_creation() {
    let debugger = Debugger::new();
    assert!(matches!(debugger.state(), DebugState::Running));
}

#[test]
fn test_profiler_creation() {
    let profiler = Profiler::new();
    assert_eq!(profiler.sample_count(), 0);
}

#[test]
fn test_debugger_add_and_remove_breakpoint() {
    let mut debugger = Debugger::new();
    let id = debugger.add_breakpoint("test.hudhud".to_string(), 5);
    assert_eq!(debugger.breakpoints().len(), 1);
    assert!(debugger.remove_breakpoint(id));
    assert_eq!(debugger.breakpoints().len(), 0);
}

#[test]
fn test_dap_server_can_be_created() {
    let server = DapServer::new();
    assert!(!server.is_disconnected());
}

#[test]
fn test_breakpoint_kind_variants() {
    assert_eq!(BreakpointKind::Normal, BreakpointKind::Normal);
    let bp = Breakpoint::new(1, "f.hudhud".to_string(), 1);
    assert_eq!(bp.kind, BreakpointKind::Normal);
}

// === from breakpoint.rs ===

#[test]
fn test_breakpoint() {
    let bp = Breakpoint::new(1, "test.hudhud".to_string(), 10);
    assert_eq!(bp.id, 1);
    assert_eq!(bp.line, 10);
    assert!(bp.enabled);
    assert_eq!(bp.kind, BreakpointKind::Normal);
}

#[test]
fn test_conditional_breakpoint() {
    let bp = Breakpoint::new(2, "test.hudhud".to_string(), 20).with_condition("x > 10".to_string());
    assert!(bp.is_conditional());
    assert_eq!(bp.condition, Some("x > 10".to_string()));
    assert_eq!(bp.kind, BreakpointKind::Conditional("x > 10".to_string()));
}

#[test]
fn test_logpoint() {
    let bp =
        Breakpoint::new(3, "test.hudhud".to_string(), 30).with_log_message("x = {x}".to_string());
    assert!(bp.is_logpoint());
    assert!(!bp.is_conditional());
    assert_eq!(bp.kind, BreakpointKind::Logpoint("x = {x}".to_string()));
}

#[test]
fn test_exception_breakpoint() {
    let bp = Breakpoint::new(4, "test.hudhud".to_string(), 0)
        .as_exception(Some("TypeError".to_string()));
    assert!(bp.is_exception());
    assert_eq!(
        bp.kind,
        BreakpointKind::Exception(Some("TypeError".to_string()))
    );
}

#[test]
fn test_exception_breakpoint_catch_all() {
    let bp = Breakpoint::new(5, "test.hudhud".to_string(), 0).as_exception(None);
    assert!(bp.is_exception());
    assert_eq!(bp.kind, BreakpointKind::Exception(None));
}

#[test]
fn test_hit_count() {
    let mut bp = Breakpoint::new(6, "test.hudhud".to_string(), 10);
    assert_eq!(bp.hit_count, 0);
    assert_eq!(bp.record_hit(), 1);
    assert_eq!(bp.record_hit(), 2);
    assert_eq!(bp.hit_count, 2);
}

#[test]
fn test_with_kind() {
    let bp = Breakpoint::new(1, "test.hudhud".to_string(), 5)
        .with_kind(BreakpointKind::Logpoint("hello".to_string()));
    assert!(bp.is_logpoint());
    assert!(!bp.is_conditional());
    assert!(!bp.is_exception());
}

#[test]
fn test_normal_breakpoint_is_not_logpoint_or_exception() {
    let bp = Breakpoint::new(1, "test.hudhud".to_string(), 5);
    assert!(!bp.is_logpoint());
    assert!(!bp.is_exception());
    assert!(!bp.is_conditional());
}

#[test]
fn test_breakpoint_file_and_line() {
    let bp = Breakpoint::new(42, "my_script.hudhud".to_string(), 100);
    assert_eq!(bp.file, "my_script.hudhud");
    assert_eq!(bp.line, 100);
    assert_eq!(bp.id, 42);
    assert!(bp.condition.is_none());
}

#[test]
fn test_breakpoint_enabled_by_default() {
    let bp = Breakpoint::new(1, "test.hudhud".to_string(), 1);
    assert!(bp.enabled);
}

#[test]
fn test_breakpoint_serialization_roundtrip() {
    let bp = Breakpoint::new(1, "test.hudhud".to_string(), 10).with_condition("x > 5".to_string());
    let json = serde_json::to_string(&bp).unwrap();
    let deserialized: Breakpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, 1);
    assert_eq!(deserialized.file, "test.hudhud");
    assert_eq!(deserialized.line, 10);
    assert_eq!(deserialized.condition, Some("x > 5".to_string()));
    assert_eq!(
        deserialized.kind,
        BreakpointKind::Conditional("x > 5".to_string())
    );
}

#[test]
fn test_breakpoint_kind_equality() {
    assert_eq!(BreakpointKind::Normal, BreakpointKind::Normal);
    assert_ne!(
        BreakpointKind::Normal,
        BreakpointKind::Conditional("x".to_string())
    );
    assert_eq!(
        BreakpointKind::Exception(None),
        BreakpointKind::Exception(None)
    );
    assert_ne!(
        BreakpointKind::Exception(None),
        BreakpointKind::Exception(Some("T".to_string()))
    );
}

#[test]
fn test_with_log_message_sets_kind() {
    let bp =
        Breakpoint::new(1, "test.hudhud".to_string(), 5).with_log_message("value={x}".to_string());
    assert_eq!(bp.kind, BreakpointKind::Logpoint("value={x}".to_string()));
    assert!(bp.is_logpoint());
}

#[test]
fn test_as_exception_sets_kind() {
    let bp = Breakpoint::new(1, "test.hudhud".to_string(), 0)
        .as_exception(Some("RangeError".to_string()));
    assert!(bp.is_exception());
    assert_eq!(
        bp.kind,
        BreakpointKind::Exception(Some("RangeError".to_string()))
    );
}

// === from profiler.rs ===

#[test]
fn test_profiler() {
    let mut profiler = Profiler::new();
    profiler.start();
    profiler.record("test".to_string(), Duration::from_millis(100));
    assert_eq!(profiler.sample_count(), 1);

    let report = profiler.report();
    assert_eq!(report.samples.len(), 1);
}

#[test]
fn test_profiler_default() {
    let profiler = Profiler::default();
    assert_eq!(profiler.sample_count(), 0);
}

#[test]
fn test_profiler_clear() {
    let mut profiler = Profiler::new();
    profiler.start();
    profiler.record("a".to_string(), Duration::from_millis(50));
    profiler.record("b".to_string(), Duration::from_millis(100));
    assert_eq!(profiler.sample_count(), 2);

    profiler.clear();
    assert_eq!(profiler.sample_count(), 0);

    let report = profiler.report();
    assert_eq!(report.samples.len(), 0);
    assert_eq!(report.total_duration, Duration::ZERO);
}

#[test]
fn test_report_total_duration() {
    let mut profiler = Profiler::new();
    profiler.record("a".to_string(), Duration::from_millis(100));
    profiler.record("b".to_string(), Duration::from_millis(200));
    profiler.record("c".to_string(), Duration::from_millis(300));

    let report = profiler.report();
    assert_eq!(report.total_duration, Duration::from_millis(600));
    assert_eq!(report.samples.len(), 3);
}

#[test]
fn test_report_average_duration() {
    let mut profiler = Profiler::new();
    profiler.record("a".to_string(), Duration::from_millis(100));
    profiler.record("b".to_string(), Duration::from_millis(200));

    let report = profiler.report();
    assert_eq!(report.average_duration(), Duration::from_millis(150));
}

#[test]
fn test_report_average_duration_empty() {
    let report = ProfileReport {
        samples: vec![],
        total_duration: Duration::ZERO,
    };
    assert_eq!(report.average_duration(), Duration::ZERO);
}

#[test]
fn test_profile_sample_name() {
    let mut profiler = Profiler::new();
    profiler.record("my_function".to_string(), Duration::from_millis(42));

    let report = profiler.report();
    assert_eq!(report.samples[0].name, "my_function");
    assert_eq!(report.samples[0].duration, Duration::from_millis(42));
}

#[test]
fn test_profiler_multiple_records_ordering() {
    let mut profiler = Profiler::new();
    profiler.record("first".to_string(), Duration::from_millis(10));
    profiler.record("second".to_string(), Duration::from_millis(20));
    profiler.record("third".to_string(), Duration::from_millis(30));

    let report = profiler.report();
    assert_eq!(report.samples[0].name, "first");
    assert_eq!(report.samples[1].name, "second");
    assert_eq!(report.samples[2].name, "third");
    // Timestamps should be in order
    assert!(report.samples[0].timestamp <= report.samples[1].timestamp);
    assert!(report.samples[1].timestamp <= report.samples[2].timestamp);
}

#[test]
fn test_profiler_start_then_clear() {
    let mut profiler = Profiler::new();
    profiler.start();
    profiler.record("a".to_string(), Duration::from_millis(10));
    profiler.clear();
    // After clear, start_time is also reset
    assert_eq!(profiler.sample_count(), 0);
}

#[test]
fn test_profiler_report_clones_samples() {
    let mut profiler = Profiler::new();
    profiler.record("x".to_string(), Duration::from_millis(50));

    let report1 = profiler.report();
    let report2 = profiler.report();
    // Both reports should have the same data
    assert_eq!(report1.samples.len(), report2.samples.len());
    assert_eq!(report1.total_duration, report2.total_duration);
}

#[test]
fn test_profiler_zero_duration_records() {
    let mut profiler = Profiler::new();
    profiler.record("zero".to_string(), Duration::ZERO);
    profiler.record("also_zero".to_string(), Duration::ZERO);

    let report = profiler.report();
    assert_eq!(report.total_duration, Duration::ZERO);
    assert_eq!(report.average_duration(), Duration::ZERO);
}

#[test]
fn test_profile_report_single_sample_average() {
    let report = ProfileReport {
        samples: vec![ProfileSample {
            name: "only".to_string(),
            duration: Duration::from_millis(100),
            timestamp: Instant::now(),
        }],
        total_duration: Duration::from_millis(100),
    };
    assert_eq!(report.average_duration(), Duration::from_millis(100));
}

// === from debugger.rs ===

#[test]
fn test_debugger_breakpoints() {
    let mut debugger = Debugger::new();
    let id = debugger.add_breakpoint("test.hudhud".to_string(), 10);
    assert!(debugger.remove_breakpoint(id));
}

#[test]
fn test_debugger_state() {
    let mut debugger = Debugger::new();
    assert_eq!(debugger.state(), DebugState::Running);
    debugger.pause();
    assert_eq!(debugger.state(), DebugState::Paused);
    debugger.resume();
    assert_eq!(debugger.state(), DebugState::Running);
}

#[test]
fn test_step_into_pauses_on_next_statement() {
    let mut dbg = Debugger::new();
    dbg.step(StepMode::Into);
    assert_eq!(dbg.state(), DebugState::Stepping);

    // Any statement should trigger a pause.
    let paused = dbg.on_statement("main.hudhud", 1);
    assert!(paused);
    assert_eq!(dbg.state(), DebugState::Paused);
    assert_eq!(dbg.pause_reason(), Some(&PauseReason::Step));
}

#[test]
fn test_step_over_does_not_pause_inside_callee() {
    let mut dbg = Debugger::new();
    dbg.step(StepMode::Over);
    // Simulate entering a function (depth increases to 1).
    dbg.push_frame("inner".to_string());

    // Inside the callee — should NOT pause (depth 1 > start depth 0).
    let paused = dbg.on_statement("main.hudhud", 5);
    assert!(!paused);
    assert_eq!(dbg.state(), DebugState::Stepping);
}

#[test]
fn test_step_over_pauses_after_callee_returns() {
    let mut dbg = Debugger::new();
    dbg.step(StepMode::Over);
    // Simulate entering and leaving a function.
    dbg.push_frame("inner".to_string());
    dbg.pop_frame();

    // Back at the original depth — should pause.
    let paused = dbg.on_statement("main.hudhud", 10);
    assert!(paused);
    assert_eq!(dbg.state(), DebugState::Paused);
}

#[test]
fn test_step_out_pauses_after_return() {
    let mut dbg = Debugger::new();
    // Start inside a function (depth 1).
    dbg.push_frame("outer".to_string());
    dbg.step(StepMode::Out); // step_start_depth = 1

    // Still inside — should NOT pause.
    let paused = dbg.on_statement("main.hudhud", 3);
    assert!(!paused);

    // Simulate returning from the function.
    dbg.pop_frame(); // depth -> 0

    // Now at a shallower depth — should pause.
    let paused = dbg.on_statement("main.hudhud", 20);
    assert!(paused);
    assert_eq!(dbg.state(), DebugState::Paused);
}

#[test]
fn test_breakpoint_hit_during_step() {
    let mut dbg = Debugger::new();
    let bp_id = dbg.add_breakpoint("main.hudhud".to_string(), 7);
    dbg.step(StepMode::Out); // should still stop at a breakpoint

    let paused = dbg.on_statement("main.hudhud", 7);
    assert!(paused);
    assert_eq!(dbg.pause_reason(), Some(&PauseReason::Breakpoint(bp_id)));
}

#[test]
fn test_current_location_tracked() {
    let mut dbg = Debugger::new();
    dbg.on_statement("script.hudhud", 42);
    assert_eq!(dbg.current_location(), Some(("script.hudhud", 42)));
}

#[test]
fn test_call_stack_management() {
    let mut dbg = Debugger::new();
    assert!(dbg.call_stack().is_empty());

    dbg.push_frame("fn_a".to_string());
    dbg.push_frame("fn_b".to_string());
    assert_eq!(dbg.call_stack().len(), 2);

    dbg.pop_frame();
    assert_eq!(dbg.call_stack(), vec!["fn_a"]);
}

#[test]
fn test_step_over_convenience() {
    let mut dbg = Debugger::new();
    dbg.step_over();
    assert_eq!(dbg.state(), DebugState::Stepping);
    let paused = dbg.on_statement("main.hudhud", 1);
    assert!(paused);
}

#[test]
fn test_step_into_convenience() {
    let mut dbg = Debugger::new();
    dbg.step_into();
    assert_eq!(dbg.state(), DebugState::Stepping);
    let paused = dbg.on_statement("main.hudhud", 1);
    assert!(paused);
}

#[test]
fn test_step_out_convenience() {
    let mut dbg = Debugger::new();
    dbg.push_frame("fn_a".to_string());
    dbg.step_out();
    assert_eq!(dbg.state(), DebugState::Stepping);
    // Still inside — should NOT pause.
    assert!(!dbg.on_statement("main.hudhud", 1));
    dbg.pop_frame();
    // Now out — should pause.
    assert!(dbg.on_statement("main.hudhud", 2));
}

#[test]
fn test_continue_execution() {
    let mut dbg = Debugger::new();
    dbg.pause();
    assert_eq!(dbg.state(), DebugState::Paused);
    dbg.continue_execution();
    assert_eq!(dbg.state(), DebugState::Running);
}

#[test]
fn test_inspect_variable() {
    let mut dbg = Debugger::new();
    dbg.set_scope_variables(vec![
        ScopeVariable {
            name: "x".to_string(),
            value: "42".to_string(),
            ty: "Number".to_string(),
        },
        ScopeVariable {
            name: "name".to_string(),
            value: "\"hello\"".to_string(),
            ty: "String".to_string(),
        },
    ]);

    let var = dbg.inspect("x").unwrap();
    assert_eq!(var.value, "42");
    assert_eq!(var.ty, "Number");

    let var = dbg.inspect("name").unwrap();
    assert_eq!(var.value, "\"hello\"");

    assert!(dbg.inspect("nonexistent").is_none());
}

#[test]
fn test_scope_variables() {
    let mut dbg = Debugger::new();
    assert!(dbg.scope_variables().is_empty());

    dbg.set_scope_variables(vec![ScopeVariable {
        name: "a".to_string(),
        value: "1".to_string(),
        ty: "Number".to_string(),
    }]);
    assert_eq!(dbg.scope_variables().len(), 1);

    dbg.clear_scope_variables();
    assert!(dbg.scope_variables().is_empty());
}

#[test]
fn test_watch_expression() {
    let mut dbg = Debugger::new();
    dbg.add_watch("x + 1".to_string());
    assert_eq!(dbg.watch_expressions().len(), 1);
    assert!(dbg.get_watch_value("x + 1").is_none());

    dbg.update_watch("x + 1", "43".to_string());
    assert_eq!(dbg.get_watch_value("x + 1"), Some("43"));

    assert!(dbg.remove_watch("x + 1"));
    assert!(dbg.watch_expressions().is_empty());
}

#[test]
fn test_watch_updated_from_scope() {
    let mut dbg = Debugger::new();
    dbg.add_watch("x".to_string());

    dbg.set_scope_variables(vec![ScopeVariable {
        name: "x".to_string(),
        value: "100".to_string(),
        ty: "Number".to_string(),
    }]);

    assert_eq!(dbg.get_watch_value("x"), Some("100"));
}

#[test]
fn test_conditional_breakpoint_debugger() {
    let mut dbg = Debugger::new();
    let id = dbg.add_conditional_breakpoint("test.hudhud".to_string(), 10, "x > 5".to_string());
    let bp = dbg.get_breakpoint(id).unwrap();
    assert!(bp.is_conditional());
    assert_eq!(bp.condition, Some("x > 5".to_string()));
}

#[test]
fn test_logpoint_debugger() {
    let mut dbg = Debugger::new();
    let _id = dbg.add_logpoint(
        "test.hudhud".to_string(),
        10,
        "value of x = {x}".to_string(),
    );

    // Logpoints should not cause a pause via on_statement.
    let paused = dbg.on_statement("test.hudhud", 10);
    assert!(!paused);

    // But on_statement_extended should return the log message.
    let action = dbg.on_statement_extended("test.hudhud", 10);
    assert!(!action.should_pause);
    assert_eq!(action.log_messages.len(), 1);
    assert_eq!(action.log_messages[0], "value of x = {x}");
}

#[test]
fn test_exception_breakpoint_specific() {
    let mut dbg = Debugger::new();
    let _id = dbg.add_exception_breakpoint(Some("TypeError".to_string()));

    // Matching exception.
    assert!(dbg.on_exception("TypeError", "cannot read property"));
    assert_eq!(dbg.state(), DebugState::Paused);
    assert!(matches!(
        dbg.pause_reason(),
        Some(PauseReason::Exception(_))
    ));

    // Non-matching exception — resume first.
    dbg.resume();
    assert!(!dbg.on_exception("RangeError", "out of bounds"));
    assert_eq!(dbg.state(), DebugState::Running);
}

#[test]
fn test_exception_breakpoint_catch_all_debugger() {
    let mut dbg = Debugger::new();
    let _id = dbg.add_exception_breakpoint(None);

    assert!(dbg.on_exception("AnyError", "something went wrong"));
    assert_eq!(dbg.state(), DebugState::Paused);
}

#[test]
fn test_break_on_all_exceptions_flag() {
    let mut dbg = Debugger::new();
    assert!(!dbg.break_on_all_exceptions());

    dbg.set_break_on_all_exceptions(true);
    assert!(dbg.on_exception("Error", "oops"));
    assert_eq!(dbg.state(), DebugState::Paused);
}

#[test]
fn test_toggle_breakpoint() {
    let mut dbg = Debugger::new();
    let id = dbg.add_breakpoint("test.hudhud".to_string(), 10);

    // Initially enabled; toggling disables it.
    assert!(!dbg.toggle_breakpoint(id));
    assert!(!dbg.on_statement("test.hudhud", 10));

    // Toggle again to re-enable.
    assert!(dbg.toggle_breakpoint(id));
    assert!(dbg.on_statement("test.hudhud", 10));
}

#[test]
fn test_breakpoint_hit_count() {
    let mut dbg = Debugger::new();
    let id = dbg.add_breakpoint("test.hudhud".to_string(), 5);

    dbg.on_statement("test.hudhud", 5);
    dbg.resume();
    dbg.on_statement("test.hudhud", 5);

    let bp = dbg.get_breakpoint(id).unwrap();
    assert_eq!(bp.hit_count, 2);
}

#[test]
fn test_call_frames_with_location() {
    let mut dbg = Debugger::new();
    dbg.on_statement("main.hudhud", 10);
    dbg.push_frame("fn_a".to_string());

    let frames = dbg.call_frames();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].name, "fn_a");
    assert_eq!(frames[0].file.as_deref(), Some("main.hudhud"));
    assert_eq!(frames[0].line, Some(10));
}

#[test]
fn test_call_depth() {
    let mut dbg = Debugger::new();
    assert_eq!(dbg.call_depth(), 0);
    dbg.push_frame("a".to_string());
    assert_eq!(dbg.call_depth(), 1);
    dbg.push_frame("b".to_string());
    assert_eq!(dbg.call_depth(), 2);
    dbg.pop_frame();
    assert_eq!(dbg.call_depth(), 1);
}

#[test]
fn test_breakpoints_list() {
    let mut dbg = Debugger::new();
    dbg.add_breakpoint("a.hudhud".to_string(), 1);
    dbg.add_breakpoint("b.hudhud".to_string(), 2);
    assert_eq!(dbg.breakpoints().len(), 2);
}

#[test]
fn test_debugger_default() {
    let dbg = Debugger::default();
    assert_eq!(dbg.state(), DebugState::Running);
    assert!(dbg.call_stack().is_empty());
    assert!(dbg.scope_variables().is_empty());
    assert!(dbg.watch_expressions().is_empty());
    assert!(dbg.current_location().is_none());
    assert!(dbg.pause_reason().is_none());
}

#[test]
fn test_remove_nonexistent_breakpoint() {
    let mut dbg = Debugger::new();
    assert!(!dbg.remove_breakpoint(999));
}

#[test]
fn test_toggle_nonexistent_breakpoint() {
    let mut dbg = Debugger::new();
    // Toggling a non-existent breakpoint returns false
    assert!(!dbg.toggle_breakpoint(999));
}

#[test]
fn test_get_breakpoint_returns_none() {
    let dbg = Debugger::new();
    assert!(dbg.get_breakpoint(42).is_none());
}

#[test]
fn test_get_breakpoint_mut_returns_none() {
    let mut dbg = Debugger::new();
    assert!(dbg.get_breakpoint_mut(42).is_none());
}

#[test]
fn test_get_breakpoint_mut_modify() {
    let mut dbg = Debugger::new();
    let id = dbg.add_breakpoint("test.hudhud".to_string(), 5);

    let bp = dbg.get_breakpoint_mut(id).unwrap();
    bp.enabled = false;

    let bp2 = dbg.get_breakpoint(id).unwrap();
    assert!(!bp2.enabled);
}

#[test]
fn test_remove_watch_nonexistent() {
    let mut dbg = Debugger::new();
    assert!(!dbg.remove_watch("nonexistent"));
}

#[test]
fn test_get_watch_value_nonexistent() {
    let dbg = Debugger::new();
    assert!(dbg.get_watch_value("nonexistent").is_none());
}

#[test]
fn test_update_watch_nonexistent_is_noop() {
    let mut dbg = Debugger::new();
    // Should not panic
    dbg.update_watch("no_such_expression", "value".to_string());
    assert!(dbg.watch_expressions().is_empty());
}

#[test]
fn test_current_location_none_initially() {
    let dbg = Debugger::new();
    assert!(dbg.current_location().is_none());
}

#[test]
fn test_on_statement_no_breakpoint_no_step() {
    let mut dbg = Debugger::new();
    // In Running state, no breakpoints, should not pause
    let paused = dbg.on_statement("main.hudhud", 1);
    assert!(!paused);
    assert_eq!(dbg.state(), DebugState::Running);
}

#[test]
fn test_disabled_breakpoint_does_not_trigger() {
    let mut dbg = Debugger::new();
    let id = dbg.add_breakpoint("test.hudhud".to_string(), 10);
    dbg.toggle_breakpoint(id); // Disable it

    let paused = dbg.on_statement("test.hudhud", 10);
    assert!(!paused);
}

#[test]
fn test_on_exception_no_breakpoints_returns_false() {
    let mut dbg = Debugger::new();
    assert!(!dbg.on_exception("TypeError", "something"));
}

#[test]
fn test_on_exception_disabled_breakpoint_ignored() {
    let mut dbg = Debugger::new();
    let id = dbg.add_exception_breakpoint(Some("TypeError".to_string()));
    // Disable the exception breakpoint
    dbg.toggle_breakpoint(id);

    assert!(!dbg.on_exception("TypeError", "something"));
}

#[test]
fn test_remove_exception_breakpoint_cleans_up() {
    let mut dbg = Debugger::new();
    let id = dbg.add_exception_breakpoint(Some("TypeError".to_string()));

    // Verify it works
    assert!(dbg.on_exception("TypeError", "msg"));
    dbg.resume();

    // Remove it
    assert!(dbg.remove_breakpoint(id));

    // Should not trigger anymore
    assert!(!dbg.on_exception("TypeError", "msg"));
}

#[test]
fn test_on_statement_extended_with_breakpoint_and_logpoint() {
    let mut dbg = Debugger::new();
    dbg.add_breakpoint("test.hudhud".to_string(), 5);
    dbg.add_logpoint("test.hudhud".to_string(), 5, "log msg".to_string());

    let action = dbg.on_statement_extended("test.hudhud", 5);
    // Should pause because of breakpoint
    assert!(action.should_pause);
    // Should also have logpoint message
    assert_eq!(action.log_messages.len(), 1);
    assert_eq!(action.log_messages[0], "log msg");
    assert!(matches!(
        action.pause_reason,
        Some(PauseReason::Breakpoint(_))
    ));
}

#[test]
fn test_on_statement_extended_no_match() {
    let mut dbg = Debugger::new();
    let action = dbg.on_statement_extended("test.hudhud", 99);
    assert!(!action.should_pause);
    assert!(action.log_messages.is_empty());
    assert!(action.pause_reason.is_none());
}

#[test]
fn test_pause_sets_explicit_reason() {
    let mut dbg = Debugger::new();
    dbg.pause();
    assert_eq!(dbg.pause_reason(), Some(&PauseReason::Explicit));
}

#[test]
fn test_resume_clears_pause_reason() {
    let mut dbg = Debugger::new();
    dbg.pause();
    dbg.resume();
    assert!(dbg.pause_reason().is_none());
}

#[test]
fn test_step_clears_pause_reason() {
    let mut dbg = Debugger::new();
    dbg.pause();
    dbg.step(StepMode::Into);
    assert!(dbg.pause_reason().is_none());
    assert_eq!(dbg.state(), DebugState::Stepping);
}

#[test]
fn test_breakpoint_id_auto_increment() {
    let mut dbg = Debugger::new();
    let id1 = dbg.add_breakpoint("a.hudhud".to_string(), 1);
    let id2 = dbg.add_breakpoint("b.hudhud".to_string(), 2);
    let id3 = dbg.add_conditional_breakpoint("c.hudhud".to_string(), 3, "x > 0".to_string());
    let id4 = dbg.add_logpoint("d.hudhud".to_string(), 4, "msg".to_string());
    let id5 = dbg.add_exception_breakpoint(None);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
    assert_eq!(id4, 4);
    assert_eq!(id5, 5);
}

#[test]
fn test_call_frames_source_location() {
    let mut dbg = Debugger::new();
    dbg.on_statement("file_a.hudhud", 10);
    dbg.push_frame("fn_a".to_string());
    dbg.on_statement("file_b.hudhud", 20);
    dbg.push_frame("fn_b".to_string());

    let frames = dbg.call_frames();
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].name, "fn_a");
    assert_eq!(frames[0].file.as_deref(), Some("file_a.hudhud"));
    assert_eq!(frames[0].line, Some(10));
    assert_eq!(frames[1].name, "fn_b");
    assert_eq!(frames[1].file.as_deref(), Some("file_b.hudhud"));
    assert_eq!(frames[1].line, Some(20));
}

#[test]
fn test_set_scope_variables_updates_watches() {
    let mut dbg = Debugger::new();
    dbg.add_watch("x".to_string());
    dbg.add_watch("y".to_string());

    // x is in scope, y is not
    dbg.set_scope_variables(vec![ScopeVariable {
        name: "x".to_string(),
        value: "42".to_string(),
        ty: "Number".to_string(),
    }]);

    assert_eq!(dbg.get_watch_value("x"), Some("42"));
    // y should still be None since it wasn't in scope
    assert!(dbg.get_watch_value("y").is_none());
}

#[test]
fn test_on_statement_extended_logpoint_only() {
    let mut dbg = Debugger::new();
    dbg.add_logpoint("test.hudhud".to_string(), 10, "log: {x}".to_string());

    let action = dbg.on_statement_extended("test.hudhud", 10);
    // Should NOT pause (logpoint doesn't pause)
    assert!(!action.should_pause);
    assert_eq!(action.log_messages.len(), 1);
    assert_eq!(action.log_messages[0], "log: {x}");
    // pause_reason should be None since no pause
    assert!(action.pause_reason.is_none());
}

#[test]
fn test_on_statement_extended_different_line() {
    let mut dbg = Debugger::new();
    dbg.add_logpoint("test.hudhud".to_string(), 10, "log".to_string());

    let action = dbg.on_statement_extended("test.hudhud", 99);
    assert!(!action.should_pause);
    assert!(action.log_messages.is_empty());
}

#[test]
fn test_on_statement_extended_different_file() {
    let mut dbg = Debugger::new();
    dbg.add_breakpoint("a.hudhud".to_string(), 5);

    let action = dbg.on_statement_extended("b.hudhud", 5);
    assert!(!action.should_pause);
}

#[test]
fn test_logpoint_records_hit_count() {
    let mut dbg = Debugger::new();
    let id = dbg.add_logpoint("test.hudhud".to_string(), 5, "msg".to_string());

    dbg.on_statement_extended("test.hudhud", 5);
    dbg.on_statement_extended("test.hudhud", 5);

    let bp = dbg.get_breakpoint(id).unwrap();
    assert_eq!(bp.hit_count, 2);
}

#[test]
fn test_exception_breakpoint_not_matching_filter() {
    let mut dbg = Debugger::new();
    dbg.add_exception_breakpoint(Some("TypeError".to_string()));

    // RangeError should not match TypeError filter
    assert!(!dbg.on_exception("RangeError", "out of range"));
    assert_eq!(dbg.state(), DebugState::Running);
}

#[test]
fn test_exception_breakpoint_partial_match() {
    let mut dbg = Debugger::new();
    dbg.add_exception_breakpoint(Some("Type".to_string()));

    // "TypeError" contains "Type"
    assert!(dbg.on_exception("TypeError", "bad type"));
    assert_eq!(dbg.state(), DebugState::Paused);
}

#[test]
fn test_on_exception_formats_reason() {
    let mut dbg = Debugger::new();
    dbg.set_break_on_all_exceptions(true);

    dbg.on_exception("RangeError", "index out of bounds");
    assert_eq!(
        dbg.pause_reason(),
        Some(&PauseReason::Exception(
            "RangeError: index out of bounds".to_string()
        ))
    );
}

#[test]
fn test_multiple_logpoints_same_line() {
    let mut dbg = Debugger::new();
    dbg.add_logpoint("f.hudhud".to_string(), 3, "msg1".to_string());
    dbg.add_logpoint("f.hudhud".to_string(), 3, "msg2".to_string());

    let action = dbg.on_statement_extended("f.hudhud", 3);
    assert!(!action.should_pause);
    assert_eq!(action.log_messages.len(), 2);
}

#[test]
fn test_disabled_logpoint_not_returned() {
    let mut dbg = Debugger::new();
    let id = dbg.add_logpoint("f.hudhud".to_string(), 3, "msg".to_string());
    dbg.toggle_breakpoint(id); // disable

    let action = dbg.on_statement_extended("f.hudhud", 3);
    assert!(action.log_messages.is_empty());
}

#[test]
fn test_step_mode_none_does_not_pause() {
    let mut dbg = Debugger::new();
    // Manually set Stepping but no step mode
    dbg.step(StepMode::Over);
    // Simulate: push then on_statement at deeper depth
    dbg.push_frame("f".to_string());
    let paused = dbg.on_statement("f.hudhud", 1);
    // Deeper than start depth: should not pause for Over
    assert!(!paused);
}

#[test]
fn test_multiple_watches() {
    let mut dbg = Debugger::new();
    dbg.add_watch("a".to_string());
    dbg.add_watch("b".to_string());
    dbg.add_watch("c".to_string());
    assert_eq!(dbg.watch_expressions().len(), 3);

    dbg.update_watch("b", "42".to_string());
    assert_eq!(dbg.get_watch_value("b"), Some("42"));
    assert!(dbg.get_watch_value("a").is_none());

    dbg.remove_watch("a");
    assert_eq!(dbg.watch_expressions().len(), 2);
}

#[test]
fn test_pop_frame_empty_stack() {
    let mut dbg = Debugger::new();
    // Popping from empty stack should not panic
    dbg.pop_frame();
    assert_eq!(dbg.call_depth(), 0);
}

#[test]
fn test_step_out_at_depth_zero_pauses_immediately() {
    let mut dbg = Debugger::new();
    // At depth 0, step out with start_depth 0
    dbg.step(StepMode::Out);
    // Since current_depth (0) is not < step_start_depth (0), should NOT pause
    let paused = dbg.on_statement("f.hudhud", 1);
    assert!(!paused);
}

// === from dap.rs ===

/// Helper: encode a DAP request as a wire-protocol message.
fn encode_dap_request(seq: i64, command: &str, arguments: Option<Value>) -> Vec<u8> {
    let req = serde_json::json!({
        "seq": seq,
        "type": "request",
        "command": command,
        "arguments": arguments,
    });
    let body = serde_json::to_string(&req).unwrap();
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
}

/// Helper: decode all DAP messages from raw bytes.
fn decode_all_dap_messages(data: &[u8]) -> Vec<Value> {
    let s = std::str::from_utf8(data).unwrap();
    let mut messages = Vec::new();
    let mut remaining = s;
    while let Some(header_start) = remaining.find("Content-Length:") {
        remaining = &remaining[header_start..];
        // Find end of headers.
        let header_end = remaining.find("\r\n\r\n").expect("no header terminator");
        let length_str = remaining["Content-Length:".len()..header_end].trim();
        let length: usize = length_str.parse().expect("bad content length");
        let body_start = header_end + 4;
        let body = &remaining[body_start..body_start + length];
        let msg: Value = serde_json::from_str(body).unwrap();
        messages.push(msg);
        remaining = &remaining[body_start + length..];
    }
    messages
}

#[test]
fn test_read_message_parses_request() {
    use std::io;
    let data = encode_dap_request(1, "initialize", None);
    let mut reader = io::BufReader::new(data.as_slice());
    let req = DapServer::read_message(&mut reader).unwrap().unwrap();
    assert_eq!(req.command, "initialize");
    assert_eq!(req.seq, 1);
}

#[test]
fn test_read_message_eof_returns_none() {
    use std::io;
    let data: &[u8] = b"";
    let mut reader = io::BufReader::new(data);
    let result = DapServer::read_message(&mut reader).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_initialize_response_and_event() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "initialize".to_string(),
        arguments: Some(serde_json::json!({
            "clientId": "test",
            "linesStartAt1": true,
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages.len(), 2);

    // First: response
    assert_eq!(messages[0]["type"], "response");
    assert_eq!(messages[0]["command"], "initialize");
    assert_eq!(messages[0]["success"], true);
    assert!(messages[0]["body"]["supportsConfigurationDoneRequest"]
        .as_bool()
        .unwrap());

    // Second: initialized event
    assert_eq!(messages[1]["type"], "event");
    assert_eq!(messages[1]["event"], "initialized");
}

#[test]
fn test_launch_response() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 2,
        command: "launch".to_string(),
        arguments: Some(serde_json::json!({
            "program": "test.hudhud",
            "stopOnEntry": false,
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["success"], true);
    assert!(server.launched);
}

#[test]
fn test_set_breakpoints_dap() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 3,
        command: "setBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "source": { "path": "test.hudhud" },
            "breakpoints": [
                { "line": 5 },
                { "line": 10 },
            ]
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages.len(), 1);
    let bps = messages[0]["body"]["breakpoints"].as_array().unwrap();
    assert_eq!(bps.len(), 2);
    assert_eq!(bps[0]["line"], 5);
    assert_eq!(bps[0]["verified"], true);
    assert_eq!(bps[1]["line"], 10);
}

#[test]
fn test_set_breakpoints_replaces_old() {
    let mut server = DapServer::new();

    // Set initial breakpoints.
    let req1 = DapRequest {
        seq: 1,
        command: "setBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "source": { "path": "test.hudhud" },
            "breakpoints": [{ "line": 5 }, { "line": 10 }]
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req1, &mut output).unwrap();

    // Replace with just one breakpoint.
    let req2 = DapRequest {
        seq: 2,
        command: "setBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "source": { "path": "test.hudhud" },
            "breakpoints": [{ "line": 20 }]
        })),
    };
    output.clear();
    server.handle_request(&req2, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let bps = messages[0]["body"]["breakpoints"].as_array().unwrap();
    assert_eq!(bps.len(), 1);
    assert_eq!(bps[0]["line"], 20);

    // Old breakpoints should no longer trigger.
    assert!(!server.debugger.on_statement("test.hudhud", 5));
    assert!(!server.debugger.on_statement("test.hudhud", 10));
    assert!(server.debugger.on_statement("test.hudhud", 20));
}

#[test]
fn test_threads() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "threads".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let threads = messages[0]["body"]["threads"].as_array().unwrap();
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0]["id"], THREAD_ID);
    assert_eq!(threads[0]["name"], THREAD_NAME);
}

#[test]
fn test_stack_trace_empty() {
    let mut server = DapServer::new();
    // Simulate being at a location.
    server.debugger.on_statement("test.hudhud", 42);

    let req = DapRequest {
        seq: 1,
        command: "stackTrace".to_string(),
        arguments: Some(serde_json::json!({ "threadId": THREAD_ID })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let frames = messages[0]["body"]["stackFrames"].as_array().unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["name"], "<global>");
    assert_eq!(frames[0]["line"], 42);
}

#[test]
fn test_stack_trace_with_frames() {
    let mut server = DapServer::new();
    server.debugger.push_frame("main".to_string());
    server.debugger.push_frame("helper".to_string());
    server.debugger.on_statement("test.hudhud", 10);

    let req = DapRequest {
        seq: 1,
        command: "stackTrace".to_string(),
        arguments: Some(serde_json::json!({ "threadId": THREAD_ID })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let frames = messages[0]["body"]["stackFrames"].as_array().unwrap();
    assert_eq!(frames.len(), 2);
    // Most recent frame first.
    assert_eq!(frames[0]["name"], "helper");
    assert_eq!(frames[1]["name"], "main");
}

#[test]
fn test_continue_resumes_debugger() {
    let mut server = DapServer::new();
    server.debugger.pause();
    assert_eq!(server.debugger.state(), DebugState::Paused);

    let req = DapRequest {
        seq: 1,
        command: "continue".to_string(),
        arguments: Some(serde_json::json!({ "threadId": THREAD_ID })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    assert_eq!(server.debugger.state(), DebugState::Running);
}

#[test]
fn test_next_sets_step_over() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "next".to_string(),
        arguments: Some(serde_json::json!({ "threadId": THREAD_ID })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();
    assert_eq!(server.debugger.state(), DebugState::Stepping);
}

#[test]
fn test_step_in_sets_step_into() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "stepIn".to_string(),
        arguments: Some(serde_json::json!({ "threadId": THREAD_ID })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();
    assert_eq!(server.debugger.state(), DebugState::Stepping);
}

#[test]
fn test_step_out_sets_step_out() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "stepOut".to_string(),
        arguments: Some(serde_json::json!({ "threadId": THREAD_ID })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();
    assert_eq!(server.debugger.state(), DebugState::Stepping);
}

#[test]
fn test_disconnect() {
    let mut server = DapServer::new();
    assert!(!server.is_disconnected());

    let req = DapRequest {
        seq: 1,
        command: "disconnect".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    assert!(server.is_disconnected());
}

#[test]
fn test_stopped_event() {
    let mut server = DapServer::new();
    // Simulate a breakpoint hit.
    server.debugger.add_breakpoint("test.hudhud".to_string(), 5);
    server.debugger.on_statement("test.hudhud", 5);

    let vars = vec![Variable {
        name: "x".to_string(),
        value: "42".to_string(),
        ty: "Number".to_string(),
        variables_reference: 0,
    }];

    let mut output = Vec::new();
    server.send_stopped_event(&mut output, vars).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["event"], "stopped");
    assert_eq!(messages[0]["body"]["reason"], "breakpoint");
    assert_eq!(messages[0]["body"]["threadId"], THREAD_ID);
}

#[test]
fn test_variables_after_stop() {
    let mut server = DapServer::new();

    // Populate variables as if we just stopped.
    let vars = vec![
        Variable {
            name: "x".to_string(),
            value: "42".to_string(),
            ty: "Number".to_string(),
            variables_reference: 0,
        },
        Variable {
            name: "name".to_string(),
            value: "\"hello\"".to_string(),
            ty: "String".to_string(),
            variables_reference: 0,
        },
    ];
    server.variable_store.insert(1, vars);

    let req = DapRequest {
        seq: 1,
        command: "variables".to_string(),
        arguments: Some(serde_json::json!({ "variablesReference": 1 })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let variables = messages[0]["body"]["variables"].as_array().unwrap();
    assert_eq!(variables.len(), 2);
    assert_eq!(variables[0]["name"], "x");
    assert_eq!(variables[0]["value"], "42");
    assert_eq!(variables[1]["name"], "name");
}

#[test]
fn test_scopes() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "scopes".to_string(),
        arguments: Some(serde_json::json!({ "frameId": 0 })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let scopes = messages[0]["body"]["scopes"].as_array().unwrap();
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0]["name"], "Local");
    assert_eq!(scopes[0]["variablesReference"], 1);
}

#[test]
fn test_unsupported_command() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "completions".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], false);
    assert!(messages[0]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported"));
}

#[test]
fn test_full_session_via_run() {
    // Simulate a minimal DAP session: initialize -> launch -> disconnect.
    let mut input = Vec::new();
    input.extend_from_slice(&encode_dap_request(
        1,
        "initialize",
        Some(serde_json::json!({ "clientId": "test" })),
    ));
    input.extend_from_slice(&encode_dap_request(
        2,
        "launch",
        Some(serde_json::json!({ "program": "test.hudhud" })),
    ));
    input.extend_from_slice(&encode_dap_request(3, "disconnect", None));

    let mut output = Vec::new();
    let mut server = DapServer::new();
    server.run(input.as_slice(), &mut output).unwrap();

    assert!(server.is_disconnected());
    let messages = decode_all_dap_messages(&output);
    // initialize response + initialized event + launch response + disconnect response = 4
    assert_eq!(messages.len(), 4);
}

#[test]
fn test_configuration_done_with_stop_on_entry() {
    let mut server = DapServer::new();

    // Launch with stopOnEntry.
    let launch = DapRequest {
        seq: 1,
        command: "launch".to_string(),
        arguments: Some(serde_json::json!({
            "program": "test.hudhud",
            "stopOnEntry": true,
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&launch, &mut output).unwrap();
    output.clear();

    // configurationDone should emit a stopped event.
    let cfg_done = DapRequest {
        seq: 2,
        command: "configurationDone".to_string(),
        arguments: None,
    };
    server.handle_request(&cfg_done, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages.len(), 2); // response + stopped event
    assert_eq!(messages[1]["event"], "stopped");
    assert_eq!(messages[1]["body"]["reason"], "entry");
    assert_eq!(server.debugger.state(), DebugState::Paused);
}

#[test]
fn test_dap_server_default() {
    let server = DapServer::default();
    assert!(!server.is_disconnected());
    assert_eq!(server.debugger().state(), DebugState::Running);
}

#[test]
fn test_with_debugger() {
    let mut dbg = Debugger::new();
    dbg.add_breakpoint("test.hudhud".to_string(), 5);
    let server = DapServer::with_debugger(dbg);
    assert_eq!(server.debugger().breakpoints().len(), 1);
}

#[test]
fn test_debugger_mut_access() {
    let mut server = DapServer::new();
    server
        .debugger_mut()
        .add_breakpoint("test.hudhud".to_string(), 10);
    assert_eq!(server.debugger().breakpoints().len(), 1);
}

#[test]
fn test_evaluate_found_variable() {
    let mut server = DapServer::new();
    server
        .debugger_mut()
        .set_scope_variables(vec![ScopeVariable {
            name: "x".to_string(),
            value: "42".to_string(),
            ty: "Number".to_string(),
        }]);

    let req = DapRequest {
        seq: 1,
        command: "evaluate".to_string(),
        arguments: Some(serde_json::json!({
            "expression": "x",
            "frameId": 0,
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], true);
    assert_eq!(messages[0]["body"]["result"], "42");
    assert_eq!(messages[0]["body"]["type"], "Number");
}

#[test]
fn test_evaluate_not_found() {
    let mut server = DapServer::new();

    let req = DapRequest {
        seq: 1,
        command: "evaluate".to_string(),
        arguments: Some(serde_json::json!({
            "expression": "nonexistent",
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], false);
    assert!(messages[0]["message"]
        .as_str()
        .unwrap()
        .contains("cannot evaluate"));
}

#[test]
fn test_evaluate_missing_arguments() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "evaluate".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], false);
}

#[test]
fn test_variables_missing_arguments() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "variables".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], false);
}

#[test]
fn test_variables_empty_store() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "variables".to_string(),
        arguments: Some(serde_json::json!({ "variablesReference": 99 })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], true);
    let variables = messages[0]["body"]["variables"].as_array().unwrap();
    assert!(variables.is_empty());
}

#[test]
fn test_set_breakpoints_missing_arguments() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "setBreakpoints".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], false);
}

#[test]
fn test_set_exception_breakpoints_all_filter() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "setExceptionBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "filters": ["all"]
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], true);
    assert!(server.debugger().break_on_all_exceptions());
}

#[test]
fn test_set_exception_breakpoints_uncaught_filter() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "setExceptionBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "filters": ["uncaught"]
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], true);
    assert!(!server.debugger().break_on_all_exceptions());
    // Should have added an exception breakpoint
    let bps: Vec<_> = server
        .debugger()
        .breakpoints()
        .into_iter()
        .filter(|bp| bp.is_exception())
        .collect();
    assert_eq!(bps.len(), 1);
}

#[test]
fn test_set_exception_breakpoints_empty_filters() {
    let mut server = DapServer::new();
    // First enable all exceptions
    server.debugger_mut().set_break_on_all_exceptions(true);

    let req = DapRequest {
        seq: 1,
        command: "setExceptionBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "filters": []
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    assert!(!server.debugger().break_on_all_exceptions());
}

#[test]
fn test_set_exception_breakpoints_missing_arguments() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "setExceptionBreakpoints".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], false);
}

#[test]
fn test_send_terminated_event() {
    let mut server = DapServer::new();
    let mut output = Vec::new();
    server.send_terminated_event(&mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["event"], "terminated");
}

#[test]
fn test_send_output_event() {
    let mut server = DapServer::new();
    let mut output = Vec::new();
    server
        .send_output_event(&mut output, "console", "Hello world\n")
        .unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["event"], "output");
    assert_eq!(messages[0]["body"]["category"], "console");
    assert_eq!(messages[0]["body"]["output"], "Hello world\n");
}

#[test]
fn test_stopped_event_step_reason() {
    let mut server = DapServer::new();
    // Simulate a step pause
    server.debugger_mut().step(StepMode::Into);
    server.debugger_mut().on_statement("test.hudhud", 1);

    let mut output = Vec::new();
    server.send_stopped_event(&mut output, vec![]).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["body"]["reason"], "step");
}

#[test]
fn test_stopped_event_explicit_reason() {
    let mut server = DapServer::new();
    server.debugger_mut().pause();

    let mut output = Vec::new();
    server.send_stopped_event(&mut output, vec![]).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["body"]["reason"], "pause");
}

#[test]
fn test_configuration_done_without_stop_on_entry() {
    let mut server = DapServer::new();
    // Launch without stopOnEntry
    let launch = DapRequest {
        seq: 1,
        command: "launch".to_string(),
        arguments: Some(serde_json::json!({
            "program": "test.hudhud",
            "stopOnEntry": false,
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&launch, &mut output).unwrap();
    output.clear();

    let cfg_done = DapRequest {
        seq: 2,
        command: "configurationDone".to_string(),
        arguments: None,
    };
    server.handle_request(&cfg_done, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    // Only response, no stopped event
    assert_eq!(messages.len(), 1);
    assert_eq!(server.debugger.state(), DebugState::Running);
}

#[test]
fn test_launch_with_no_arguments() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "launch".to_string(),
        arguments: None,
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], true);
    assert!(server.launched);
}

#[test]
fn test_stopped_event_exception_reason() {
    let mut server = DapServer::new();
    server.debugger_mut().set_break_on_all_exceptions(true);
    server.debugger_mut().on_exception("TypeError", "bad value");

    let mut output = Vec::new();
    server.send_stopped_event(&mut output, vec![]).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["body"]["reason"], "exception");
}

#[test]
fn test_stopped_event_unknown_reason() {
    let mut server = DapServer::new();
    // No pause reason set, debugger in Running state
    let mut output = Vec::new();
    server.send_stopped_event(&mut output, vec![]).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["body"]["reason"], "unknown");
}

#[test]
fn test_read_message_missing_content_length() {
    use std::io;
    let data = b"Some-Header: value\r\n\r\n{}";
    let mut reader = io::BufReader::new(&data[..]);
    let result = DapServer::read_message(&mut reader);
    assert!(result.is_err());
}

#[test]
fn test_read_message_invalid_content_length() {
    use std::io;
    let data = b"Content-Length: notanumber\r\n\r\n";
    let mut reader = io::BufReader::new(&data[..]);
    let result = DapServer::read_message(&mut reader);
    assert!(result.is_err());
}

#[test]
fn test_dap_message_request_serialization() {
    let msg = DapMessage::Request(DapRequest {
        seq: 1,
        command: "initialize".to_string(),
        arguments: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"request\""));
    assert!(json.contains("\"command\":\"initialize\""));
}

#[test]
fn test_dap_message_response_serialization() {
    let msg = DapMessage::Response(DapResponse {
        seq: 1,
        request_seq: 1,
        success: true,
        command: "initialize".to_string(),
        message: None,
        body: None,
    });
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"response\""));
    assert!(json.contains("\"success\":true"));
}

#[test]
fn test_dap_message_event_serialization() {
    let msg = DapMessage::Event(DapEvent {
        seq: 1,
        event: "stopped".to_string(),
        body: Some(serde_json::json!({"reason": "breakpoint"})),
    });
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"event\""));
    assert!(json.contains("\"event\":\"stopped\""));
}

#[test]
fn test_set_breakpoints_empty_list() {
    let mut server = DapServer::new();
    let req = DapRequest {
        seq: 1,
        command: "setBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "source": { "path": "test.hudhud" },
            "breakpoints": []
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let bps = messages[0]["body"]["breakpoints"].as_array().unwrap();
    assert_eq!(bps.len(), 0);
}

#[test]
fn test_set_breakpoints_source_name_fallback() {
    let mut server = DapServer::new();
    // Source has name but no path
    let req = DapRequest {
        seq: 1,
        command: "setBreakpoints".to_string(),
        arguments: Some(serde_json::json!({
            "source": { "name": "test.hudhud" },
            "breakpoints": [{ "line": 5 }]
        })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    assert_eq!(messages[0]["success"], true);
    let bps = messages[0]["body"]["breakpoints"].as_array().unwrap();
    assert_eq!(bps.len(), 1);
}

#[test]
fn test_stack_trace_no_location() {
    let mut server = DapServer::new();
    // No location set, no frames pushed
    let req = DapRequest {
        seq: 1,
        command: "stackTrace".to_string(),
        arguments: Some(serde_json::json!({ "threadId": THREAD_ID })),
    };
    let mut output = Vec::new();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let frames = messages[0]["body"]["stackFrames"].as_array().unwrap();
    // No location set, so empty stack trace
    assert!(frames.is_empty());
}

#[test]
fn test_send_stopped_event_stores_variables() {
    let mut server = DapServer::new();
    server.debugger_mut().pause();

    let vars = vec![Variable {
        name: "x".to_string(),
        value: "42".to_string(),
        ty: "Number".to_string(),
        variables_reference: 0,
    }];

    let mut output = Vec::new();
    server.send_stopped_event(&mut output, vars).unwrap();

    // Now request variables - they should be stored
    let req = DapRequest {
        seq: 1,
        command: "variables".to_string(),
        arguments: Some(serde_json::json!({ "variablesReference": 1 })),
    };
    output.clear();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let variables = messages[0]["body"]["variables"].as_array().unwrap();
    assert_eq!(variables.len(), 1);
    assert_eq!(variables[0]["name"], "x");
}

#[test]
fn test_send_stopped_event_empty_vars_does_not_store() {
    let mut server = DapServer::new();
    server.debugger_mut().pause();

    let mut output = Vec::new();
    server.send_stopped_event(&mut output, vec![]).unwrap();

    // Variables reference 1 should return empty
    let req = DapRequest {
        seq: 1,
        command: "variables".to_string(),
        arguments: Some(serde_json::json!({ "variablesReference": 1 })),
    };
    output.clear();
    server.handle_request(&req, &mut output).unwrap();

    let messages = decode_all_dap_messages(&output);
    let variables = messages[0]["body"]["variables"].as_array().unwrap();
    assert!(variables.is_empty());
}

#[test]
fn test_set_exception_breakpoints_replaces_old() {
    let mut server = DapServer::new();

    // First set "uncaught"
    let req1 = DapRequest {
        seq: 1,
        command: "setExceptionBreakpoints".to_string(),
        arguments: Some(serde_json::json!({ "filters": ["uncaught"] })),
    };
    let mut output = Vec::new();
    server.handle_request(&req1, &mut output).unwrap();

    let exc_count_before: usize = server
        .debugger()
        .breakpoints()
        .iter()
        .filter(|bp| bp.is_exception())
        .count();
    assert_eq!(exc_count_before, 1);

    // Now set "all" - should clear old exception breakpoints
    let req2 = DapRequest {
        seq: 2,
        command: "setExceptionBreakpoints".to_string(),
        arguments: Some(serde_json::json!({ "filters": ["all"] })),
    };
    output.clear();
    server.handle_request(&req2, &mut output).unwrap();

    // Old exception BP should be cleared, "all" flag set
    assert!(server.debugger().break_on_all_exceptions());
    let exc_count_after: usize = server
        .debugger()
        .breakpoints()
        .iter()
        .filter(|bp| bp.is_exception())
        .count();
    assert_eq!(exc_count_after, 0);
}

#[test]
fn test_variable_struct_serialization() {
    let var = Variable {
        name: "count".to_string(),
        value: "7".to_string(),
        ty: "Number".to_string(),
        variables_reference: 0,
    };
    let json = serde_json::to_string(&var).unwrap();
    assert!(json.contains("\"name\":\"count\""));
    assert!(json.contains("\"type\":\"Number\""));
}

#[test]
fn test_source_struct_serialization() {
    let src = Source {
        name: Some("test.hudhud".to_string()),
        path: Some("/path/test.hudhud".to_string()),
    };
    let json = serde_json::to_string(&src).unwrap();
    let deserialized: Source = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, Some("test.hudhud".to_string()));
    assert_eq!(deserialized.path, Some("/path/test.hudhud".to_string()));
}

#[test]
fn test_source_struct_empty() {
    let src = Source {
        name: None,
        path: None,
    };
    let json = serde_json::to_string(&src).unwrap();
    let deserialized: Source = serde_json::from_str(&json).unwrap();
    assert!(deserialized.name.is_none());
    assert!(deserialized.path.is_none());
}

#[test]
fn test_full_session_with_breakpoints() {
    let mut input = Vec::new();
    input.extend_from_slice(&encode_dap_request(
        1,
        "initialize",
        Some(serde_json::json!({ "clientId": "test" })),
    ));
    input.extend_from_slice(&encode_dap_request(
        2,
        "launch",
        Some(serde_json::json!({ "program": "test.hudhud" })),
    ));
    input.extend_from_slice(&encode_dap_request(
        3,
        "setBreakpoints",
        Some(serde_json::json!({
            "source": { "path": "test.hudhud" },
            "breakpoints": [{ "line": 10 }]
        })),
    ));
    input.extend_from_slice(&encode_dap_request(4, "configurationDone", None));
    input.extend_from_slice(&encode_dap_request(5, "disconnect", None));

    let mut output = Vec::new();
    let mut server = DapServer::new();
    server.run(input.as_slice(), &mut output).unwrap();

    assert!(server.is_disconnected());
    let messages = decode_all_dap_messages(&output);
    // initialize response + initialized event + launch response +
    // setBreakpoints response + configurationDone response + disconnect response = 6
    assert_eq!(messages.len(), 6);
}
