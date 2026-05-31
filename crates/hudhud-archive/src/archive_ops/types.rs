//! Archive operation identifiers and zero-cost dispatch.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, HudHudResult};

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    CreateTarGz,
    ExtractTarGz,
    CreateZip,
    ExtractZip,
    List,
    Compress,
    Decompress,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create_tar_gz" => Ok(Self::CreateTarGz),
            "extract_tar_gz" => Ok(Self::ExtractTarGz),
            "create_zip" => Ok(Self::CreateZip),
            "extract_zip" => Ok(Self::ExtractZip),
            "list" => Ok(Self::List),
            "compress" => Ok(Self::Compress),
            "decompress" => Ok(Self::Decompress),
            _ => Err(super::utils::runtime_error(format!(
                "Unknown method: {}",
                s
            ))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::CreateTarGz => super::tar::create_tar_gz(args),
        ScriptMethodId::ExtractTarGz => super::tar::extract_tar_gz(args),
        ScriptMethodId::CreateZip => super::zip::create_zip(args),
        ScriptMethodId::ExtractZip => super::zip::extract_zip(args),
        ScriptMethodId::List => super::list::list_archive(args),
        ScriptMethodId::Compress => super::compress::compress(args),
        ScriptMethodId::Decompress => super::compress::decompress(args),
    }
}
