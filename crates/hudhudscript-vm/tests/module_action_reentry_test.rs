//! Production-shaped module agent-action regression test.

use hudhudscript_bytecode::Bytecode;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;
use std::path::{Path, PathBuf};

fn compile_fixture(path: &Path, module_base_dir: &Path) -> Bytecode {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let ast = parse(&source)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()));
    let mut compiler = Compiler::new();
    compiler.set_module_base_dir(module_base_dir.to_path_buf());
    compiler
        .compile(&ast)
        .unwrap_or_else(|error| panic!("failed to compile {}: {error}", path.display()))
}

#[test]
fn production_shape_imported_action_calls_module_function_once() {
    let fixture_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/module_action_reentry");
    let module = compile_fixture(
        &fixture_dir.join("iteration_agent_process.hud"),
        &fixture_dir,
    );
    let main = compile_fixture(&fixture_dir.join("main.hud"), &fixture_dir);

    assert_eq!(
        main.function_name_at(8).expect("parent function index 8"),
        "parent_loop"
    );
    assert_eq!(
        module.function_name_at(8).expect("module function index 8"),
        "invoke_agent"
    );

    let mut vm = VM::new();
    vm.execute(&main)
        .expect("production-shaped fixture must execute");

    let result = vm
        .get_variable("production_result")
        .expect("production_result must be published");
    let object = result
        .as_object()
        .expect("production_result must be an object");

    assert_eq!(
        object
            .get("parent_loop_calls")
            .and_then(|value| value.as_int()),
        Some(0),
        "the parent function at the stale module index must never execute"
    );
    assert_eq!(
        object
            .get("invoke_agent_calls")
            .and_then(|value| value.as_int()),
        Some(1),
        "the imported invoke_agent function must execute exactly once"
    );
    assert_eq!(
        object.get("result").and_then(|value| value.as_str()),
        Some("invoke-agent-ok")
    );
    assert_eq!(object.len(), 3, "the result contract must have exact keys");
}
