//! Tests extracted from hudhudscript-tools-ai/src/conversation.rs
//! Skipped (already in tools_ai_test_lib.rs): test_role_display,
//! test_message_new, test_message_tool_result, test_message_assistant_with_tools,
//! test_message_estimated_tokens, test_conversation_new,
//! test_conversation_with_system_prompt, test_stream_accumulator_finish,
//! test_stream_accumulator_reset

use hudhudscript_tools_ai::{
    Conversation, ConversationError, Message, Role, StreamAccumulator, ToolCall,
};

#[test]
fn test_role_serde_roundtrip() {
    let json = serde_json::to_string(&Role::Assistant).unwrap();
    assert_eq!(json, "\"assistant\"");
    let parsed: Role = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, Role::Assistant);
}

#[test]
fn test_add_messages() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_user("Hi");
    conv.add_assistant("Hello!");
    conv.add_system("Note: be concise");
    conv.add_tool_result("call_1", "42");

    assert_eq!(conv.message_count(), 4);
    assert_eq!(conv.messages()[0].role, Role::User);
    assert_eq!(conv.messages()[1].role, Role::Assistant);
    assert_eq!(conv.messages()[2].role, Role::System);
    assert_eq!(conv.messages()[3].role, Role::Tool);
}

#[test]
fn test_last_message() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    assert!(conv.last_message().is_none());

    conv.add_user("first");
    conv.add_assistant("second");
    assert_eq!(conv.last_message().unwrap().content, "second");
}

#[test]
fn test_clear() {
    let mut conv = Conversation::with_system_prompt("gpt-4o", 4096, "system");
    conv.add_user("hello");
    conv.add_assistant("hi");
    conv.clear();

    assert_eq!(conv.message_count(), 0);
    // System prompt is preserved
    assert_eq!(conv.system_prompt(), Some("system"));
}

#[test]
fn test_total_tokens() {
    let mut conv = Conversation::with_system_prompt("gpt-4o", 4096, "Be helpful.");
    conv.add_user("Hello");
    let tokens = conv.total_tokens();
    // Should be > 0 and account for system prompt + message
    assert!(tokens > 0);
}

#[test]
fn test_truncate_to_fit() {
    let mut conv = Conversation::new("gpt-4o", 100);
    // Add many messages to exceed the budget
    for i in 0..50 {
        conv.add_user(&format!(
            "Message number {} with some extra text to use tokens",
            i
        ));
    }

    let before = conv.message_count();
    let removed = conv.truncate_to_fit(100);
    assert!(removed > 0);
    assert!(conv.message_count() < before);
    assert!(conv.total_tokens() <= 100);
}

#[test]
fn test_truncate_preserves_system_messages() {
    let mut conv = Conversation::new("gpt-4o", 100);
    conv.add_system("Important context");
    for i in 0..20 {
        conv.add_user(&format!("User message {} with padding text here", i));
    }

    conv.truncate_to_fit(50);

    // All remaining system messages should be preserved
    for msg in conv.messages() {
        if msg.role == Role::System {
            assert_eq!(msg.content, "Important context");
        }
    }
}

#[test]
fn test_truncate_within_budget_noop() {
    let mut conv = Conversation::new("gpt-4o", 10000);
    conv.add_user("Hi");
    let removed = conv.truncate_to_fit(10000);
    assert_eq!(removed, 0);
    assert_eq!(conv.message_count(), 1);
}

#[test]
fn test_messages_for_api_basic() {
    let mut conv = Conversation::with_system_prompt("gpt-4o", 4096, "You are helpful.");
    conv.add_user("Hello");
    conv.add_assistant("Hi there!");

    let api = conv.messages_for_api();
    assert_eq!(api.len(), 3); // system + user + assistant
    assert_eq!(api[0]["role"], "system");
    assert_eq!(api[0]["content"], "You are helpful.");
    assert_eq!(api[1]["role"], "user");
    assert_eq!(api[2]["role"], "assistant");
}

#[test]
fn test_messages_for_api_with_tool_calls() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_user("What is the weather?");

    let tool_calls = vec![ToolCall {
        id: "call_abc".into(),
        name: "get_weather".into(),
        arguments: r#"{"city":"Dubai"}"#.into(),
    }];
    conv.add_assistant_with_tools("", tool_calls);
    conv.add_tool_result("call_abc", r#"{"temp": 35}"#);

    let api = conv.messages_for_api();
    assert_eq!(api.len(), 3);

    // Assistant message should have tool_calls
    assert!(api[1]["tool_calls"].is_array());
    assert_eq!(api[1]["tool_calls"][0]["function"]["name"], "get_weather");

    // Tool result should have tool_call_id
    assert_eq!(api[2]["tool_call_id"], "call_abc");
}

#[test]
fn test_messages_for_api_no_system() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_user("Hi");
    let api = conv.messages_for_api();
    assert_eq!(api.len(), 1);
    assert_eq!(api[0]["role"], "user");
}

#[test]
fn test_save_and_load() {
    let dir = std::env::temp_dir();
    let path = dir.join("hudhud_conv_test.json");

    // Build a conversation
    let mut conv = Conversation::with_system_prompt("gpt-4o", 4096, "Be helpful.");
    conv.add_user("Hello");
    conv.add_assistant("Hi!");
    conv.add_tool_result("call_1", "result");

    // Save
    conv.save(&path).unwrap();

    // Load
    let loaded = Conversation::load(&path).unwrap();
    assert_eq!(loaded.model(), "gpt-4o");
    assert_eq!(loaded.max_tokens(), 4096);
    assert_eq!(loaded.system_prompt(), Some("Be helpful."));
    assert_eq!(loaded.message_count(), 3);
    assert_eq!(loaded.messages()[0].role, Role::User);
    assert_eq!(loaded.messages()[0].content, "Hello");
    assert_eq!(loaded.messages()[2].role, Role::Tool);
    assert_eq!(loaded.messages()[2].tool_call_id.as_deref(), Some("call_1"));

    // Clean up
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_load_nonexistent_file() {
    let result = Conversation::load("/tmp/nonexistent_hudhud_conv_12345.json");
    assert!(result.is_err());
}

#[test]
fn test_stream_accumulator_content() {
    let mut acc = StreamAccumulator::new();
    assert_eq!(acc.content(), "");
    assert_eq!(acc.token_count(), 0);
    assert!(!acc.is_finished());

    acc.push_content("Hello");
    acc.push_content(" ");
    acc.push_content("world");

    assert_eq!(acc.content(), "Hello world");
    assert_eq!(acc.token_count(), 3);
}

#[test]
fn test_stream_accumulator_with_tool_calls() {
    let mut acc = StreamAccumulator::new();
    acc.push_content("Let me check.");
    acc.push_tool_call(ToolCall {
        id: "call_1".into(),
        name: "search".into(),
        arguments: r#"{"q":"test"}"#.into(),
    });

    let msg = acc.finish();
    assert_eq!(msg.role, Role::Assistant);
    assert!(msg.tool_calls.is_some());
    assert_eq!(msg.tool_calls.as_ref().unwrap().len(), 1);
}

#[test]
fn test_tool_call_serde() {
    let tc = ToolCall {
        id: "call_123".into(),
        name: "get_weather".into(),
        arguments: r#"{"city":"Dubai"}"#.into(),
    };
    let json = serde_json::to_string(&tc).unwrap();
    let parsed: ToolCall = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.id, "call_123");
    assert_eq!(parsed.name, "get_weather");
}

#[test]
fn test_conversation_set_system_prompt() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    assert!(conv.system_prompt().is_none());
    conv.set_system_prompt("New system prompt");
    assert_eq!(conv.system_prompt(), Some("New system prompt"));
}

#[test]
fn test_conversation_error_display() {
    let e = ConversationError::Empty;
    assert!(format!("{}", e).contains("Conversation is empty"));
}

#[test]
fn test_message_estimated_tokens_with_tool_calls() {
    let calls = vec![
        ToolCall {
            id: "c1".into(),
            name: "search".into(),
            arguments: r#"{"query":"test"}"#.into(),
        },
        ToolCall {
            id: "c2".into(),
            name: "calculate".into(),
            arguments: r#"{"expr":"1+1"}"#.into(),
        },
    ];
    let msg = Message::assistant_with_tools("", calls);
    let tokens = msg.estimated_tokens();
    // 4 (overhead) + 0 (empty content) + tool tokens > 4
    assert!(tokens > 4);
}

#[test]
fn test_truncate_only_system_messages() {
    let mut conv = Conversation::new("gpt-4o", 10);
    conv.add_system("System message 1");
    conv.add_system("System message 2");
    // Only system messages — truncate should not remove them
    let removed = conv.truncate_to_fit(0);
    // Since all are system, we can't remove any even if over budget
    assert_eq!(removed, 0);
}

#[test]
fn test_stream_accumulator_default() {
    let acc = StreamAccumulator::default();
    assert_eq!(acc.content(), "");
    assert_eq!(acc.token_count(), 0);
    assert!(!acc.is_finished());
}

#[test]
fn test_conversation_error_io_variant() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no file");
    let conv_err: ConversationError = io_err.into();
    let s = format!("{}", conv_err);
    assert!(s.contains("IO error"));
}

#[test]
fn test_conversation_error_serialization_variant() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let conv_err: ConversationError = json_err.into();
    let s = format!("{}", conv_err);
    assert!(s.contains("Serialization error"));
}

#[test]
fn test_full_tool_use_loop() {
    // Simulate: user asks -> assistant requests tool -> tool returns -> assistant responds
    let mut conv = Conversation::with_system_prompt("gpt-4o", 4096, "You are a weather bot.");

    // User asks
    conv.add_user("What is the weather in Dubai?");

    // Assistant requests tool call
    let tool_calls = vec![ToolCall {
        id: "call_weather_1".into(),
        name: "get_weather".into(),
        arguments: r#"{"city":"Dubai"}"#.into(),
    }];
    conv.add_assistant_with_tools("", tool_calls);

    // Tool returns result
    conv.add_tool_result("call_weather_1", r#"{"temp_c": 35, "condition": "sunny"}"#);

    // Assistant gives final answer
    conv.add_assistant("It is 35 degrees Celsius and sunny in Dubai.");

    assert_eq!(conv.message_count(), 4);

    let api = conv.messages_for_api();
    assert_eq!(api.len(), 5); // system + 4 messages
    assert_eq!(api[0]["role"], "system");
    assert_eq!(api[4]["role"], "assistant");
    assert_eq!(
        api[4]["content"],
        "It is 35 degrees Celsius and sunny in Dubai."
    );
}
