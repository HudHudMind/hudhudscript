//! WebSocket builtin tests (Kural 7 follow-up).
//!
//! After the `hudhudscript-builtins` crate was deleted, the
//! `create_ws_module()`-based structural test
//! (`test_ws_module_has_all_methods`) lost its subject — that layout was
//! `Value::NativeFunction`-wrapped, i.e. interpreter-era infrastructure
//! with no shared counterpart. The end-to-end round-trip tests below
//! stay and continue to exercise the shared dispatcher, which is the
//! same code both runtimes call at runtime.

use hudhudscript_bytecode::Value16;

#[test]
fn test_ws_serve_and_connect() {
    // Start a local WS server through the shared dispatcher — the same
    // function path both runtimes hit from `ws.serve(...)`.
    let server = hudhudscript_shared_builtins::ws_ops::dispatch(
        "serve",
        &[
            Value16::string("127.0.0.1".to_string()),
            Value16::number(0.0),
        ],
    )
    .unwrap();

    let addr = if let Some(obj) = server.as_object() {
        match obj.get("address").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            _ => panic!("No address"),
        }
    } else {
        panic!("Expected object");
    };

    // Client thread: connect, send, receive, close.
    let addr_clone = addr.clone();
    let client_handle = std::thread::spawn(move || {
        let conn = hudhudscript_shared_builtins::ws_ops::dispatch(
            "connect",
            &[Value16::string(format!("ws://{}", addr_clone))],
        )
        .unwrap();
        hudhudscript_shared_builtins::ws_ops::dispatch(
            "send",
            &[conn.clone(), Value16::string("hello ws".to_string())],
        )
        .unwrap();
        let msg = hudhudscript_shared_builtins::ws_ops::dispatch("recv", &[conn.clone()]).unwrap();
        hudhudscript_shared_builtins::ws_ops::dispatch("close", &[conn]).unwrap();
        msg
    });

    // Server side: accept the incoming connection.
    let client_conn =
        hudhudscript_shared_builtins::ws_ops::dispatch("accept", &[server.clone()]).unwrap();

    // Read what the client sent.
    let msg =
        hudhudscript_shared_builtins::ws_ops::dispatch("recv", &[client_conn.clone()]).unwrap();
    if let Some(obj) = &msg.as_object() {
        assert_eq!(obj.get("type"), Some(&Value16::string("text".to_string())));
        assert_eq!(
            obj.get("data"),
            Some(&Value16::string("hello ws".to_string()))
        );
    } else {
        panic!("Expected object");
    }

    // Echo it back.
    hudhudscript_shared_builtins::ws_ops::dispatch(
        "send",
        &[client_conn.clone(), Value16::string("hello ws".to_string())],
    )
    .unwrap();
    hudhudscript_shared_builtins::ws_ops::dispatch("close", &[client_conn]).unwrap();

    // The client should have received the echo.
    let client_msg = client_handle.join().unwrap();
    if let Some(obj) = &client_msg.as_object() {
        assert_eq!(
            obj.get("data"),
            Some(&Value16::string("hello ws".to_string()))
        );
    }
}

#[test]
fn test_ws_connect_invalid_url() {
    // Same shared dispatcher path — connecting to a closed port must
    // surface an error rather than succeeding silently.
    let result = hudhudscript_shared_builtins::ws_ops::dispatch(
        "connect",
        &[Value16::string("ws://127.0.0.1:1".to_string())],
    );
    assert!(result.is_err());
}
