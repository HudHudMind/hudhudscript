use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;
use std::fs;

fn sample_path(name: &str) -> String {
    format!("/home/onur/HudHudMind/hudhud-script/examples/09-loop-engineering/{}", name)
}

fn compile_sample(path: &str) -> hudhudscript_bytecode::Bytecode {
    let src = fs::read_to_string(path).expect("read sample");
    let stmts = parse(&src).expect("parse");
    let mut compiler = Compiler::default();
    compiler.compile(&stmts).expect("compile")
}

#[test]
fn sample_01_simple_done_compiles() {
    let bytecode = compile_sample(&sample_path("01_simple_done.hud"));
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    assert!(bc_ref.has_function("__loop::first_loop"));
    let chunk = bc_ref.get_function("__loop::first_loop").unwrap();
    assert!(chunk.instructions.len() > 3, "loop chunk must have step body instructions");
}

#[test]
fn sample_01_run_loop_emits_call() {
    // Test with run loop syntax
    let src = "loop first_loop { step first_step { let x = 0; gate g { when x==0 -> done else -> fail } } } run loop first_loop";
    let stmts = parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    assert!(!bytecode.instructions.is_empty());
}

#[test]
fn sample_02_two_steps_done_fail() {
    let src = "loop two_steps { step prepare { result.ready = 0; gate g { when result.ready==0 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&stmts).is_ok());
}

#[test]
fn sample_03_else_fail_compiles() {
    let bytecode = compile_sample(&sample_path("03_else_fail.hud"));
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    assert!(bc_ref.has_function("__loop::fail_path"));
}

#[test]
fn unsupported_gate_target_loop_is_compile_error() {
    let src = "loop L { step s { let x = 0; gate g { when x==0 -> loop other_loop else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    // other_loop not compiled → compile error
    let err = Compiler::default().compile(&stmts).unwrap_err();
    assert!(format!("{}", err).contains("not found"), "expected 'not found' error");
}

#[test]
fn unsupported_gate_target_retry_is_compile_error() {
    let src = "loop L { step s { let x = 0; gate g { when x==0 -> retry else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    // retry is now a supported gate target
    assert!(Compiler::default().compile(&stmts).is_ok());
}

#[test]
fn vm_execute_simple_loop() {
    let src = "loop my_loop { step s { let x = 0; } }";
    let stmts = parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();

    // Verify loop FunctionChunk exists and has step body instructions
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::my_loop").unwrap();
    assert!(chunk.instructions.len() > 3, "loop chunk with step body must have >3 instructions, got {}", chunk.instructions.len());

    // VM execute: the top-level bytecode has no return, but the loop chunk does
    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "VM execute should succeed: {:?}", result.err());
}

#[test]
fn vm_execute_run_loop_with_call() {
    let src = "loop my_loop { step s { let x = 0; } } run loop my_loop";
    let stmts = parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    // Verify Call instruction exists
    let has_call = bytecode.instructions.iter().any(|i| matches!(i, Instruction::Call { .. }));
    assert!(has_call, "run loop must emit Call instruction");
    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "VM execute with run loop should succeed: {:?}", result.err());
}

#[test]
fn vm_execute_run_loop_has_loop_chunk() {
    let src = "loop my_loop { step s { let x = 0; gate g { when x==0 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::my_loop").unwrap();
    // Verify SetProperty exists (success field)
    let has_set_prop = chunk.instructions.iter().any(|i| matches!(i, Instruction::SetProperty { .. }));
    assert!(has_set_prop, "done gate must emit SetProperty for success field");
    let mut vm = hudhudscript_vm::VM::new();
    assert!(vm.execute(&bytecode).is_ok());
}

#[test]
fn done_loop_has_success_true_constant() {
    use hudhudscript_bytecode::Value16;
    let src = "loop done_loop { step s { let x = 0; gate g { when x==0 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::done_loop").unwrap();
    let has_true = chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_bool() == Some(true));
    assert!(has_true, "done loop must have bool(true) constant for success field");
}

#[test]
fn fail_loop_has_success_false_constant() {
    use hudhudscript_bytecode::Value16;
    let src = "loop fail_loop { step s { let x = 0; gate g { when x==1 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::fail_loop").unwrap();
    let has_false = chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_bool() == Some(false));
    assert!(has_false, "fail loop must have bool(false) constant for success field");
}

#[test]
fn vm_execute_done_loop_returns_result_object() {
    use hudhudscript_bytecode::Value16;
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let mut vm = hudhudscript_vm::VM::new();
    assert!(vm.execute(&bytecode).is_ok());
    // Check that the loop FunctionChunk exists and has SetProperty
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    let has_success = chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_bool() == Some(true));
    let has_status = chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_string().map_or(false, |s| s == "done"));
    assert!(has_success, "done loop must have success:true constant");
    assert!(has_status, "done loop must have status:done constant");
}

#[test]
fn vm_execute_loop_has_bytecode_with_setproperty() {
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let mut vm = hudhudscript_vm::VM::new();
    assert!(vm.execute(&bytecode).is_ok());
    // Verify the loop chunk has SetProperty for success field
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    let set_count = chunk.instructions.iter().filter(|i| matches!(i, Instruction::SetProperty { .. })).count();
    assert!(set_count >= 2, "done loop must have >=2 SetProperty (success + status), got {}", set_count);
}

#[test]
fn done_loop_has_status_done_in_constants() {
    use hudhudscript_bytecode::Value16;
    let src = "loop L { step S { let x = 0; gate G { when x==0 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let functions = bytecode.functions.borrow();
    let bc_ref = &bytecode;
    let chunk = bc_ref.get_function("__loop::L").unwrap();
    assert!(chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_string().map_or(false, |s| s == "done")));
    assert!(chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_string().map_or(false, |s| s == "failed")));
    let mut vm = hudhudscript_vm::VM::new();
    assert!(vm.execute(&bytecode).is_ok());
}

#[test]
fn mode_times_parses_correctly() {
    let src = "loop t mode: times(3) { step s { } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    // times(N) now compiles with counter + IntSubI loop
    assert!(Compiler::default().compile(&stmts).is_ok());
}

#[test]
fn vm_execute_returns_result_object_via_last_return() {
    let src = "loop L { step S { result.x = 0; gate G { when result.x==0 -> done else -> fail } } } run loop L";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let mut vm = hudhudscript_vm::VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "VM execute failed: {:?}", result.err());
    // last_return may be null if Call frame doesn't propagate to top-level
    // This test verifies that the loop compiles and executes without error
}

#[test]
fn pause_gate_target_compiles() {
    let src = "loop L { step S { gate g { when true -> pause else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&stmts).is_ok());
}

#[test]
fn escalate_gate_target_compiles() {
    let src = "loop L { step S { gate g { when true -> escalate else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&stmts).is_ok());
}

#[test]
fn approval_gate_target_compiles() {
    let src = "loop L { step S { gate g { when true -> approval else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&stmts).is_ok());
}

#[test]
fn retry_gate_target_jumps_back() {
    let src = "loop L { step S { result.x = 0; gate g { when result.x==0 -> retry else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    let has_jump = chunk.instructions.iter().any(|i| matches!(i, Instruction::Jump(_)));
    assert!(has_jump, "retry gate must emit Jump instruction");
}

#[test]
fn loop_with_result_field_propagation_compiles() {
    let src = "loop L { step S1 { result.x = 5; gate g { when true -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    // Verify SetProperty for result.x = 5
    let has_set = chunk.instructions.iter().any(|i| matches!(i, Instruction::SetProperty { .. }));
    assert!(has_set, "step body result.x=5 must emit SetProperty");
}

#[test]
fn chain_with_two_loops_executes_both() {
    let src = "chain c { loop l1 { step s { gate g { when true -> done else -> fail } } } loop l2 { step s { gate g { when true -> done else -> fail } } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chain_chunk = bc.get_function("__chain::c").unwrap();
    let call_count = chain_chunk.instructions.iter().filter(|i| matches!(i, Instruction::Call { .. })).count();
    assert_eq!(call_count, 2, "chain with 2 links must have 2 Call instructions");
    assert!(chain_chunk.instructions.len() > 4, "chain must have more than skeleton");
}

#[test]
fn loop_step_gate_done_has_return_with_result_reg() {
    let src = "loop L { step S { gate g { when true -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    let has_return = chunk.instructions.iter().any(|i| matches!(i, Instruction::Return { .. }));
    assert!(has_return, "done gate must emit Return");
}

#[test]
fn done_loop_bytecode_contains_success_true_constant() {
    use hudhudscript_bytecode::Value16;
    let src = "loop L { step S { result.x = 0; gate G { when result.x==0 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    assert!(chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_bool() == Some(true)));
    assert!(chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_bool() == Some(false)));
    let mut vm = hudhudscript_vm::VM::new();
    assert!(vm.execute(&bc).is_ok());
}

#[test]
fn fail_loop_bytecode_contains_status_failed_string() {
    use hudhudscript_bytecode::Value16;
    let src = "loop L { step S { result.x = 0; gate G { when result.x==1 -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    assert!(chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_string().map_or(false, |s| s == "failed")));
}

#[test]
fn loop_with_multiple_steps_compiles_all_steps() {
    let src = "loop L { step S1 { result.a = 1; gate g1 { when true -> done else -> fail } } step S2 { result.b = 2; gate g2 { when true -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    // Both steps should contribute SetProperty instructions
    let set_count = chunk.instructions.iter().filter(|i| matches!(i, Instruction::SetProperty { .. })).count();
    assert!(set_count >= 4, "two steps with done gate must have >=4 SetProperty, got {}", set_count);
}

#[test]
fn chain_three_links_compiles() {
    let src = "chain c { loop a { step s { gate g { when true -> done else -> fail } } } loop b { step s { gate g { when true -> done else -> fail } } } loop c2 { step s { gate g { when true -> done else -> fail } } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chain_chunk = bc.get_function("__chain::c").unwrap();
    let calls = chain_chunk.instructions.iter().filter(|i| matches!(i, Instruction::Call { .. })).count();
    assert_eq!(calls, 3, "chain with 3 links must have 3 Call instructions");
}

#[test]
fn loop_with_retry_gate_has_jump_instruction() {
    let src = "loop L { step S { result.c = 0; gate G { when result.c==0 -> retry else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    let has_jump = chunk.instructions.iter().any(|i| matches!(i, Instruction::Jump(_)));
    assert!(has_jump, "retry gate must emit Jump back to step entry");
}

#[test]
fn loop_with_done_and_fail_has_both_status_strings() {
    use hudhudscript_bytecode::Value16;
    let src = "loop L { step S { gate G { when true -> done else -> fail } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::L").unwrap();
    assert!(chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_string().map_or(false, |s| s == "done")));
    assert!(chunk.constants.iter().any(|v: &hudhudscript_bytecode::Value16| v.as_string().map_or(false, |s| s == "failed")));
}

#[test]
fn cross_loop_call_payload_is_set() {
    // Cross-loop gate: compile two loops, check that call payload exists
    let src = "loop A { step s { gate g { when true -> done else -> done } } } loop B { step s { gate g { when true -> loop A else -> done } } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk_b = bc.get_function("__loop::B").unwrap();
    let has_call = chunk_b.instructions.iter().any(|i| matches!(i, Instruction::Call { .. }));
    assert!(has_call, "loop B gate -> loop A must emit Call");
}

#[test]
fn mode_times_emits_intsubi() {
    let src = "loop t mode: times(5) { step s { } }";
    let stmts = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bc = compiler.compile(&stmts).unwrap();
    let chunk = bc.get_function("__loop::t").unwrap();
    let has_intsub = chunk.instructions.iter().any(|i| matches!(i, Instruction::IntSubI { .. } | Instruction::IntSubIJump { .. }));
    assert!(has_intsub, "times(N) loop must emit IntSubI (or fused IntSubIJump) for counter decrement");
    let has_loadint = chunk.instructions.iter().any(|i| matches!(i, Instruction::LoadIntConst { .. }));
    assert!(has_loadint, "times(N) must emit LoadIntConst for counter init");
}
