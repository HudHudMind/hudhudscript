use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::plugin_ops as plugin_impl;
use std::collections::HashMap;

fn plugin_register(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    plugin_impl::plugin_register(args)
}
fn plugin_unregister(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    plugin_impl::plugin_unregister(args)
}
fn plugin_list(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    plugin_impl::plugin_list(args)
}
fn plugin_reload(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    plugin_impl::plugin_reload(args)
}
fn plugin_create(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    plugin_impl::plugin_create(args)
}

#[test]
fn test_register_plugin() {
    let mut opts = HashMap::new();
    opts.insert("name".to_string(), Value16::string("my-plugin".to_string()));
    opts.insert("version".to_string(), Value16::string("1.0.0".to_string()));
    opts.insert(
        "capabilities".to_string(),
        Value16::array(vec![Value16::string("http".to_string())]),
    );

    let result = plugin_register(&[Value16::object(opts)]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(
            obj.get("name"),
            Some(&Value16::string("my-plugin".to_string()))
        );
        assert_eq!(
            obj.get("version"),
            Some(&Value16::string("1.0.0".to_string()))
        );
        assert_eq!(obj.get("loaded"), Some(&Value16::boolean(true)));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_register_requires_name() {
    let opts: std::collections::HashMap<hudhudscript_bytecode::sym::SymId, hudhudscript_bytecode::Value16> = std::collections::HashMap::new();
    assert!(plugin_register(&[Value16::object(opts)]).is_err());
}

#[test]
fn test_unregister() {
    assert!(plugin_unregister(&[Value16::string("my-plugin".to_string())]).is_ok());
    assert!(plugin_unregister(&[Value16::number(42.0)]).is_err());
}

#[test]
fn test_list_returns_array() {
    let result = plugin_list(&[]).unwrap();
    assert!(result.as_array().is_some());
}

#[test]
fn test_reload() {
    // Register first, then reload (reload requires the plugin to exist)
    let mut opts = std::collections::HashMap::new();
    opts.insert(
        "name".to_string(),
        Value16::string("reload-test-plugin".to_string()),
    );
    opts.insert("version".to_string(), Value16::string("1.0.0".to_string()));
    plugin_register(&[Value16::object(opts)]).unwrap();

    let result = plugin_reload(&[Value16::string("reload-test-plugin".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("reloaded"), Some(&Value16::boolean(true)));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_create_with_isolation() {
    let mut opts = HashMap::new();
    opts.insert(
        "name".to_string(),
        Value16::string("sandbox-plugin".to_string()),
    );
    opts.insert("isolated".to_string(), Value16::boolean(true));

    let result = plugin_create(&[Value16::object(opts)]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("isolated"), Some(&Value16::boolean(true)));
    } else {
        panic!("expected object");
    }
}
