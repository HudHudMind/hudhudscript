//! Comprehensive public API tests for hudhudscript-debug

use hudhudscript_debug::{
    Breakpoint, BreakpointKind, DapEvent, DapMessage, DapRequest, DapResponse, DapServer,
    DebugState, Debugger, PauseReason, Profiler, StepMode, Variable,
};
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════
// Debugger — new / state
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_new_default_running() {
    let d = Debugger::new();
    assert_eq!(d.state(), DebugState::Running);
}

#[test]
fn debugger_default_trait() {
    let d = Debugger::default();
    assert_eq!(d.state(), DebugState::Running);
}

#[test]
fn debugger_no_breakpoints_initially() {
    let d = Debugger::new();
    assert!(d.breakpoints().is_empty());
}

#[test]
fn debugger_no_pause_reason_initially() {
    let d = Debugger::new();
    assert!(d.pause_reason().is_none());
}

#[test]
fn debugger_current_location_none_initially() {
    let d = Debugger::new();
    assert!(d.current_location().is_none());
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — add_breakpoint / remove / toggle / get
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_add_breakpoint() {
    let mut d = Debugger::new();
    let id = d.add_breakpoint("main.hudhud".into(), 10);
    assert!(id > 0);
    assert_eq!(d.breakpoints().len(), 1);
}

#[test]
fn debugger_add_multiple_breakpoints() {
    let mut d = Debugger::new();
    let id1 = d.add_breakpoint("a.hudhud".into(), 1);
    let id2 = d.add_breakpoint("b.hudhud".into(), 5);
    assert_ne!(id1, id2);
    assert_eq!(d.breakpoints().len(), 2);
}

#[test]
fn debugger_remove_breakpoint() {
    let mut d = Debugger::new();
    let id = d.add_breakpoint("f.hudhud".into(), 1);
    assert!(d.remove_breakpoint(id));
    assert!(d.breakpoints().is_empty());
}

#[test]
fn debugger_remove_nonexistent() {
    let mut d = Debugger::new();
    assert!(!d.remove_breakpoint(999));
}

#[test]
fn debugger_toggle_breakpoint() {
    let mut d = Debugger::new();
    let id = d.add_breakpoint("f.hudhud".into(), 1);
    // Initially enabled
    assert!(d.get_breakpoint(id).unwrap().enabled);
    let new_state = d.toggle_breakpoint(id);
    assert!(!new_state);
    assert!(!d.get_breakpoint(id).unwrap().enabled);
    // Toggle back
    let new_state2 = d.toggle_breakpoint(id);
    assert!(new_state2);
}

#[test]
fn debugger_toggle_nonexistent() {
    let mut d = Debugger::new();
    assert!(!d.toggle_breakpoint(999));
}

#[test]
fn debugger_get_breakpoint() {
    let mut d = Debugger::new();
    let id = d.add_breakpoint("f.hudhud".into(), 42);
    let bp = d.get_breakpoint(id).unwrap();
    assert_eq!(bp.file, "f.hudhud");
    assert_eq!(bp.line, 42);
}

#[test]
fn debugger_get_breakpoint_none() {
    let d = Debugger::new();
    assert!(d.get_breakpoint(999).is_none());
}

#[test]
fn debugger_get_breakpoint_mut() {
    let mut d = Debugger::new();
    let id = d.add_breakpoint("f.hudhud".into(), 1);
    let bp = d.get_breakpoint_mut(id).unwrap();
    bp.enabled = false;
    assert!(!d.get_breakpoint(id).unwrap().enabled);
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — conditional breakpoint
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_add_conditional_breakpoint() {
    let mut d = Debugger::new();
    let id = d.add_conditional_breakpoint("f.hudhud".into(), 5, "x > 10".into());
    let bp = d.get_breakpoint(id).unwrap();
    assert!(bp.is_conditional());
    assert_eq!(bp.condition, Some("x > 10".into()));
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — logpoint
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_add_logpoint() {
    let mut d = Debugger::new();
    let id = d.add_logpoint("f.hudhud".into(), 3, "x = {x}".into());
    let bp = d.get_breakpoint(id).unwrap();
    assert!(bp.is_logpoint());
    assert!(!bp.is_conditional());
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — exception breakpoint
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_add_exception_breakpoint_all() {
    let mut d = Debugger::new();
    let id = d.add_exception_breakpoint(None);
    let bp = d.get_breakpoint(id).unwrap();
    assert!(bp.is_exception());
}

#[test]
fn debugger_add_exception_breakpoint_filtered() {
    let mut d = Debugger::new();
    let id = d.add_exception_breakpoint(Some("TypeError".into()));
    let bp = d.get_breakpoint(id).unwrap();
    assert!(bp.is_exception());
}

#[test]
fn debugger_break_on_all_exceptions() {
    let mut d = Debugger::new();
    assert!(!d.break_on_all_exceptions());
    d.set_break_on_all_exceptions(true);
    assert!(d.break_on_all_exceptions());
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — pause / resume / step
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_pause() {
    let mut d = Debugger::new();
    d.pause();
    assert_eq!(d.state(), DebugState::Paused);
    assert_eq!(d.pause_reason(), Some(&PauseReason::Explicit));
}

#[test]
fn debugger_resume() {
    let mut d = Debugger::new();
    d.pause();
    d.resume();
    assert_eq!(d.state(), DebugState::Running);
    assert!(d.pause_reason().is_none());
}

#[test]
fn debugger_continue_execution() {
    let mut d = Debugger::new();
    d.pause();
    d.continue_execution();
    assert_eq!(d.state(), DebugState::Running);
}

#[test]
fn debugger_step_over() {
    let mut d = Debugger::new();
    d.step_over();
    assert_eq!(d.state(), DebugState::Stepping);
}

#[test]
fn debugger_step_into() {
    let mut d = Debugger::new();
    d.step_into();
    assert_eq!(d.state(), DebugState::Stepping);
}

#[test]
fn debugger_step_out() {
    let mut d = Debugger::new();
    d.step_out();
    assert_eq!(d.state(), DebugState::Stepping);
}

#[test]
fn debugger_step_with_mode() {
    let mut d = Debugger::new();
    d.step(StepMode::Over);
    assert_eq!(d.state(), DebugState::Stepping);
    d.step(StepMode::Into);
    assert_eq!(d.state(), DebugState::Stepping);
    d.step(StepMode::Out);
    assert_eq!(d.state(), DebugState::Stepping);
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — on_statement (breakpoint hit)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_on_statement_hits_breakpoint() {
    let mut d = Debugger::new();
    d.add_breakpoint("test.hudhud".into(), 5);
    let paused = d.on_statement("test.hudhud", 5);
    assert!(paused);
    assert_eq!(d.state(), DebugState::Paused);
}

#[test]
fn debugger_on_statement_no_breakpoint() {
    let mut d = Debugger::new();
    let paused = d.on_statement("test.hudhud", 5);
    assert!(!paused);
}

#[test]
fn debugger_on_statement_disabled_breakpoint() {
    let mut d = Debugger::new();
    let id = d.add_breakpoint("test.hudhud".into(), 5);
    d.toggle_breakpoint(id); // disable
    let paused = d.on_statement("test.hudhud", 5);
    assert!(!paused);
}

#[test]
fn debugger_on_statement_step_into_pauses() {
    let mut d = Debugger::new();
    d.step_into();
    let paused = d.on_statement("test.hudhud", 1);
    assert!(paused);
    assert_eq!(d.pause_reason(), Some(&PauseReason::Step));
}

#[test]
fn debugger_on_statement_extended_logpoint() {
    let mut d = Debugger::new();
    d.add_logpoint("f.hudhud".into(), 3, "value = {x}".into());
    let action = d.on_statement_extended("f.hudhud", 3);
    assert!(!action.should_pause);
    assert_eq!(action.log_messages.len(), 1);
    assert!(action.log_messages[0].contains("value"));
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — on_exception
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_on_exception_break_all() {
    let mut d = Debugger::new();
    d.set_break_on_all_exceptions(true);
    assert!(d.on_exception("TypeError", "bad type"));
    assert_eq!(d.state(), DebugState::Paused);
}

#[test]
fn debugger_on_exception_no_break() {
    let mut d = Debugger::new();
    assert!(!d.on_exception("TypeError", "bad type"));
}

#[test]
fn debugger_on_exception_filtered() {
    let mut d = Debugger::new();
    d.add_exception_breakpoint(Some("TypeError".into()));
    assert!(d.on_exception("TypeError", "bad type"));
    assert!(!d.on_exception("RangeError", "out of range"));
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — call stack
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_push_pop_frame() {
    let mut d = Debugger::new();
    assert_eq!(d.call_depth(), 0);
    d.push_frame("main".into());
    assert_eq!(d.call_depth(), 1);
    assert_eq!(d.call_stack(), vec!["main"]);
    d.push_frame("helper".into());
    assert_eq!(d.call_depth(), 2);
    d.pop_frame();
    assert_eq!(d.call_depth(), 1);
    d.pop_frame();
    assert_eq!(d.call_depth(), 0);
}

#[test]
fn debugger_call_frames() {
    let mut d = Debugger::new();
    d.on_statement("f.hudhud", 1);
    d.push_frame("myFunc".into());
    let frames = d.call_frames();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].name, "myFunc");
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — watch expressions
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_add_watch() {
    let mut d = Debugger::new();
    d.add_watch("x + y".into());
    assert_eq!(d.watch_expressions().len(), 1);
}

#[test]
fn debugger_remove_watch() {
    let mut d = Debugger::new();
    d.add_watch("x".into());
    assert!(d.remove_watch("x"));
    assert!(d.watch_expressions().is_empty());
}

#[test]
fn debugger_remove_watch_nonexistent() {
    let mut d = Debugger::new();
    assert!(!d.remove_watch("y"));
}

#[test]
fn debugger_update_watch() {
    let mut d = Debugger::new();
    d.add_watch("x".into());
    d.update_watch("x", "42".into());
    assert_eq!(d.get_watch_value("x"), Some("42"));
}

#[test]
fn debugger_get_watch_value_none() {
    let d = Debugger::new();
    assert!(d.get_watch_value("z").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// Debugger — scope variables
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debugger_scope_variables() {
    use hudhudscript_debug::ScopeVariable;
    let mut d = Debugger::new();
    d.set_scope_variables(vec![ScopeVariable {
        name: "x".into(),
        value: "42".into(),
        ty: "number".into(),
    }]);
    assert_eq!(d.scope_variables().len(), 1);
    assert!(d.inspect("x").is_some());
    assert!(d.inspect("y").is_none());
}

#[test]
fn debugger_clear_scope_variables() {
    use hudhudscript_debug::ScopeVariable;
    let mut d = Debugger::new();
    d.set_scope_variables(vec![ScopeVariable {
        name: "x".into(),
        value: "1".into(),
        ty: "number".into(),
    }]);
    d.clear_scope_variables();
    assert!(d.scope_variables().is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Breakpoint
// ═══════════════════════════════════════════════════════════════════

#[test]
fn breakpoint_new() {
    let bp = Breakpoint::new(1, "f.hudhud".into(), 10);
    assert_eq!(bp.id, 1);
    assert_eq!(bp.file, "f.hudhud");
    assert_eq!(bp.line, 10);
    assert!(bp.enabled);
    assert_eq!(bp.hit_count, 0);
    assert_eq!(bp.kind, BreakpointKind::Normal);
}

#[test]
fn breakpoint_with_condition() {
    let bp = Breakpoint::new(1, "f.hudhud".into(), 5).with_condition("x > 0".into());
    assert!(bp.is_conditional());
    assert_eq!(bp.condition, Some("x > 0".into()));
}

#[test]
fn breakpoint_with_log_message() {
    let bp = Breakpoint::new(1, "f.hudhud".into(), 5).with_log_message("val={x}".into());
    assert!(bp.is_logpoint());
    assert!(!bp.is_conditional());
}

#[test]
fn breakpoint_as_exception() {
    let bp = Breakpoint::new(1, "f.hudhud".into(), 0).as_exception(Some("Error".into()));
    assert!(bp.is_exception());
}

#[test]
fn breakpoint_with_kind() {
    let bp = Breakpoint::new(1, "f".into(), 1).with_kind(BreakpointKind::Normal);
    assert_eq!(bp.kind, BreakpointKind::Normal);
}

#[test]
fn breakpoint_record_hit() {
    let mut bp = Breakpoint::new(1, "f.hudhud".into(), 1);
    assert_eq!(bp.record_hit(), 1);
    assert_eq!(bp.record_hit(), 2);
    assert_eq!(bp.hit_count, 2);
}

// ═══════════════════════════════════════════════════════════════════
// Profiler
// ═══════════════════════════════════════════════════════════════════

#[test]
fn profiler_new_empty() {
    let p = Profiler::new();
    assert_eq!(p.sample_count(), 0);
}

#[test]
fn profiler_default() {
    let p = Profiler::default();
    assert_eq!(p.sample_count(), 0);
}

#[test]
fn profiler_record() {
    let mut p = Profiler::new();
    p.record("test_fn".into(), Duration::from_millis(50));
    assert_eq!(p.sample_count(), 1);
}

#[test]
fn profiler_report() {
    let mut p = Profiler::new();
    p.record("a".into(), Duration::from_millis(100));
    p.record("b".into(), Duration::from_millis(200));
    let report = p.report();
    assert_eq!(report.samples.len(), 2);
    assert_eq!(report.total_duration, Duration::from_millis(300));
}

#[test]
fn profiler_report_average() {
    let mut p = Profiler::new();
    p.record("a".into(), Duration::from_millis(100));
    p.record("b".into(), Duration::from_millis(300));
    let report = p.report();
    assert_eq!(report.average_duration(), Duration::from_millis(200));
}

#[test]
fn profiler_report_empty_average() {
    let p = Profiler::new();
    assert_eq!(p.report().average_duration(), Duration::ZERO);
}

#[test]
fn profiler_clear() {
    let mut p = Profiler::new();
    p.record("x".into(), Duration::from_secs(1));
    p.clear();
    assert_eq!(p.sample_count(), 0);
    assert_eq!(p.report().total_duration, Duration::ZERO);
}

#[test]
fn profiler_start() {
    let mut p = Profiler::new();
    p.start();
    // start should not panic
}

// ═══════════════════════════════════════════════════════════════════
// DapServer
// ═══════════════════════════════════════════════════════════════════

#[test]
fn dap_server_new() {
    let _server = DapServer::new();
}

// ═══════════════════════════════════════════════════════════════════
// DapMessage variants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn dap_request_create() {
    let req = DapRequest {
        seq: 1,
        command: "initialize".into(),
        arguments: None,
    };
    assert_eq!(req.seq, 1);
    assert_eq!(req.command, "initialize");
}

#[test]
fn dap_response_create() {
    let resp = DapResponse {
        seq: 1,
        request_seq: 1,
        success: true,
        command: "initialize".into(),
        message: None,
        body: None,
    };
    assert!(resp.success);
}

#[test]
fn dap_event_create() {
    let evt = DapEvent {
        seq: 1,
        event: "stopped".into(),
        body: None,
    };
    assert_eq!(evt.event, "stopped");
}

#[test]
fn dap_message_request_variant() {
    let msg = DapMessage::Request(DapRequest {
        seq: 1,
        command: "launch".into(),
        arguments: None,
    });
    assert!(matches!(msg, DapMessage::Request(_)));
}

#[test]
fn dap_message_response_variant() {
    let msg = DapMessage::Response(DapResponse {
        seq: 1,
        request_seq: 1,
        success: true,
        command: "launch".into(),
        message: None,
        body: None,
    });
    assert!(matches!(msg, DapMessage::Response(_)));
}

#[test]
fn dap_message_event_variant() {
    let msg = DapMessage::Event(DapEvent {
        seq: 1,
        event: "initialized".into(),
        body: None,
    });
    assert!(matches!(msg, DapMessage::Event(_)));
}

// ═══════════════════════════════════════════════════════════════════
// Variable / Source
// ═══════════════════════════════════════════════════════════════════

#[test]
fn variable_create() {
    let v = Variable {
        name: "x".into(),
        value: "42".into(),
        ty: "number".into(),
        variables_reference: 0,
    };
    assert_eq!(v.name, "x");
    assert_eq!(v.value, "42");
    assert_eq!(v.ty, "number");
}

#[test]
fn variable_clone() {
    let v = Variable {
        name: "y".into(),
        value: "true".into(),
        ty: "boolean".into(),
        variables_reference: 0,
    };
    let v2 = v.clone();
    assert_eq!(v2.name, "y");
}

// ═══════════════════════════════════════════════════════════════════
// DebugState / PauseReason / StepMode equality
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debug_state_eq() {
    assert_eq!(DebugState::Running, DebugState::Running);
    assert_eq!(DebugState::Paused, DebugState::Paused);
    assert_ne!(DebugState::Running, DebugState::Paused);
}

#[test]
fn pause_reason_eq() {
    assert_eq!(PauseReason::Step, PauseReason::Step);
    assert_eq!(PauseReason::Explicit, PauseReason::Explicit);
    assert_ne!(PauseReason::Step, PauseReason::Explicit);
}

#[test]
fn step_mode_eq() {
    assert_eq!(StepMode::Over, StepMode::Over);
    assert_eq!(StepMode::Into, StepMode::Into);
    assert_eq!(StepMode::Out, StepMode::Out);
    assert_ne!(StepMode::Over, StepMode::Into);
}
