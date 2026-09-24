//! M9 sayısal ABI: polymorphic hızlı yollar (num_add/sub/mul/cmp/div/rem),
//! açık BigInt ABI'si ve JIT trusted helper'ları. H.6 disiplini: her op'ta
//! int×int donanım yolu İLK; limb/num-bigint yolları cold.

use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

use crate::bigint::{
    alloc_bigint, handle_of, hudhud_bigint_from_bigint, to_bigint, untag_bigint,
    HudBigInt,
};
use crate::bigint_arith::{
    demote_or, mul_limbs, operand, pick, signed_add, split_i64, val_cmp,
};

// ── Açık BigInt ABI (isimler/semantikler korunur) ─────────────────────

fn arg_limbs(b: *mut HudBigInt) -> (i32, &'static [u64]) {
    unsafe {
        let real = untag_bigint(b);
        if real.is_null() {
            (0, &[])
        } else {
            ((*real).sign, (*real).limbs())
        }
    }
}

fn zero_handle() -> *mut HudBigInt {
    handle_of(alloc_bigint(0, 0)) as *mut HudBigInt
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_add(a: *mut HudBigInt, b: *mut HudBigInt) -> *mut HudBigInt {
    let (sa, la) = arg_limbs(a);
    let (sb, lb) = arg_limbs(b);
    signed_add(sa, la, sb, lb) as *mut HudBigInt
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_sub(a: *mut HudBigInt, b: *mut HudBigInt) -> *mut HudBigInt {
    let (sa, la) = arg_limbs(a);
    let (sb, lb) = arg_limbs(b);
    let neg_sb = if sb == 0 { 0 } else { -sb };
    signed_add(sa, la, neg_sb, lb) as *mut HudBigInt
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_mul(a: *mut HudBigInt, b: *mut HudBigInt) -> *mut HudBigInt {
    let (sa, la) = arg_limbs(a);
    let (sb, lb) = arg_limbs(b);
    mul_limbs(sa, la, sb, lb) as *mut HudBigInt
}

fn to_materialized(sign: i32, limbs: &[u64]) -> BigInt {
    let mag = num_bigint::BigUint::new(crate::bigint::u64_limbs_to_digits(limbs));
    BigInt::from_biguint(crate::bigint::nb_sign(sign), mag)
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_div(a: *mut HudBigInt, b: *mut HudBigInt) -> *mut HudBigInt {
    // Soğuk yol: num-bigint bölmesi (sıcak döngülerde bölme yok)
    let (sa, la) = arg_limbs(a);
    let (sb, lb) = arg_limbs(b);
    let ba = to_materialized(sa, la);
    let bb = to_materialized(sb, lb);
    if bb.is_zero() {
        return zero_handle();
    }
    hudhud_bigint_from_bigint(ba / bb)
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_rem(a: *mut HudBigInt, b: *mut HudBigInt) -> *mut HudBigInt {
    let (sa, la) = arg_limbs(a);
    let (sb, lb) = arg_limbs(b);
    let ba = to_materialized(sa, la);
    let bb = to_materialized(sb, lb);
    if bb.is_zero() {
        return zero_handle();
    }
    hudhud_bigint_from_bigint(ba % bb)
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_pow(a: *mut HudBigInt, exp: u32) -> *mut HudBigInt {
    let real = untag_bigint(a);
    if real.is_null() {
        return zero_handle();
    }
    hudhud_bigint_from_bigint(to_bigint(&*real).pow(exp))
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_cmp(a: *mut HudBigInt, b: *mut HudBigInt) -> i64 {
    let (sa, la) = arg_limbs(a);
    let (sb, lb) = arg_limbs(b);
    match val_cmp(sa, la, sb, lb) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

// ── Polymorphic Fast-Path Arithmetic for JIT/AOT Promotion ────────────

#[no_mangle]
#[link_section = ".hudhud_hot.num_add"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_num_add(a: i64, b: i64) -> i64 {
    let a_is_big = crate::type_ops::is_bigint(a as u64);
    let b_is_big = crate::type_ops::is_bigint(b as u64);
    if !a_is_big && !b_is_big {
        if let Some(sum) = a.checked_add(b) {
            return sum;
        }
    }
    let (sa, ba, na, biga) = operand(a, a_is_big);
    let la: &[u64] = pick(&ba, na, biga);
    let (sb, bb, nb, bigb) = operand(b, b_is_big);
    let lb: &[u64] = pick(&bb, nb, bigb);
    let r = signed_add(sa, la, sb, lb);
    // VM oracle: BigInt+BigInt demote OLMAZ; karışık/taşma yolları demote eder
    if a_is_big && b_is_big {
        return r;
    }
    demote_or(r)
}

#[no_mangle]
#[link_section = ".hudhud_hot.num_sub"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_num_sub(a: i64, b: i64) -> i64 {
    let a_is_big = crate::type_ops::is_bigint(a as u64);
    let b_is_big = crate::type_ops::is_bigint(b as u64);
    if !a_is_big && !b_is_big {
        if let Some(diff) = a.checked_sub(b) {
            return diff;
        }
    }
    // a - b = a + (-b). -(i64::MIN) = +2^63 — iki limb, stack'te kur.
    let (sa, ba, na, biga) = operand(a, a_is_big);
    let la: &[u64] = pick(&ba, na, biga);
    let r = if b_is_big {
        let p = untag_bigint(b as *mut HudBigInt);
        let s = (*p).sign;
        signed_add(sa, la, if s == 0 { 0 } else { -s }, (*p).limbs())
    } else if b == i64::MIN {
        signed_add(sa, la, 1, &[1u64 << 63, 0][..1]) // -(MIN) = +2^63 tek limb
    } else {
        let (s, buf, n) = split_i64(-b);
        signed_add(sa, la, s, &buf[..n])
    };
    // VM oracle (bigint_sub): her yolda demote
    demote_or(r)
}

#[no_mangle]
#[link_section = ".hudhud_hot.num_mul"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_num_mul(a: i64, b: i64) -> i64 {
    let a_is_big = crate::type_ops::is_bigint(a as u64);
    let b_is_big = crate::type_ops::is_bigint(b as u64);
    if !a_is_big && !b_is_big {
        if let Some(prod) = a.checked_mul(b) {
            return prod;
        }
    }
    let (sa, ba, na, biga) = operand(a, a_is_big);
    let la: &[u64] = pick(&ba, na, biga);
    let (sb, bb, nb, bigb) = operand(b, b_is_big);
    let lb: &[u64] = pick(&bb, nb, bigb);
    let r = mul_limbs(sa, la, sb, lb);
    // VM oracle: BigInt*BigInt demote OLMAZ
    if a_is_big && b_is_big {
        return r;
    }
    demote_or(r)
}

#[no_mangle]
#[link_section = ".hudhud_hot.num_cmp"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_num_cmp(a: i64, b: i64) -> i64 {
    let a_is_big = crate::type_ops::is_bigint(a as u64);
    let b_is_big = crate::type_ops::is_bigint(b as u64);
    if !a_is_big && !b_is_big {
        return match a.cmp(&b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };
    }
    let (sa, ba, na, biga) = operand(a, a_is_big);
    let la: &[u64] = pick(&ba, na, biga);
    let (sb, bb, nb, bigb) = operand(b, b_is_big);
    let lb: &[u64] = pick(&bb, nb, bigb);
    match val_cmp(sa, la, sb, lb) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

#[no_mangle]
#[link_section = ".hudhud_hot.num_div"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_num_div(a: i64, b: i64) -> i64 {
    let a_is_big = crate::type_ops::is_bigint(a as u64);
    let b_is_big = crate::type_ops::is_bigint(b as u64);
    if !a_is_big && !b_is_big {
        if b != 0 && !(a == i64::MIN && b == -1) {
            return a / b;
        }
    }
    num_div_rem(a, b, a_is_big, b_is_big, true)
}

#[no_mangle]
#[link_section = ".hudhud_hot.num_rem"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_num_rem(a: i64, b: i64) -> i64 {
    let a_is_big = crate::type_ops::is_bigint(a as u64);
    let b_is_big = crate::type_ops::is_bigint(b as u64);
    if !a_is_big && !b_is_big {
        if b != 0 && !(a == i64::MIN && b == -1) {
            return a % b;
        }
    }
    num_div_rem(a, b, a_is_big, b_is_big, false)
}

#[cold]
#[inline(never)]
unsafe fn num_div_rem(a: i64, b: i64, a_is_big: bool, b_is_big: bool, is_div: bool) -> i64 {
    let ba = if a_is_big {
        to_bigint(&*untag_bigint(a as *mut HudBigInt))
    } else {
        BigInt::from(a)
    };
    let bb = if b_is_big {
        to_bigint(&*untag_bigint(b as *mut HudBigInt))
    } else {
        BigInt::from(b)
    };
    if bb.is_zero() {
        return 0;
    }
    let q = if is_div { ba / bb } else { ba % bb };
    if let Some(i) = q.to_i64() {
        return i;
    }
    hudhud_bigint_from_bigint(q) as i64
}

// ── Trusted helper'lar (JIT tag-check'ten gelir; precondition: both-big) ──

#[no_mangle]
#[link_section = ".hudhud_hot.bigint_add_trusted"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_bigint_add_trusted(a: i64, b: i64) -> i64 {
    let ra = untag_bigint(a as *mut HudBigInt);
    let rb = untag_bigint(b as *mut HudBigInt);
    // VM oracle: BigInt+BigInt demote OLMAZ
    signed_add((*ra).sign, (*ra).limbs(), (*rb).sign, (*rb).limbs())
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_sub_trusted(a: i64, b: i64) -> i64 {
    let ra = untag_bigint(a as *mut HudBigInt);
    let rb = untag_bigint(b as *mut HudBigInt);
    let sb = (*rb).sign;
    let neg = if sb == 0 { 0 } else { -sb };
    let r = signed_add((*ra).sign, (*ra).limbs(), neg, (*rb).limbs());
    demote_or(r) // VM oracle: sub demote eder
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_mul_trusted(a: i64, b: i64) -> i64 {
    let ra = untag_bigint(a as *mut HudBigInt);
    let rb = untag_bigint(b as *mut HudBigInt);
    // VM oracle: BigInt*BigInt demote OLMAZ
    mul_limbs((*ra).sign, (*ra).limbs(), (*rb).sign, (*rb).limbs())
}
