use hudhudscript_compiler::compiler::Compiler;
use hudhudscript_parser::parse;

fn compile(src: &str) -> Result<hudhudscript_bytecode::Bytecode, String> {
    let stmts = parse(src).map_err(|e| e.to_string())?;
    let mut compiler = Compiler::default();
    compiler.compile(&stmts).map_err(|e| e.to_string())
}

#[test]
fn test_attach_semicolon_parsing() {
    // Semicolon'lu ve semicolon'suz formların parse edilmesi
    let src1 = "attach gate G to s;";
    let src2 = "attach gate G to s";
    let src3 = "attach step s with gate G to C;";
    let src4 = "attach loop L to chain C;";
    let src5 = "attach loop L to chain C on_done: done on_fail: fail;";

    assert!(
        parse(src1).is_ok(),
        "Failed to parse attach gate with semicolon"
    );
    assert!(
        parse(src2).is_ok(),
        "Failed to parse attach gate without semicolon"
    );
    assert!(
        parse(src3).is_ok(),
        "Failed to parse attach step with semicolon"
    );
    assert!(
        parse(src4).is_ok(),
        "Failed to parse attach loop with semicolon"
    );
    assert!(
        parse(src5).is_ok(),
        "Failed to parse attach loop with modifiers and semicolon"
    );
}

#[test]
fn test_one_step_one_gate_parser_enforcement() {
    let src_two_inline_gates = "
    loop L {
        step s {
            gate g1 { when true -> done else -> fail }
            gate g2 { when true -> done else -> fail }
        }
    }";
    let res = parse(src_two_inline_gates);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("cannot have more than one gate"));
}

#[test]
fn test_one_step_one_gate_compiler_enforcement_multiple_attached() {
    let src = "
    gate g1 { when true -> done else -> fail }
    gate g2 { when true -> done else -> fail }
    loop L {
        step s { let x = 1; }
        attach gate g1 to s;
        attach gate g2 to s;
    }";
    let res = compile(src);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("cannot have more than one attached gate"));
}

#[test]
fn test_one_step_one_gate_compiler_enforcement_inline_and_attached() {
    let src = "
    gate external { when true -> done else -> fail }
    loop L {
        step s {
            let x = 1;
            gate inline { when true -> done else -> fail }
        }
        attach gate external to s;
    }";
    let res = compile(src);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("already has an inline gate"));
}

#[test]
fn test_valid_multiple_when_branches() {
    let src = "
    loop L {
        step s {
            let x = 1;
            gate decide {
                when x == 1 -> done
                when x == 2 -> retry
                else -> fail
            }
        }
    }";
    assert!(
        compile(src).is_ok(),
        "Failed to compile valid multi-branch gate"
    );
}
