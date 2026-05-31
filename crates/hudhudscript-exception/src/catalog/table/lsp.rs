use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TABLE: [ExceptionEntry; 2] = [
    ExceptionEntry {
        code: ExceptionCode(122),
        long_code: "HHS_E_LSP_RUNTIME_START_FAILED",
        short_code: "E0122",
        title: "LSP Failed To Start Tokio Runtime",
        short_description: "The language server could not initialize its async runtime and therefore cannot accept any client connections.",
        long_description: "`hudhudscript-lsp` uses Tokio as its async runtime. The runtime is constructed when the server boots, and any failure in that construction — thread spawn denied, resource limit hit, conflicting global runtime — leaves the server unable to handle even the initialize handshake.

This error happens before the server reads anything from stdin or any socket, so the editor will see the LSP process exit immediately on launch. There is no partial mode to fall back to.

Look at the wrapped message for the OS-level reason. Common causes are restrictive ulimits, sandbox containers without thread privileges, or another component already constructing a global Tokio runtime.",
        hints: &["Inspect the wrapped message for the OS-level reason", "Check thread and file-descriptor ulimits in restricted environments", "Confirm no other component is constructing a global Tokio runtime", "Try running the LSP manually outside the editor to reproduce"],
        example_bad: None,
        example_good: None,
        see_also: &["LspServer"],
        since_version: "0.4.0",
        category: ExceptionCategory::Lsp,
    },

    ExceptionEntry {
        code: ExceptionCode(123),
        long_code: "HHS_E_LSP_SERVER",
        short_code: "E0123",
        title: "Generic LSP Server Error",
        short_description: "The language server encountered an error during normal operation that does not match a more specific LSP variant.",
        long_description: "This variant wraps any error raised inside the running language server after a successful start: protocol decoding failures, request handler panics caught at the boundary, file watcher hiccups, or unexpected client messages. The wrapped message is the primary signal.

Because the LSP runs as a long-lived process attached to an editor, transient errors here may cause the editor to dim diagnostics or restart the server. Persistent errors usually indicate a real bug that should be reported.

Read the wrapped message, capture the editor-side LSP log if available, and report a reproducer if the error keeps coming back. The LSP log on the editor side often shows the exact request that triggered the failure.",
        hints: &["Read the wrapped message — it carries the actual cause", "Capture the editor's LSP log to see which request triggered the error", "Persistent errors warrant a bug report with a reproducer", "Restart the LSP if the editor does not auto-recover"],
        example_bad: None,
        example_good: None,
        see_also: &["LspRuntimeStartFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Lsp,
    }
];
