use hudhudscript_bytecode::{Instruction, Value16};
use hudhudscript_compiler::{Compiler, VM};
use hudhudscript_parser::parse;

fn main() {
    let src = r#"
        function loop_rec(n) {
            if (n == 0) { return 0 }
            return loop_rec(n - 1)
        }
    "#;
    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");
    let funcs = bc.functions.borrow();
    let chunk = funcs.get("loop_rec").expect("chunk");
    println!("=== loop_rec (local_count={}) ===", chunk.local_count);
    for (i, inst) in chunk.instructions.iter().enumerate() {
        println!("  {:3}: {:?}", i, inst);
    }
}
