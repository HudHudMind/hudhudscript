use hudhudscript_parser::parse;
use hudhudscript_compiler::compiler::decl::loop_engine::validate_loop_structure;
use hudhudscript_compiler::compiler::Compiler;

#[test]
fn gate_without_else_is_parse_error() {
    assert!(parse("gate g { when x==0 -> done }").is_err());
}

#[test]
fn valid_loop_validates() {
    let src = "loop ci { step s1 { let r = 0; gate g1 { when r==0 -> done else -> fail } } }";
    let ast = parse(src).unwrap();
    assert!(validate_loop_structure(&ast).is_ok());
}

#[test]
fn empty_loop_is_error() {
    let ast = parse("loop x { }").unwrap();
    assert!(validate_loop_structure(&ast).is_err());
}

#[test]
fn duplicate_loop_is_error() {
    let src = "loop x { step s { } } loop x { step t { } }";
    let ast = parse(src).unwrap();
    assert!(validate_loop_structure(&ast).is_err());
}

#[test]
fn chain_validates() {
    let src = "chain c { loop l1 { step s1 { } } loop l2 { step s2 { } } }";
    let ast = parse(src).unwrap();
    assert!(validate_loop_structure(&ast).is_ok());
}

#[test]
fn no_body_exec_file_exists() {
    let body_exec = std::path::Path::new(
        "crates/hudhudscript-compiler/src/compiler/decl/body_exec.rs"
    );
    let gate_eval = std::path::Path::new(
        "crates/hudhudscript-compiler/src/compiler/decl/gate_eval.rs"
    );
    assert!(!body_exec.exists(), "body_exec.rs must be deleted");
    assert!(!gate_eval.exists(), "gate_eval.rs must be deleted");
}
