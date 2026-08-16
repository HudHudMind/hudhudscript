// `mcp` declaration transport selection and the sandbox gate it drives.
//
// Before this: the parser built a default `ServerConfig` for every declaration
// (transport: Stdio) and the compiler synthesized a `transport: "stdio"` field
// from it, then skipped any user field whose key was already present — so a
// script's own `transport: "sse"` was silently discarded and EVERY mcp server
// was treated as stdio. Stdio demands `allow_process`, which the CLI could not
// grant through any flag or hudhud.toml key, so SSE servers failed with
// "process execution denied" while asking for a capability unrelated to what
// they declared.
//
// Now `mcp_decl.fields` is the single authoritative source for codegen, the VM
// reads `transport` straight off the compiled object, and an unrecognised
// transport is an error instead of a silent downgrade to stdio.
//
// These tests discriminate by error message because the message names the
// capability the declaration actually asked for — which is exactly what the
// bug got wrong.
use hudhudscript_vm::VM;

// Returns `Ok(())` when the declaration was accepted; the VM itself is not
// needed here since every assertion is about which gate the declaration hit.
fn run_with(src: &str, allow_network: bool, allow_process: bool) -> Result<(), String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler
        .compile(&stmts)
        .map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    if allow_network {
        vm.allow_network();
    }
    if allow_process {
        vm.allow_process();
    }
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(())
}

const SSE_DECL: &str = r#"
mcp Web { transport: "sse"; url: "https://example.invalid/sse"; }
"#;

const STDIO_DECL: &str = r#"
mcp Loc { transport: "stdio"; command: "some-mcp-server"; }
"#;

// ======================================================================
// 1 — THE BUG: an sse declaration must ask for network, never for process
// ======================================================================
#[test]
fn sse_declaration_requires_network_not_process() {
    let err = run_with(SSE_DECL, false, false).unwrap_err();
    assert!(
        err.contains("network access denied"),
        "an sse declaration must be gated on network access; got: {}",
        err
    );
    assert!(
        !err.contains("process execution denied"),
        "regression: the user's transport:\"sse\" is being dropped and the \
         server treated as stdio; got: {}",
        err
    );
}

// ======================================================================
// 2 — Granting process alone must NOT open an sse declaration
// ======================================================================
#[test]
fn allow_process_does_not_grant_sse() {
    let err = run_with(SSE_DECL, false, true).unwrap_err();
    assert!(
        err.contains("network access denied"),
        "capabilities must be per-transport, not interchangeable; got: {}",
        err
    );
}

// ======================================================================
// 3 — With network granted, an sse declaration passes the sandbox gate and
//     reaches the MCP client layer (proof the gate opened, no I/O needed)
// ======================================================================
#[test]
fn sse_declaration_passes_gate_when_network_allowed() {
    let err = run_with(SSE_DECL, true, false).unwrap_err();
    assert!(
        !err.contains("denied"),
        "network was granted, so no sandbox denial should remain; got: {}",
        err
    );
}

// ======================================================================
// 4 — A stdio declaration is gated on process
// ======================================================================
#[test]
fn stdio_declaration_requires_process() {
    let err = run_with(STDIO_DECL, false, false).unwrap_err();
    assert!(
        err.contains("process execution denied"),
        "a stdio declaration must be gated on process spawning; got: {}",
        err
    );
}

#[test]
fn allow_network_does_not_grant_stdio() {
    let err = run_with(STDIO_DECL, true, false).unwrap_err();
    assert!(
        err.contains("process execution denied"),
        "network must not stand in for process spawning; got: {}",
        err
    );
}

// ======================================================================
// 5 — allow_process actually reaches the VM: with it granted the process gate
//     is passed and the command deny-list is what stops a denied command.
//     (`rm` is in SandboxConfig's default denied_commands, so this asserts the
//     grant without spawning anything.)
// ======================================================================
#[test]
fn allow_process_opens_stdio_gate_and_deny_list_still_applies() {
    let src = r#"
mcp Danger { transport: "stdio"; command: "rm"; }
"#;
    let err = run_with(src, false, true).unwrap_err();
    assert!(
        err.contains("is denied for mcp server"),
        "expected the command deny-list to reject 'rm' AFTER the process gate \
         opened; got: {}",
        err
    );
    assert!(
        !err.contains("process execution denied"),
        "allow_process() did not reach the VM sandbox; got: {}",
        err
    );
}

// ======================================================================
// 6 — An absent transport defaults to stdio (documented default, one place)
// ======================================================================
#[test]
fn absent_transport_defaults_to_stdio() {
    let src = r#"
mcp Def { command: "some-mcp-server"; }
"#;
    let err = run_with(src, false, false).unwrap_err();
    assert!(
        err.contains("process execution denied"),
        "a declaration with no transport must default to stdio; got: {}",
        err
    );
}

// ======================================================================
// 7 — An unrecognised transport is an error, not a silent stdio downgrade
//     (Kural 7c: no fallback)
// ======================================================================
#[test]
fn unknown_transport_is_an_error() {
    let src = r#"
mcp Bad { transport: "grpc"; url: "https://example.invalid"; }
"#;
    let err = run_with(src, true, true).unwrap_err();
    assert!(
        err.contains("unknown transport 'grpc'"),
        "expected an explicit unknown-transport error; got: {}",
        err
    );
}

#[test]
fn unknown_transport_error_names_the_valid_options() {
    let src = r#"
mcp Bad { transport: "websocket"; }
"#;
    let err = run_with(src, true, true).unwrap_err();
    assert!(
        err.contains("stdio") && err.contains("sse"),
        "the error must tell the author what is accepted; got: {}",
        err
    );
}

// ======================================================================
// 8 — Case tolerance kept for the forms the VM already accepted
// ======================================================================
#[test]
fn uppercase_sse_is_still_accepted() {
    let src = r#"
mcp Web { transport: "SSE"; url: "https://example.invalid/sse"; }
"#;
    let err = run_with(src, false, false).unwrap_err();
    assert!(
        err.contains("network access denied"),
        "\"SSE\" must resolve to the sse transport; got: {}",
        err
    );
}
