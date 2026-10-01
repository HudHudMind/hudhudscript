//! Architecture / OS / ABI primitives (JIT_AOT_ARCHITECTURE.md §8).

use std::fmt;

/// CPU architecture family — deliberately excludes vendor/device names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Architecture {
    X86_64,
    X86,
    Arm,
    Aarch64,
    Riscv64,
}

impl Architecture {
    /// Pointer width in bytes for this architecture's default data model.
    pub fn pointer_width(self) -> u8 {
        match self {
            Architecture::X86_64 | Architecture::Aarch64 | Architecture::Riscv64 => 8,
            Architecture::X86 | Architecture::Arm => 4,
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Architecture::X86_64 => "x86_64",
            Architecture::X86 => "x86",
            Architecture::Arm => "arm",
            Architecture::Aarch64 => "aarch64",
            Architecture::Riscv64 => "riscv64",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatingSystem {
    Linux,
    Windows,
    Macos,
    Android,
    Ios,
}

impl fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OperatingSystem::Linux => "linux",
            OperatingSystem::Windows => "windows",
            OperatingSystem::Macos => "macos",
            OperatingSystem::Android => "android",
            OperatingSystem::Ios => "ios",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Abi {
    Gnu,
    Musl,
    Msvc,
    Android,
    Eabi,
    Eabihf,
    Darwin,
}

impl fmt::Display for Abi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Abi::Gnu => "gnu",
            Abi::Musl => "musl",
            Abi::Msvc => "msvc",
            Abi::Android => "android",
            Abi::Eabi => "eabi",
            Abi::Eabihf => "eabihf",
            Abi::Darwin => "darwin",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Endianness {
    Little,
    Big,
}
