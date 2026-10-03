//! Assignment fusion pattern tests — `try_fma_pattern` (NumMulAddAssign) and
//! `try_self_sub_int` (self `x - imm` decrement), plus a Compiler integration
//! check that the Horner pattern emits `NumMulAddIndexed`.

use hudhudscript_compiler::compiler::stmt_shared::assignment_fusions::{try_fma_pattern, try_self_sub_int};
use hudhudscript_ast::*;

// ── helpers ───────────────────────────────────────────────────────

fn ident(name: &str) -> Expr {
    Expr::Identifier(name.to_string(), Span::default())
}

fn mul(l: Expr, r: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(l),
        op: BinaryOp::Mul,
        right: Box::new(r),
        span: Span::default(),
    }
}

fn add(l: Expr, r: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(l),
        op: BinaryOp::Add,
        right: Box::new(r),
        span: Span::default(),
    }
}

fn literal_int(n: i64) -> Expr {
    Expr::Literal(Literal::Int(n), Span::default())
}

fn sub(l: Expr, r: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(l),
        op: BinaryOp::Sub,
        right: Box::new(r),
        span: Span::default(),
    }
}

// ── fma pattern ───────────────────────────────────────────────────

#[test]
fn test_pattern_dest_mul_add() {
    // result = result * x + y
    let val = add(mul(ident("result"), ident("x")), ident("y"));
    let r = try_fma_pattern(&val, "result");
    assert!(r.is_some());
    let (mul_expr, add_expr) = r.unwrap();
    assert!(matches!(mul_expr, Expr::Identifier(n, _) if n == "x"));
    assert!(matches!(add_expr, Expr::Identifier(n, _) if n == "y"));
}

#[test]
fn test_pattern_mul_commutative() {
    // result = x * result + y
    let val = add(mul(ident("x"), ident("result")), ident("y"));
    let r = try_fma_pattern(&val, "result");
    assert!(r.is_some());
    let (mul_expr, add_expr) = r.unwrap();
    assert!(matches!(mul_expr, Expr::Identifier(n, _) if n == "x"));
    assert!(matches!(add_expr, Expr::Identifier(n, _) if n == "y"));
}

#[test]
fn test_pattern_not_fma_different_dest() {
    // result = other * x + y
    let val = add(mul(ident("other"), ident("x")), ident("y"));
    let r = try_fma_pattern(&val, "result");
    assert!(r.is_none());
}

#[test]
fn test_pattern_not_fma_no_mul() {
    // result = result + y  (no mul)
    let val = add(ident("result"), ident("y"));
    let r = try_fma_pattern(&val, "result");
    assert!(r.is_none());
}

// ── horner integration (I2-A5) ────────────────────────────────────

#[test]
fn test_horner_fma_emitted_with_distinct_operands() {
    // Compiler integration: horner accumulation must fuse to NumMulAddIndexed.
    use hudhudscript_compiler::Compiler;
    let src = "fn horner_test(coeffs, x) { let result = coeffs[2]; let i = 1; while (i >= 0) { result = result * x + coeffs[i]; i = i - 1; } return result; } horner_test([1,2,3], 10);";
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();

    let horner = bc
        .get_function("horner_test")
        .expect("horner_test function not found");

    let mut has_fma = false;
    for instr in &horner.instructions {
        if let hudhudscript_bytecode::Instruction::NumMulAddIndexed {
            acc: _a,
            mul,
            arr,
            idx: _i,
        } = instr
        {
            has_fma = true;
            assert_ne!(
                *mul, *arr,
                "NumMulAddIndexed mul and arr must be different registers (both {mul})"
            );
        }
    }
    assert!(has_fma, "NumMulAddIndexed not emitted for horner pattern");
}

// ── self sub int ──────────────────────────────────────────────────

#[test]
fn test_self_sub_int_detected() {
    // i = i - 1
    let val = sub(ident("i"), literal_int(1));
    let r = try_self_sub_int(&val, "i");
    assert_eq!(r, Some(1));
}

#[test]
fn test_self_sub_int_larger() {
    // i = i - 42
    let val = sub(ident("i"), literal_int(42));
    let r = try_self_sub_int(&val, "i");
    assert_eq!(r, Some(42));
}

#[test]
fn test_self_sub_int_not_same_local() {
    // i = j - 1
    let val = sub(ident("j"), literal_int(1));
    let r = try_self_sub_int(&val, "i");
    assert!(r.is_none());
}

#[test]
fn test_self_sub_int_right_not_literal() {
    // i = i - x
    let val = sub(ident("i"), ident("x"));
    let r = try_self_sub_int(&val, "i");
    assert!(r.is_none());
}

#[test]
fn test_self_sub_int_not_sub() {
    // i = i + 1 (not Sub)
    let val = add(ident("i"), literal_int(1));
    let r = try_self_sub_int(&val, "i");
    assert!(r.is_none());
}
