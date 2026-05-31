//! `LanguageServer` trait implementation for `HudHudLanguageServer`.

use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

use hudhudscript_parser::parse;

use crate::server::backend::HudHudLanguageServer;
use crate::server::helpers::{completion_kind_to_lsp, position_to_offset};

#[tower_lsp::async_trait]
impl LanguageServer for HudHudLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "hudhudscript-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "HudHudScript language server initialized",
            )
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri_str = params.text_document.uri.to_string();
        let text = params.text_document.text.clone();
        let version = params.text_document.version;

        {
            let mut state = self.state.write().await;
            state.documents.insert(
                uri_str.clone(),
                crate::document::Document::new(uri_str, text.clone()),
            );
        }

        self.publish_diagnostics(params.text_document.uri, &text, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // We use FULL sync — the last content change is the full document text.
        if let Some(change) = params.content_changes.last() {
            let uri_str = params.text_document.uri.to_string();
            let text = change.text.clone();
            let version = params.text_document.version;

            {
                let mut state = self.state.write().await;
                if let Some(doc) = state.documents.get_mut(&uri_str) {
                    doc.update(text.clone());
                } else {
                    state.documents.insert(
                        uri_str,
                        crate::document::Document::new(
                            params.text_document.uri.to_string(),
                            text.clone(),
                        ),
                    );
                }
            }

            self.publish_diagnostics(params.text_document.uri, &text, Some(version))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri_str = params.text_document.uri.to_string();
        let mut state = self.state.write().await;
        state.documents.remove(&uri_str);
        // Clear diagnostics for the closed file.
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    // ── Completion (Issue #300) ──────────────────────────────────────────

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri_str = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let state = self.state.read().await;
        let text = state
            .documents
            .get(&uri_str)
            .map(|d| d.text().to_string())
            .unwrap_or_default();

        // Compute byte offset for context-aware completion
        let offset = position_to_offset(&text, position);

        let items: Vec<CompletionItem> = state
            .completion_provider
            .complete(&text, offset)
            .into_iter()
            .map(|ci| CompletionItem {
                label: ci.label,
                kind: Some(completion_kind_to_lsp(ci.kind)),
                detail: ci.detail,
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    // ── Hover (Issue #296) ───────────────────────────────────────────────

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri_str = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        let state = self.state.read().await;
        let text = state
            .documents
            .get(&uri_str)
            .map(|d| d.text().to_string())
            .unwrap_or_default();

        Ok(crate::hover::hover_at(&text, position))
    }

    // ── Go-to-definition (Issue #297) ────────────────────────────────────

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let uri_str = uri.to_string();
        let position = params.text_document_position_params.position;

        let state = self.state.read().await;
        let text = state
            .documents
            .get(&uri_str)
            .map(|d| d.text().to_string())
            .unwrap_or_default();

        Ok(crate::definition::goto_definition(&uri, &text, position))
    }

    // ── Document symbols (Issue #299) ────────────────────────────────────

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri_str = params.text_document.uri.to_string();

        let state = self.state.read().await;
        let text = state
            .documents
            .get(&uri_str)
            .map(|d| d.text().to_string())
            .unwrap_or_default();

        let ast = match parse(&text) {
            Ok(ast) => ast,
            Err(_) => return Ok(None),
        };

        let syms = crate::symbols::extract_symbols(&ast);
        Ok(Some(DocumentSymbolResponse::Nested(syms)))
    }

    // ── Find References (Issue #298) ─────────────────────────────────────

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let uri_str = uri.to_string();
        let position = params.text_document_position.position;

        let state = self.state.read().await;
        let text = match state.documents.get(&uri_str) {
            Some(doc) => doc.text().to_string(),
            None => return Ok(None),
        };

        let name = match crate::references::identifier_at_position(&text, &position) {
            Some(n) => n,
            None => return Ok(Some(vec![])),
        };

        let locations = crate::references::find_references(&text, &name, &uri);
        Ok(Some(locations))
    }

    // ── Rename (Issue #976) ───────────────────────────────────────────────

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspResult<Option<PrepareRenameResponse>> {
        let uri_str = params.text_document.uri.to_string();
        let position = params.position;

        let state = self.state.read().await;
        let text = match state.documents.get(&uri_str) {
            Some(doc) => doc.text().to_string(),
            None => return Ok(None),
        };

        Ok(crate::rename::prepare_rename(&text, &position))
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let uri_str = uri.to_string();
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        let state = self.state.read().await;
        let text = match state.documents.get(&uri_str) {
            Some(doc) => doc.text().to_string(),
            None => return Ok(None),
        };

        Ok(crate::rename::rename(&text, &position, &new_name, &uri))
    }

    // ── Document Formatting (#714) ──────────────────────────────────────

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> LspResult<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        let state = self.state.read().await;
        let text = match state.documents.get(&uri) {
            Some(doc) => doc.text().to_string(),
            None => return Ok(None),
        };
        drop(state);

        // Format the document using the HudHudScript formatter
        let mut formatter = hudhudscript_formatter::Formatter::new();
        match hudhudscript_parser::parse(&text) {
            Ok(ast) => {
                let formatted = formatter.format_program(&ast);
                // Return a single edit that replaces the entire document
                let line_count = text.lines().count() as u32;
                let last_line_len = text.lines().last().map(|l| l.len()).unwrap_or(0) as u32;
                Ok(Some(vec![TextEdit {
                    range: tower_lsp::lsp_types::Range {
                        start: tower_lsp::lsp_types::Position {
                            line: 0,
                            character: 0,
                        },
                        end: tower_lsp::lsp_types::Position {
                            line: line_count,
                            character: last_line_len,
                        },
                    },
                    new_text: formatted,
                }]))
            }
            Err(_) => Ok(None), // Can't format invalid syntax
        }
    }
}
