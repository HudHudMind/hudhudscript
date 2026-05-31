//! Batch 10: AI, Plugin & Server tests
//! Tests for EventBus (#597), Plugin (#598), McpServer (#600),
//! Server (#602), PluginConfig (#610).

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(src: &str) -> Value16 {
    let ast = parse(src).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
    interp.get_variable("result").unwrap_or(Value16::null())
}

fn run_ok(src: &str) {
    let ast = parse(src).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
}

// ── EventBus (#597) ─────────────────────────────────────────────────────

#[test]
fn test_event_bus_emit() {
    // Clear any state from previous tests, then emit
    let src = r#"
        EventBus.clear();
        var result = EventBus.emit("user.created", { name: "Alice" });
    "#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("event"),
            Some(&Value16::string("user.created".to_string()))
        );
        // No subscribers after clear → delivered is false
        assert_eq!(obj.get("delivered"), Some(&Value16::boolean(false)));
    } else {
        panic!("Expected object, got {:?}", val)
    }
}

#[test]
fn test_event_bus_on() {
    let src = r#"var result = EventBus.on("user.*", "handle_user");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("pattern"),
            Some(&Value16::string("user.*".to_string()))
        );
        assert_eq!(obj.get("active"), Some(&Value16::boolean(true)));
        assert_eq!(obj.get("once"), Some(&Value16::boolean(false)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_event_bus_once() {
    let src = r#"var result = EventBus.once("shutdown");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("once"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_event_bus_off() {
    // off() with a nonexistent subscription ID → false (real behavior)
    let src = r#"var result = EventBus.off("sub_123");"#;
    assert_eq!(run(src), Value16::boolean(false));
}

#[test]
fn test_event_bus_has_listeners() {
    let src = r#"var result = EventBus.has_listeners("test");"#;
    assert_eq!(run(src), Value16::boolean(false));
}

#[test]
fn test_event_bus_channels() {
    let src = r#"var result = EventBus.channels();"#;
    let val = run(src);
    assert!(val.is_array());
}

// ── Plugin (#598) ───────────────────────────────────────────────────────

#[test]
fn test_plugin_register() {
    let src = r#"var result = Plugin.register({ name: "my-plugin", version: "1.0.0" });"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
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
        panic!("Expected object")
    }
}

#[test]
fn test_plugin_unregister() {
    // Unregistering a non-registered plugin → false (real behavior)
    let src = r#"var result = Plugin.unregister("nonexistent-plugin");"#;
    assert_eq!(run(src), Value16::boolean(false));
}

#[test]
fn test_plugin_list() {
    let src = r#"var result = Plugin.list();"#;
    assert!(run(src).is_array());
}

#[test]
fn test_plugin_reload() {
    // Register first, then reload (reload requires the plugin to exist)
    let src = r#"
        Plugin.register({ name: "reload-test", version: "1.0" });
        var result = Plugin.reload("reload-test");
    "#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("reloaded"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object, got {:?}", val)
    }
}

#[test]
fn test_plugin_enable_disable() {
    run_ok(
        r#"
        var e = Plugin.enable("my-plugin");
        var d = Plugin.disable("my-plugin");
    "#,
    );
}

#[test]
fn test_plugin_create_isolated() {
    let src = r#"var result = Plugin.create({ name: "sandbox-plugin", isolated: true });"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("isolated"), Some(&Value16::boolean(true)));
        assert_eq!(obj.get("loaded"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object")
    }
}

// ── McpServer (#600) ────────────────────────────────────────────────────

#[test]
fn test_mcp_server_create() {
    let src = r#"var result = McpServer.create("my-server");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("name"),
            Some(&Value16::string("my-server".to_string()))
        );
        assert_eq!(
            obj.get("transport"),
            Some(&Value16::string("stdio".to_string()))
        );
        assert_eq!(obj.get("running"), Some(&Value16::boolean(false)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_mcp_server_create_sse() {
    let src =
        r#"var result = McpServer.create({ name: "sse-server", transport: "sse", port: 3000 });"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("transport"),
            Some(&Value16::string("sse".to_string()))
        );
        assert_eq!(obj.get("port"), Some(&Value16::number(3000.0)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_mcp_server_add_tool() {
    let src = r#"
        McpServer.create("test-server");
        var result = McpServer.add_tool({ name: "greet", description: "Greet a user" });
    "#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("name"), Some(&Value16::string("greet".to_string())));
        assert_eq!(obj.get("registered"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_mcp_server_add_resource() {
    let src = r#"
        McpServer.create("test-server");
        var result = McpServer.add_resource({ uri: "file:///data.json", name: "data" });
    "#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("uri"),
            Some(&Value16::string("file:///data.json".to_string()))
        );
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_mcp_server_start_stop() {
    let src = r#"var result = McpServer.start();"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("running"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object")
    }

    let src2 = r#"var result = McpServer.stop();"#;
    let val2 = run(src2);
    if let Some(obj) = val2.as_object() {
        assert_eq!(obj.get("running"), Some(&Value16::boolean(false)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_mcp_server_status() {
    let src = r#"var result = McpServer.status();"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert!(obj.contains_key("protocol_version"));
    } else {
        panic!("Expected object")
    }
}

// ── HTTP Server (#602) ──────────────────────────────────────────────────

#[test]
fn test_server_create() {
    let src = r#"var result = Server.create({ port: 3000 });"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("port"), Some(&Value16::number(3000.0)));
        assert_eq!(obj.get("running"), Some(&Value16::boolean(false)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_server_routes() {
    run_ok(
        r#"
        var get_route = Server.get("/api/users", "list_users");
        var post_route = Server.post("/api/users", "create_user");
        var put_route = Server.put("/api/users/:id", "update_user");
        var del_route = Server.delete("/api/users/:id", "delete_user");
    "#,
    );
}

#[test]
fn test_server_get_route() {
    let src = r#"var result = Server.get("/api/health");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("method"), Some(&Value16::string("GET".to_string())));
        assert_eq!(
            obj.get("path"),
            Some(&Value16::string("/api/health".to_string()))
        );
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_server_middleware() {
    let src = r#"var result = Server.middleware("rate_limit", { max_requests: 100 });"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("name"),
            Some(&Value16::string("rate_limit".to_string()))
        );
        assert_eq!(obj.get("enabled"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_server_listen_stop() {
    let src = r#"var result = Server.listen(3000);"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("listening"), Some(&Value16::boolean(true)));
        assert_eq!(obj.get("port"), Some(&Value16::number(3000.0)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_server_static_files() {
    let src = r#"var result = Server.static_files("./public");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("directory"),
            Some(&Value16::string("./public".to_string()))
        );
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_server_websocket() {
    let src = r#"var result = Server.websocket("/ws", "on_message");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("type"),
            Some(&Value16::string("websocket".to_string()))
        );
        assert_eq!(obj.get("path"), Some(&Value16::string("/ws".to_string())));
    } else {
        panic!("Expected object")
    }
}

// ── PluginConfig (#610) ─────────────────────────────────────────────────

#[test]
fn test_plugin_config_load() {
    let src = r#"var result = PluginConfig.load("test-plugin");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("__plugin"),
            Some(&Value16::string("test-plugin".to_string()))
        );
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_plugin_config_get_set() {
    let src = r#"
        var config = PluginConfig.load("test-plugin");
        var config2 = PluginConfig.set(config, "port", 3000);
        var result = PluginConfig.get(config2, "port");
    "#;
    assert_eq!(run(src), Value16::number(3000.0));
}

#[test]
fn test_plugin_config_merge() {
    let src = r#"
        var base = { a: 1, b: 2 };
        var overlay = { b: 20, c: 3 };
        var result = PluginConfig.merge(base, overlay);
    "#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("b"), Some(&Value16::number(20.0)));
        assert_eq!(obj.get("c"), Some(&Value16::number(3.0)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_plugin_config_defaults() {
    let src = r#"var result = PluginConfig.defaults("my-plugin", { timeout: 30 });"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("timeout"), Some(&Value16::number(30.0)));
        assert_eq!(obj.get("__defaults_applied"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_plugin_config_paths() {
    let src = r#"var result = PluginConfig.paths("my-plugin");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert!(obj.contains_key("system"));
        assert!(obj.contains_key("user"));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_plugin_config_watch() {
    let src = r#"var result = PluginConfig.watch("/tmp/test.toml");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(obj.get("watching"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object")
    }
}
