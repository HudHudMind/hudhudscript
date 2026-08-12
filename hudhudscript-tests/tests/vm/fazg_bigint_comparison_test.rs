//! FAZ G regression: BigInt (>=10^20) comparison operators — < > <= >= == !=
//! Mixed Int/BigInt directions.

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

fn result_bool(src: &str) -> String {
    let vm = run(src).unwrap();
    vm.last_return_value().display_string()
}

/// Two real BigInts (>10^20): strict comparison and equality in both directions
#[test]
fn fazg_bigint_vs_bigint_all_ops() {
    // a = 10^20, b = 9*10^19
    let big = "let a=1000000000;let x=a*a*100;let y=a*a*90;";
    // All six ops as r0..r5 then comma-bool return
    let src = format!(
        "{}let r0=0;if(x<y){{r0=1;}}let r1=0;if(x<=y){{r1=1;}}let r2=0;if(x>y){{r2=1;}}let r3=0;if(x>=y){{r3=1;}}let r4=0;if(x==y){{r4=1;}}let r5=0;if(x!=y){{r5=1;}}return r0+r1+r2+r3+r4+r5;",
        big
    );
    // Expected: x<y=0 x<=y=0 x>y=1 x>=y=1 x==y=0 x!=y=1 → sum = 3
    assert_eq!(result_bool(&src), "3");
}

/// BigInt vs immediate Int (x < 5 where x ≈ 10^20)
#[test]
fn fazg_bigint_vs_int_directions() {
    let big = "let a=1000000000;let x=a*a*100;";
    // x < 10 → false(=0), x > 10 → true(=1), x == x → true(=1)
    let src = format!(
        "{}let r0=0;if(x<10){{r0=1;}}let r1=0;if(x>10){{r1=1;}}let r2=0;if(x==x){{r2=1;}}return r0+r1+r2;",
        big
    );
    assert_eq!(result_bool(&src), "2");
}

/// Int immediate vs BigInt (5 < x where x ≈ 10^20)
#[test]
fn fazg_int_vs_bigint_directions() {
    let big = "let a=1000000000;let x=a*a*100;";
    let src = format!(
        "{}let r0=0;if(10<x){{r0=1;}}let r1=0;if(10>x){{r1=1;}}return r0+r1;",
        big
    );
    assert_eq!(result_bool(&src), "1");
}

/// BigInt self-equality
#[test]
fn fazg_bigint_self_equality() {
    let src = "let a=1000000000;let x=a*a*100;return x==x;";
    assert_eq!(result_bool(src), "true");
}

/// BigInt self-inequality
#[test]
fn fazg_bigint_self_inequality() {
    let src = "let a=1000000000;let x=a*a*100;return x!=x;";
    assert_eq!(result_bool(src), "false");
}

/// BigInt <= and >= on equal values
#[test]
fn fazg_bigint_ge_le_equal() {
    let src = "let a=1000000000;let x=a*a*100;let y=a*a*100;let r0=0;if(x<=y){r0=1;}let r1=0;if(x>=y){r1=1;}return r0+r1;";
    assert_eq!(result_bool(src), "2");
}
