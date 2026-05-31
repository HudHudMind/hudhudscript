use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::env_ops::EnvMethodId;
use hudhudscript_shared_builtins::os_ops::OsMethodId;

fn env_get(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    EnvMethodId::Get.dispatch(args)
}
fn env_set(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    EnvMethodId::Set.dispatch(args)
}
fn env_has(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    EnvMethodId::Has.dispatch(args)
}
fn env_remove(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    EnvMethodId::Remove.dispatch(args)
}
fn env_all(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    EnvMethodId::All.dispatch(args)
}
fn os_name(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    OsMethodId::Name.dispatch(args)
}
fn os_arch(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    OsMethodId::Arch.dispatch(args)
}
fn os_hostname(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    OsMethodId::Hostname.dispatch(args)
}
fn os_cpus(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    OsMethodId::Cpus.dispatch(args)
}
fn os_pid(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    OsMethodId::Pid.dispatch(args)
}

#[test]
fn test_env_get_set_remove() {
    env_set(&[
        Value16::string("_HUDHUD_ENV_TEST_".to_string()),
        Value16::string("test_value".to_string()),
    ])
    .unwrap();

    let result = env_get(&[Value16::string("_HUDHUD_ENV_TEST_".to_string())]).unwrap();
    assert_eq!(result, Value16::string("test_value".to_string()));

    let has = env_has(&[Value16::string("_HUDHUD_ENV_TEST_".to_string())]).unwrap();
    assert_eq!(has, Value16::boolean(true));

    env_remove(&[Value16::string("_HUDHUD_ENV_TEST_".to_string())]).unwrap();

    let result = env_get(&[Value16::string("_HUDHUD_ENV_TEST_".to_string())]).unwrap();
    assert_eq!(result, Value16::null());
}

#[test]
fn test_env_get_default() {
    let result = env_get(&[
        Value16::string("_HUDHUD_NONEXISTENT_".to_string()),
        Value16::string("default_val".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::string("default_val".to_string()));
}

#[test]
fn test_os_name() {
    let result = os_name(&[]).unwrap();
    assert_eq!(result, Value16::string("linux".to_string()));
}

#[test]
fn test_os_arch() {
    let result = os_arch(&[]).unwrap();
    if let Some(arch) = result.as_str() {
        assert!(!arch.is_empty());
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_os_hostname() {
    let result = os_hostname(&[]).unwrap();
    if let Some(h) = result.as_str() {
        assert!(!h.is_empty());
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_os_cpus() {
    let result = os_cpus(&[]).unwrap();
    if let Some(n) = result.as_number() {
        assert!(n >= 1.0);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_os_pid() {
    let result = os_pid(&[]).unwrap();
    if let Some(n) = result.as_number() {
        assert!(n > 0.0);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_env_all() {
    let result = env_all(&[]).unwrap();
    if let Some(obj) = result.as_object() {
        assert!(!obj.is_empty());
    } else {
        panic!("Expected object");
    }
}
