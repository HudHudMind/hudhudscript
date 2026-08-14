//! Comprehensive deep nested lexical scope tests (up to 8+ levels of nesting)
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(src: &str) -> VM {
    let stmts = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&stmts).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bc).expect("runtime failed");
    vm
}

#[test]
fn test_8_level_block_scope_shadowing_and_unwinding() {
    let vm = run(r#"
        let x = 1;
        let trace = "";
        {
            let x = 2;
            {
                let x = 3;
                {
                    let x = 4;
                    {
                        let x = 5;
                        {
                            let x = 6;
                            {
                                let x = 7;
                                {
                                    let x = 8;
                                    trace = trace + x + ",";
                                }
                                trace = trace + x + ",";
                            }
                            trace = trace + x + ",";
                        }
                        trace = trace + x + ",";
                    }
                    trace = trace + x + ",";
                }
                trace = trace + x + ",";
            }
            trace = trace + x + ",";
        }
        trace = trace + x;
        let final_x = x;
    "#);

    assert_eq!(
        vm.get_variable("final_x").and_then(|v| v.as_int()),
        Some(1),
        "Outermost x must remain 1 after 8-level nesting unwinds"
    );
    assert_eq!(
        vm.get_variable("trace").and_then(|v| v.as_string()),
        Some("8,7,6,5,4,3,2,1".to_string()),
        "Scope unwinding must resolve each level's shadowed variable in exact reverse order"
    );
}

#[test]
fn test_8_level_nested_if_blocks_with_variable_mutation_and_shadowing() {
    let vm = run(r#"
        let outer_acc = 0;
        let shadow_var = 100;
        if true {
            let shadow_var = 200;
            outer_acc = outer_acc + 1;
            if true {
                let shadow_var = 300;
                outer_acc = outer_acc + 1;
                if true {
                    let shadow_var = 400;
                    outer_acc = outer_acc + 1;
                    if true {
                        let shadow_var = 500;
                        outer_acc = outer_acc + 1;
                        if true {
                            let shadow_var = 600;
                            outer_acc = outer_acc + 1;
                            if true {
                                let shadow_var = 700;
                                outer_acc = outer_acc + 1;
                                if true {
                                    let shadow_var = 800;
                                    outer_acc = outer_acc + 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    "#);

    assert_eq!(
        vm.get_variable("outer_acc").and_then(|v| v.as_int()),
        Some(7),
        "Outer variable mutated across 7 nested if-blocks must accumulate 7"
    );
    assert_eq!(
        vm.get_variable("shadow_var").and_then(|v| v.as_int()),
        Some(100),
        "Outermost shadow_var must remain 100 unaffected by inner block-scoped lets"
    );
}

#[test]
fn test_8_level_interleaved_variables_in_mixed_constructs() {
    let vm = run(r#"
        let a = 10;
        let b = 20;
        let result = 0;
        {
            let a = 11;
            if true {
                let b = 21;
                {
                    let a = 12;
                    if true {
                        let b = 22;
                        {
                            let a = 13;
                            if true {
                                let b = 23;
                                {
                                    let a = 14;
                                    result = a + b;
                                }
                            }
                        }
                    }
                }
            }
        }
        let final_a = a;
        let final_b = b;
    "#);

    assert_eq!(
        vm.get_variable("result").and_then(|v| v.as_int()),
        Some(37),
        "Deepest level expression a (14) + b (23) must equal 37"
    );
    assert_eq!(
        vm.get_variable("final_a").and_then(|v| v.as_int()),
        Some(10),
        "Outer variable 'a' must remain 10"
    );
    assert_eq!(
        vm.get_variable("final_b").and_then(|v| v.as_int()),
        Some(20),
        "Outer variable 'b' must remain 20"
    );
}

#[test]
fn test_8_level_loop_engineering_step_deep_nesting() {
    let vm = run(r#"
        let observed_outer = 0;
        let innermost_val = 0;
        loop DeepStepLoop {
            step execute {
                let s = 1;
                if true {
                    let s = 2;
                    {
                        let s = 3;
                        if true {
                            let s = 4;
                            {
                                let s = 5;
                                if true {
                                    let s = 6;
                                    {
                                        let s = 7;
                                        if true {
                                            let s = 8;
                                            innermost_val = s;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                observed_outer = s;
                gate check { when true -> done else -> done }
            }
        }
        run loop DeepStepLoop;
    "#);

    assert_eq!(
        vm.get_variable("innermost_val").and_then(|v| v.as_int()),
        Some(8),
        "Innermost step scope must observe s = 8"
    );
    assert_eq!(
        vm.get_variable("observed_outer").and_then(|v| v.as_int()),
        Some(1),
        "Step-level s must remain 1 after 8-level nesting in step completes"
    );
}
