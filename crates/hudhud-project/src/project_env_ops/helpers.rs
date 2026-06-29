use std::collections::HashMap;
use std::path::Path;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

pub(crate) fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub(crate) fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

pub(crate) fn require_string(args: &[Value16], idx: usize, op: &str) -> HudHudResult<String> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| type_error("string", v.type_name_str(), op)),
        None => Err(type_error("string", "missing", op)),
    }
}

pub(crate) fn read_file(path: &Path, op: &str) -> HudHudResult<String> {
    std::fs::read_to_string(path)
        .map_err(|e| runtime_error(format!("{}: cannot read '{}': {}", op, path.display(), e)))
}

pub(crate) struct Marker {
    pub(crate) file: &'static str,
    pub(crate) language: &'static str,
    pub(crate) toolchain: &'static str,
    pub(crate) package_manager: &'static str,
}

pub(crate) const MARKERS: &[Marker] = &[
    Marker {
        file: "package.json",
        language: "JavaScript",
        toolchain: "Node.js",
        package_manager: "npm",
    },
    Marker {
        file: "yarn.lock",
        language: "JavaScript",
        toolchain: "Node.js",
        package_manager: "yarn",
    },
    Marker {
        file: "pnpm-lock.yaml",
        language: "JavaScript",
        toolchain: "Node.js",
        package_manager: "pnpm",
    },
    Marker {
        file: "requirements.txt",
        language: "Python",
        toolchain: "CPython",
        package_manager: "pip",
    },
    Marker {
        file: "pyproject.toml",
        language: "Python",
        toolchain: "CPython",
        package_manager: "pip",
    },
    Marker {
        file: "Pipfile",
        language: "Python",
        toolchain: "CPython",
        package_manager: "pipenv",
    },
    Marker {
        file: "Cargo.toml",
        language: "Rust",
        toolchain: "rustc",
        package_manager: "cargo",
    },
    Marker {
        file: "go.mod",
        language: "Go",
        toolchain: "go",
        package_manager: "go modules",
    },
    Marker {
        file: "CMakeLists.txt",
        language: "C/C++",
        toolchain: "cmake",
        package_manager: "cmake",
    },
    Marker {
        file: "Makefile",
        language: "C/C++",
        toolchain: "make",
        package_manager: "make",
    },
    Marker {
        file: "Gemfile",
        language: "Ruby",
        toolchain: "ruby",
        package_manager: "bundler",
    },
];

pub(crate) fn make_dep(name: &str, version: &str) -> Value16 {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("name".to_string(), Value16::string(name.to_string()));
    obj.insert("version".to_string(), Value16::string(version.to_string()));
    Value16::object(obj)
}
