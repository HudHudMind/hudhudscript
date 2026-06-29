//! PROVIDER0004: Integration tests for provider declaration → call dispatch.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> Result<String, String> {
    let ast = parse(source).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&ast)
        .map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.allow_network();
    std::env::set_var("PROVIDER_TEST_KEY", "test-key");
    match vm.execute(&bytecode) {
        Ok(_) => Ok("ok".into()),
        Err(e) => Err(format!("{}", e)),
    }
}

#[test]
fn test_provider_decl_obj_has_fields() {
    let src = r#"
        provider TestAI {
            type: "deepseek",
            api_key: env("PROVIDER_TEST_KEY"),
            model: "test-model"
        }
        print(TestAI["type"])
        print(TestAI["model"])
    "#;
    // Just verify it compiles and runs — provider object accessible
    let r = run(src);
    // Should succeed even without network (provider obj is just data)
    assert!(r.is_ok(), "Provider decl failed: {:?}", r.err());
}

#[test]
fn test_provider_decl_missing_type_errors() {
    let src = r#"
        provider Bad {
            api_key: env("PROVIDER_TEST_KEY")
        }
        Bad.call({ prompt: "hi" })
    "#;
    let r = run(src);
    assert!(r.is_err(), "Should fail on missing type");
    let err = r.unwrap_err();
    assert!(
        err.contains("missing required") || err.contains("type"),
        "Error should mention missing type: {}",
        err
    );
}

#[test]
fn test_provider_decl_unknown_type_errors() {
    let src = r#"
        provider Bad {
            type: "nonexistent_llm_xyz",
            api_key: env("PROVIDER_TEST_KEY")
        }
        Bad.call({ prompt: "hi" })
    "#;
    let r = run(src);
    assert!(r.is_err(), "Should fail on unknown provider type");
    let err = r.unwrap_err();
    assert!(
        err.contains("Unknown") || err.contains("nonexistent_llm"),
        "Error should mention unknown type: {}",
        err
    );
}

#[test]
fn test_provider_decl_unused_no_network() {
    // Declare but never call — should NOT touch network
    let src = r#"
        provider Unused {
            type: "deepseek",
            api_key: env("PROVIDER_TEST_KEY")
        }
        print("ok")
    "#;
    let r = run(src);
    assert!(
        r.is_ok(),
        "Unused provider decl should not fail: {:?}",
        r.err()
    );
}
