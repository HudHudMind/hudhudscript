use hudhudscript_utils::error::*;

#[test]
fn test_hudhud_result_ok() {
    let result: HudHudResult<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_hudhud_result_err() {
    let result: HudHudResult<i32> = Err("something went wrong".to_string());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "something went wrong");
}

#[test]
fn test_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let hudhud_err = io_err.into_hudhud_error();
    assert!(hudhud_err.contains("IO error"));
    assert!(hudhud_err.contains("file missing"));
}
