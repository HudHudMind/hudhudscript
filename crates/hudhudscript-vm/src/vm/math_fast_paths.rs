//! GATE-1: unified arithmetic hot paths.
//! Hot signatures match bigint_arith (Value16, Value16) -> Result<Value16, ErrorCode>.
//! No VM/Bytecode/ip in hot path — cold error materialization only.

use hudhudscript_bytecode::{ReprTag, Value16};
use hudhudscript_errors::catalog::ErrorCode;

#[cfg(feature = "telemetry")]
std::thread_local! {
    pub(crate) static INT_ADD_SLOW_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(feature = "telemetry")]
pub fn take_int_add_slow_count() -> u64 {
    INT_ADD_SLOW_COUNT.with(|c| {
        let val = c.get();
        c.set(0);
        val
    })
}

#[cfg(not(feature = "telemetry"))]
pub fn take_int_add_slow_count() -> u64 {
    0
}

macro_rules! int_math_op {
    ($name:ident, $slow:ident, $checked:ident, $promote:ident, $float_op:expr, $bigint:ident, $is_add:expr) => {
        #[inline(always)]
        pub(crate) fn $name(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
            let (ta, pa) = a.split_tag();
            let (tb, pb) = b.split_tag();
            match (ta, tb) {
                (ReprTag::Int, ReprTag::Int) => {
                    let x = pa as i64;
                    let y = pb as i64;
                    match x.$checked(y) {
                        Some(r) => Ok(Value16::int(r)),
                        None => Ok(crate::vm::bigint_arith::$promote(x, y)),
                    }
                }
                // H.6.2: float arms inline, immediately after the int arm. These
                // used to fall through to `$slow`, which is a real call — every
                // float-containing arithmetic op paid it (do_int_add_slow was
                // 5.8% Ir in rk4, 8.4% in fib_iterative) even though bigint
                // promotion, the reason `$slow` exists, was never hit there.
                // `split_tag` already ran once above; do not repeat it (H.6.1).
                (ReprTag::Int | ReprTag::Number, ReprTag::Int | ReprTag::Number) => {
                    let x = if ta == ReprTag::Int { pa as i64 as f64 } else { f64::from_bits(pa) };
                    let y = if tb == ReprTag::Int { pb as i64 as f64 } else { f64::from_bits(pb) };
                    Ok(Value16::number($float_op(x, y)))
                }
                _ => $slow(a, b),
            }
        }

        /// Only string concatenation and BigInt reach here — every Int/Number
        /// combination is handled inline by the caller.
        #[cold]
        #[inline(never)]
        fn $slow(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
            #[cfg(feature = "telemetry")]
            if stringify!($name) == "do_int_add" {
                crate::vm::math_fast_paths::INT_ADD_SLOW_COUNT.with(|c| c.set(c.get() + 1));
            }
            if $is_add {
                if let (Some(av), Some(bv)) = (a.as_string(), b.as_string()) {
                    return Ok(Value16::string(av + &bv));
                }
            }
            crate::vm::bigint_arith::$bigint(a, b)
        }
    };
}

int_math_op!(do_int_add, do_int_add_slow, checked_add, promote_add, |x:f64,y:f64| x+y, bigint_add, true);
int_math_op!(do_int_sub, do_int_sub_slow, checked_sub, promote_sub, |x:f64,y:f64| x-y, bigint_sub, false);
int_math_op!(do_int_mul, do_int_mul_slow, checked_mul, promote_mul, |x:f64,y:f64| x*y, bigint_mul, false);

macro_rules! int_math_op_i {
    ($name:ident, $slow:ident, $checked:ident, $promote:ident, $float_op:expr, $bigint:ident) => {
        #[inline(always)]
        pub(crate) fn $name(a: Value16, imm: i64) -> Result<Value16, ErrorCode> {
            let (ta, pa) = a.split_tag();
            match ta {
                ReprTag::Int => {
                    let x = pa as i64;
                    match x.$checked(imm) {
                        Some(r) => Ok(Value16::int(r)),
                        None => Ok(crate::vm::bigint_arith::$promote(x, imm)),
                    }
                }
                // Inline, and — critically — using `$float_op`. This arm used to
                // live in `$slow` with a hardcoded `x + imm`, so every op shared
                // addition: `f() - 2` and `f() * 2` on a float both returned
                // `f() + 2`. The bug only showed when the compiler could not
                // prove the operand was a float (e.g. it came from a call), which
                // is why `let b = 5.5; b - 2` was right while `f() - 2` was not.
                ReprTag::Number => {
                    let x = f64::from_bits(pa);
                    Ok(Value16::number($float_op(x, imm as f64)))
                }
                _ => $slow(a, imm),
            }
        }

        /// Only BigInt reaches here — Int and Number are handled inline above.
        #[cold]
        #[inline(never)]
        fn $slow(a: Value16, imm: i64) -> Result<Value16, ErrorCode> {
            #[cfg(feature = "telemetry")]
            if stringify!($name) == "do_int_add_i" {
                crate::vm::math_fast_paths::INT_ADD_SLOW_COUNT.with(|c| c.set(c.get() + 1));
            }
            // Dynamic (BigInt) — use original Value16, not pointer payload
            crate::vm::bigint_arith::$bigint(a, Value16::int(imm))
        }
    };
}

int_math_op_i!(do_int_add_i, do_int_add_slow_i, checked_add, promote_add, |x:f64,y:f64| x+y, bigint_add);
int_math_op_i!(do_int_sub_i, do_int_sub_slow_i, checked_sub, promote_sub, |x:f64,y:f64| x-y, bigint_sub);
int_math_op_i!(do_int_mul_i, do_int_mul_slow_i, checked_mul, promote_mul, |x:f64,y:f64| x*y, bigint_mul);
