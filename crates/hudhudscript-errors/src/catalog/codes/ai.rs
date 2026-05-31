use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum AiErrorCode {
    /// E0046 — Conversation Has No Messages
    ConversationEmpty = 46,
    /// E0047 — Conversation Persistence I/O Failure
    ConversationIo = 47,
    /// E0048 — Conversation Failed To Serialize Or Parse
    ConversationSerialization = 48,
    /// E0080 — Every Fallback Provider Failed In Sequence
    FallbackAllProvidersExhausted = 80,
    /// E0081 — Fallback Chain Has No Providers Configured
    FallbackEmptyChain = 81,
    /// E0104 — Failed to deserialize HuggingFace response
    HfDeserialize = 104,
    /// E0105 — HTTP request to HuggingFace failed
    HfHttp = 105,
    /// E0124 — Long-Term Memory Backend Failed
    MemoryBackend = 124,
    /// E0125 — Memory Entry Missing By Key
    MemoryNotFound = 125,
    /// E0126 — Memory Entry Serialization Or Parse Failure
    MemorySerialization = 126,
}
