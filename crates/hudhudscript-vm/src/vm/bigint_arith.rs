use hudhudscript_bytecode::{ReprTag, Value16};
use hudhudscript_errors::catalog::ErrorCode;
use std::cmp::Ordering;

// ── Cold overflow promotion — #[cold] #[inline(never)] ──────────

#[cold] #[inline(never)]
pub(crate) fn promote_mul(x: i64, y: i64) -> Value16 {
    Value16::bigint(num_bigint::BigInt::from(x) * num_bigint::BigInt::from(y))
}
#[cold] #[inline(never)]
pub(crate) fn promote_add(x: i64, y: i64) -> Value16 {
    Value16::bigint(num_bigint::BigInt::from(x) + num_bigint::BigInt::from(y))
}
#[cold] #[inline(never)]
pub(crate) fn promote_sub(x: i64, y: i64) -> Value16 {
    Value16::bigint(num_bigint::BigInt::from(x) - num_bigint::BigInt::from(y))
}

// ── Cold BigInt fallback — #[cold] #[inline(never)] ─────────────

#[cold] #[inline(never)]
pub(crate) fn bigint_mul(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_bigint()) {
        return Ok(Value16::bigint_no_demote(x * y));
    }
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_int()) {
        return Ok(Value16::bigint(x * num_bigint::BigInt::from(y)));
    }
    if let (Some(x), Some(y)) = (a.as_int(), b.as_bigint()) {
        return Ok(Value16::bigint(num_bigint::BigInt::from(x) * y));
    }
    Err(ErrorCode(310))
}

#[cold] #[inline(never)]
pub(crate) fn bigint_add(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_bigint()) {
        return Ok(Value16::bigint_no_demote(x + y));
    }
    if let (Some(x), Some(y)) = (a.as_bigint(), b.as_int()) {
        return Ok(Value16::bigint(x + num_bigint::BigInt::from(y)));
    }
    if let (Some(x), Some(y)) = (a.as_int(), b.as_bigint()) {
        return Ok(Value16::bigint(num_bigint::BigInt::from(x) + y));
    }
    Err(ErrorCode(310))
}

#[cold] #[inline(never)]
pub(crate) fn bigint_sub(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    Ok(Value16::bigint(x - y))
}

// ── Cold BigInt division/modulo/comparison — #[cold] #[inline(never)] ─

#[cold] #[inline(never)]
pub(crate) fn bigint_div(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    if y == num_bigint::BigInt::ZERO {
        return Err(ErrorCode(399));
    }
    Ok(Value16::bigint(x / y))
}

#[cold] #[inline(never)]
pub(crate) fn bigint_mod(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    if y == num_bigint::BigInt::ZERO {
        return Err(ErrorCode(399));
    }
    Ok(Value16::bigint(x % y))
}

#[cold] #[inline(never)]
fn bigint_cmp(a: Value16, b: Value16) -> Result<Ordering, ErrorCode> {
    let x = a.to_bigint_value().ok_or(ErrorCode(310))?;
    let y = b.to_bigint_value().ok_or(ErrorCode(310))?;
    Ok(x.cmp(&y))
}

/// Split once, match once. Int+Int is the first branch — no Float
/// or BigInt check before the hottest code path.
macro_rules! int_binop {
    ($name:ident, $checked:ident, $promote:ident, $bigint:ident, $float_op:expr) => {
        #[inline(always)]
        pub fn $name(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
            let (ta, pa) = a.split_tag();
            let (tb, pb) = b.split_tag();
            match (ta, tb) {
                // ── HOT PATH: Int * Int ──
                (ReprTag::Int, ReprTag::Int) => {
                    let x = pa as i64;
                    let y = pb as i64;
                    match x.$checked(y) {
                        Some(r) => Ok(Value16::int(r)),
                        None => Ok($promote(x, y)),
                    }
                }
                // ── Cold: Number paths ──
                (ReprTag::Number, ReprTag::Number) => {
                    Ok(Value16::number($float_op(f64::from_bits(pa), f64::from_bits(pb))))
                }
                (ReprTag::Number, ReprTag::Int) => {
                    Ok(Value16::number($float_op(f64::from_bits(pa), pb as i64 as f64)))
                }
                (ReprTag::Int, ReprTag::Number) => {
                    Ok(Value16::number($float_op(pa as i64 as f64, f64::from_bits(pb))))
                }
                // ── Cold: BigInt or mixed ──
                _ => $bigint(a, b),
            }
        }
    };
}

int_binop!(int_add, checked_add, promote_add, bigint_add, |x, y| x + y);
int_binop!(int_sub, checked_sub, promote_sub, bigint_sub, |x, y| x - y);
int_binop!(int_mul, checked_mul, promote_mul, bigint_mul, |x, y| x * y);

#[inline(always)]
pub fn int_div(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    let (ta, pa) = a.split_tag();
    let (tb, pb) = b.split_tag();
    match (ta, tb) {
        (ReprTag::Int, ReprTag::Int) => {
            let y = pb as i64;
            if y == 0 { return Err(ErrorCode(310)); }
            match (pa as i64).checked_div(y) {
                Some(q) => Ok(Value16::int(q)),
                None => Ok(Value16::bigint(
                    num_bigint::BigInt::from(pa as i64) / num_bigint::BigInt::from(y),
                )),
            }
        }
        (ReprTag::Number, ReprTag::Number) => {
            let divisor = f64::from_bits(pb);
            if divisor == 0.0 { return Err(ErrorCode(310)); }
            Ok(Value16::number(f64::from_bits(pa) / divisor))
        }
        (ReprTag::Number, ReprTag::Int) => {
            let y = pb as i64;
            if y == 0 { return Err(ErrorCode(310)); }
            Ok(Value16::number(f64::from_bits(pa) / y as f64))
        }
        (ReprTag::Int, ReprTag::Number) => {
            let divisor = f64::from_bits(pb);
            if divisor == 0.0 { return Err(ErrorCode(310)); }
            Ok(Value16::number(pa as i64 as f64 / divisor))
        }
        _ => bigint_div(a, b),
    }
}

#[inline(always)]
pub fn int_mod(a: Value16, b: Value16) -> Result<Value16, ErrorCode> {
    let (ta, pa) = a.split_tag();
    let (tb, pb) = b.split_tag();
    match (ta, tb) {
        (ReprTag::Int, ReprTag::Int) => {
            let y = pb as i64;
            if y == 0 { return Err(ErrorCode(310)); }
            match (pa as i64).checked_rem(y) {
                Some(r) => Ok(Value16::int(r)),
                None => Ok(Value16::int(0)),
            }
        }
        (ReprTag::Number, ReprTag::Number) => {
            let divisor = f64::from_bits(pb);
            if divisor == 0.0 { return Err(ErrorCode(310)); }
            Ok(Value16::number(f64::from_bits(pa) % divisor))
        }
        (ReprTag::Number, ReprTag::Int) => {
            let y = pb as i64;
            if y == 0 { return Err(ErrorCode(310)); }
            Ok(Value16::number(f64::from_bits(pa) % y as f64))
        }
        (ReprTag::Int, ReprTag::Number) => {
            let divisor = f64::from_bits(pb);
            if divisor == 0.0 { return Err(ErrorCode(310)); }
            Ok(Value16::number(pa as i64 as f64 % divisor))
        }
        _ => bigint_mod(a, b),
    }
}

#[inline]
pub fn int_cmp(a: Value16, b: Value16) -> Result<Ordering, ErrorCode> {
    let (ta, pa) = a.split_tag();
    let (tb, pb) = b.split_tag();
    match (ta, tb) {
        (ReprTag::Int, ReprTag::Int) => Ok((pa as i64).cmp(&(pb as i64))),
        (ReprTag::Number, ReprTag::Number) => Ok(f64::from_bits(pa).total_cmp(&f64::from_bits(pb))),
        (ReprTag::Number, ReprTag::Int) => Ok(f64::from_bits(pa).total_cmp(&(pb as i64 as f64))),
        (ReprTag::Int, ReprTag::Number) => Ok((pa as i64 as f64).total_cmp(&f64::from_bits(pb))),
        _ => bigint_cmp(a, b),
    }
}
