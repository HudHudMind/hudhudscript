use hudhudscript_compiler::compiler::Compiler;
use hudhudscript_parser::parse;

fn compile_err(src: &str) -> String {
    let stmts = parse(src).unwrap();
    let err = Compiler::default().compile(&stmts).unwrap_err();
    format!("{}", err)
}

fn compile_ok(src: &str) -> hudhudscript_bytecode::Bytecode {
    let stmts = parse(src).unwrap();
    Compiler::default().compile(&stmts).unwrap()
}

#[test]
fn duplicate_loop_compile_error() {
    let e = compile_err("loop x { step s { } } loop x { step t { } }");
    assert!(
        e.contains("duplicate"),
        "expected duplicate error, got: {}",
        e
    );
}

#[test]
#[test]
fn chain_empty_compile_error() {
    let e = compile_err("chain c { }");
    assert!(e.contains("must have at least one link"), "got: {}", e);
}

#[test]
fn run_loop_missing_compile_error() {
    let e = compile_err("run loop ghost_loop");
    assert!(e.contains("not found"), "got: {}", e);
}

#[test]
fn gate_without_else_parse_error() {
    assert!(parse("gate g { when x==0 -> done }").is_err());
}

#[test]
fn duplicate_chain_compile_error() {
    let e = compile_err("chain c { loop l { step s { } } } chain c { loop l2 { step s2 { } } }");
    assert!(e.contains("duplicate"), "got: {}", e);
}

#[test]
fn step_body_compiles_with_let_assignment() {
    let src = "loop L { step S { let x = 1; } }";
    let stmts = parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    assert!(
        chunk.instructions.len() >= 3,
        "step body must produce instructions"
    );
}

#[test]
fn loop_with_gate_produces_conditional_bytecode() {
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done else -> fail } } }";
    let bytecode = compile_ok(src);
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    // Step body + gate conditions must produce substantial bytecode
    assert!(
        chunk.instructions.len() >= 5,
        "gate must expand bytecode, got {} instructions",
        chunk.instructions.len()
    );
}

#[test]
fn loop_compiled_function_chunk_not_empty() {
    let src = "loop L { step S { let x = 0; } }";
    let bytecode = compile_ok(src);
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn anti_fake_no_body_exec_module() {
    assert!(
        !std::path::Path::new("crates/hudhudscript-compiler/src/compiler/decl/body_exec.rs")
            .exists()
    );
    assert!(
        !std::path::Path::new("crates/hudhudscript-compiler/src/compiler/decl/gate_eval.rs")
            .exists()
    );
}

#[test]
fn anti_fake_no_orchestration_loop_engine_dir() {
    assert!(!std::path::Path::new("crates/hudhudscript-orchestration/src/loop_engine").exists());
}

#[test]
fn run_chain_parses() {
    let src = "run chain my_chain";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(
        matches!(&stmts[0], hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::RunChain { name, .. }) if name == "my_chain")
    );
}

#[test]
fn run_chain_missing_compile_error() {
    let src = "run chain ghost_chain";
    let stmts = parse(src).unwrap();
    let err = Compiler::default().compile(&stmts).unwrap_err();
    assert!(format!("{}", err).contains("not found"));
}

#[test]
fn run_chain_emits_call_instruction() {
    let src = "chain c { loop l { step s { } } } run chain c";
    let stmts = parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let has_call = bytecode
        .instructions
        .iter()
        .any(|i| matches!(i, hudhudscript_bytecode::Instruction::Call { .. }));
    assert!(has_call, "run chain must emit Call instruction");
}

#[test]
fn chain_unknown_link_compiles_inline() {
    // Inline loops in chains are compiled, so unknown links work as inline
    let src = "chain c { loop unknown_s { step s { } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut c = hudhudscript_compiler::compiler::Compiler::default();
    assert!(c.compile(&stmts).is_ok());
}

#[test]
fn duplicate_step_in_loop_compile_error() {
    let src = "loop L { step s { } step s { } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let err = hudhudscript_compiler::compiler::Compiler::default()
        .compile(&stmts)
        .unwrap_err();
    assert!(
        format!("{}", err).contains("duplicate"),
        "expected duplicate step error"
    );
}

#[test]
fn gate_target_unknown_step_compile_error() {
    let src = "loop L { step s { gate g { when true -> nonexistent else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let err = hudhudscript_compiler::compiler::Compiler::default()
        .compile(&stmts)
        .unwrap_err();
    assert!(
        format!("{}", err).contains("not found"),
        "expected 'not found' error, got: {}",
        err
    );
}

#[test]
fn mode_cyclic_without_stopper_compile_error() {
    let src = "loop L mode: cyclic { step s { } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let err = hudhudscript_compiler::compiler::Compiler::default()
        .compile(&stmts)
        .unwrap_err();
    assert!(
        format!("{}", err).contains("cyclic"),
        "expected cyclic error, got: {}",
        err
    );
}

#[test]
#[test]
fn run_loop_unknown_target_compile_error() {
    let src = "run loop nonexistent_loop";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let err = hudhudscript_compiler::compiler::Compiler::default()
        .compile(&stmts)
        .unwrap_err();
    assert!(format!("{}", err).contains("not found"));
}

#[test]
fn gate_target_loop_step_parses_correctly() {
    let src = "loop L { step S { gate g { when true -> done else -> other.step_s } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let err = hudhudscript_compiler::compiler::Compiler::default().compile(&stmts);
    // other loop doesn't exist, but the AST should parse correctly
    assert!(err.is_err(), "unknown loop should produce compile error");
}

#[test]
#[test]
fn mode_times_n_syntax_parsed() {
    let src = "loop t mode: times(3) { step s { } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    // times(N) now compiles with counter loop
    assert!(hudhudscript_compiler::compiler::Compiler::default()
        .compile(&stmts)
        .is_ok());
}

#[test]
fn mode_until_converged_compile_error() {
    let src = "loop u mode: until_converged { step s { } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let err = hudhudscript_compiler::compiler::Compiler::default()
        .compile(&stmts)
        .unwrap_err();
    assert!(
        format!("{}", err).contains("until"),
        "expected until mode error"
    );
}
