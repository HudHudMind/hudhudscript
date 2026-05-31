use super::{Message, Role, ToolCall};

/// Accumulates streaming response tokens into a complete message.
///
/// As tokens arrive one by one from a streaming API, the accumulator buffers
/// them and can produce the final assistant message once the stream ends.
#[derive(Debug, Clone)]
pub struct StreamAccumulator {
    /// Content tokens accumulated so far.
    content_buffer: String,
    /// Tool calls being built up from streaming deltas.
    tool_calls: Vec<ToolCall>,
    /// Total tokens received so far.
    token_count: usize,
    /// Whether the stream has been finalised.
    finished: bool,
}

impl StreamAccumulator {
    /// Create a new empty accumulator.
    pub fn new() -> Self {
        Self {
            content_buffer: String::new(),
            tool_calls: Vec::new(),
            token_count: 0,
            finished: false,
        }
    }

    /// Push a content token/chunk into the accumulator.
    pub fn push_content(&mut self, token: &str) {
        self.content_buffer.push_str(token);
        self.token_count += 1;
    }

    /// Push a complete tool call (typically arrives as a single delta).
    pub fn push_tool_call(&mut self, tool_call: ToolCall) {
        self.tool_calls.push(tool_call);
    }

    /// The accumulated content so far.
    pub fn content(&self) -> &str {
        &self.content_buffer
    }

    /// The tool calls accumulated so far.
    pub fn tool_calls(&self) -> &[ToolCall] {
        &self.tool_calls
    }

    /// Number of content chunks received.
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Whether the stream has been finalised.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Mark the stream as finished and produce the final [`Message`].
    pub fn finish(&mut self) -> Message {
        self.finished = true;

        if self.tool_calls.is_empty() {
            Message::new(Role::Assistant, &self.content_buffer)
        } else {
            Message::assistant_with_tools(&self.content_buffer, self.tool_calls.clone())
        }
    }

    /// Reset the accumulator for reuse.
    pub fn reset(&mut self) {
        self.content_buffer.clear();
        self.tool_calls.clear();
        self.token_count = 0;
        self.finished = false;
    }
}

impl Default for StreamAccumulator {
    fn default() -> Self {
        Self::new()
    }
}
