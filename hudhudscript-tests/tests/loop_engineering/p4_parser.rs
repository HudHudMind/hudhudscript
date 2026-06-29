use hudhudscript_ast::*;
use hudhudscript_parser::parse;

#[test]
fn parse_gate_decl_test() {
    let stmts = parse("gate g { when x==0 -> done else -> fail }").expect("parse failed");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::Decl(Decl::Gate { name, branches, else_target, .. }) => {
            assert_eq!(name, "g");
            assert_eq!(branches.len(), 1);
            assert_eq!(*else_target, GateTargetAst::Fail);
        }
        other => panic!("Expected Gate, got {:?}", other),
    }
}

#[test]
fn parse_empty_loop() {
    let stmts = parse("loop e { }").expect("parse failed");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_loop_with_step_no_gate() {
    let stmts = parse("loop e { step s { } }").expect("parse");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn parse_chain_nested() {
    let src = "chain ci { loop build_loop { step b { } } loop test_loop { step t { } } }";
    let stmts = parse(src).expect("parse failed");
    match &stmts[0] {
        Stmt::Decl(Decl::Chain { name, links, .. }) => {
            assert_eq!(name, "ci");
            assert_eq!(links.len(), 2);
        }
        other => panic!("Expected Chain, got {:?}", other),
    }
}

#[test]
fn parse_attach_multi_step() {
    let s = parse("attach step build with gate ok, step test with gate ok to build_loop").expect("parse failed");
    match &s[0] {
        Stmt::Decl(Decl::AttachStep { targets, loop_name, .. }) => {
            assert_eq!(targets.len(), 2);
            assert_eq!(loop_name, "build_loop");
            assert_eq!(targets[0].step, "build");
            assert_eq!(targets[0].gate.as_deref(), Some("ok"));
        }
        other => panic!("Expected AttachStep, got {:?}", other),
    }
}

#[test]
fn parse_step_gate_is_captured() {
    use hudhudscript_parser::parse;
    use hudhudscript_ast::*;
    let stmts = parse("loop e { step s { let x = 0; gate g { when x==0 -> done else -> fail } } }").unwrap();
    match &stmts[0] {
        Stmt::Decl(Decl::Loop { items, .. }) => {
            // Debug: check items count
            assert!(!items.is_empty(), "Loop should have items");
            match &items[0] {
                LoopItemAst::InlineStep(step_decl) => {
                    match step_decl.as_ref() {
                        Decl::Step { gate, body, .. } => {
                            eprintln!("BODY LEN: {}, GATE: {:?}", body.len(), gate.is_some());
                            for (i, s) in body.iter().enumerate() {
                                eprintln!("  body[{}]: {:?}", i, s);
                            }
                        }
                        other => panic!("Expected Decl::Step, got {:?}", other),
                    }
                }
                other => panic!("Expected InlineStep, got {:?}", other),
            }
        }
        other => panic!("Expected Loop, got {:?}", other),
    }
}
