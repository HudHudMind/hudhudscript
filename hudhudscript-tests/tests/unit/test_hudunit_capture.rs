//! Tests for hudhudscript-hudunit stdout capture — thread-local print
//! buffer drain and isolation between captures.

use hudhudscript_hudunit::capture::capture_prints;

// ── capture ───────────────────────────────────────────────────────

#[test]
fn captures_printed_output() {
    let (value, output) = capture_prints(|| {
        hudhud_print::print_ops::print_line("hudunit-capture-test");
        42
    });
    assert_eq!(value, 42);
    assert!(output.contains("hudunit-capture-test"), "got: {:?}", output);
}

#[test]
fn sequential_captures_do_not_leak() {
    let (_, first) = capture_prints(|| {
        hudhud_print::print_ops::print_line("first");
    });
    let (_, second) = capture_prints(|| {
        hudhud_print::print_ops::print_line("second");
    });
    assert_eq!(first, "first\n");
    assert_eq!(second, "second\n");
}
