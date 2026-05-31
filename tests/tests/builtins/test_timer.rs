use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::schedule_ops::ScriptMethodId;
use hudhudscript_shared_builtins::timer_ops::{shared_clear_timer, shared_set_interval};

fn clear_timeout(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    shared_clear_timer(args, "clearTimeout")
}
fn set_interval(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    shared_set_interval(args)
}
fn parse_cron(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::schedule_ops::dispatch(ScriptMethodId::ParseCron, args)
}
fn schedule_cron(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::schedule_ops::dispatch(ScriptMethodId::Cron, args)
}

#[test]
fn test_clear_timeout_null() {
    let result = clear_timeout(&[Value16::number(1.0)]).unwrap();
    assert_eq!(result, Value16::null());
}

#[test]
fn test_set_interval_returns_descriptor() {
    let result = set_interval(&[Value16::null(), Value16::number(100.0)]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(
            obj.get("type"),
            Some(&Value16::string("interval".to_string()))
        );
        assert_eq!(obj.get("ms"), Some(&Value16::number(100.0)));
        assert_eq!(obj.get("active"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_parse_cron() {
    let result = parse_cron(&[Value16::string("*/5 * * * *".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("minute"), Some(&Value16::string("*/5".to_string())));
        assert_eq!(obj.get("hour"), Some(&Value16::string("*".to_string())));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_parse_cron_invalid() {
    let result = parse_cron(&[Value16::string("* *".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_schedule_cron() {
    let result = schedule_cron(&[Value16::string("0 9 * * 1-5".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("type"), Some(&Value16::string("cron".to_string())));
        assert_eq!(
            obj.get("expression"),
            Some(&Value16::string("0 9 * * 1-5".to_string()))
        );
    } else {
        panic!("Expected object");
    }
}
