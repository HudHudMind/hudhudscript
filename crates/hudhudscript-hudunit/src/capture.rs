//! Test stdout capture.
//!
//! The VM routes every `print` through `hudhud_print::print_ops`, which has
//! a built-in thread-local capture buffer (`start_capture`/`stop_capture`)
//! intended exactly for this purpose. Capturing therefore needs no file
//! descriptor tricks: enable the buffer around the test execution and drain
//! it afterwards.

/// Run `f` with the current thread's HudHudScript print output captured and
/// return `(f's value, captured output)`.
pub fn capture_prints<T>(f: impl FnOnce() -> T) -> (T, String) {
    hudhud_print::print_ops::start_capture();
    let value = f();
    let captured = hudhud_print::print_ops::stop_capture().unwrap_or_default();
    (value, captured)
}
