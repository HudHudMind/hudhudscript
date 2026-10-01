//! Date and Time Native ABI for fast monotonic and epoch time operations.
//! Zero-allocation fast paths for Date.now(), Date.to_millis(), high-res timers,
//! and pure-integer Gregorian calendar decomposition.

use std::ffi::{c_char, CString};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static MONOTONIC_START: OnceLock<Instant> = OnceLock::new();

#[inline(always)]
fn monotonic_start() -> &'static Instant {
    MONOTONIC_START.get_or_init(Instant::now)
}

/// Millisecond resolution Unix epoch timestamp (Date.to_millis).
/// Fast path: uses std::time::SystemTime without timezone or calendar overhead.
#[no_mangle]
#[link_section = ".hudhud_hot.date_millis"]
#[inline(never)]
pub extern "C" fn hudhud_date_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Fractional seconds Unix epoch timestamp (Date.now / Date.timestamp).
#[no_mangle]
pub extern "C" fn hudhud_date_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Monotonic nanoseconds elapsed since process initialization.
/// Ideal for high-precision benchmarking without clock jumps.
#[no_mangle]
pub extern "C" fn hudhud_time_nanos() -> i64 {
    monotonic_start().elapsed().as_nanos() as i64
}

/// Monotonic microseconds elapsed since process initialization.
#[no_mangle]
pub extern "C" fn hudhud_time_micros() -> i64 {
    monotonic_start().elapsed().as_micros() as i64
}

/// Sleep for specified milliseconds.
#[no_mangle]
pub extern "C" fn hudhud_sleep_millis(ms: i64) {
    if ms > 0 {
        std::thread::sleep(Duration::from_millis(ms as u64));
    }
}

/// Sleep for specified microseconds.
#[no_mangle]
pub extern "C" fn hudhud_sleep_micros(us: i64) {
    if us > 0 {
        std::thread::sleep(Duration::from_micros(us as u64));
    }
}

// ── Pure-integer Gregorian calendar decomposition ─────────────────────────
// Based on Howard Hinnant's civil date algorithm (C++20 std::chrono reference).
// O(1), pure integer arithmetic, zero heap allocation.

#[inline(always)]
fn decompose_civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let final_y = y + if m <= 2 { 1 } else { 0 };
    (final_y, m, d)
}

#[inline(always)]
fn decompose_epoch_millis(ts_millis: i64) -> (i64, i64, i64, i64, i64, i64, i64) {
    let total_secs = ts_millis.div_euclid(1000);
    let rem_ms = ts_millis.rem_euclid(1000);
    let days = total_secs.div_euclid(86400);
    let sec_of_day = total_secs.rem_euclid(86400);

    let (year, month, day) = decompose_civil(days);
    let hour = sec_of_day / 3600;
    let rem_h = sec_of_day % 3600;
    let minute = rem_h / 60;
    let second = rem_h % 60;

    (year, month, day, hour, minute, second, rem_ms)
}

/// Extract UTC year from timestamp in milliseconds.
#[no_mangle]
pub extern "C" fn hudhud_date_year(ts_millis: i64) -> i64 {
    decompose_epoch_millis(ts_millis).0
}

/// Extract UTC month (1..=12) from timestamp in milliseconds.
#[no_mangle]
pub extern "C" fn hudhud_date_month(ts_millis: i64) -> i64 {
    decompose_epoch_millis(ts_millis).1
}

/// Extract UTC day of month (1..=31) from timestamp in milliseconds.
#[no_mangle]
pub extern "C" fn hudhud_date_day(ts_millis: i64) -> i64 {
    decompose_epoch_millis(ts_millis).2
}

/// Extract UTC hour (0..=23) from timestamp in milliseconds.
#[no_mangle]
pub extern "C" fn hudhud_date_hour(ts_millis: i64) -> i64 {
    decompose_epoch_millis(ts_millis).3
}

/// Extract UTC minute (0..=59) from timestamp in milliseconds.
#[no_mangle]
pub extern "C" fn hudhud_date_minute(ts_millis: i64) -> i64 {
    decompose_epoch_millis(ts_millis).4
}

/// Extract UTC second (0..=59) from timestamp in milliseconds.
#[no_mangle]
pub extern "C" fn hudhud_date_second(ts_millis: i64) -> i64 {
    decompose_epoch_millis(ts_millis).5
}

/// Format timestamp as ISO-8601 string: "YYYY-MM-DDTHH:MM:SSZ".
/// Caller must free with `hudhud_string_free`.
#[no_mangle]
pub extern "C" fn hudhud_date_iso(ts_millis: i64) -> *mut c_char {
    let (y, m, d, hh, mm, ss, _) = decompose_epoch_millis(ts_millis);
    let formatted = format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z");
    match CString::new(formatted) {
        Ok(c) => {
            let ptr = c.into_raw();
            crate::type_ops::register_string(ptr as usize);
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}
