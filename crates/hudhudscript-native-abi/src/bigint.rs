//! BigInt Native ABI for HudHudScript — arena destekli inline-limb repr.
//!
//! M9: sıcak yolda num-bigint YOK — büyüklük u64 limb dizisi olarak taşınır,
//! sonuçlar thread-local bump arena'dan ayrılır (ölçüm: Vec malloc ~33ns vs
//! arena ~5ns). num-bigint yalnızca soğuk yollarda (dizge, bölme, pow)
//! geçici olarak maddileştirilir. magic offset 0'da kalır — is_bigint
//! (tag+MAGIC) sözleşmesi değişmez.

use std::ffi::{c_char, CStr, CString};
use num_bigint::BigInt;

pub const HUD_BIGINT_MAGIC: u32 = 0x48554442; // 'HUDB'
// Tag kodlaması 64-bit ham kelime üzerindedir (i64/pointer şeridi):
// 32-bit hedeflerde usize dar olurdu — u64 ile bit-korur taşınır (v0.9.25).
pub const HUD_BIGINT_TAG: u64 = 0xB161_0000_0000_0000;
pub const HUD_BIGINT_PTR_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

#[inline(always)]
pub unsafe fn untag_bigint(b: *mut HudBigInt) -> *mut HudBigInt {
    ((b as u64) & HUD_BIGINT_PTR_MASK) as *mut HudBigInt
}

/// Değişken boyutlu BigInt: header + len adet u64 limb (flexible-array).
/// sign: -1 | 0 | +1; len==0 ⇔ değer 0 (limb'ler normalize: sondaki
/// sıfır limb yok).
#[repr(C)]
pub struct HudBigInt {
    pub magic: u32,
    pub sign: i32,
    pub len: u32,
    pub limbs: [u64; 1],
}

impl HudBigInt {
    #[inline(always)]
    pub fn limbs(&self) -> &[u64] {
        unsafe { std::slice::from_raw_parts(self.limbs.as_ptr(), self.len as usize) }
    }
}

/// Etiketli handle (i64/pointer şeridi) — tüm dış dönüşler bu biçimde.
#[inline(always)]
pub fn handle_of(p: *mut HudBigInt) -> i64 {
    ((p as u64) | HUD_BIGINT_TAG) as i64
}

// ── Thread-local bump arena (leak-on-exit lane sözleşmesi) ─────────────
// UnsafeCell + ham işaretçi: RefCell borrow maliyeti yok (~10ns tasarruf).

const ARENA_SLAB: usize = 1 << 21; // 2 MiB
struct Arena {
    slab: *mut u8,
    used: usize,
}
thread_local! {
    static ARENA: std::cell::UnsafeCell<Arena> = std::cell::UnsafeCell::new(unsafe { arena_new() });
}

unsafe fn arena_new() -> Arena {
    let layout = std::alloc::Layout::from_size_align(ARENA_SLAB, 16).unwrap();
    let p = std::alloc::alloc(layout);
    Arena { slab: p, used: 0 }
}

/// 16-bayt hizalı blok; slab dolunca yenisi (eskisi leak — sözleşme).
#[inline(always)]
unsafe fn arena_alloc(n_bytes: usize) -> *mut u8 {
    ARENA.with(|cell| {
        let a = &mut *cell.get();
        let need = (n_bytes + 15) & !15;
        if a.used + need > ARENA_SLAB {
            *a = arena_new();
        }
        let p = a.slab.add(a.used);
        a.used += need;
        p
    })
}

/// sign + len limb'lik BigInt için arena'dan yer ayır, header'ı yaz.
pub fn alloc_bigint(sign: i32, len: usize) -> *mut HudBigInt {
    let bytes = std::mem::size_of::<HudBigInt>() + len.saturating_sub(1) * 8;
    let p = unsafe { arena_alloc(bytes) } as *mut HudBigInt;
    unsafe {
        (*p).magic = HUD_BIGINT_MAGIC;
        (*p).sign = sign;
        (*p).len = len as u32;
    }
    p
}

/// Limb diliminden BigInt handle üret (diziyi kopyalar, normalize eder).
pub fn from_limbs(sign: i32, limbs: &[u64]) -> i64 {
    let mut n = limbs.len();
    while n > 0 && limbs[n - 1] == 0 {
        n -= 1;
    }
    if n == 0 {
        return ((alloc_bigint(0, 0) as u64) | HUD_BIGINT_TAG) as i64;
    }
    let p = alloc_bigint(sign, n);
    unsafe {
        std::ptr::copy_nonoverlapping(limbs.as_ptr(), (*p).limbs.as_mut_ptr(), n);
    }
    ((p as u64) | HUD_BIGINT_TAG) as i64
}

/// num-bigint değeri → arena repr'ı (soğuk köprü: promote/bölme/pow yolları).
pub fn hudhud_bigint_from_bigint(val: BigInt) -> *mut HudBigInt {
    let (sign_u32, digits) = val.to_u64_digits();
    let sign = match sign_u32 {
        num_bigint::Sign::Minus => -1,
        num_bigint::Sign::NoSign => 0,
        num_bigint::Sign::Plus => 1,
    };
    from_limbs(sign, &digits) as *mut HudBigInt
}

/// u64 limb'ler → u32 digit'ler (num-bigint 0.4 BigDigit=u32; little-endian
/// çiftler, sondaki sıfırlar atılır — BigUint::new trailing-zero'da panic).
pub fn u64_limbs_to_digits(limbs: &[u64]) -> Vec<u32> {
    let mut digits = Vec::with_capacity(limbs.len() * 2);
    for &l in limbs {
        digits.push(l as u32);
        digits.push((l >> 32) as u32);
    }
    while digits.last() == Some(&0) {
        digits.pop();
    }
    digits
}

/// num-bigint Sign üret.
pub fn nb_sign(sign: i32) -> num_bigint::Sign {
    match sign {
        -1 => num_bigint::Sign::Minus,
        0 => num_bigint::Sign::NoSign,
        _ => num_bigint::Sign::Plus,
    }
}

/// Arena repr'ı → num-bigint (soğuk: dizge/bölme/pow/i64-dış dönüşüm).
pub fn to_bigint(b: &HudBigInt) -> BigInt {
    let mag = num_bigint::BigUint::new(u64_limbs_to_digits(b.limbs()));
    BigInt::from_biguint(nb_sign(b.sign), mag)
}

/// BigInt'i ondalık dizgeye çevir (soğuk yol — num-bigint).
pub fn to_str(b: &HudBigInt) -> String {
    to_bigint(b).to_string()
}

/// Değer i64'e sığıyorsa Some (işaret düşünülür; ±2^63 kenarları doğru).
pub fn fits_i64(sign: i32, limbs: &[u64]) -> Option<i64> {
    if limbs.is_empty() {
        return Some(0);
    }
    if limbs.len() > 1 {
        return None;
    }
    let v = limbs[0];
    if sign >= 0 {
        if v <= i64::MAX as u64 {
            Some(v as i64)
        } else {
            None
        }
    } else if v <= (i64::MAX as u64) + 1 {
        Some((v as i128).wrapping_neg() as i64)
    } else {
        None
    }
}

// ── C ABI (isimler korunur) ────────────────────────────────────────────

/// Create a new BigInt from an `i64`.
#[no_mangle]
pub extern "C" fn hudhud_bigint_from_i64(val: i64) -> *mut HudBigInt {
    from_i64_raw(val) as *mut HudBigInt
}

/// i64 → etiketli arena BigInt handle.
pub fn from_i64_raw(val: i64) -> i64 {
    if val == 0 {
        let p = alloc_bigint(0, 0);
        return handle_of(p);
    }
    let (sign, mag) = if val > 0 {
        (1, val as u64)
    } else if val > i64::MIN {
        (-1, (val as i128).unsigned_abs() as u64)
    } else {
        (-1, 1u64 << 63)
    };
    from_limbs(sign, &[mag])
}

/// Create a new BigInt from a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_from_str(s: *const c_char) -> *mut HudBigInt {
    if s.is_null() {
        return handle_of(alloc_bigint(0, 0)) as *mut HudBigInt;
    }
    let rust_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s.trim(),
        Err(_) => return handle_of(alloc_bigint(0, 0)) as *mut HudBigInt,
    };
    let val = rust_str.parse::<BigInt>().unwrap_or_else(|_| BigInt::from(0));
    hudhud_bigint_from_bigint(val)
}

/// Convert a BigInt to `i64` (truncated if overflow).
#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_to_i64(b: *mut HudBigInt) -> i64 {
    let real = untag_bigint(b);
    if real.is_null() {
        return 0;
    }
    fits_i64((*real).sign, (*real).limbs()).unwrap_or(0)
}

/// Convert a BigInt to a newly allocated C-string (caller frees with `hudhud_string_free`).
#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_to_str(b: *mut HudBigInt) -> *mut c_char {
    let real = untag_bigint(b);
    if real.is_null() {
        return std::ptr::null_mut();
    }
    let s = to_str(&*real);
    CString::new(s).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut())
}

/// Print a BigInt to stdout.
#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_print(b: *mut HudBigInt) {
    let real = untag_bigint(b);
    if !real.is_null() {
        println!("{}", to_str(&*real));
    }
}

/// Free a BigInt handle. Arena repr: bellek leak-on-exit sözleşmesiyle
/// yaşar (free kayıt temizliği dışında no-op); ABI uyumu için korunur.
#[no_mangle]
pub unsafe extern "C" fn hudhud_bigint_free(b: *mut HudBigInt) {
    let real = untag_bigint(b);
    if !real.is_null() {
        crate::type_ops::unregister_bigint(real as usize);
    }
}

// Aritmetik + polymorphic hızlı yollar bigint_arith.rs'te.

/// i64 bit deseni → f64 (uniform ABI'yi geçerken bit koru — sayısal çevrim DEĞİL)
#[no_mangle]
pub extern "C" fn hudhud_f64_bits_from_i64(v: i64) -> f64 {
    f64::from_bits(v as u64)
}

/// f64 bit deseni → i64
#[no_mangle]
pub extern "C" fn hudhud_i64_bits_from_f64(v: f64) -> i64 {
    v.to_bits() as i64
}
