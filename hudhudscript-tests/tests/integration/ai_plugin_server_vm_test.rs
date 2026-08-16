//! Batch 10: AI, Plugin & Server VM tests
//! Tests for EventBus (#597), Plugin (#598), McpServer (#600),
//! Server (#602), PluginConfig (#610).

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn vm_run(code: &str, var: &str) -> (hudhudscript_vm::VM, hudhudscript_bytecode::Value16) {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("VM execution failed");
    let val = vm
        .get_variable(var)
        .cloned()
        .map(|v| v)
        .unwrap_or_else(|| panic!("variable \'{}\' not found", var));
    (vm, val)
}

fn vm_run_ok(code: &str) {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("VM execution failed");
}

fn assert_bool((_vm, val): (hudhudscript_vm::VM, hudhudscript_bytecode::Value16), expected: bool) {
    if let Some(b) = val.as_bool() {
        assert_eq!(b, expected);
    } else {
        panic!("Expected Boolean({}), got {:?}", expected, val);
    }
}

fn assert_number((_vm, val): (hudhudscript_vm::VM, hudhudscript_bytecode::Value16), expected: f64) {
    if let Some(n) = val.as_number() {
        assert!(
            (n - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            n
        );
    } else {
        panic!("Expected Number({}), got {:?}", expected, val);
    }
}

fn assert_obj_bool(val: &(hudhudscript_vm::VM, Value16), key: &str, expected: bool) {
    if let Some(obj) = val.1.as_object() {
        match obj.get(key).and_then(|v| v.as_bool()) {
            Some(b) => assert_eq!(
                b, expected,
                "key '{}': expected {}, got {}",
                key, expected, b
            ),
            other => panic!("key '{}': expected Boolean, got {:?}", key, other),
        }
    } else {
        panic!("Expected Object, got {:?}", val.1);
    }
}

fn assert_obj_string(val: &(hudhudscript_vm::VM, Value16), key: &str, expected: &str) {
    if let Some(obj) = val.1.as_object() {
        match obj.get(key).and_then(|v| v.as_str()) {
            Some(s) => assert_eq!(
                s, expected,
                "key '{}': expected '{}', got '{}'",
                key, expected, s
            ),
            other => panic!("key '{}': expected String, got {:?}", key, other),
        }
    } else {
        panic!("Expected Object, got {:?}", val.1);
    }
}

fn assert_obj_number(val: &(hudhudscript_vm::VM, Value16), key: &str, expected: f64) {
    if let Some(obj) = val.1.as_object() {
        match obj.get(key) {
            Some(v) => {
                if let Some(n) = v.as_number() {
                    assert!(
                        (n - expected).abs() < 0.001,
                        "key '{}': expected {}, got {}",
                        key,
                        expected,
                        n
                    );
                } else {
                    panic!("key '{}': expected Number, got {:?}", key, v);
                }
            }
            None => panic!("key '{}': not found", key),
        }
    } else {
        panic!("Expected Object, got {:?}", val.1);
    }
}

// ── EventBus (#597) ─────────────────────────────────────────────────────

#[test]
fn test_vm_event_bus_emit() {
    let _guard = crate::EVENT_BUS_LOCK.lock().unwrap();
    // Register a listener first so emit has someone to deliver to
    let val = vm_run(
        r#"EventBus.on("user.created", "handler");
var result = EventBus.emit("user.created");"#,
        "result",
    );
    assert_obj_string(&val, "event", "user.created");
    assert_obj_bool(&val, "delivered", true);
}

#[test]
fn test_vm_event_bus_on() {
    let _guard = crate::EVENT_BUS_LOCK.lock().unwrap();
    let val = vm_run(
        r#"var result = EventBus.on("user.*", "handler");"#,
        "result",
    );
    assert_obj_string(&val, "pattern", "user.*");
    assert_obj_bool(&val, "active", true);
    assert_obj_bool(&val, "once", false);
}

#[test]
fn test_vm_event_bus_once() {
    let _guard = crate::EVENT_BUS_LOCK.lock().unwrap();
    let val = vm_run(r#"var result = EventBus.once("shutdown");"#, "result");
    assert_obj_bool(&val, "once", true);
}

#[test]
fn test_vm_event_bus_off() {
    let _guard = crate::EVENT_BUS_LOCK.lock().unwrap();
    // Subscribe first, then unsubscribe by the returned subscription id
    let val = vm_run(
        r#"var sub = EventBus.on("test.event", "handler");
var result = EventBus.off(sub.id);"#,
        "result",
    );
    assert_bool(val, true);
}

#[test]
fn test_vm_event_bus_channels() {
    let _guard = crate::EVENT_BUS_LOCK.lock().unwrap();
    let val = vm_run(r#"var result = EventBus.channels();"#, "result");
    assert!(val.1.is_array());
}

// ── Plugin (#598) ───────────────────────────────────────────────────────

#[test]
fn test_vm_plugin_register() /* uses register-plugin */
{
    let val = vm_run(
        r#"var result = Plugin.register({ name: "register-plugin", version: "1.0.0" });"#,
        "result",
    );
    assert_obj_string(&val, "name", "register-plugin");
    assert_obj_string(&val, "version", "1.0.0");
    assert_obj_bool(&val, "loaded", true);
}

#[test]
fn test_vm_plugin_unregister() {
    /* uses unregister-plugin */
    // Register the plugin first, then unregister it
    let val = vm_run(
        r#"Plugin.register({ name: "unregister-plugin", version: "1.0.0" });
var result = Plugin.unregister("unregister-plugin");"#,
        "result",
    );
    assert_bool(val, true);
}

#[test]
fn test_vm_plugin_list() {
    let val = vm_run(r#"var result = Plugin.list();"#, "result");
    assert!(val.1.is_array());
}

#[test]
fn test_vm_plugin_reload() {
    /* uses reload-plugin */
    // Register the plugin first, then reload it
    let val = vm_run(
        r#"Plugin.register({ name: "reload-plugin", version: "1.0.0" });
var result = Plugin.reload("reload-plugin");"#,
        "result",
    );
    assert_obj_bool(&val, "reloaded", true);
}

#[test]
fn test_vm_plugin_create_isolated() {
    let val = vm_run(
        r#"var result = Plugin.create({ name: "sandbox-plugin", isolated: true });"#,
        "result",
    );
    assert_obj_bool(&val, "isolated", true);
    assert_obj_bool(&val, "loaded", true);
}

// ── McpServer (#600) ────────────────────────────────────────────────────

#[test]
fn test_vm_mcp_server_create() {
    let val = vm_run(r#"var result = McpServer.create("my-server");"#, "result");
    assert_obj_string(&val, "name", "my-server");
    assert_obj_string(&val, "transport", "stdio");
    assert_obj_bool(&val, "running", false);
}

#[test]
fn test_vm_mcp_server_create_sse() {
    let val = vm_run(
        r#"var result = McpServer.create({ name: "sse-server", transport: "sse", port: 3000 });"#,
        "result",
    );
    assert_obj_string(&val, "transport", "sse");
    assert_obj_number(&val, "port", 3000.0);
}

#[test]
fn test_vm_mcp_server_add_tool() {
    // Create the MCP server first, then add a tool
    let val = vm_run(
        r#"McpServer.create("my-server");
var result = McpServer.add_tool({ name: "greet", description: "Greet" });"#,
        "result",
    );
    assert_obj_string(&val, "name", "greet");
    assert_obj_bool(&val, "registered", true);
}

#[test]
fn test_vm_mcp_server_start_stop() {
    let val = vm_run(r#"var result = McpServer.start();"#, "result");
    assert_obj_bool(&val, "running", true);

    let val2 = vm_run(r#"var result = McpServer.stop();"#, "result");
    assert_obj_bool(&val2, "running", false);
}

#[test]
fn test_vm_mcp_server_status() {
    let val = vm_run(r#"var result = McpServer.status();"#, "result");
    assert_obj_string(&val, "protocol_version", "2024-11-05");
}

// ── HTTP Server (#602) ──────────────────────────────────────────────────

#[test]
fn test_vm_server_create() {
    let val = vm_run(r#"var result = Server.create({ port: 3000 });"#, "result");
    assert_obj_number(&val, "port", 3000.0);
    assert_obj_bool(&val, "running", false);
}

#[test]
fn test_vm_server_get_route() {
    let val = vm_run(r#"var result = Server.get("/api/health");"#, "result");
    assert_obj_string(&val, "method", "GET");
    assert_obj_string(&val, "path", "/api/health");
}

#[test]
fn test_vm_server_post_route() {
    let val = vm_run(
        r#"var result = Server.post("/api/users", "create");"#,
        "result",
    );
    assert_obj_string(&val, "method", "POST");
    assert_obj_string(&val, "handler", "create");
}

#[test]
fn test_vm_server_all_methods() {
    vm_run_ok(
        r#"
        var g = Server.get("/a");
        var p = Server.post("/b");
        var u = Server.put("/c");
        var d = Server.delete("/d");
        var pa = Server.patch("/e");
    "#,
    );
}

#[test]
fn test_vm_server_middleware() {
    let val = vm_run(
        r#"var result = Server.middleware("auth", { secret: "key123" });"#,
        "result",
    );
    assert_obj_string(&val, "name", "auth");
    assert_obj_bool(&val, "enabled", true);
}

#[test]
#[ignore = "requires HUDHUD_REAL_NETWORK_TESTS=1 and localhost bind permission"]
fn test_vm_server_listen_stop() {
    let val = vm_run(r#"var result = Server.listen(4000);"#, "result");
    assert_obj_bool(&val, "listening", true);
    assert_obj_number(&val, "port", 4000.0);

    let val2 = vm_run(r#"var result = Server.stop();"#, "result");
    assert_obj_bool(&val2, "stopped", true);
}

#[test]
fn test_vm_server_static_files() {
    let val = vm_run(r#"var result = Server.static_files("./public");"#, "result");
    assert_obj_string(&val, "directory", "./public");
    assert_obj_string(&val, "type", "static");
}

#[test]
fn test_vm_server_websocket() {
    let val = vm_run(r#"var result = Server.websocket("/ws");"#, "result");
    assert_obj_string(&val, "type", "websocket");
    assert_obj_string(&val, "path", "/ws");
}

// ── PluginConfig (#610) ─────────────────────────────────────────────────

#[test]
fn test_vm_plugin_config_load() {
    let val = vm_run(
        r#"var result = PluginConfig.load("test-plugin");"#,
        "result",
    );
    assert_obj_string(&val, "__plugin", "test-plugin");
}

#[test]
fn test_vm_plugin_config_get_set() {
    let val = vm_run(
        r#"
        var config = PluginConfig.load("test");
        var config2 = PluginConfig.set(config, "port", 3000);
        var result = PluginConfig.get(config2, "port");
        "#,
        "result",
    );
    assert_number(val, 3000.0);
}

#[test]
fn test_vm_plugin_config_merge() {
    let val = vm_run(
        r#"
        var base = { a: 1, b: 2 };
        var overlay = { b: 20, c: 3 };
        var result = PluginConfig.merge(base, overlay);
        "#,
        "result",
    );
    assert_obj_number(&val, "b", 20.0);
    assert_obj_number(&val, "c", 3.0);
}

#[test]
fn test_vm_plugin_config_defaults() {
    let val = vm_run(
        r#"var result = PluginConfig.defaults("reload-plugin-cfg", { timeout: 30 });"#,
        "result",
    );
    assert_obj_number(&val, "timeout", 30.0);
    assert_obj_bool(&val, "__defaults_applied", true);
}

#[test]
fn test_vm_plugin_config_paths() {
    let val = vm_run(
        r#"var result = PluginConfig.paths("reload-plugin-cfg");"#,
        "result",
    );
    if let Some(obj) = val.1.as_object() {
        assert!(obj.contains_key("system"));
        assert!(obj.contains_key("user"));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_vm_plugin_config_watch() {
    let val = vm_run(
        r#"var result = PluginConfig.watch("/tmp/test.toml");"#,
        "result",
    );
    assert_obj_bool(&val, "watching", true);
}
