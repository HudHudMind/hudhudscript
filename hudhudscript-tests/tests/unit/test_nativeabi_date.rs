//! Tests for hudhudscript-native-abi date/time ABI — epoch/monotonic
//! clocks and pure-integer Gregorian calendar decomposition.

use hudhudscript_native_abi::{
    hudhud_date_day, hudhud_date_hour, hudhud_date_iso, hudhud_date_millis,
    hudhud_date_minute, hudhud_date_month, hudhud_date_now, hudhud_date_second,
    hudhud_date_year, hudhud_string_free, hudhud_time_micros, hudhud_time_nanos,
};

// ── clocks ───────────────────────────────────────────────────────

#[test]
fn test_date_millis_and_now() {
    let ms = hudhud_date_millis();
    let now = hudhud_date_now();
    assert!(ms > 1_700_000_000_000); // Year 2023+
    assert!(now > 1_700_000_000.0);
    let diff = (now * 1000.0) - (ms as f64);
    assert!(diff.abs() < 1000.0);
}

#[test]
fn test_monotonic_timers() {
    let t1 = hudhud_time_nanos();
    let m1 = hudhud_time_micros();
    assert!(t1 >= 0);
    assert!(m1 >= 0);
    let t2 = hudhud_time_nanos();
    assert!(t2 >= t1);
}

// ── Gregorian calendar decomposition ─────────────────────────────

#[test]
fn test_civil_decomposition_known_dates() {
    // 2026-09-18 09:00:00 UTC = 1789722000 seconds = 1789722000000 ms
    let ts: i64 = 1789722000000;
    assert_eq!(hudhud_date_year(ts), 2026);
    assert_eq!(hudhud_date_month(ts), 9);
    assert_eq!(hudhud_date_day(ts), 18);
    assert_eq!(hudhud_date_hour(ts), 9);
    assert_eq!(hudhud_date_minute(ts), 0);
    assert_eq!(hudhud_date_second(ts), 0);

    // Epoch 0: 1970-01-01 00:00:00 UTC
    assert_eq!(hudhud_date_year(0), 1970);
    assert_eq!(hudhud_date_month(0), 1);
    assert_eq!(hudhud_date_day(0), 1);
    assert_eq!(hudhud_date_hour(0), 0);
    assert_eq!(hudhud_date_minute(0), 0);
    assert_eq!(hudhud_date_second(0), 0);

    // Leap year 2000-02-29 12:30:45 UTC
    // 2000-02-29 12:30:45 = 951827445000 ms
    let ts_leap: i64 = 951827445000;
    assert_eq!(hudhud_date_year(ts_leap), 2000);
    assert_eq!(hudhud_date_month(ts_leap), 2);
    assert_eq!(hudhud_date_day(ts_leap), 29);
    assert_eq!(hudhud_date_hour(ts_leap), 12);
    assert_eq!(hudhud_date_minute(ts_leap), 30);
    assert_eq!(hudhud_date_second(ts_leap), 45);
}

// ── ISO-8601 formatting ──────────────────────────────────────────

#[test]
fn test_date_iso() {
    let ts: i64 = 1789722000000;
    let s = hudhud_date_iso(ts);
    assert!(!s.is_null());
    let cstr = unsafe { std::ffi::CStr::from_ptr(s).to_str().unwrap() };
    assert_eq!(cstr, "2026-09-18T09:00:00Z");
    unsafe { hudhud_string_free(s) };
}
