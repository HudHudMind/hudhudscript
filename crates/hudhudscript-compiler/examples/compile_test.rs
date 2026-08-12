use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use std::fs;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = fs::read_to_string("examples/arabic_working.hudhud")?;
    let ast = parse(&src)?;
    let bc = Compiler::new().compile(&ast)?;
    for (i, inst) in bc.instructions.iter().enumerate() { println!("  [{}] {:?}", i, inst); }
    for (fi, c) in bc.functions.borrow().iter().enumerate() { println!("  F{}:",fi); for (j,i) in c.instructions.iter().enumerate() { println!("    [{}] {:?}",j,i); } }
    Ok(())
}
