fn main() {
    let src = r#"let c = 0; for (let i = 0; i < 10; i = i + 1) { if (i == 3) { continue; } c = c + 1; }"#;
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    for (j, instr) in bc.instructions.iter().enumerate() {
        println!("{}: {:?}", j, instr);
    }
}
