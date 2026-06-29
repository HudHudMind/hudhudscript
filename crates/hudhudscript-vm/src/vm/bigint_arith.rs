use hudhudscript_bytecode::{ReprTag, Value16};
use hudhudscript_errors::catalog::ErrorCode;
use std::cmp::Ordering;

/// F2: Cold overflow — #[cold] #[inline(never)] so hot path stays clean.
#[cold]
#[inline(never)]
fn promote_mul(x: i64, y: i64) -> Value16 {
    Value16::bigint(num_bigint::BigInt::from(x) * num_bigint::BigInt::from(y))
}
#[cold]
#[inline(never)]
fn promote_add(x: i64, y: i64) -> Value16 {
    Value16::bigint(num_bigint::BigInt::from(x) + num_bigint::BigInt::from(y))
}
#[cold]
#[inline(never)]
fn promote_sub(x: i64, y: i64) -> Value16 {
    Value16::bigint(num_bigint::BigInt::from(x) - num_bigint::BigInt::from(y))
}
#[cold]
#[inline(never)]
fn bigint_mul(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    // P5-B2: avoid to_bigint_value() — handle operand combinations directly,
    // mirroring P5-A1 bigint_add. No redundant Int->BigInt conversion.
    // BigInt * BigInt: result is always ≥ product of two BigInts — skip demotion.
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_bigint()) {
        return Ok(Value16::bigint_no_demote(x * y));
    }
    // BigInt * Int: use reference multiplication to avoid clone+mutate.
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_int()) {
        return Ok(Value16::bigint(x * num_bigint::BigInt::from(y)));
    }
    // Int * BigInt
    if let (Some(x), Some(y)) = (a.as_int(), b.as_bigint()) {
        return Ok(Value16::bigint(num_bigint::BigInt::from(x) * y));
    }
    // Int * Int overflow
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        return Ok(match x.checked_mul(y) {
            Some(v) => Value16::int(v),
            None => promote_mul(x, y),
        });
    }
    Err(ErrorCode(310))
}
#[cold]
#[inline(never)]
fn bigint_add(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    // P5-A1: avoid to_bigint_value() — handle operand combinations directly.
    // BigInt + BigInt: result always large — skip demotion.
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_bigint()) {
        return Ok(Value16::bigint_no_demote(x + y));
    }
    // BigInt + Int: use reference addition to avoid clone+mutate.
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_int()) {
        return Ok(Value16::bigint(x + num_bigint::BigInt::from(y)));
    }
    // Int + BigInt
    if let (Some(x), Some(y)) = (a.as_int(), b.as_bigint()) {
        return Ok(Value16::bigint(num_bigint::BigInt::from(x) + y));
    }
    // Int + Int overflow
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        return Ok(match x.checked_add(y) {
            Some(v) => Value16::int(v),
            None => promote_add(x, y),
        });
    }
    Err(ErrorCode(310))
}
#[cold]
#[inline(never)]
fn bigint_sub(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    Ok(Value16::bigint(x - y))
}

#[inline]
fn check_mix(a: Value16, b: Value16) -> Result<(), ErrorCode> {
    if (a.is_bigint() && b.is_number()) || (a.is_number() && b.is_bigint()) {
        Err(ErrorCode(310))
    } else {
        Ok(())
    }
}

/// Pure float op — Number*Number or Number*Int (P1: mixed int-float fast path).
#[inline(always)]
fn num_op(a: Value16, b: Value16, op: fn(f64, f64) -> f64) -> Option<Value16> {
    let (ta, pa) = a.split_tag();
    let (tb, pb) = b.split_tag();
    match (ta, tb) {
        (ReprTag::Number, ReprTag::Number) => {
            Some(Value16::number(op(f64::from_bits(pa), f64::from_bits(pb))))
        }
        (ReprTag::Number, ReprTag::Int) => {
            Some(Value16::number(op(f64::from_bits(pa), pb as i64 as f64)))
        }
        (ReprTag::Int, ReprTag::Number) => {
            Some(Value16::number(op(pa as i64 as f64, f64::from_bits(pb))))
        }
        _ => None,
    }
}

/// Mixed Int/Float fallback — never BigInt.
#[inline]
fn try_num_op(a: Value16, b: Value16, op: fn(f64, f64) -> f64) -> Option<Value16> {
    if a.is_bigint() || b.is_bigint() {
        return None;
    }
    let a_num = a.as_number().or_else(|| a.as_int().map(|i| i as f64))?;
    let b_num = b.as_number().or_else(|| b.as_int().map(|i| i as f64))?;
    Some(Value16::number(op(a_num, b_num)))
}

// ── F2: ALL hot-path helpers #[inline(always)], cold overflow #[cold] ──

#[inline(always)]
pub fn int_mul(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    check_mix(a, b)?;
    if let Some(r) = num_op(a, b, |x, y| x * y) {
        return Ok(r);
    }
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        return Ok(match x.checked_mul(y) {
            Some(r) => Value16::int(r),
            None => promote_mul(x, y),
        });
    }
    if let Some(r) = try_num_op(a, b, |x, y| x * y) {
        return Ok(r);
    }
    bigint_mul(a, b)
}

#[inline(always)]
pub fn int_add(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    check_mix(a, b)?;
    if let Some(r) = num_op(a, b, |x, y| x + y) {
        return Ok(r);
    }
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        return Ok(match x.checked_add(y) {
            Some(r) => Value16::int(r),
            None => promote_add(x, y),
        });
    }
    if let Some(r) = try_num_op(a, b, |x, y| x + y) {
        return Ok(r);
    }
    bigint_add(a, b)
}

#[inline(always)]
pub fn int_sub(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    check_mix(a, b)?;
    if let Some(r) = num_op(a, b, |x, y| x - y) {
        return Ok(r);
    }
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        return Ok(match x.checked_sub(y) {
            Some(r) => Value16::int(r),
            None => promote_sub(x, y),
        });
    }
    if let Some(r) = try_num_op(a, b, |x, y| x - y) {
        return Ok(r);
    }
    bigint_sub(a, b)
}

#[inline(always)]
pub fn int_div(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    check_mix(a, b)?;
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        if y == 0 {
            return Err(ErrorCode(310));
        }
        return Ok(Value16::int(x / y));
    }
    if let Some(r) = num_op(a, b, |x, y| if y == 0.0 { f64::NAN } else { x / y }) {
        if r.as_number().map_or(false, |n| n.is_nan()) {
            return Err(ErrorCode(310));
        }
        return Ok(r);
    }
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    if y == num_bigint::BigInt::ZERO {
        return Err(ErrorCode(310));
    }
    Ok(Value16::bigint(x / y))
}

#[inline(always)]
pub fn int_mod(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    check_mix(a, b)?;
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        if y == 0 {
            return Err(ErrorCode(310));
        }
        return Ok(Value16::int(x % y));
    }
    if let Some(r) = num_op(a, b, |x, y| if y == 0.0 { f64::NAN } else { x % y }) {
        if r.as_number().map_or(false, |n| n.is_nan()) {
            return Err(ErrorCode(310));
        }
        return Ok(r);
    }
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    if y == num_bigint::BigInt::ZERO {
        return Err(ErrorCode(310));
    }
    Ok(Value16::bigint(x % y))
}

#[inline]
pub fn int_cmp(a: Value16, b: Value16) -> Result<Ordering, ErrorCode> {
    check_mix(a, b)?;
    if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
        return Ok(x.cmp(&y));
    }
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    Ok(x.cmp(&y))
}
