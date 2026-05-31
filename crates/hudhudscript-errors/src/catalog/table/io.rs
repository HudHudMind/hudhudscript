use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const STREAM_CHANNEL_CLOSED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(267),
        long_code: "HHS_E_STREAM_CHANNEL_CLOSED",
        short_code: "E0267",
        title: "Text Stream Channel Closed Unexpectedly",
        short_description: "The underlying channel for a text stream was closed before the framing layer expected end-of-stream.",
        long_description: "`hudhudscript-text-stream` reads framed messages from an underlying channel (pipe, socket, in-process queue). When that channel closes mid-frame or between expected frames, the reader cannot continue and emits this error.

The stream itself does not retain partial frames after the close. Anything that was waiting on the next message gets this error in place of a value.

Decide whether the close was orderly (peer signalled shutdown) or unexpected (peer crashed), and respond accordingly. Wrapping stream consumers in supervisors that can re-establish the channel is a common pattern.",
        hints: &["Check whether the peer closed cleanly or crashed", "Wrap consumers in a supervisor that can recreate the channel", "Avoid treating this as end-of-stream — it is an error condition", "Cancel any awaits on the stream when this fires"],
        example_bad: None,
        example_good: None,
        see_also: &["StreamDecodeError", "StreamEncodeError"],
        since_version: "0.4.0",
        category: ErrorCategory::Io,
    };

pub const STREAM_DECODE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(268),
        long_code: "HHS_E_STREAM_DECODE_ERROR",
        short_code: "E0268",
        title: "Text Stream Decode Error",
        short_description: "A frame received on a text stream could not be deserialized into the expected message type.",
        long_description: "Each frame on a text stream is decoded into a typed message via the configured serializer. When the bytes do not parse — wrong format, version mismatch, truncated payload — this error is raised with the underlying serializer message.

The stream remains open after a decode failure, but the offending frame is consumed and lost. Whether to continue reading depends on whether the stream is delimited per frame or whether one bad frame implies a corrupted state.

Inspect the wrapped cause, verify peer/version compatibility, and decide whether to drop the frame or close the stream entirely.",
        hints: &["Verify peer and local versions of the message schema match", "Inspect the wrapped serializer message for the failing field", "Decide framing policy: drop one bad frame or close the stream", "Log the raw bytes (with care) when investigating malformed frames"],
        example_bad: None,
        example_good: None,
        see_also: &["StreamEncodeError", "StreamChannelClosed"],
        since_version: "0.4.0",
        category: ErrorCategory::Io,
    };

pub const STREAM_ENCODE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(269),
        long_code: "HHS_E_STREAM_ENCODE_ERROR",
        short_code: "E0269",
        title: "Text Stream Encode Error",
        short_description: "A message could not be serialized for transmission on a text stream.",
        long_description: "Encode failures happen when the typed message you tried to send cannot be turned into bytes by the configured serializer. Common causes include non-serializable types embedded in the message, fields that violate the schema's invariants, or recursion that exceeds the serializer's depth limit.

Nothing is written to the channel when encoding fails, so the peer is unaffected. The error is local to the sender.

Fix the message construction so it conforms to the schema. If the schema needs to change, version it deliberately so the peer can keep up.",
        hints: &["Verify the message conforms to its schema before sending", "Watch for non-serializable types embedded in messages", "Avoid deep recursion that exceeds the serializer depth limit", "Version schema changes deliberately so peers can adapt"],
        example_bad: None,
        example_good: None,
        see_also: &["StreamDecodeError", "StreamChannelClosed"],
        since_version: "0.4.0",
        category: ErrorCategory::Io,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    STREAM_CHANNEL_CLOSED,
    STREAM_DECODE_ERROR,
    STREAM_ENCODE_ERROR,
];
