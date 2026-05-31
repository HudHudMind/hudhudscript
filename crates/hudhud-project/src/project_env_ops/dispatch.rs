use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, HudHudResult};

use super::helpers::runtime_error;

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Detect,
    DetectVenv,
    ParseEnvFile,
    ToolchainVersion,
    Dependencies,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "detect" => Ok(Self::Detect),
            "detect_venv" => Ok(Self::DetectVenv),
            "parse_env_file" => Ok(Self::ParseEnvFile),
            "toolchain_version" => Ok(Self::ToolchainVersion),
            "dependencies" => Ok(Self::Dependencies),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Detect => super::env_core::detect(args),
        ScriptMethodId::DetectVenv => super::env_core::detect_venv(args),
        ScriptMethodId::ParseEnvFile => super::config_ops::parse_env_file(args),
        ScriptMethodId::ToolchainVersion => super::toolchain::toolchain_version(args),
        ScriptMethodId::Dependencies => super::dependencies::dependencies(args),
    }
}
