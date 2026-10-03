//! Tests for hudhudscript-target arch primitives — pointer widths per
//! architecture and Display names for arch/OS/ABI.

use hudhudscript_target::{Abi, Architecture, OperatingSystem};

// ── pointer widths ───────────────────────────────────────────────

#[test]
fn pointer_widths() {
    assert_eq!(Architecture::X86_64.pointer_width(), 8);
    assert_eq!(Architecture::Aarch64.pointer_width(), 8);
    assert_eq!(Architecture::Riscv64.pointer_width(), 8);
    assert_eq!(Architecture::X86.pointer_width(), 4);
    assert_eq!(Architecture::Arm.pointer_width(), 4);
}

// ── display names ────────────────────────────────────────────────

#[test]
fn display_names() {
    assert_eq!(Architecture::Aarch64.to_string(), "aarch64");
    assert_eq!(OperatingSystem::Android.to_string(), "android");
    assert_eq!(Abi::Eabihf.to_string(), "eabihf");
    assert_eq!(Abi::Msvc.to_string(), "msvc");
}
