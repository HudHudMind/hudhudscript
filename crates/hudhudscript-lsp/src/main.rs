/*---------------------------------------------------------------------------------------------
 *  Copyright (c) HudHud Mind. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! HudHudScript Language Server binary
//!
//! Starts a stdio-based LSP server using tower-lsp.

use tower_lsp::{LspService, Server};

use hudhudscript_lsp::server::HudHudLanguageServer;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(HudHudLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
