use hudhudscript_bytecode::Value16;

fn daemon_pid(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::daemon_ops::dispatch("pid", args)
}
fn daemon_is_running(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::daemon_ops::dispatch("is_running", args)
}
fn daemon_write_pid(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::daemon_ops::dispatch("write_pid", args)
}
fn daemon_remove_pid(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::daemon_ops::dispatch("remove_pid", args)
}

#[test]
fn test_daemon_pid() {
    let result = daemon_pid(&[]).unwrap();
    if let Some(pid) = result.as_number() {
        assert!(pid > 0.0);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_daemon_is_running_self() {
    let pid = std::process::id() as f64;
    let result = daemon_is_running(&[Value16::number(pid)]).unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_daemon_is_running_nonexistent() {
    // PID 99999999 is very unlikely to exist
    let result = daemon_is_running(&[Value16::number(99999999.0)]).unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn test_daemon_write_and_remove_pid() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    drop(tmp);

    daemon_write_pid(&[Value16::string(path.clone())]).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, std::process::id().to_string());

    daemon_remove_pid(&[Value16::string(path.clone())]).unwrap();
    assert!(!std::path::Path::new(&path).exists());
}
