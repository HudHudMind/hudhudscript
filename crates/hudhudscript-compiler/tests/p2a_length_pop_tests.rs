// P2a: Compiler tests — ArrayLen/StringLen/ArrayPop emission guards.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_bytecode::Instruction;

fn compile_instructions(src: &str) -> Vec<Instruction> {
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile failed");
    let mut all = bc.instructions.clone();
    for chunk in bc.functions.borrow().iter() {
        all.extend_from_slice(&chunk.instructions);
    }
    all
}

fn has_instruction<F>(insns: &[Instruction], pred: F) -> bool
where
    F: Fn(&Instruction) -> bool,
{
    insns.iter().any(pred)
}

#[test]
fn typed_array_length_emits_arraylen() {
    let insns = compile_instructions("let a = [1,2,3]; let n = a.length;");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::ArrayLen { .. })),
        "typed array .length must emit ArrayLen"
    );
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::GetProperty { .. })),
        "typed array .length must NOT emit GetProperty"
    );
}

#[test]
fn typed_string_length_emits_stringlen() {
    let insns = compile_instructions("let s = \"hello\"; let n = s.length;");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::StringLen { .. })),
        "typed string .length must emit StringLen"
    );
}

#[test]
fn uncalled_function_length_uses_getproperty() {
    let insns = compile_instructions("fn f(x) { return x.length; }");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::GetProperty { .. })),
        "uncalled function .length must emit GetProperty"
    );
}

#[test]
fn called_with_array_length_uses_arraylen() {
    let insns = compile_instructions("fn f(x) { return x.length; } print(f([1,2]));");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::ArrayLen { .. })),
        "array-called function .length must emit ArrayLen"
    );
}

#[test]
fn typed_array_pop_emits_arraypop() {
    let insns = compile_instructions("let a = [1,2,3]; let x = a.pop();");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::ArrayPop { .. })),
        "typed array .pop() must emit ArrayPop"
    );
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::MethodCall { .. })),
        "typed array .pop() must NOT emit MethodCall"
    );
}

#[test]
fn uncalled_function_pop_uses_methodcall() {
    let insns = compile_instructions("fn f(x) { return x.pop(); }");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::MethodCall { .. })),
        "uncalled function .pop() must emit MethodCall"
    );
}

#[test]
fn called_with_array_pop_uses_arraypop() {
    let insns = compile_instructions("fn f(x) { return x.pop(); } print(f([1,2]));");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::ArrayPop { .. })),
        "array-called function .pop() must emit ArrayPop"
    );
}

#[test]
fn push_skip_wbr_for_typed_array() {
    let insns = compile_instructions("let a = [1,2]; a.push(3);");
    // push on typed local array → ArrayPush/ArrayPushIntConst + no WriteBackReceiver
    let has_push = has_instruction(&insns, |i| {
        matches!(i, Instruction::ArrayPush { .. }
            | Instruction::ArrayPushIntConst { .. }
            | Instruction::ArrayPushConst { .. })
    });
    assert!(
        has_push,
        "typed array push must emit ArrayPush variant"
    );
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::WriteBackReceiver(..))),
        "typed array push must NOT emit WriteBackReceiver"
    );
}
