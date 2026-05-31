//! WASM compatibility notes.
//!
//! The VM core (instruction dispatch, stack, scopes) is platform-independent.
//! The following features are NOT available on wasm32:
//! - File system operations (std::fs)
//! - Network operations (std::net)
//! - Process operations (std::process)
//! - Unix-specific APIs (std::os::unix)
//!
//! These are in external package crates (hudhud-fs, hudhud-net, hudhud-exec)
//! which are NOT compiled for wasm32 targets.
