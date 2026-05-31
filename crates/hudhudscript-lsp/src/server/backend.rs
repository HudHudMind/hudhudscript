//! Backend state and server struct.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::Client;

use crate::completion::CompletionProvider;
use crate::document::{Document, DocumentUri};

/// Shared mutable state wrapped for async access
pub(crate) struct BackendState {
    pub(crate) documents: HashMap<DocumentUri, Document>,
    pub(crate) completion_provider: CompletionProvider,
}

impl BackendState {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
            completion_provider: CompletionProvider::new(),
        }
    }
}

/// The tower-lsp backend that implements `LanguageServer`
pub struct HudHudLanguageServer {
    pub(crate) client: Client,
    pub(crate) state: Arc<RwLock<BackendState>>,
}

impl HudHudLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(BackendState::new())),
        }
    }

    /// Parse `text` and publish diagnostics for `uri`.
    pub(crate) async fn publish_diagnostics(
        &self,
        uri: tower_lsp::lsp_types::Url,
        text: &str,
        version: Option<i32>,
    ) {
        let diagnostics = crate::server::helpers::parse_diagnostics(text);
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }
}
