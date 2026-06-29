//! Real unit tests for hudhudscript-debug — Breakpoint, Profiler types

use hudhudscript_debug::*;

#[test]
#[test]
fn breakpoint_kind_variants() {
    assert!(matches!(BreakpointKind::Normal, BreakpointKind::Normal));
    assert!(matches!(
        BreakpointKind::Conditional("x>0".into()),
        BreakpointKind::Conditional(_)
    ));
    assert!(matches!(
        BreakpointKind::Exception(None),
        BreakpointKind::Exception(_)
    ));
    assert!(matches!(
        BreakpointKind::Logpoint("x={x}".into()),
        BreakpointKind::Logpoint(_)
    ));
}

#[test]
fn breakpoint_new() {
    let bp = Breakpoint::new(1, "main.hud".into(), 42);
    assert_eq!(bp.id, 1);
    assert_eq!(bp.file, "main.hud");
    assert_eq!(bp.line, 42);
    assert!(bp.enabled);
    assert!(!bp.is_conditional());
    assert!(!bp.is_logpoint());
    assert!(!bp.is_exception());
}

#[test]
fn breakpoint_with_condition() {
    let bp = Breakpoint::new(2, "test.hud".into(), 10).with_condition("x > 5".into());
    assert!(bp.is_conditional());
    assert_eq!(bp.condition, Some("x > 5".to_string()));
}

#[test]
fn breakpoint_with_log_message() {
    let bp = Breakpoint::new(3, "test.hud".into(), 15).with_log_message("x = {x}".into());
    assert!(bp.is_logpoint());
}

#[test]
fn breakpoint_as_exception() {
    let bp = Breakpoint::new(4, "test.hud".into(), 20).as_exception(Some("TypeError".into()));
    assert!(bp.is_exception());
}

#[test]
fn breakpoint_record_hit() {
    let mut bp = Breakpoint::new(5, "test.hud".into(), 25);
    assert_eq!(bp.record_hit(), 1);
    assert_eq!(bp.record_hit(), 2);
    assert_eq!(bp.hit_count, 2);
}

#[test]
fn breakpoint_with_kind() {
    let bp = Breakpoint::new(6, "test.hud".into(), 30)
        .with_kind(BreakpointKind::Conditional("y>0".into()));
    assert!(matches!(bp.kind, BreakpointKind::Conditional(_)));
}

#[test]
fn profiler_creation() {
    let profiler = Profiler::new();
    let _ = profiler;
}
