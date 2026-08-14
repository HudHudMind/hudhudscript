//! Dedicated unit tests for unparenthesized if/while control flow and G2.2 verifier checks.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> VM {
    let stmts = parse(source).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute failed");
    vm
}

#[test]
fn test_unparenthesized_if_false_skips_then_branch() {
    let vm = run(r#"
        let cond = false;
        let executed = false;
        if cond {
            executed = true;
        }
    "#);
    assert_eq!(
        vm.get_variable("executed").and_then(|v| v.as_bool()),
        Some(false),
        "unparenthesized if false must skip then branch"
    );
}

#[test]
fn test_unparenthesized_if_true_executes_then_branch() {
    let vm = run(r#"
        let cond = true;
        let executed = false;
        if cond {
            executed = true;
        }
    "#);
    assert_eq!(
        vm.get_variable("executed").and_then(|v| v.as_bool()),
        Some(true),
        "unparenthesized if true must execute then branch"
    );
}

#[test]
fn test_unparenthesized_if_else_branching() {
    let vm = run(r#"
        let cond = false;
        let branch = 0;
        if cond {
            branch = 1;
        } else {
            branch = 2;
        }
    "#);
    assert_eq!(
        vm.get_variable("branch").and_then(|v| v.as_int()),
        Some(2),
        "unparenthesized if false must jump to else branch"
    );
}

#[test]
fn test_unparenthesized_else_if_chain() {
    let vm = run(r#"
        let x = 20;
        let category = 0;
        if x == 10 {
            category = 1;
        } else if x == 20 {
            category = 2;
        } else {
            category = 3;
        }
    "#);
    assert_eq!(
        vm.get_variable("category").and_then(|v| v.as_int()),
        Some(2),
        "unparenthesized else-if chain must match correct branch"
    );
}

#[test]
fn test_unparenthesized_while_loop_count() {
    let vm = run(r#"
        let i = 0;
        let sum = 0;
        while i < 5 {
            sum = sum + i;
            i = i + 1;
        }
    "#);
    assert_eq!(
        vm.get_variable("sum").and_then(|v| v.as_int()),
        Some(10),
        "unparenthesized while loop must execute 5 iterations"
    );
}

#[test]
fn test_unparenthesized_while_false_does_not_execute() {
    let vm = run(r#"
        let ran = false;
        while false {
            ran = true;
        }
    "#);
    assert_eq!(
        vm.get_variable("ran").and_then(|v| v.as_bool()),
        Some(false),
        "unparenthesized while false must not execute"
    );
}

#[test]
fn test_unparenthesized_control_flow_inside_state_machine_loop() {
    let vm = run(r#"
        let flag = false;
        loop TestLoop {
            step run {
                let cond = false;
                if cond {
                    flag = false;
                } else {
                    let i = 0;
                    let count = 0;
                    while i < 3 {
                        count = count + 1;
                        i = i + 1;
                    }
                    if count == 3 {
                        flag = true;
                    }
                }
                gate check { when true -> done else -> done }
            }
        }
        run loop TestLoop;
    "#);
    assert_eq!(
        vm.get_variable("flag").and_then(|v| v.as_bool()),
        Some(true),
        "unparenthesized if and while inside loop step must branch and iterate correctly"
    );
}

#[test]
fn test_g2_2_verifier_with_nested_function_and_optimizer() {
    let vm = run(r#"
        function isPalindrome(s) {
            let n = s.length;
            for (let i = 0; i < n / 2; i = i + 1) {
                if (s[i] != s[n - 1 - i]) {
                    return false;
                }
            }
            return true;
        }
        let check1 = isPalindrome("racecar");
        let check2 = isPalindrome("hello");
    "#);
    assert_eq!(
        vm.get_variable("check1").and_then(|v| v.as_bool()),
        Some(true),
        "isPalindrome('racecar') should be true without G2.2 verifier panic"
    );
    assert_eq!(
        vm.get_variable("check2").and_then(|v| v.as_bool()),
        Some(false),
        "isPalindrome('hello') should be false without G2.2 verifier panic"
    );
}

#[test]
fn test_sample_if_test_step_execution() {
    let vm = run(r#"
        let bypass_detected = false;
        loop IfTest {
            step run {
                let cond = false;
                if cond == true {
                    bypass_detected = true;
                }
                gate check { when true -> done else -> done }
            }
        }
        run loop IfTest;
    "#);
    assert_eq!(
        vm.get_variable("bypass_detected").and_then(|v| v.as_bool()),
        Some(false),
        "IfTest: cond == false must not execute if branch"
    );
}

#[test]
fn test_sample_while_test_step_execution() {
    let vm = run(r#"
        let bypass_detected = false;
        loop WhileTest {
            step run {
                while false {
                    bypass_detected = true;
                }
                gate check { when true -> done else -> done }
            }
        }
        run loop WhileTest;
    "#);
    assert_eq!(
        vm.get_variable("bypass_detected").and_then(|v| v.as_bool()),
        Some(false),
        "WhileTest: while false must not execute loop body"
    );
}

#[test]
fn test_issue2_large_multistep_loop_backward_gate_jump() {
    let vm = run(r#"
        let total_iterations = 0;
        loop HeavyLoop {
            step prepare {
                let a1 = 1; let a2 = 2; let a3 = 3; let a4 = 4;
                let a5 = 5; let a6 = 6; let a7 = 7; let a8 = 8;
                let a9 = 9; let a10 = 10; let a11 = 11; let a12 = 12;
                let sum = a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9 + a10 + a11 + a12;
                total_iterations = total_iterations + 1;
                gate to_next {
                    when sum > 50 -> process
                    else -> done
                }
            }
            step process {
                let b1 = 10; let b2 = 20; let b3 = 30; let b4 = 40;
                let bsum = b1 + b2 + b3 + b4;
                gate check_repeat {
                    when total_iterations < 3 -> prepare
                    else -> done
                }
            }
        }
        run loop HeavyLoop;
    "#);
    assert_eq!(
        vm.get_variable("total_iterations").and_then(|v| v.as_int()),
        Some(3),
        "Multi-step backward gate jump with >32 instructions must succeed without G2.2 verifier panic"
    );
}

#[test]
fn test_block_scope_shadowing_preserves_outer_variable() {
    let vm = run(r#"
        let val = 10;
        if true {
            let val = 20;
        }
    "#);
    assert_eq!(
        vm.get_variable("val").and_then(|v| v.as_int()),
        Some(10),
        "Block scope shadowing: outer variable must remain 10 after inner block exits"
    );
}

#[test]
fn test_loop_step_block_scope_shadowing() {
    let vm = run(r#"
        let outer_val = 0;
        loop ScopeTest {
            step run {
                let val = 10;
                if true {
                    let val = 20;
                }
                outer_val = val;
                gate check { when true -> done else -> done }
            }
        }
        run loop ScopeTest;
    "#);
    assert_eq!(
        vm.get_variable("outer_val").and_then(|v| v.as_int()),
        Some(10),
        "Loop step block scope shadowing: outer_val must be 10, not mutated to 20"
    );
}



