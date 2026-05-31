use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::timer_ops::sleep;

#[test]
fn test_sleep_zero() {
    let result = sleep(&[Value16::number(0.0)]);
    assert_eq!(result.unwrap(), Value16::null());
}

#[test]
fn test_sleep_small_duration() {
    let start = std::time::Instant::now();
    let result = sleep(&[Value16::number(10.0)]);
    assert!(result.is_ok());
    assert!(start.elapsed().as_millis() >= 9);
}

#[test]
fn test_sleep_negative_error() {
    let result = sleep(&[Value16::number(-1.0)]);
    assert!(result.is_err());
}

#[test]
fn test_sleep_type_error() {
    let result = sleep(&[Value16::string("100".to_string())]);
    assert!(result.is_err());
}
