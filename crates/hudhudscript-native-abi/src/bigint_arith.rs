//! M9 çekirdek limb aritmetiği: schoolbook add/sub/mul u64 limb'ler üzerinde,
//! sonuçlar doğrudan arena'dan (header + limb'ler tek blok; ara Vec/malloc YOK
//! — arena relocation yapmaz, ödünç dilimler sağlam). H.6: limb yolları
//! #[inline(never)]; int×int donanım yolu bigint_num.rs'te daima ilk sırada.

use crate::bigint::{alloc_bigint, from_limbs, handle_of, untag_bigint, HudBigInt};

// ── Ham limb işlemleri (normalize: çağıran len'i düzeltir) ─────────────

/// |a| + |b| → out (out.len >= max(alen,blen)+1); gerçek limb sayısı döner.
#[inline(never)]
pub(crate) unsafe fn limb_add_into(a: &[u64], b: &[u64], out: &mut [u64]) -> usize {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    let mut carry = 0u64;
    for i in 0..short.len() {
        let (s1, c1) = a[i].overflowing_add(b[i]);
        let (s2, c2) = s1.overflowing_add(carry);
        out[i] = s2;
        carry = (c1 as u64) | (c2 as u64);
    }
    for i in short.len()..long.len() {
        let (s, c) = long[i].overflowing_add(carry);
        out[i] = s;
        carry = c as u64;
    }
    let mut n = long.len();
    if carry != 0 {
        out[n] = carry;
        n += 1;
    }
    n
}

/// |a| - |b| (a >= b ZORUNLU) → out; normalize edilmiş uzunluk döner.
#[inline(never)]
pub(crate) unsafe fn limb_sub_into(a: &[u64], b: &[u64], out: &mut [u64]) -> usize {
    let mut borrow = 0u64;
    for i in 0..a.len() {
        let bi = if i < b.len() { b[i] } else { 0 };
        let (s1, br1) = a[i].overflowing_sub(bi);
        let (s2, br2) = s1.overflowing_sub(borrow);
        out[i] = s2;
        borrow = (br1 as u64) | (br2 as u64);
    }
    let mut n = a.len();
    while n > 0 && out[n - 1] == 0 {
        n -= 1;
    }
    n
}

/// |a|·|b| → out (alen+blen slot); normalize uzunluk döner.
#[inline(never)]
pub(crate) unsafe fn limb_mul_into(a: &[u64], b: &[u64], out: &mut [u64]) -> usize {
    // Küçük-operand hızlı yolu (fact/power deseni: BigInt × küçük int):
    // tek geçiş mul-with-carry — genel schoolbook'un memset+çift döngüsü
    // bu durumda 3.8× yavaştı (bigbench fact A/B: 24.7ms vs 93.6ms)
    if b.len() == 1 {
        return limb_mul_small(a, b[0], out);
    }
    if a.len() == 1 {
        return limb_mul_small(b, a[0], out);
    }
    for slot in out.iter_mut() {
        *slot = 0;
    }
    for i in 0..a.len() {
        let mut carry: u128 = 0;
        let ai = a[i] as u128;
        for j in 0..b.len() {
            let cur = out[i + j] as u128 + ai * b[j] as u128 + carry;
            out[i + j] = cur as u64;
            carry = cur >> 64;
        }
        let mut k = i + b.len();
        while carry > 0 {
            let cur = out[k] as u128 + carry;
            out[k] = cur as u64;
            carry = cur >> 64;
            k += 1;
        }
    }
    let mut n = a.len() + b.len();
    while n > 0 && out[n - 1] == 0 {
        n -= 1;
    }
    n
}

/// a × küçük u64 → out (a.len()+1 slot); normalize uzunluk döner.
/// Klasik mul-with-carry: cur = a[i]*m + carry (u128) — memset yok.
#[inline(never)]
unsafe fn limb_mul_small(a: &[u64], m: u64, out: &mut [u64]) -> usize {
    let mut carry: u128 = 0;
    for i in 0..a.len() {
        let cur = a[i] as u128 * m as u128 + carry;
        out[i] = cur as u64;
        carry = cur >> 64;
    }
    let mut n = a.len();
    if carry > 0 {
        out[n] = carry as u64;
        n += 1;
    }
    // normalize (m==0 veya trailing sıfır olmazsa da güvenli)
    while n > 0 && out[n - 1] == 0 {
        n -= 1;
    }
    n
}

// ── Karşılaştırma ve işaret yardımcıları ──────────────────────────────

/// Büyüklük karşılaştırması (uzunluk, sonra yüksek limben aşağı).
pub(crate) fn limb_cmp(a: &[u64], b: &[u64]) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    if a.len() != b.len() {
        return if a.len() < b.len() { Less } else { Greater };
    }
    for i in (0..a.len()).rev() {
        if a[i] != b[i] {
            return if a[i] < b[i] { Less } else { Greater };
        }
    }
    Equal
}

/// Değer karşılaştırması (işaret + sıfır durumları doğru).
pub(crate) fn val_cmp(sa: i32, a: &[u64], sb: i32, b: &[u64]) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let za = a.is_empty();
    let zb = b.is_empty();
    if za || zb {
        if za && zb {
            return Equal;
        }
        let other_sign = if za { sb } else { sa };
        return if other_sign < 0 { Greater } else { Less };
    }
    if sa != sb {
        return if sa < sb { Less } else { Greater };
    }
    let m = limb_cmp(a, b);
    if sa < 0 {
        m.reverse()
    } else {
        m
    }
}

/// i64 → (sign, stack magnitude); |i64::MIN| = 2^63 iki limb.
#[inline(always)]
pub(crate) fn split_i64(v: i64) -> (i32, [u64; 2], usize) {
    if v == 0 {
        return (0, [0; 2], 0);
    }
    if v > 0 {
        (1, [v as u64, 0], 1)
    } else if v > i64::MIN {
        (-1, [(-(v as i128)) as u64, 0], 1)
    } else {
        (-1, [1u64 << 63, 0], 1) // |MIN| = 2^63 — tek limb (64 bite sığar)
    }
}

#[inline(always)]
pub(crate) unsafe fn limbs_mut<'a>(p: *mut HudBigInt) -> &'a mut [u64] {
    std::slice::from_raw_parts_mut((*p).limbs.as_mut_ptr(), (*p).len as usize)
}

// ── İşaretli işlemler → arena handle ──────────────────────────────────

/// İşaretli topla (sonuç arena'da; ara allocation yok).
pub(crate) unsafe fn signed_add(sa: i32, la: &[u64], sb: i32, lb: &[u64]) -> i64 {
    let za = la.is_empty();
    let zb = lb.is_empty();
    if za {
        return from_limbs(sb, lb);
    }
    if zb {
        return from_limbs(sa, la);
    }
    if sa == sb {
        let n = la.len().max(lb.len()) + 1;
        let p = alloc_bigint(sa, n);
        let len = limb_add_into(la, lb, limbs_mut(p));
        (*p).len = len as u32;
        return handle_of(p);
    }
    match limb_cmp(la, lb) {
        std::cmp::Ordering::Equal => from_limbs(0, &[]), // zıt işaret, eş büyüklük → 0
        std::cmp::Ordering::Greater => {
            let p = alloc_bigint(sa, la.len());
            let len = limb_sub_into(la, lb, limbs_mut(p));
            (*p).len = len as u32;
            handle_of(p)
        }
        std::cmp::Ordering::Less => {
            let p = alloc_bigint(sb, lb.len());
            let len = limb_sub_into(lb, la, limbs_mut(p));
            (*p).len = len as u32;
            handle_of(p)
        }
    }
}

/// İşaretli çarp (sonuç arena'da).
pub(crate) unsafe fn mul_limbs(sa: i32, la: &[u64], sb: i32, lb: &[u64]) -> i64 {
    let sign = sa * sb;
    if la.is_empty() || lb.is_empty() {
        return from_limbs(0, &[]);
    }
    let p = alloc_bigint(sign, la.len() + lb.len());
    let len = limb_mul_into(la, lb, limbs_mut(p));
    (*p).len = len as u32;
    handle_of(p)
}

/// Sonuç handle'ı i64'e sığıyorsa demote et, değilse handle döner.
pub(crate) unsafe fn demote_or(r: i64) -> i64 {
    let p = untag_bigint(r as *mut HudBigInt);
    if let Some(i) = crate::bigint::fits_i64((*p).sign, (*p).limbs()) {
        return i;
    }
    r
}

/// İşlenen operandı limb dilimi olarak ödünç al (bigint → arena dilimi,
/// int → stack dizisi). Arena relocation yok — ödünç sağlam.
#[inline(always)]
pub(crate) unsafe fn operand(
    v: i64,
    is_big: bool,
) -> (i32, [u64; 2], usize, Option<&'static [u64]>) {
    if is_big {
        let p = untag_bigint(v as *mut HudBigInt);
        ((*p).sign, [0; 2], 0, Some((*p).limbs()))
    } else {
        let (s, buf, n) = split_i64(v);
        (s, buf, n, None)
    }
}

/// Stack/big alternatifinden dilimi seç.
#[inline(always)]
pub(crate) fn pick<'a>(buf: &'a [u64; 2], n: usize, big: Option<&'a [u64]>) -> &'a [u64] {
    big.unwrap_or(&buf[..n])
}
