use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

#[test]
fn test_dump_ins() {
    let src = "
        let a = 10;
        let b = 20;
        let c = a + b;
    ";
    let ast = parse(src).unwrap();
    let bc = Compiler::new().compile(&ast).unwrap();
    println!("DUMP_START");
    println!("main_local_names = {:?}", bc.main_local_names);
    println!("main_local_count = {}", bc.main_local_count);
    for (i, ins) in bc.main_chunk.instructions.iter().enumerate() {
        println!("{:02}: {:?}", i, ins);
    }
    println!("DUMP_END");
    panic!("Force fail to see output");
}
