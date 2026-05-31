//! Runtime + bytecode bundle for single-file distribution (Issue #605)
//!
//! A bundle is a simple archive consisting of:
//! 1. A 4-byte big-endian length prefix for the manifest JSON
//! 2. The manifest JSON bytes
//! 3. The raw bytecode bytes (the remainder of the file)

use crate::error::{compile_codes, CompileResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata describing a bundled application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Human-readable application name.
    pub name: String,
    /// Semantic version string (e.g. "1.0.0").
    pub version: String,
    /// Entry-point module or file name.
    pub entry_point: String,
    /// List of dependency identifiers (for informational purposes).
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Create a bundle file from a manifest and a pre-compiled `.hudc` bytecode file.
///
/// The bundle is written to `output_path` and contains the manifest JSON
/// followed by the raw bytecode bytes.
pub fn create_bundle(
    manifest: &BundleManifest,
    bytecode_path: &Path,
    output_path: &Path,
) -> CompileResult<()> {
    let bytecode_data = std::fs::read(bytecode_path)
        .map_err(|e| compile_codes::runtime_error(format!("Failed to read bytecode file: {e}")))?;

    let manifest_json = serde_json::to_vec(manifest)
        .map_err(|e| compile_codes::runtime_error(format!("Failed to serialize manifest: {e}")))?;

    let manifest_len = manifest_json.len() as u32;

    let mut out = Vec::with_capacity(4 + manifest_json.len() + bytecode_data.len());
    out.extend_from_slice(&manifest_len.to_be_bytes());
    out.extend_from_slice(&manifest_json);
    out.extend_from_slice(&bytecode_data);

    std::fs::write(output_path, out)
        .map_err(|e| compile_codes::runtime_error(format!("Failed to write bundle file: {e}")))
}

/// Load a bundle file, returning the manifest and raw bytecode bytes.
pub fn load_bundle(path: &Path) -> CompileResult<(BundleManifest, Vec<u8>)> {
    let data = std::fs::read(path)
        .map_err(|e| compile_codes::runtime_error(format!("Failed to read bundle file: {e}")))?;

    if data.len() < 4 {
        return Err(compile_codes::invalid_bytecode("Bundle file too small"));
    }

    let manifest_len = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;

    if data.len() < 4 + manifest_len {
        return Err(compile_codes::invalid_bytecode(
            "Bundle file truncated (manifest exceeds file size)",
        ));
    }

    let manifest: BundleManifest = serde_json::from_slice(&data[4..4 + manifest_len])
        .map_err(|e| compile_codes::invalid_bytecode(format!("Invalid bundle manifest: {e}")))?;

    let bytecode = data[4 + manifest_len..].to_vec();

    Ok((manifest, bytecode))
}

// Tests moved to hudhud-script-tests/tests/compiler_test_inline.rs
