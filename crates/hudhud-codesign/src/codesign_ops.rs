//! Shared plugin code-signing (GPG sign/verify, file hashing, manifest)
//! — single source of truth for VM + interpreter (Kural 7).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Sign,
    Verify,
    HashFile,
    GenerateManifest,
    VerifyManifest,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sign" => Ok(Self::Sign),
            "verify" => Ok(Self::Verify),
            "hash_file" => Ok(Self::HashFile),
            "generate_manifest" => Ok(Self::GenerateManifest),
            "verify_manifest" => Ok(Self::VerifyManifest),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Sign => codesign_sign(args),
        ScriptMethodId::Verify => codesign_verify(args),
        ScriptMethodId::HashFile => codesign_hash_file(args),
        ScriptMethodId::GenerateManifest => codesign_generate_manifest(args),
        ScriptMethodId::VerifyManifest => codesign_verify_manifest(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn codesign_sign(args: &[Value16]) -> HudHudResult<Value16> {
    let file_path = require_str(args, 0, "codesign.sign")?.to_string();
    let key_path = require_str(args, 1, "codesign.sign")?.to_string();

    if !std::path::Path::new(&file_path).exists() {
        return Err(runtime_error(format!(
            "codesign.sign: file not found: {}",
            file_path
        )));
    }

    let signature_path = format!("{}.sig", file_path);
    let output = std::process::Command::new("gpg")
        .args([
            "--batch",
            "--yes",
            "--detach-sign",
            "--armor",
            "--default-key",
            &key_path,
            "--output",
            &signature_path,
            &file_path,
        ])
        .output()
        .map_err(|e| runtime_error(format!("codesign.sign: failed to execute gpg: {}", e)))?;

    let mut result = HashMap::new();
    if output.status.success() {
        result.insert(
            "signature_path".to_string(),
            Value16::string(signature_path),
        );
        result.insert("ok".to_string(), Value16::bool_(true));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        result.insert("signature_path".to_string(), Value16::string(String::new()));
        result.insert("ok".to_string(), Value16::bool_(false));
        result.insert("error".to_string(), Value16::string(stderr));
    }
    Ok(Value16::object(result))
}

pub fn codesign_verify(args: &[Value16]) -> HudHudResult<Value16> {
    let file_path = require_str(args, 0, "codesign.verify")?.to_string();
    let signature_path = require_str(args, 1, "codesign.verify")?.to_string();
    let key_path = require_str(args, 2, "codesign.verify")?.to_string();

    if !std::path::Path::new(&file_path).exists() {
        return Err(runtime_error(format!(
            "codesign.verify: file not found: {}",
            file_path
        )));
    }
    if !std::path::Path::new(&signature_path).exists() {
        return Err(runtime_error(format!(
            "codesign.verify: signature file not found: {}",
            signature_path
        )));
    }

    let output = std::process::Command::new("gpg")
        .args([
            "--batch",
            "--yes",
            "--keyring",
            &key_path,
            "--verify",
            &signature_path,
            &file_path,
        ])
        .output()
        .map_err(|e| runtime_error(format!("codesign.verify: failed to execute gpg: {}", e)))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let valid = output.status.success();
    let signer = stderr
        .lines()
        .find(|line| line.contains("Good signature from"))
        .map(|line| line.to_string())
        .unwrap_or_default();

    let mut result = HashMap::new();
    result.insert("valid".to_string(), Value16::bool_(valid));
    result.insert("signer".to_string(), Value16::string(signer));
    result.insert("message".to_string(), Value16::string(stderr));
    Ok(Value16::object(result))
}

pub fn codesign_hash_file(args: &[Value16]) -> HudHudResult<Value16> {
    let file_path = require_str(args, 0, "codesign.hash_file")?.to_string();
    let algorithm = args
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("sha256")
        .to_string();

    let data = std::fs::read(&file_path).map_err(|e| {
        runtime_error(format!(
            "codesign.hash_file: cannot read '{}': {}",
            file_path, e
        ))
    })?;

    let hex_result = match algorithm.to_lowercase().as_str() {
        "sha256" | "sha-256" => {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        }
        "sha512" | "sha-512" => {
            let mut hasher = Sha512::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        }
        other => {
            return Err(runtime_error(format!(
                "codesign.hash_file: unsupported algorithm '{}'. Supported: sha256, sha512",
                other
            )));
        }
    };
    Ok(Value16::string(hex_result))
}

pub fn codesign_generate_manifest(args: &[Value16]) -> HudHudResult<Value16> {
    let dir_path = require_str(args, 0, "codesign.generate_manifest")?.to_string();
    let path = std::path::Path::new(&dir_path);
    if !path.is_dir() {
        return Err(runtime_error(format!(
            "codesign.generate_manifest: not a directory: {}",
            dir_path
        )));
    }
    let mut manifest: HashMap<String, Value16> = HashMap::new();
    collect_file_hashes(path, path, &mut manifest)?;
    Ok(Value16::object(manifest))
}

fn collect_file_hashes(
    base: &std::path::Path,
    dir: &std::path::Path,
    manifest: &mut HashMap<String, Value16>,
) -> HudHudResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        runtime_error(format!(
            "codesign.generate_manifest: cannot read directory '{}': {}",
            dir.display(),
            e
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            runtime_error(format!(
                "codesign.generate_manifest: directory entry error: {}",
                e
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_file_hashes(base, &path, manifest)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let data = std::fs::read(&path).map_err(|e| {
                runtime_error(format!(
                    "codesign.generate_manifest: cannot read '{}': {}",
                    path.display(),
                    e
                ))
            })?;
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hex::encode(hasher.finalize());
            manifest.insert(relative, Value16::string(hash));
        }
    }
    Ok(())
}

pub fn codesign_verify_manifest(args: &[Value16]) -> HudHudResult<Value16> {
    let dir_path = require_str(args, 0, "codesign.verify_manifest")?.to_string();
    let manifest = args.get(1).and_then(|v| v.as_object()).ok_or_else(|| {
        args.get(1).map_or_else(
            || runtime_error("codesign.verify_manifest: missing manifest argument"),
            |v| type_error("object", v.type_name_str(), "codesign.verify_manifest"),
        )
    })?;

    let base = std::path::Path::new(&dir_path);
    if !base.is_dir() {
        return Err(runtime_error(format!(
            "codesign.verify_manifest: not a directory: {}",
            dir_path
        )));
    }

    let mut valid = true;
    let mut mismatches: Vec<Value16> = Vec::new();
    let mut missing: Vec<Value16> = Vec::new();

    for (rel_path, expected_hash) in manifest {
        let expected = match expected_hash.as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };

        let full_path = base.join(rel_path);
        if !full_path.exists() {
            valid = false;
            missing.push(Value16::string(rel_path.clone()));
            continue;
        }

        let data = std::fs::read(&full_path).map_err(|e| {
            runtime_error(format!(
                "codesign.verify_manifest: cannot read '{}': {}",
                full_path.display(),
                e
            ))
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let actual = hex::encode(hasher.finalize());

        if actual != expected {
            valid = false;
            mismatches.push(Value16::string(rel_path.clone()));
        }
    }

    let mut result = HashMap::new();
    result.insert("valid".to_string(), Value16::bool_(valid));
    result.insert("mismatches".to_string(), Value16::array(mismatches));
    result.insert("missing".to_string(), Value16::array(missing));
    Ok(Value16::object(result))
}

fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            method, idx
        ))),
    }
}
