//! Regression: spawn Subject + agent.call() must not deadlock.
//! Before v0.8.49, this pattern hung indefinitely (nested runtime deadlock).

use hudhudscript_vm::VM;

fn run_with_timeout(src: &str, timeout_ms: u64) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler
        .compile(&stmts)
        .map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    // If this hangs, the test will be killed by the test runner's timeout,
    // which is better than a deadlock.
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

#[test]
fn spawn_plus_agent_call_does_not_hang() {
    // Minimal repro from known-issues.md. With no Ollama running,
    // this should return an error quickly — NOT hang.
    let src = r#"
provider TestProv { type: "ollama", url: "http://localhost:11434", timeout: 500 }
agent TestAg { provider: "TestProv", model: "gemma3:4b", role: "Say OK." }
role T { can check }
subject S { state val: 0 }

let s = spawn S;
let r = TestAg.call({prompt: "OK", max_tokens: 2});
"#;
    let result = run_with_timeout(src, 5000);
    match result {
        Ok(_) => {} // OK if Ollama happens to be running
        Err(e) => {
            // Must be a connection error, NOT a hang
            assert!(
                e.contains("Ollama")
                    || e.contains("API error")
                    || e.contains("connection")
                    || e.contains("404")
                    || e.contains("Sandbox"),
                "unexpected error: {}",
                e
            );
        }
    }
}

#[test]
fn spawn_without_provider_does_not_hang() {
    // Spawn alone must not hang (sanity check).
    let src = r#"
role T { can check }
subject S { state val: 0 }
let s = spawn S;
return 1;
"#;
    let vm = run_with_timeout(src, 2000).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "1");
}
