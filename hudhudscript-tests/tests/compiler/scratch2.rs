#[test]
fn test_scratch() {
    use hudhudscript_bytecode::{Instruction, Value16};
    use hudhudscript_compiler::Compiler;
    use hudhudscript_vm::VM;
    use hudhudscript_parser::parse;

    let src = r#"
        function loop_rec(n) {
            if (n == 0) { return 0 }
            return loop_rec(n - 1)
        }
        let result = loop_rec(5)
    "#;
    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");
    let mut vm = VM::new();
    if let Err(e) = vm.execute(&bc) {
        panic!("VM Error: {:?}", e);
    }
}
