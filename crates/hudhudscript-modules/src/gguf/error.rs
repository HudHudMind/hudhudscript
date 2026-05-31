//! GGUF errors — unified error catalog bridge.

/// Errors that can occur while parsing a GGUF header.
#[derive(Debug)]
pub enum GgufError {
    TooShort,
    InvalidMagic(u32),
    UnsupportedVersion(u32),
    UnexpectedEof,
    InvalidUtf8,
}

impl std::fmt::Display for GgufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            GgufError::TooShort => write!(f, "Data too short to contain a valid GGUF header"),
            GgufError::InvalidMagic(got) => write!(
                f,
                "Invalid GGUF magic number: expected 0x{:08X}, got 0x{:08X}",
                super::GGUF_MAGIC,
                got
            ),
            GgufError::UnsupportedVersion(v) => write!(f, "Unsupported GGUF version: {}", v),
            GgufError::UnexpectedEof => write!(f, "Unexpected end of data while parsing header"),
            GgufError::InvalidUtf8 => write!(f, "Invalid UTF-8 in metadata key or string value"),
        }
    }
}

impl std::error::Error for GgufError {}

impl GgufError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            GgufError::InvalidMagic(..) => hudhudscript_errors::ErrorCode::GgufInvalidMagic,
            GgufError::InvalidUtf8 => hudhudscript_errors::ErrorCode::GgufInvalidUtf8,
            GgufError::TooShort => hudhudscript_errors::ErrorCode::GgufTooShort,
            GgufError::UnexpectedEof => hudhudscript_errors::ErrorCode::GgufUnexpectedEof,
            GgufError::UnsupportedVersion(..) => {
                hudhudscript_errors::ErrorCode::GgufUnsupportedVersion
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<GgufError> for hudhudscript_errors::Error {
    fn from(e: GgufError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
