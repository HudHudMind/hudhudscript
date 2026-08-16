//! Shared filesystem operations builtin — used by both VM and interpreter.
//!
//! Provides: fs.chmod, fs.chown, fs.symlink, fs.readlink, fs.stat, fs.mkdir_p,
//!           fs.copy, fs.rename, fs.realpath, fs.watch

use crate::patch;
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
use std::collections::HashMap;

/// Execute an fs method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "chmod" => {
            let path = require_str(args, 0, "fs.chmod")?;
            let mode = match args.get(1) {
                Some(v) if v.as_number().is_some() => v.as_number().unwrap() as u32,
                Some(v) if v.as_str().is_some() => {
                    let s = v.as_str().unwrap();
                    u32::from_str_radix(s, 8).map_err(|_| {
                        runtime_error(format!("fs.chmod: invalid octal mode '{}'", s))
                    })?
                }
                _ => return Err(runtime_error("fs.chmod: expected mode".to_string())),
            };
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(mode);
                std::fs::set_permissions(&path, perms)
                    .map_err(|e| runtime_error(format!("fs.chmod error: {}", e)))?;
                Ok(Value16::null())
            }
            #[cfg(not(unix))]
            {
                let _ = (path, mode);
                Err(runtime_error(
                    "fs.chmod not supported on this platform".to_string(),
                ))
            }
        }
        "chown" => {
            let path = require_str(args, 0, "fs.chown")?;
            let uid = match args.get(1).and_then(|v| v.as_number()) {
                Some(n) => n as u32,
                None => return Err(runtime_error("fs.chown: expected uid number".to_string())),
            };
            let gid = match args.get(2).and_then(|v| v.as_number()) {
                Some(n) => n as u32,
                None => return Err(runtime_error("fs.chown: expected gid number".to_string())),
            };
            #[cfg(unix)]
            {
                let c_path = std::ffi::CString::new(path.as_str())
                    .map_err(|e| runtime_error(format!("fs.chown: invalid path: {}", e)))?;
                let result = unsafe { libc::chown(c_path.as_ptr(), uid, gid) };
                if result != 0 {
                    return Err(runtime_error(format!(
                        "fs.chown error: {}",
                        std::io::Error::last_os_error()
                    )));
                }
                Ok(Value16::null())
            }
            #[cfg(not(unix))]
            {
                let _ = (path, uid, gid);
                Err(runtime_error(
                    "fs.chown not supported on this platform".to_string(),
                ))
            }
        }
        "symlink" => {
            let target = require_str(args, 0, "fs.symlink target")?;
            let link_path = require_str(args, 1, "fs.symlink link_path")?;
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&target, &link_path)
                    .map_err(|e| runtime_error(format!("fs.symlink error: {}", e)))?;
            }
            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_file(&target, &link_path)
                    .map_err(|e| runtime_error(format!("fs.symlink error: {}", e)))?;
            }
            #[cfg(not(any(unix, windows)))]
            {
                let _ = (&target, &link_path);
                return Err(runtime_error(
                    "fs.symlink not supported on this platform".to_string(),
                ));
            }
            Ok(Value16::null())
        }
        "readlink" => {
            let path = require_str(args, 0, "fs.readlink")?;
            let target = std::fs::read_link(&path)
                .map_err(|e| runtime_error(format!("fs.readlink error: {}", e)))?;
            Ok(Value16::string(target.to_string_lossy().to_string()))
        }
        "stat" => {
            let path = require_str(args, 0, "fs.stat")?;
            let meta = std::fs::metadata(&path)
                .map_err(|e| runtime_error(format!("fs.stat error: {}", e)))?;
            let mut result = hudhudscript_bytecode::ObjMap::default();
            result.insert("size".to_string(), Value16::number(meta.len() as f64));
            result.insert("is_file".to_string(), Value16::bool_(meta.is_file()));
            result.insert("is_dir".to_string(), Value16::bool_(meta.is_dir()));
            result.insert(
                "is_symlink".to_string(),
                Value16::bool_(meta.file_type().is_symlink()),
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                result.insert("mode".to_string(), Value16::number(meta.mode() as f64));
                result.insert("uid".to_string(), Value16::number(meta.uid() as f64));
                result.insert("gid".to_string(), Value16::number(meta.gid() as f64));
            }
            #[cfg(not(unix))]
            {
                result.insert("mode".to_string(), Value16::number(0.0));
                result.insert("uid".to_string(), Value16::number(0.0));
                result.insert("gid".to_string(), Value16::number(0.0));
            }
            if let Ok(modified) = meta.modified() {
                if let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH) {
                    result.insert("modified".to_string(), Value16::number(dur.as_secs_f64()));
                }
            }
            if let Ok(created) = meta.created() {
                if let Ok(dur) = created.duration_since(std::time::UNIX_EPOCH) {
                    result.insert("created".to_string(), Value16::number(dur.as_secs_f64()));
                }
            }
            Ok(Value16::object(result))
        }
        "mkdir_p" => {
            let path = require_str(args, 0, "fs.mkdir_p")?;
            std::fs::create_dir_all(&path)
                .map_err(|e| runtime_error(format!("fs.mkdir_p error: {}", e)))?;
            Ok(Value16::null())
        }
        "copy" => {
            let src = require_str(args, 0, "fs.copy src")?;
            let dst = require_str(args, 1, "fs.copy dst")?;
            let bytes = std::fs::copy(&src, &dst)
                .map_err(|e| runtime_error(format!("fs.copy error: {}", e)))?;
            Ok(Value16::number(bytes as f64))
        }
        "rename" => {
            let src = require_str(args, 0, "fs.rename src")?;
            let dst = require_str(args, 1, "fs.rename dst")?;
            std::fs::rename(&src, &dst)
                .map_err(|e| runtime_error(format!("fs.rename error: {}", e)))?;
            Ok(Value16::null())
        }
        "realpath" => {
            let path = require_str(args, 0, "fs.realpath")?;
            let canonical = std::fs::canonicalize(&path)
                .map_err(|e| runtime_error(format!("fs.realpath error: {}", e)))?;
            Ok(Value16::string(canonical.to_string_lossy().to_string()))
        }
        "watch" => {
            let path = require_str(args, 0, "fs.watch")?;
            let mut result = hudhudscript_bytecode::ObjMap::default();
            result.insert("path".to_string(), Value16::string(path.clone()));
            result.insert(
                "exists".to_string(),
                Value16::bool_(std::path::Path::new(&path).exists()),
            );
            if let Ok(meta) = std::fs::metadata(&path) {
                result.insert("size".to_string(), Value16::number(meta.len() as f64));
                if let Ok(modified) = meta.modified() {
                    if let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH) {
                        result.insert("modified".to_string(), Value16::number(dur.as_secs_f64()));
                    }
                }
            }
            Ok(Value16::object(result))
        }
        "patch.apply" | "patch_apply" => {
            let file_path = require_str(args, 0, "patch.apply")?;
            let search = require_str(args, 1, "patch.apply")?;
            let replace = require_str(args, 2, "patch.apply")?;
            let r =
                patch::patch_apply(&file_path, &search, &replace).map_err(|e| runtime_error(e))?;
            let mut obj = hudhudscript_bytecode::ObjMap::default();
            obj.insert(
                "replacements".to_string(),
                Value16::int(r.replacements as i64),
            );
            obj.insert("modified".to_string(), Value16::bool_(r.modified));
            Ok(Value16::object(obj))
        }
        "read" | "write" | "remove" | "exists" | "copy" | "list" => {
            crate::file_ops::dispatch(method, args)
        }
        _ => Err(runtime_error(format!("Unknown fs method: {}", method))),
    }
}

fn require_str(args: &[Value16], idx: usize, context: &str) -> HudHudResult<String> {
    match args.get(idx).and_then(|v| v.as_str()) {
        Some(s) => Ok(s.to_string()),
        None => Err(runtime_error(format!("{}: expected string", context))),
    }
}
