//! Differential parity: MIR constant folding vs. the real VM
//! (JIT_AOT_ARCHITECTURE.md §22.1 — the VM is the oracle).
//!
//! AŞAMA-0 proof: for `print(2 + 3)` the lowered MIR folds the argument
//! to the integer the VM actually computes for the same source. Overflow
//! deliberately does NOT fold — the VM promotes to BigInt, so folding
//! would change observable semantics (§18).

use hudhudscript_compiler::Compiler;
use hudhudscript_mir::MirInst;
use hudhudscript_mir_opt::const_fold;
use hudhudscript_parser::parse;
use hudhudscript_types::{HirBinOp, HirExpr, HirFunction, HirStmt, Type};
use hudhudscript_vm::VM;

/// Build HIR for `fn main() { print(<expr>) }` with the given literal pair.
fn main_print(a: i64, b: i64) -> HirFunction {
    HirFunction {
        name: "main".into(),
        params: vec![],
        return_type: Type::Null,
        body: vec![HirStmt::Expr(HirExpr::Call {
            callee: "print".into(),
            args: vec![HirExpr::Binary {
                op: HirBinOp::Add,
                lhs: Box::new(HirExpr::IntLit(a)),
                rhs: Box::new(HirExpr::IntLit(b)),
                ty: Type::Number,
            }],
            ty: Type::Null,
        })],
    }
}

/// Run `let x = a + b` through the REAL engine (parser→compiler→VM).
fn vm_computed(a: i64, b: i64) -> i64 {
    let source = format!("let x = {a} + {b}");
    let ast = parse(&source).expect("parse");
    let bc = Compiler::new().compile(&ast).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_variable_owned("x")
        .and_then(|v| v.as_int())
        .expect("x must be an int")
}

/// Extract the folded argument of the print call from MIR.
fn folded_print_arg(mir: &hudhudscript_mir::MirFunction) -> i64 {
    for inst in &mir.blocks[0].insts {
        if let MirInst::CallNative { args, .. } = inst {
            assert_eq!(args.len(), 1);
            let arg = args[0];
            for i in &mir.blocks[0].insts {
                if let MirInst::ConstInt { dst, value, .. } = i {
                    if *dst == arg {
                        return *value;
                    }
                }
            }
        }
    }
    panic!("no folded print argument found");
}

#[test]
fn print_2_plus_3_mir_matches_vm() {
    let hir = main_print(2, 3);
    let mir = hudhudscript_mir::lower_function(&hir).expect("lower");
    let (folded, n) = const_fold(&mir);
    assert_eq!(n, 1, "the 2+3 addition must fold");
    hudhudscript_mir::verify_function(&folded).expect("folded MIR must verify");

    let mir_value = folded_print_arg(&folded);
    let vm_value = vm_computed(2, 3);
    assert_eq!(mir_value, vm_value, "MIR folding must match the VM oracle");
    assert_eq!(mir_value, 5);
}

#[test]
fn larger_sums_match_vm() {
    for (a, b) in [(0, 0), (123, 456), (-50, 50), (1_000_000, 1)] {
        let hir = main_print(a, b);
        let mir = hudhudscript_mir::lower_function(&hir).expect("lower");
        let (folded, _) = const_fold(&mir);
        assert_eq!(
            folded_print_arg(&folded),
            vm_computed(a, b),
            "parity broken for {a}+{b}"
        );
    }
}

#[test]
fn overflow_never_folds_because_vm_promotes_to_bigint() {
    // i64::MAX + 1: VM promotes to BigInt (observable semantics);
    // therefore MIR folding MUST leave the add in place.
    let hir = main_print(i64::MAX, 1);
    let mir = hudhudscript_mir::lower_function(&hir).expect("lower");
    let (folded, n) = const_fold(&mir);
    assert_eq!(n, 0, "overflow add must NOT fold");
    let has_add = folded.blocks[0]
        .insts
        .iter()
        .any(|i| matches!(i, MirInst::Add { .. }));
    assert!(has_add, "the checked add must remain for the runtime lane");
}

// ── Tam otomatik boru hattı (AŞAMA-0.5): Kaynak → Parse → AST→HIR →
// HIR→MIR → const_fold, VM oracle'a karşı ─────────────────────────────

use hudhudscript_types::lower_module;

#[test]
fn automatic_pipeline_matches_vm() {
    let source = "function main() { print(2 + 3) }";
    let ast = parse(source).expect("parse");
    let module = lower_module(&ast).expect("AST→HIR");
    let hir_main = module.functions.get("main").expect("main in module");

    let mir = hudhudscript_mir::lower_function(hir_main).expect("HIR→MIR");
    let (folded, n) = const_fold(&mir);
    assert_eq!(n, 1);
    hudhudscript_mir::verify_function(&folded).expect("folded MIR must verify");
    assert_eq!(folded_print_arg(&folded), vm_computed(2, 3));
    assert_eq!(folded_print_arg(&folded), 5);
}

#[test]
fn automatic_pipeline_multiple_functions() {
    let source = "function main() { print(40 + 2) }\nfunction helper() { return 1 }";
    let ast = parse(source).expect("parse");
    let module = lower_module(&ast).expect("AST→HIR");
    assert_eq!(module.functions.len(), 2);
    assert!(module.functions.contains_key("helper"));

    let mir = hudhudscript_mir::lower_function(&module.functions["main"]).expect("lower");
    let (folded, n) = const_fold(&mir);
    assert_eq!(n, 1);
    assert_eq!(folded_print_arg(&folded), vm_computed(40, 2));
    assert_eq!(folded_print_arg(&folded), 42);
}
