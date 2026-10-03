//! Tests for hudhudscript-target `TargetSpec` — triple parsing, host
//! detection, native-tier eligibility, and CpuSpec display.

use hudhudscript_target::{parse_triple, Abi, Architecture, CpuSpec, OperatingSystem, TargetSpec};

// ── triple parsing ───────────────────────────────────────────────

#[test]
fn parses_documented_triples() {
    let t = parse_triple("x86_64-unknown-linux-gnu").unwrap();
    assert_eq!(t.arch, Architecture::X86_64);
    assert_eq!(t.os, OperatingSystem::Linux);
    assert_eq!(t.abi, Abi::Gnu);
    assert_eq!(t.pointer_width, 8);

    let t = parse_triple("x86_64-pc-windows-msvc").unwrap();
    assert_eq!(t.os, OperatingSystem::Windows);
    assert_eq!(t.abi, Abi::Msvc);

    let t = parse_triple("aarch64-unknown-linux-gnu").unwrap();
    assert_eq!(t.arch, Architecture::Aarch64);
    assert_eq!(t.pointer_width, 8);
    assert!(t.supports_cranelift());

    let t = parse_triple("aarch64-linux-android").unwrap();
    assert_eq!(t.os, OperatingSystem::Android);
    assert_eq!(t.abi, Abi::Android);

    let t = parse_triple("armv7-unknown-linux-gnueabihf").unwrap();
    assert_eq!(t.arch, Architecture::Arm);
    assert_eq!(t.abi, Abi::Eabihf);
    assert_eq!(t.pointer_width, 4);
    assert!(!t.supports_cranelift()); // §8.2: 32-bit = native yok

    let t = parse_triple("i686-unknown-linux-gnu").unwrap();
    assert_eq!(t.arch, Architecture::X86);
    assert_eq!(t.pointer_width, 4);
    assert!(!t.supports_cranelift());

    let t = parse_triple("riscv64-unknown-linux-gnu").unwrap();
    assert!(t.supports_cranelift());
}

#[test]
fn rejects_garbage() {
    assert!(parse_triple("sparc-sun-solaris").is_err());
    assert!(parse_triple("x86_64").is_err());
    let e = parse_triple("mips-unknown-linux-gnu").unwrap_err();
    assert!(e.to_string().contains("unsupported architecture"));
}

// ── host detection ───────────────────────────────────────────────

#[test]
fn host_matches_build_process() {
    let h = TargetSpec::host();
    // Bu derlemenin process genişliğiyle birebir olmalı:
    assert_eq!(h.pointer_width, std::mem::size_of::<usize>() as u8);
    assert_eq!(h.triple.contains("unknown"), true);
}

// ── CPU spec display ─────────────────────────────────────────────

#[test]
fn cpu_spec_display() {
    assert_eq!(CpuSpec::Generic.to_string(), "generic");
    assert_eq!(CpuSpec::Native.to_string(), "native");
    assert_eq!(CpuSpec::Named("cortex-a8".into()).to_string(), "cortex-a8");
}
