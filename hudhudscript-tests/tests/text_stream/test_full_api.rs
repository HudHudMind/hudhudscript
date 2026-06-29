//! Real unit tests for hudhudscript-text-stream — agent pipe, StreamMessage, TextStreamAdapter

use hudhudscript_text_stream::*;
use serde_json::json;

// ── StreamMessage construction ────────────────────────────────────────────

#[test]
fn message_text_constructor() {
    let msg = StreamMessage::text("hello world");
    match msg {
        StreamMessage::Data {
            payload,
            content_type,
        } => {
            assert_eq!(payload, "hello world");
            assert_eq!(content_type, Some("text/plain".to_string()));
        }
        _ => panic!("expected Data variant"),
    }
}

#[test]
fn message_json_constructor() {
    let val = json!({"key": "value", "num": 42});
    let msg = StreamMessage::json(&val);
    match msg {
        StreamMessage::Data {
            payload,
            content_type,
        } => {
            assert!(payload.contains("key"));
            assert!(payload.contains("42"));
            assert_eq!(content_type, Some("application/json".to_string()));
        }
        _ => panic!("expected Data variant"),
    }
}

#[test]
fn message_error_constructor() {
    let msg = StreamMessage::error("TIMEOUT", "request took too long");
    match msg {
        StreamMessage::Error { code, message } => {
            assert_eq!(code, "TIMEOUT");
            assert_eq!(message, "request took too long");
        }
        _ => panic!("expected Error variant"),
    }
}

#[test]
fn message_eof_is_eof() {
    let msg = StreamMessage::Eof;
    assert!(msg.is_eof());
}

#[test]
fn message_data_is_not_eof() {
    let msg = StreamMessage::text("hello");
    assert!(!msg.is_eof());
}

// ── NDJSON serialization ─────────────────────────────────────────────────

#[test]
fn ndjson_roundtrip_data() {
    let original = StreamMessage::text("test payload");
    let line = original.to_ndjson().unwrap();
    let parsed = StreamMessage::from_ndjson(&line).unwrap();
    assert_eq!(original, parsed);
}

#[test]
fn ndjson_roundtrip_error() {
    let original = StreamMessage::error("E001", "something broke");
    let line = original.to_ndjson().unwrap();
    let parsed = StreamMessage::from_ndjson(&line).unwrap();
    assert_eq!(original, parsed);
}

#[test]
fn ndjson_roundtrip_eof() {
    let original = StreamMessage::Eof;
    let line = original.to_ndjson().unwrap();
    let parsed = StreamMessage::from_ndjson(&line).unwrap();
    assert_eq!(original, parsed);
}

#[test]
fn ndjson_deserializes_json_data() {
    let val = json!({"items": [1, 2, 3]});
    let msg = StreamMessage::json(&val);
    let line = msg.to_ndjson().unwrap();
    let parsed = StreamMessage::from_ndjson(&line).unwrap();
    assert_eq!(msg, parsed);
}

#[test]
fn display_impl_produces_json() {
    let msg = StreamMessage::text("hi");
    let s = format!("{}", msg);
    assert!(s.contains("hi"));
    assert!(s.contains("text_plain") || s.contains("Data") || s.contains("type"));
}

// ── Agent pipe (agent_pipe) ──────────────────────────────────────────────

#[tokio::test]
async fn agent_pipe_write_and_read_one() {
    let (writer, mut reader) = agent_pipe(16);
    writer.write_text("hello from agent").await.unwrap();

    let msg = reader.next().await.unwrap();
    match msg {
        StreamMessage::Data { payload, .. } => assert_eq!(payload, "hello from agent"),
        _ => panic!("expected data"),
    }
}

#[tokio::test]
async fn agent_pipe_write_and_read_multiple() {
    let (writer, mut reader) = agent_pipe(16);
    writer.write_text("first").await.unwrap();
    writer.write_text("second").await.unwrap();
    writer.write_text("third").await.unwrap();

    assert_eq!(reader.next().await.unwrap(), StreamMessage::text("first"));
    assert_eq!(reader.next().await.unwrap(), StreamMessage::text("second"));
    assert_eq!(reader.next().await.unwrap(), StreamMessage::text("third"));
}

#[tokio::test]
async fn agent_pipe_write_json_value() {
    let (mut writer, mut reader) = agent_pipe(16);
    let val = json!({"agent": "alpha", "score": 0.95});
    writer.write_json(&val).await.unwrap();
    writer.close().await.unwrap();

    let msg = reader.next().await.unwrap();
    match msg {
        StreamMessage::Data { payload, .. } => {
            assert!(payload.contains("alpha"));
            assert!(payload.contains("0.95"));
        }
        _ => panic!("expected data"),
    }
}

#[tokio::test]
async fn agent_pipe_close_sends_eof() {
    let (mut writer, mut reader) = agent_pipe(16);
    writer.write_text("data").await.unwrap();
    writer.close().await.unwrap();

    // First message is the data
    let msg1 = reader.next().await.unwrap();
    assert!(!msg1.is_eof());
    // Second message is Eof
    let msg2 = reader.next().await.unwrap();
    assert!(msg2.is_eof());
}

#[tokio::test]
async fn agent_pipe_close_is_idempotent() {
    let (mut writer, _reader) = agent_pipe(16);
    writer.close().await.unwrap();
    // Closing again should not panic
    writer.close().await.unwrap();
}

#[tokio::test]
async fn agent_pipe_collect_all() {
    let (mut writer, mut reader) = agent_pipe(16);
    writer.write_text("a").await.unwrap();
    writer.write_text("b").await.unwrap();
    writer.close().await.unwrap();

    let all = reader.collect_all().await;
    assert_eq!(all.len(), 3); // a, b, Eof
    assert!(all[2].is_eof());
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
async fn agent_pipe_collect_text_skips_non_data() {
    let (writer, mut reader) = agent_pipe(16);
    writer.write_text("data1").await.unwrap();
    writer
        .write(StreamMessage::error("WARN", "something"))
        .await
        .unwrap();
    writer.write_text("data2").await.unwrap();
    drop(writer); // sends Eof on drop

    let text = reader.collect_text().await;
    assert_eq!(text, "data1data2");
}

// ── TextStreamAdapter ────────────────────────────────────────────────────

#[tokio::test]
async fn adapter_send_and_receive_value() {
    let (writer, mut reader) = agent_pipe(16);
    let val = json!({"result": "ok", "count": 3});

    TextStreamAdapter::send_value(&writer, &val).await.unwrap();
    drop(writer);

    let received = TextStreamAdapter::receive_value(&mut reader).await.unwrap();
    assert_eq!(received, val);
}

#[tokio::test]
async fn adapter_receive_empty_is_error() {
    let (_writer, mut reader) = agent_pipe(16);
    drop(_writer); // send Eof with no data

    let result = TextStreamAdapter::receive_value(&mut reader).await;
    assert!(result.is_err());
}

// ── StreamError Display ──────────────────────────────────────────────────

#[test]
fn stream_error_display() {
    let e1 = StreamError::ChannelClosed;
    assert!(format!("{}", e1).contains("channel closed"));

    let e2 = StreamError::EncodeError("bad json".to_string());
    assert!(format!("{}", e2).contains("serialise"));
    assert!(format!("{}", e2).contains("bad json"));

    let e3 = StreamError::DecodeError("invalid utf8".to_string());
    assert!(format!("{}", e3).contains("deserialise"));
    assert!(format!("{}", e3).contains("invalid utf8"));
}

#[test]
fn stream_error_code() {
    let e = StreamError::ChannelClosed;
    let code = e.code();
    assert!(!code.short_code().is_empty());
    assert!(!code.title().is_empty());
}
