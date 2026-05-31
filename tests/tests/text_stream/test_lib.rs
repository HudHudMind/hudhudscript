use hudhudscript_text_stream::*;
use serde_json::json;

// =========================================================================
// StreamMessage — constructors
// =========================================================================

#[test]
fn stream_message_text() {
    let msg = StreamMessage::text("hello");
    if let StreamMessage::Data {
        payload,
        content_type,
    } = &msg
    {
        assert_eq!(payload, "hello");
        assert_eq!(content_type.as_deref(), Some("text/plain"));
    } else {
        panic!("expected Data");
    }
}

#[test]
fn stream_message_json() {
    let value = json!({"key": "value"});
    let msg = StreamMessage::json(&value);
    if let StreamMessage::Data {
        payload,
        content_type,
    } = &msg
    {
        assert!(payload.contains("key"));
        assert_eq!(content_type.as_deref(), Some("application/json"));
    } else {
        panic!("expected Data");
    }
}

#[test]
fn stream_message_error() {
    let msg = StreamMessage::error("TIMEOUT", "request timed out");
    if let StreamMessage::Error { code, message } = &msg {
        assert_eq!(code, "TIMEOUT");
        assert_eq!(message, "request timed out");
    } else {
        panic!("expected Error");
    }
}

#[test]
fn stream_message_eof() {
    let msg = StreamMessage::Eof;
    assert!(msg.is_eof());
}

// =========================================================================
// StreamMessage — is_eof
// =========================================================================

#[test]
fn stream_message_data_is_not_eof() {
    let msg = StreamMessage::text("data");
    assert!(!msg.is_eof());
}

#[test]
fn stream_message_error_is_not_eof() {
    let msg = StreamMessage::error("ERR", "fail");
    assert!(!msg.is_eof());
}

// =========================================================================
// StreamMessage — NDJSON serialization
// =========================================================================

#[test]
fn stream_message_to_ndjson_data() {
    let msg = StreamMessage::text("hello");
    let json = msg.to_ndjson().unwrap();
    assert!(json.contains("\"type\":\"data\""));
    assert!(json.contains("\"payload\":\"hello\""));
}

#[test]
fn stream_message_to_ndjson_eof() {
    let msg = StreamMessage::Eof;
    let json = msg.to_ndjson().unwrap();
    assert!(json.contains("\"type\":\"eof\""));
}

#[test]
fn stream_message_to_ndjson_error() {
    let msg = StreamMessage::error("RATE_LIMITED", "slow down");
    let json = msg.to_ndjson().unwrap();
    assert!(json.contains("\"type\":\"error\""));
    assert!(json.contains("RATE_LIMITED"));
}

#[test]
fn stream_message_from_ndjson_data() {
    let json = r#"{"type":"data","payload":"hello","content_type":"text/plain"}"#;
    let msg = StreamMessage::from_ndjson(json).unwrap();
    if let StreamMessage::Data { payload, .. } = msg {
        assert_eq!(payload, "hello");
    } else {
        panic!("expected Data");
    }
}

#[test]
fn stream_message_from_ndjson_eof() {
    let json = r#"{"type":"eof"}"#;
    let msg = StreamMessage::from_ndjson(json).unwrap();
    assert!(msg.is_eof());
}

#[test]
fn stream_message_from_ndjson_error() {
    let json = r#"{"type":"error","code":"FAIL","message":"something broke"}"#;
    let msg = StreamMessage::from_ndjson(json).unwrap();
    if let StreamMessage::Error { code, message } = msg {
        assert_eq!(code, "FAIL");
        assert_eq!(message, "something broke");
    } else {
        panic!("expected Error");
    }
}

#[test]
fn stream_message_from_ndjson_with_whitespace() {
    let json = "  {\"type\":\"eof\"}  \n";
    let msg = StreamMessage::from_ndjson(json).unwrap();
    assert!(msg.is_eof());
}

#[test]
fn stream_message_from_ndjson_invalid() {
    let result = StreamMessage::from_ndjson("not json");
    assert!(result.is_err());
}

// =========================================================================
// StreamMessage — roundtrip
// =========================================================================

#[test]
fn stream_message_roundtrip_data() {
    let original = StreamMessage::text("roundtrip test");
    let json = original.to_ndjson().unwrap();
    let deserialized = StreamMessage::from_ndjson(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn stream_message_roundtrip_eof() {
    let original = StreamMessage::Eof;
    let json = original.to_ndjson().unwrap();
    let deserialized = StreamMessage::from_ndjson(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn stream_message_roundtrip_error() {
    let original = StreamMessage::error("CODE", "msg");
    let json = original.to_ndjson().unwrap();
    let deserialized = StreamMessage::from_ndjson(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn stream_message_roundtrip_json_data() {
    let value = json!({"nested": {"array": [1, 2, 3]}});
    let original = StreamMessage::json(&value);
    let json = original.to_ndjson().unwrap();
    let deserialized = StreamMessage::from_ndjson(&json).unwrap();
    assert_eq!(original, deserialized);
}

// =========================================================================
// StreamMessage — Display
// =========================================================================

#[test]
fn stream_message_display_data() {
    let msg = StreamMessage::text("hello");
    let display = format!("{}", msg);
    assert!(display.contains("data"));
    assert!(display.contains("hello"));
}

#[test]
fn stream_message_display_eof() {
    let msg = StreamMessage::Eof;
    let display = format!("{}", msg);
    assert!(display.contains("eof"));
}

// =========================================================================
// StreamMessage — eq / clone
// =========================================================================

#[test]
fn stream_message_eq() {
    assert_eq!(StreamMessage::Eof, StreamMessage::Eof);
    assert_eq!(StreamMessage::text("a"), StreamMessage::text("a"));
    assert_ne!(StreamMessage::text("a"), StreamMessage::text("b"));
    assert_ne!(StreamMessage::Eof, StreamMessage::text("x"));
}

#[test]
fn stream_message_clone() {
    let msg = StreamMessage::text("clone me");
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

// =========================================================================
// StreamMessage — content_type skipped if None
// =========================================================================

#[test]
fn stream_message_data_no_content_type() {
    let msg = StreamMessage::Data {
        payload: "raw".to_string(),
        content_type: None,
    };
    let json = msg.to_ndjson().unwrap();
    assert!(!json.contains("content_type"));
}

// =========================================================================
// StreamError
// =========================================================================

#[test]
fn stream_error_channel_closed() {
    let err = StreamError::ChannelClosed;
    let msg = format!("{err}");
    assert!(msg.contains("channel closed"));
}

#[test]
fn stream_error_encode_error() {
    let err = StreamError::EncodeError("bad data".to_string());
    assert!(format!("{err}").contains("bad data"));
}

#[test]
fn stream_error_decode_error() {
    let err = StreamError::DecodeError("invalid json".to_string());
    assert!(format!("{err}").contains("invalid json"));
}

// =========================================================================
// agent_pipe — basic writer/reader
// =========================================================================

#[tokio::test]
async fn agent_pipe_basic_write_read() {
    let (writer, mut reader) = agent_pipe(16);
    writer.write_text("hello").await.unwrap();
    let msg = reader.next().await.unwrap();
    if let StreamMessage::Data { payload, .. } = msg {
        assert_eq!(payload, "hello");
    } else {
        panic!("expected Data");
    }
}

#[tokio::test]
async fn agent_pipe_close_sends_eof() {
    let (mut writer, mut reader) = agent_pipe(16);
    writer.write_text("data").await.unwrap();
    writer.close().await.unwrap();
    let msg1 = reader.next().await.unwrap();
    assert!(!msg1.is_eof());
    let msg2 = reader.next().await.unwrap();
    assert!(msg2.is_eof());
}

#[tokio::test]
async fn agent_pipe_collect_all() {
    let (mut writer, mut reader) = agent_pipe(16);
    writer.write_text("one").await.unwrap();
    writer.write_text("two").await.unwrap();
    writer.close().await.unwrap();
    let frames = reader.collect_all().await;
    assert_eq!(frames.len(), 3); // data, data, eof
    assert!(frames[2].is_eof());
}

#[tokio::test]
async fn agent_pipe_collect_text() {
    let (mut writer, mut reader) = agent_pipe(16);
    writer.write_text("hello ").await.unwrap();
    writer.write_text("world").await.unwrap();
    writer.close().await.unwrap();
    let text = reader.collect_text().await;
    assert_eq!(text, "hello world");
}

#[tokio::test]
async fn agent_pipe_write_json() {
    let (writer, mut reader) = agent_pipe(16);
    let value = json!({"key": "val"});
    writer.write_json(&value).await.unwrap();
    let msg = reader.next().await.unwrap();
    if let StreamMessage::Data {
        payload,
        content_type,
    } = msg
    {
        assert!(payload.contains("key"));
        assert_eq!(content_type.as_deref(), Some("application/json"));
    } else {
        panic!("expected Data");
    }
}

#[tokio::test]
async fn agent_pipe_write_error_frame() {
    let (writer, mut reader) = agent_pipe(16);
    writer
        .write(StreamMessage::error("ERR", "fail"))
        .await
        .unwrap();
    let msg = reader.next().await.unwrap();
    assert!(matches!(msg, StreamMessage::Error { .. }));
}

#[tokio::test]
async fn agent_pipe_collect_text_ignores_errors_and_eof() {
    let (mut writer, mut reader) = agent_pipe(16);
    writer.write_text("data1").await.unwrap();
    writer
        .write(StreamMessage::error("ERR", "oops"))
        .await
        .unwrap();
    writer.write_text("data2").await.unwrap();
    writer.close().await.unwrap();
    let text = reader.collect_text().await;
    assert_eq!(text, "data1data2");
}

#[tokio::test]
async fn agent_pipe_double_close() {
    let (mut writer, _reader) = agent_pipe(16);
    writer.close().await.unwrap();
    // Second close should be a no-op (already closed)
    writer.close().await.unwrap();
}

// =========================================================================
// TextStreamAdapter
// =========================================================================

#[tokio::test]
async fn adapter_send_value() {
    let (writer, mut reader) = agent_pipe(16);
    let value = json!({"hello": "world"});
    TextStreamAdapter::send_value(&writer, &value)
        .await
        .unwrap();
    let msg = reader.next().await.unwrap();
    if let StreamMessage::Data { payload, .. } = msg {
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["hello"], "world");
    }
}

#[tokio::test]
async fn adapter_receive_value() {
    let (mut writer, mut reader) = agent_pipe(16);
    let value = json!({"number": 42});
    writer.write_json(&value).await.unwrap();
    writer.close().await.unwrap();
    let received = TextStreamAdapter::receive_value(&mut reader).await.unwrap();
    assert_eq!(received["number"], 42);
}

#[tokio::test]
async fn adapter_receive_value_invalid_json() {
    let (mut writer, mut reader) = agent_pipe(16);
    writer.write_text("not valid json {{").await.unwrap();
    writer.close().await.unwrap();
    let result = TextStreamAdapter::receive_value(&mut reader).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), StreamError::DecodeError(_)));
}

// =========================================================================
// Drop behavior — writer sends EOF on drop
// =========================================================================

#[tokio::test]
async fn writer_drop_sends_eof() {
    let (writer, mut reader) = agent_pipe(16);
    writer.write_text("before drop").await.unwrap();
    drop(writer); // should trigger EOF via Drop
    let frames = reader.collect_all().await;
    // Should have the data frame and an EOF
    assert!(frames.last().unwrap().is_eof());
}

// =========================================================================
// Large payload
// =========================================================================

#[tokio::test]
async fn agent_pipe_large_payload() {
    let (mut writer, mut reader) = agent_pipe(16);
    let large = "x".repeat(100_000);
    writer.write_text(&large).await.unwrap();
    writer.close().await.unwrap();
    let text = reader.collect_text().await;
    assert_eq!(text.len(), 100_000);
}

// =========================================================================
// Multiple frames ordering
// =========================================================================

#[tokio::test]
async fn agent_pipe_ordering_preserved() {
    let (mut writer, mut reader) = agent_pipe(16);
    for i in 0..10 {
        writer.write_text(&format!("msg{i}")).await.unwrap();
    }
    writer.close().await.unwrap();
    let text = reader.collect_text().await;
    assert_eq!(text, "msg0msg1msg2msg3msg4msg5msg6msg7msg8msg9");
}
