//! TargetSpec — full-triple target description and host detection
//! (JIT_AOT_ARCHITECTURE.md §8).
//!
//! JIT/AOT eligibility keys off `pointer_width` (the PROCESS width, not
//! the CPU): a 64-bit CPU running a 32-bit OS/process has no native tier.

use std::fmt;
use std::str::FromStr;

use crate::arch::{Abi, Architecture, Endianness, OperatingSystem};

/// CPU tuning selection. Device names are forbidden by design — only
/// generic baselines, host detection, or real CPU names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuSpec {
    Generic,
    Native,
    Named(String),
}

impl fmt::Display for CpuSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuSpec::Generic => f.write_str("generic"),
            CpuSpec::Native => f.write_str("native"),
            CpuSpec::Named(n) => write!(f, "{n}"),
        }
    }
}

/// A fully resolved target. `triple` stays the canonical identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetSpec {
    pub triple: String,
    pub arch: Architecture,
    pub os: OperatingSystem,
    pub abi: Abi,
    pub pointer_width: u8,
    pub endian: Endianness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTargetError {
    pub triple: String,
    pub reason: String,
}

impl fmt::Display for ParseTargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown target triple `{}`: {}", self.triple, self.reason)
    }
}

impl std::error::Error for ParseTargetError {}

/// Parse a Rust-style target triple: `arch[-vendor]-os[-abi]`.
pub fn parse_triple(triple: &str) -> Result<TargetSpec, ParseTargetError> {
    let err = |reason: &str| ParseTargetError { triple: triple.to_string(), reason: reason.into() };
    let parts: Vec<&str> = triple.split('-').collect();
    if parts.len() < 2 {
        return Err(err("expected arch[-vendor]-os[-abi]"));
    }

    let arch = match parts[0] {
        "x86_64" | "amd64" => Architecture::X86_64,
        "i686" | "i386" | "i586" => Architecture::X86,
        "arm" | "armv6" | "armv7" => Architecture::Arm,
        "aarch64" | "arm64" => Architecture::Aarch64,
        "riscv64" => Architecture::Riscv64,
        other => return Err(err(&format!("unsupported architecture `{other}`"))),
    };

    // Find OS part (skip vendor when 4 parts) and ABI suffix.
    let (os_str, abi) = if parts.len() >= 3 {
        (parts[1], parts[2]) // 3-part triple: arch-os-abi
    } else {
        (parts[0], parts[1]) // 2-part fallback handled below
    };
    let _ = os_str;

    let joined = parts[1..].join("-");
    let (os, abi) = if joined.contains("linux-android") {
        (OperatingSystem::Android, Abi::Android)
    } else if joined.contains("linux-musl") {
        (OperatingSystem::Linux, Abi::Musl)
    } else if joined.contains("linux-gnueabihf") {
        (OperatingSystem::Linux, Abi::Eabihf)
    } else if joined.contains("linux-gnueabi") {
        (OperatingSystem::Linux, Abi::Eabi)
    } else if joined.contains("linux-gnu") || joined.contains("unknown-linux") || joined.contains("linux") {
        (OperatingSystem::Linux, Abi::Gnu)
    } else if joined.contains("windows-msvc") {
        (OperatingSystem::Windows, Abi::Msvc)
    } else if joined.contains("windows-gnu") {
        (OperatingSystem::Windows, Abi::Gnu)
    } else if joined.contains("darwin") || joined.contains("macos") {
        (OperatingSystem::Macos, Abi::Darwin)
    } else if joined.contains("ios") {
        (OperatingSystem::Ios, Abi::Darwin)
    } else {
        return Err(err("unrecognized OS/ABI"));
    };

    let pointer_width = arch.pointer_width();
    let endian = match arch {
        Architecture::Arm => Endianness::Little, // Hudhud hedefleri: hepsi LE
        _ => Endianness::Little,
    };

    Ok(TargetSpec {
        triple: triple.to_string(),
        arch,
        os,
        abi,
        pointer_width,
        endian,
    })
}

impl FromStr for TargetSpec {
    type Err = ParseTargetError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_triple(s)
    }
}

impl TargetSpec {
    /// The host this process runs on (JIT always targets this).
    pub fn host() -> TargetSpec {
        let host_ptr_bytes = std::mem::size_of::<usize>() as u8;
        let arch = match (std::env::consts::ARCH, host_ptr_bytes) {
            ("x86_64", _) => Architecture::X86_64,
            ("aarch64", _) => Architecture::Aarch64,
            // 64-bit CPU with a 32-bit process: keep the process truth.
            ("x86", 4) | ("i686", 4) | ("i386", 4) => Architecture::X86,
            ("arm", 4) => Architecture::Arm,
            ("riscv64", _) => Architecture::Riscv64,
            (other, w) => panic!("unsupported host arch {other} (ptr={w})"),
        };
        let os = match std::env::consts::OS {
            "linux" => OperatingSystem::Linux,
            "windows" => OperatingSystem::Windows,
            "macos" => OperatingSystem::Macos,
            "android" => OperatingSystem::Android,
            other => panic!("unsupported host OS {other}"),
        };
        let abi = match os {
            OperatingSystem::Windows => Abi::Msvc,
            OperatingSystem::Macos => Abi::Darwin,
            OperatingSystem::Android => Abi::Android,
            OperatingSystem::Linux => {
                if cfg!(target_env = "musl") {
                    Abi::Musl
                } else {
                    Abi::Gnu
                }
            }
            OperatingSystem::Ios => Abi::Darwin,
        };
        let triple = format!("{arch}-unknown-{os}-{abi}");
        TargetSpec {
            triple,
            arch,
            os,
            abi,
            pointer_width: arch.pointer_width(),
            endian: Endianness::Little,
        }
    }

    /// Native tier (JIT/AOT) availability for this target on THIS build.
    /// Cranelift has no 32-bit ISA — 32-bit processes stay on the VM.
    pub fn supports_cranelift(&self) -> bool {
        self.pointer_width == 8
            && matches!(
                self.arch,
                Architecture::X86_64 | Architecture::Aarch64 | Architecture::Riscv64
            )
    }
}

impl fmt::Display for TargetSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.triple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn host_matches_build_process() {
        let h = TargetSpec::host();
        // Bu derlemenin process genişliğiyle birebir olmalı:
        assert_eq!(h.pointer_width, std::mem::size_of::<usize>() as u8);
        assert_eq!(h.triple.contains("unknown"), true);
    }

    #[test]
    fn cpu_spec_display() {
        assert_eq!(CpuSpec::Generic.to_string(), "generic");
        assert_eq!(CpuSpec::Native.to_string(), "native");
        assert_eq!(CpuSpec::Named("cortex-a8".into()).to_string(), "cortex-a8");
    }
}
