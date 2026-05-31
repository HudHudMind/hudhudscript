//! Shared Transmission RPC client builtins (Issue #639).
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).
//! Talks to Transmission daemon JSON-RPC (with X-Transmission-Session-Id
//! CSRF handshake on 409).

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

mod client_ops;
mod download_ops;
mod helpers;
mod metadata_ops;

pub(crate) use client_ops::{build_client, check_rpc_result, rpc_call};
pub use download_ops::{torrent_add, torrent_pause, torrent_remove, torrent_resume};
pub(crate) use helpers::{
    default_rpc_url, ok_message, optional_string, require_i64, require_string, status_string,
};
pub use metadata_ops::{torrent_info, torrent_list};

pub fn call_torrent_method(method: &str, args: &[Value16]) -> SharedResult<Value16> {
    match method {
        "list" => metadata_ops::torrent_list(args),
        "add" => download_ops::torrent_add(args),
        "remove" => download_ops::torrent_remove(args),
        "pause" => download_ops::torrent_pause(args),
        "resume" => download_ops::torrent_resume(args),
        "info" => metadata_ops::torrent_info(args),
        _ => Err(runtime_error(format!("Unknown torrent method: {}", method))),
    }
}
