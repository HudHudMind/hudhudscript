//! MCP server declaration compilation for `CompileTarget`.

use super::*;

impl Compiler {

    pub fn compile_mcp_server(
        &mut self,
        mcp_decl: &hudhudscript_ast::McpServerDecl,
    ) -> CompileResult<()> {
        // `mcp_decl.fields` — the fields the user actually wrote in the script
        // body — is the SINGLE authoritative source for codegen (Kural 7).
        //
        // `mcp_decl.config` is a parser-side view that is NEVER synthesized
        // into the emitted object. Synthesizing it is what broke `transport`:
        // the parser leaves `config` at its default for every declaration
        // (`transport: Stdio`, everything else `None`), the synthesized
        // `transport: "stdio"` entry then claimed the key, and the merge loop
        // skipped keys that were already present — so a user's
        // `transport: "sse"` was silently dropped and every server was treated
        // as stdio (which demands `allow_process`).
        //
        // The VM reads `transport` straight off the compiled object and
        // defaults to "stdio" when the key is absent, so defaults live in one
        // place instead of being baked in here.
        self.compile_decl_as_object("mcp_server", &mcp_decl.name, &mcp_decl.fields)
    }
}
