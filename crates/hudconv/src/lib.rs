//! hudconv — C++/CMake/Conan to HudHudScript converter library.
//!
//! This crate is also a binary tool; the lib target exposes all parsing
//! and generation modules so that the external test suite can import them.

pub mod cmake_parser;
pub mod conan_parser;
pub mod header_parser;
pub mod hud_generator;
pub mod manifest_generator;
