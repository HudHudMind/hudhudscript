use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum PackageOllamaExceptionCode {
    /// E0155 — Failed to deserialize Ollama response
    OllamaDeserialize = 155,
    /// E0156 — HTTP request to Ollama failed
    OllamaHttp = 156,
}
