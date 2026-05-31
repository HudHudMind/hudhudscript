//! HudHudScript Language Server
//!
//! This crate provides a Language Server Protocol implementation for HudHudScript
//! built on top of `tower-lsp`.  The [`HudHudLanguageServer`] type implements the
//! `tower_lsp::LanguageServer` trait and can be wired into a stdio/TCP transport
//! using the standard `tower_lsp::LspService` builder.

pub mod completion;
pub mod definition;
pub mod document;
pub mod hover;
pub mod references;
pub mod rename;
pub mod server;
pub mod symbols;

pub use completion::{CompletionItem, CompletionKind, CompletionProvider, CursorContext};
pub use definition::{collect_definitions, goto_definition, DefinitionEntry};
pub use document::{Document, DocumentUri};
pub use hover::{hover_at, word_at_position as hover_word_at_position};
pub use references::{find_references, identifier_at_position, span_to_range};
pub use rename::{prepare_rename, rename};
pub use server::{
    completion_kind_to_lsp, parse_diagnostics, position_to_offset, HudHudLanguageServer,
    HudHudServerCapabilities,
};
pub use symbols::extract_symbols;

/// LSP server error type. (v0.4.47.9 — Issue #845)
#[derive(Debug)]
pub enum LspError {
    RuntimeStartFailed(std::io::Error),
    Server(String),
}

impl std::fmt::Display for LspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            LspError::RuntimeStartFailed(e) => write!(f, "Failed to start tokio runtime: {}", e),
            LspError::Server(s) => write!(f, "LSP server error: {}", s),
        }
    }
}

impl std::error::Error for LspError {}

impl From<std::io::Error> for LspError {
    fn from(e: std::io::Error) -> Self {
        LspError::RuntimeStartFailed(e)
    }
}

impl From<String> for LspError {
    fn from(s: String) -> Self {
        LspError::Server(s)
    }
}

/// Start the LSP server on stdio transport (#713).
pub fn run_stdio() -> Result<(), LspError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) = tower_lsp::LspService::new(HudHudLanguageServer::new);
        tower_lsp::Server::new(stdin, stdout, socket)
            .serve(service)
            .await;
    });
    Ok(())
}

/// Start the LSP server on TCP transport (#713).
pub fn run_tcp(port: u16) -> Result<(), LspError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .map_err(|e| format!("Cannot bind LSP TCP port {}: {}", port, e))?;
        eprintln!("LSP server listening on 127.0.0.1:{}", port);
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| format!("Accept error: {}", e))?;
        let (read, write) = tokio::io::split(stream);
        let (service, socket) = tower_lsp::LspService::new(HudHudLanguageServer::new);
        tower_lsp::Server::new(read, write, socket)
            .serve(service)
            .await;
        Ok::<(), String>(())
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl LspError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            LspError::RuntimeStartFailed(..) => {
                hudhudscript_errors::ErrorCode::LspRuntimeStartFailed
            }
            LspError::Server(..) => hudhudscript_errors::ErrorCode::LspServer,
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

impl From<LspError> for hudhudscript_errors::Error {
    fn from(e: LspError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
