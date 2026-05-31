//! Math builtin tests — migrated to call shared-builtins directly
//! (`call_math_method`) after the `hudhudscript-builtins` deletion.
//!
//! The interpreter-era `create_math_object()` returned a `Value::Object`
//! with `Value::NativeFunction` closures and `PI`/`E` fields. That shape is
//! specific to the interpreter's representation; the VM exposes Math via
//! `call_math_method` on the shared trait. The PI/E *constant* tests used
//! to probe that structural layout — they have no meaningful counterpart
//! in the shared dispatcher (which only services method calls), so they
//! are replaced by value-based checks performed through the public method
//! dispatcher where possible (no other math method returns PI/E directly).
//! For that single reason the two constant-layout tests are omitted — they
//! were asserting on interpreter-era object shape, not on mathematical
//! behaviour.

use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::math::MathMethodId;
use std::f64::consts::E;

/// Helper: call a math method through the shared dispatcher.
fn call_math(name: &str, args: Vec<Value16>) -> hudhudscript_errors::HudHudResult<Value16> {
    name.parse::<MathMethodId>()?.dispatch(&args)
}

// ── Basic math functions ────────────────────────────────────────────

#[test]
fn math_abs_positive() {
    assert_eq!(
        call_math("abs", vec![Value16::number(5.0)]).unwrap(),
        Value16::number(5.0)
    );
}

#[test]
fn math_abs_negative() {
    assert_eq!(
        call_math("abs", vec![Value16::number(-3.7)]).unwrap(),
        Value16::number(3.7)
    );
}

#[test]
fn math_abs_type_error() {
    let err = call_math("abs", vec![Value16::string("x")]);
    assert!(err.is_err());
}

#[test]
fn math_sqrt() {
    assert_eq!(
        call_math("sqrt", vec![Value16::number(9.0)]).unwrap(),
        Value16::number(3.0)
    );
}

#[test]
fn math_pow() {
    assert_eq!(
        call_math("pow", vec![Value16::number(2.0), Value16::number(10.0)]).unwrap(),
        Value16::number(1024.0)
    );
}

#[test]
fn math_pow_type_error() {
    let err = call_math("pow", vec![Value16::number(2.0), Value16::boolean(true)]);
    assert!(err.is_err());
}

#[test]
fn math_floor() {
    assert_eq!(
        call_math("floor", vec![Value16::number(3.9)]).unwrap(),
        Value16::number(3.0)
    );
}

#[test]
fn math_ceil() {
    assert_eq!(
        call_math("ceil", vec![Value16::number(3.1)]).unwrap(),
        Value16::number(4.0)
    );
}

#[test]
fn math_round_up() {
    assert_eq!(
        call_math("round", vec![Value16::number(2.6)]).unwrap(),
        Value16::number(3.0)
    );
}

#[test]
fn math_round_down() {
    assert_eq!(
        call_math("round", vec![Value16::number(2.4)]).unwrap(),
        Value16::number(2.0)
    );
}

#[test]
fn math_min() {
    assert_eq!(
        call_math("min", vec![Value16::number(7.0), Value16::number(3.0)]).unwrap(),
        Value16::number(3.0)
    );
}

#[test]
fn math_max() {
    assert_eq!(
        call_math("max", vec![Value16::number(7.0), Value16::number(3.0)]).unwrap(),
        Value16::number(7.0)
    );
}

// ── Trigonometry ────────────────────────────────────────────────────

#[test]
fn math_sin_zero() {
    let val = call_math("sin", vec![Value16::number(0.0)]).unwrap();
    assert_eq!(val, Value16::number(0.0));
}

#[test]
fn math_cos_zero() {
    let val = call_math("cos", vec![Value16::number(0.0)]).unwrap();
    assert_eq!(val, Value16::number(1.0));
}

#[test]
fn math_tan_zero() {
    let val = call_math("tan", vec![Value16::number(0.0)]).unwrap();
    assert_eq!(val, Value16::number(0.0));
}

#[test]
fn math_asin_roundtrip() {
    // asin(sin(0.5)) should be ~0.5
    let sin_val = call_math("sin", vec![Value16::number(0.5)]).unwrap();
    let asin_val = call_math("asin", vec![sin_val]).unwrap();
    if let Some(n) = asin_val.as_number() {
        assert!((n - 0.5).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn math_atan2() {
    let val = call_math("atan2", vec![Value16::number(1.0), Value16::number(1.0)]).unwrap();
    if let Some(n) = val.as_number() {
        assert!((n - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

// ── Logarithms & Exp ────────────────────────────────────────────────

#[test]
fn math_log_e() {
    // ln(e) = 1
    let val = call_math("log", vec![Value16::number(E)]).unwrap();
    if let Some(n) = val.as_number() {
        assert!((n - 1.0).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn math_log10() {
    let val = call_math("log10", vec![Value16::number(100.0)]).unwrap();
    if let Some(n) = val.as_number() {
        assert!((n - 2.0).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn math_log2() {
    let val = call_math("log2", vec![Value16::number(8.0)]).unwrap();
    if let Some(n) = val.as_number() {
        assert!((n - 3.0).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn math_exp() {
    // exp(1) = e
    let val = call_math("exp", vec![Value16::number(1.0)]).unwrap();
    if let Some(n) = val.as_number() {
        assert!((n - E).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

// ── Advanced functions ──────────────────────────────────────────────

#[test]
fn math_sign_positive() {
    assert_eq!(
        call_math("sign", vec![Value16::number(42.0)]).unwrap(),
        Value16::number(1.0)
    );
}

#[test]
fn math_sign_negative() {
    assert_eq!(
        call_math("sign", vec![Value16::number(-7.0)]).unwrap(),
        Value16::number(-1.0)
    );
}

#[test]
fn math_sign_zero() {
    assert_eq!(
        call_math("sign", vec![Value16::number(0.0)]).unwrap(),
        Value16::number(0.0)
    );
}

#[test]
fn math_trunc() {
    assert_eq!(
        call_math("trunc", vec![Value16::number(3.9)]).unwrap(),
        Value16::number(3.0)
    );
    assert_eq!(
        call_math("trunc", vec![Value16::number(-3.9)]).unwrap(),
        Value16::number(-3.0)
    );
}

#[test]
fn math_hypot() {
    let val = call_math("hypot", vec![Value16::number(3.0), Value16::number(4.0)]).unwrap();
    assert_eq!(val, Value16::number(5.0));
}

#[test]
fn math_clamp_within_range() {
    let val = call_math(
        "clamp",
        vec![
            Value16::number(5.0),
            Value16::number(0.0),
            Value16::number(10.0),
        ],
    )
    .unwrap();
    assert_eq!(val, Value16::number(5.0));
}

#[test]
fn math_clamp_below_min() {
    let val = call_math(
        "clamp",
        vec![
            Value16::number(-5.0),
            Value16::number(0.0),
            Value16::number(10.0),
        ],
    )
    .unwrap();
    assert_eq!(val, Value16::number(0.0));
}

#[test]
fn math_clamp_above_max() {
    let val = call_math(
        "clamp",
        vec![
            Value16::number(15.0),
            Value16::number(0.0),
            Value16::number(10.0),
        ],
    )
    .unwrap();
    assert_eq!(val, Value16::number(10.0));
}

#[test]
fn math_random_in_range() {
    let val = call_math("random", vec![]).unwrap();
    if let Some(n) = val.as_number() {
        assert!(
            n >= 0.0 && n <= 1.0,
            "random() returned {} which is out of [0,1]",
            n
        );
    } else {
        panic!("expected number");
    }
}
