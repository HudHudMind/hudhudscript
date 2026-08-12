use hudhudscript_compiler::compiler::decl::loop_engine::validate_loop_structure;
use hudhudscript_compiler::compiler::Compiler;
use hudhudscript_parser::parse;

#[test]
fn parse_and_validate_loop() {
    let src = "loop ci { step s1 { let r = 0; gate g1 { when r==0 -> done else -> fail } } }";
    let ast = parse(src).unwrap();
    assert!(validate_loop_structure(&ast).is_ok());
}

#[test]
fn parse_and_compile_chain() {
    let src = "chain c { loop l1 { step s1 { } } loop l2 { step s2 { } } }";
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::default();
    assert!(compiler.compile(&ast).is_ok());
}

#[test]
fn parse_validate_loop_registry() {
    let src = "loop a { step s { } }";
    let ast = parse(src).unwrap();
    assert!(validate_loop_structure(&ast).is_ok());
}
