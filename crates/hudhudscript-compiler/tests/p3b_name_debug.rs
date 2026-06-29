// P3b: Function name mapping debug test.
// Verifies that function names in bytecode match source order.

#[cfg(test)]
mod tests {
    use hudhudscript_compiler::Compiler;
    use hudhudscript_parser::parse;

    #[test]
    fn function_names_match_source_order() {
        let src = "
fn first(x) { return x + 1; }
fn second(y) { return first(y); }
print(second(41));
";
        let ast = parse(src).expect("parse");
        let mut compiler = Compiler::new();
        let bc = compiler.compile(&ast).expect("compile");

        let names = bc.function_names.borrow();
        let keys: Vec<&String> = names.keys().collect();
        eprintln!("Registry keys: {:?}", keys);
        eprintln!("Function count: {}", names.len());

        // first and second should both be in the registry
        assert!(names.contains_key("first"), "first should be in function names");
        assert!(names.contains_key("second"), "second should be in function names");
    }

    #[test]
    fn two_functions_declared_in_order_caller_can_inline_callee() {
        let src = "
fn add1(x) { return x + 1; }
fn caller(y) { return add1(y); }
";
        let ast = parse(src).expect("parse");
        let mut compiler = Compiler::new();
        let bc = compiler.compile(&ast).expect("compile");

        let funcs = bc.functions.borrow();
        let names = bc.function_names.borrow();
        eprintln!("Function names: {:?}", names.keys().collect::<Vec<_>>());

        // Check that caller's body does NOT have a Call to add1
        if let Some(&caller_idx) = names.get("caller") {
            if let Some(chunk) = funcs.get(caller_idx) {
                let has_call_to_add1 = chunk.instructions.iter().any(|i| {
                    matches!(i, hudhudscript_bytecode::Instruction::Call { .. })
                });
                assert!(
                    !has_call_to_add1,
                    "caller body should not have Call after inlining add1. Instructions: {:?}",
                    chunk.instructions
                );
            }
        }
    }
}
