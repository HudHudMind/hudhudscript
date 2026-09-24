//! BigInt Native ABI tests.

use std::ffi::CString;
use hudhudscript_native_abi::*;

#[test]
fn test_bigint_creation_and_to_str() {
    let b = hudhud_bigint_from_i64(123456789);
    assert!(!b.is_null());
    assert_eq!(unsafe { hudhud_bigint_to_i64(b) }, 123456789);

    let s = unsafe { hudhud_bigint_to_str(b) };
    let rust_s = unsafe { std::ffi::CStr::from_ptr(s).to_str().unwrap() };
    assert_eq!(rust_s, "123456789");
    unsafe { hudhud_string_free(s) };
    unsafe { hudhud_bigint_free(b) };
}

#[test]
fn test_bigint_from_str_large() {
    let large_str = "434665576869374564356885276750406258025646605173717804024817290895365554179490518904038798400792551692959225930803226347752096896232398733224711616429964409065331879382989696499285160037044761377951668492288750000";
    let cs = CString::new(large_str).unwrap();
    let b = unsafe { hudhud_bigint_from_str(cs.as_ptr()) };
    assert!(!b.is_null());

    let s = unsafe { hudhud_bigint_to_str(b) };
    let rust_s = unsafe { std::ffi::CStr::from_ptr(s).to_str().unwrap() };
    assert_eq!(rust_s, large_str);
    unsafe { hudhud_string_free(s) };
    unsafe { hudhud_bigint_free(b) };
}

#[test]
fn test_bigint_arithmetic() {
    let a = hudhud_bigint_from_i64(1_000_000_000_000_000_000);
    let b = hudhud_bigint_from_i64(2_000_000_000_000_000_000);

    let sum = unsafe { hudhud_bigint_add(a, b) };
    assert_eq!(unsafe { hudhud_bigint_to_i64(sum) }, 3_000_000_000_000_000_000);

    let diff = unsafe { hudhud_bigint_sub(sum, a) };
    assert_eq!(unsafe { hudhud_bigint_to_i64(diff) }, 2_000_000_000_000_000_000);

    let mul = unsafe { hudhud_bigint_mul(a, b) };
    let s = unsafe { hudhud_bigint_to_str(mul) };
    let rust_s = unsafe { std::ffi::CStr::from_ptr(s).to_str().unwrap() };
    assert_eq!(rust_s, "2000000000000000000000000000000000000");
    unsafe { hudhud_string_free(s) };

    unsafe {
        hudhud_bigint_free(a);
        hudhud_bigint_free(b);
        hudhud_bigint_free(sum);
        hudhud_bigint_free(diff);
        hudhud_bigint_free(mul);
    }
}

#[test]
fn test_polymorphic_add_overflow_promotion() {
    // Normal addition without overflow
    let sum1 = unsafe { hudhud_num_add(10, 20) };
    assert_eq!(sum1, 30);
    assert!(!is_bigint(sum1 as u64));

    // Addition with overflow promotes to BigInt
    let sum2 = unsafe { hudhud_num_add(i64::MAX, 1) };
    assert!(is_bigint(sum2 as u64));

    let b = sum2 as *mut HudBigInt;
    let s = unsafe { hudhud_bigint_to_str(b) };
    let rust_s = unsafe { std::ffi::CStr::from_ptr(s).to_str().unwrap() };
    assert_eq!(rust_s, "9223372036854775808");
    unsafe { hudhud_string_free(s) };

    // Further addition on promoted BigInt
    let sum3 = unsafe { hudhud_num_add(sum2, 2) };
    assert!(is_bigint(sum3 as u64));
    let b3 = sum3 as *mut HudBigInt;
    let s3 = unsafe { hudhud_bigint_to_str(b3) };
    let rust_s3 = unsafe { std::ffi::CStr::from_ptr(s3).to_str().unwrap() };
    assert_eq!(rust_s3, "9223372036854775810");
    unsafe { hudhud_string_free(s3) };

    // Test hudhud_int_to_string on promoted BigInt tagged pointer
    let s_promoted = hudhud_int_to_string(sum3);
    assert!(!s_promoted.is_null());
    let rust_sp = unsafe { std::ffi::CStr::from_ptr(s_promoted).to_str().unwrap() };
    assert_eq!(rust_sp, "9223372036854775810");

    // Test string concat with promoted BigInt
    let prefix = CString::new("Result: ").unwrap();
    let concatenated = unsafe { hudhud_string_concat(prefix.as_ptr(), s_promoted) };
    assert!(!concatenated.is_null());
    let rust_concat = unsafe { std::ffi::CStr::from_ptr(concatenated).to_str().unwrap() };
    assert_eq!(rust_concat, "Result: 9223372036854775810");
    unsafe {
        hudhud_string_free(s_promoted);
        hudhud_string_free(concatenated);
    }

    // Direct concat with raw BigInt handle (defensive check)
    let direct_concat = unsafe { hudhud_string_concat(prefix.as_ptr(), sum3 as *const std::ffi::c_char) };
    assert!(!direct_concat.is_null());
    let rust_dc = unsafe { std::ffi::CStr::from_ptr(direct_concat).to_str().unwrap() };
    assert_eq!(rust_dc, "Result: 9223372036854775810");
    unsafe { hudhud_string_free(direct_concat); }

    // Test print on promoted BigInt
    assert_eq!(hudhud_print_int(sum3), HUDHUD_PRINT_OK);
}

#[test]
fn no_demote_add_zero_stays_bigint() {
    // VM paritesi: BigInt+BigInt=0 → BigInt kalır; typeof "bigint".
    // (0-big i64::MIN'e demote olur — both-big için 2^64 kullanılır)
    unsafe {
        let big = hudhudscript_native_abi::hudhud_num_add(i64::MAX, 1); // 2^63 BigInt
        let big2 = hudhudscript_native_abi::hudhud_num_add(big, big);   // 2^64 BigInt
        let neg2 = hudhudscript_native_abi::hudhud_num_sub(0, big2);    // -2^64 (sığmaz → BigInt)
        let s = hudhudscript_native_abi::hudhud_num_add(big2, neg2);    // both-big → 0
        let ty = hudhudscript_native_abi::hudhud_typeof(s);
        let cstr = std::ffi::CStr::from_ptr(ty).to_str().unwrap();
        assert_eq!(cstr, "bigint", "s handle={s:#x} typeof={cstr}");
    }
}

#[test]
fn demote_sub_zero_becomes_int() {
    // VM: sub her yolda demote eder → big-big=0 INT kodlamasına döner.
    // (typeof(0) lane'de "null"dir — int-0/null kodlama belirsizliği ayrı konu;
    // burada değer-kodlaması doğrulanır: d == düz 0, tagged handle DEĞİL)
    unsafe {
        let big = hudhudscript_native_abi::hudhud_num_add(i64::MAX, 1);
        let d = hudhudscript_native_abi::hudhud_num_sub(big, big);
        assert_eq!(d, 0, "sub demote: 0 int olarak dönmeli (tagged handle değil)");
    }
}

#[test]
fn m9_limb_arith_sign_edges() {
    unsafe {
        // m = 2^126 BigInt; neg = -2^126 BigInt (soğuk kurulum, taşmayla)
        let m = hudhudscript_native_abi::hudhud_num_mul(i64::MIN, i64::MIN); // 2^126
        let neg = hudhudscript_native_abi::hudhud_num_sub(0, m);             // -2^126
        let cstr = |h: i64| {
            std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_bigint_to_str(h as *mut _))
                .to_str().unwrap().to_string()
        };
        // aynı işaret: neg + neg = -2^127
        assert_eq!(cstr(hudhudscript_native_abi::hudhud_num_add(neg, neg)), "-170141183460469231731687303715884105728");
        // zıt işaret eşit: neg + m = BigInt 0 (both-big no-demote)
        let z = hudhudscript_native_abi::hudhud_num_add(neg, m);
        assert_eq!(cstr(z), "0");
        assert_eq!(
            std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_typeof(z))
                .to_str().unwrap(),
            "bigint"
        );
        // büyüklük farkı: neg - m = -2^127 (sub demote: sığmaz → BigInt)
        assert_eq!(cstr(hudhudscript_native_abi::hudhud_num_sub(neg, m)), "-170141183460469231731687303715884105728");
        // mul işareti: neg × neg = +2^252
        assert_eq!(
            cstr(hudhudscript_native_abi::hudhud_num_mul(neg, neg)),
            "7237005577332262213973186563042994240829374041602535252466099000494570602496"
        );
        // cmp: neg < m; m < 2^252
        assert_eq!(hudhudscript_native_abi::hudhud_num_cmp(neg, m), -1);
        assert_eq!(hudhudscript_native_abi::hudhud_num_cmp(m, m), 0);
    }
}

#[test]
fn m9_limb_i64_min_edges() {
    unsafe {
        // i64::MIN - 1 = -2^63-1 (BigInt, taşma promote)
        let v = hudhudscript_native_abi::hudhud_num_sub(i64::MIN, 1);
        let v_str = hudhudscript_native_abi::hudhud_bigint_to_str(v as *mut _);
        assert_eq!(
            std::ffi::CStr::from_ptr(v_str).to_str().unwrap(),
            "-9223372036854775809"
        );
        // 0 - i64::MIN = +2^63 (BigInt — i64'e sığmaz)
        let w = hudhudscript_native_abi::hudhud_num_sub(0, i64::MIN);
        let w_str = hudhudscript_native_abi::hudhud_bigint_to_str(w as *mut _);
        assert_eq!(
            std::ffi::CStr::from_ptr(w_str).to_str().unwrap(),
            "9223372036854775808"
        );
        // i64::MIN * i64::MIN = 2^126
        let p = hudhudscript_native_abi::hudhud_num_mul(i64::MIN, i64::MIN);
        let p_str = hudhudscript_native_abi::hudhud_bigint_to_str(p as *mut _);
        assert_eq!(
            std::ffi::CStr::from_ptr(p_str).to_str().unwrap(),
            "85070591730234615865843651857942052864"
        );
    }
}

#[test]
fn m9_div_pow_bridge() {
    unsafe {
        let big = hudhudscript_native_abi::hudhud_num_add(i64::MAX, 1); // 2^63
        let big2 = hudhudscript_native_abi::hudhud_num_add(big, big);   // 2^64
        // bölme köprüsü: 2^64 / 2^63 = 2
        let q = hudhudscript_native_abi::hudhud_num_div(big2, big);
        assert_eq!(q, 2);
        // kalan: 2^64+1 % 2^63 = 1
        let big2p1 = hudhudscript_native_abi::hudhud_num_add(big2, 1);
        let r = hudhudscript_native_abi::hudhud_num_rem(big2p1, big);
        assert_eq!(r, 1);
        // pow köprüsü: (2^63)^2 = 2^126
        let pw = hudhudscript_native_abi::hudhud_bigint_pow(big as *mut _, 2);
        let pw_str = hudhudscript_native_abi::hudhud_bigint_to_str(pw);
        assert_eq!(
            std::ffi::CStr::from_ptr(pw_str).to_str().unwrap(),
            "85070591730234615865843651857942052864"
        );
    }
}
