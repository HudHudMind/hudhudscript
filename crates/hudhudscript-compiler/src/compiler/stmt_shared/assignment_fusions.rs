//! B3: NumMulAddAssign fusion pattern detection.
//! Detects `dest = dest * mul_expr + add_expr` for Horner-style accumulation.

use hudhudscript_ast::{BinaryOp, Expr, Literal};

/// If `value` matches `name * mul_expr + add_expr`, returns `Some((mul_expr, add_expr))`.
/// Handles commutativity: `name * X + Y` and `X * name + Y` both match.
pub(crate) fn try_fma_pattern<'a>(value: &'a Expr, name: &str) -> Option<(&'a Expr, &'a Expr)> {
    // outer: Add
    if let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right: add_expr,
        ..
    } = value
    {
        // inner: Mul with Identifier(name) on either side
        if let Expr::Binary {
            op: BinaryOp::Mul,
            left: mul_left,
            right: mul_right,
            ..
        } = left.as_ref()
        {
            if is_ident(mul_left, name) {
                return Some((mul_right, add_expr));
            }
            if is_ident(mul_right, name) {
                return Some((mul_left, add_expr));
            }
        }
    }
    None
}

/// If `value` matches `name - positive_int_literal`, returns `Some(imm)`.
/// Only matches when left side is `Identifier(name)` — no commutativity for subtract.
pub(crate) fn try_self_sub_int<'a>(value: &'a Expr, name: &str) -> Option<i16> {
    if let Expr::Binary {
        op: BinaryOp::Sub,
        left,
        right,
        ..
    } = value
    {
        if !is_ident(left, name) {
            return None;
        }
        if let Expr::Literal(Literal::Number(n, false), _) = right.as_ref() {
            let i = *n as i64;
            if i > 0 && i <= i16::MAX as i64 {
                return Some(i as i16);
            }
        }
    }
    None
}

/// If `value` matches `name + positive_int_literal` or `positive_int_literal + name`,
/// returns `Some(imm)`. Only matches when one side is Identifier(name).
pub(super) fn try_self_add_int<'a>(value: &'a Expr, name: &str) -> Option<i16> {
    if let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
        ..
    } = value
    {
        if let Expr::Literal(Literal::Number(n, false), _) = right.as_ref() {
            let i = *n as i64;
            if i > 0 && i <= i16::MAX as i64 && is_ident(left, name) {
                return Some(i as i16);
            }
        }
        if let Expr::Literal(Literal::Number(n, false), _) = left.as_ref() {
            let i = *n as i64;
            if i > 0 && i <= i16::MAX as i64 && is_ident(right, name) {
                return Some(i as i16);
            }
        }
    }
    None
}

fn is_ident(expr: &Expr, name: &str) -> bool {
    matches!(expr, Expr::Identifier(n, _) if n == name)
}

#[cfg(test)]
mod tests {
    use hudhudscript_ast::*;
    use crate::compiler::stmt_shared::assignment_fusions::{try_fma_pattern, try_self_sub_int};

    fn ident(name: &str) -> Expr {
        Expr::Identifier(name.to_string(), Span::default())
    }

    fn mul(l: Expr, r: Expr) -> Expr {
        Expr::Binary { left: Box::new(l), op: BinaryOp::Mul, right: Box::new(r), span: Span::default() }
    }

    fn add(l: Expr, r: Expr) -> Expr {
        Expr::Binary { left: Box::new(l), op: BinaryOp::Add, right: Box::new(r), span: Span::default() }
    }

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

    #[test]
    fn test_horner_fma_emitted_with_distinct_operands() {
        // Public Compiler integration test moved to external test suite (I2-A5).
        // Kept here as a thin sanity check until the external suite is the norm.
        use crate::compiler::Compiler;
        let src = "fn horner_test(coeffs, x) { let result = coeffs[2]; let i = 1; while (i >= 0) { result = result * x + coeffs[i]; i = i - 1; } return result; } horner_test([1,2,3], 10);";
        let ast = hudhudscript_parser::parse(src).unwrap();
        let mut compiler = Compiler::new();
        let bc = compiler.compile(&ast).unwrap();
        
        let horner = bc.get_function("horner_test")
            .expect("horner_test function not found");
        
        let mut has_fma = false;
        for instr in &horner.instructions {
            if let hudhudscript_bytecode::Instruction::NumMulAddIndexed { acc: _a, mul, arr, idx: _i } = instr {
                has_fma = true;
                assert_ne!(*mul, *arr, "NumMulAddIndexed mul and arr must be different registers (both {mul})");
            }
        }
        assert!(has_fma, "NumMulAddIndexed not emitted for horner pattern");
    }

    fn literal_int(n: f64) -> Expr {
        Expr::Literal(Literal::Number(n, false), Span::default())
    }

    fn sub(l: Expr, r: Expr) -> Expr {
        Expr::Binary { left: Box::new(l), op: BinaryOp::Sub, right: Box::new(r), span: Span::default() }
    }

    #[test]
    fn test_self_sub_int_detected() {
        // i = i - 1
        let val = sub(ident("i"), literal_int(1.0));
        let r = try_self_sub_int(&val, "i");
        assert_eq!(r, Some(1));
    }

    #[test]
    fn test_self_sub_int_larger() {
        // i = i - 42
        let val = sub(ident("i"), literal_int(42.0));
        let r = try_self_sub_int(&val, "i");
        assert_eq!(r, Some(42));
    }

    #[test]
    fn test_self_sub_int_not_same_local() {
        // i = j - 1
        let val = sub(ident("j"), literal_int(1.0));
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
        let val = add(ident("i"), literal_int(1.0));
        let r = try_self_sub_int(&val, "i");
        assert!(r.is_none());
    }
}
