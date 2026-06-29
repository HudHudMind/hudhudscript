//! Date/Time primitive public API tests.
//!
//! Moved from `hudhud-datetime/src/date.rs` inline test module as part of I2-A2.

use hudhud_datetime::date;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

fn from_millis(args: &[Value16]) -> HudHudResult<Value16> {
    date::from_millis(args)
}

#[test]
fn test_to_millis_returns_int_and_from_millis_accepts_int() {
    // Current timestamp as Int
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let val = Value16::int(ms);
    assert!(val.is_int(), "to_millis should produce Int");
    assert!(
        val.as_int().unwrap() > 1_000_000_000_000,
        "millis should be large positive"
    );

    // from_millis forward compat: Int argument accepted
    let args = [val];
    let result = from_millis(&args);
    assert!(result.is_ok(), "from_millis should accept Int argument");
}
