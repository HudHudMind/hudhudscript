use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ToolHttpExceptionCode {
    /// E0106 — HTTP request URL is malformed
    HttpToolInvalidUrl = 106,
    /// E0107 — HTTP response body parse failed
    HttpToolParseError = 107,
    /// E0108 — HTTP request transport failed
    HttpToolRequestFailed = 108,
    /// E0109 — HTTP request exceeded timeout
    HttpToolTimeout = 109,
}
