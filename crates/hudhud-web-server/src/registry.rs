//! Connection and listener registries for hudhud-web-server.
//!
//! Global registries (per-process, per-worker) for TCP listeners
//! and active connections.

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::Mutex;

/// Global listener registry: listener_id → TcpListener.
pub(crate) fn listener_registry() -> &'static Mutex<HashMap<u64, std::net::TcpListener>> {
    use std::sync::OnceLock;
    static REG: OnceLock<Mutex<HashMap<u64, std::net::TcpListener>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Global connection registry: conn_id → TcpStream.
pub(crate) fn conn_registry() -> &'static Mutex<HashMap<u64, TcpStream>> {
    use std::sync::OnceLock;
    static REG: OnceLock<Mutex<HashMap<u64, TcpStream>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}
