//! VM stdlib module registration.
//!
//! Single source of truth (Kural 7) for registering the shared-builtin
//! namespaced modules (`Math`, `Stats`, `EventBus`, `Plugin`, …) on a VM
//! instance.  Both the CLI (`hudi run`, bytecode runner) and the external
//! test harness (`hudhud-script-tests`) call this function so that a script
//! sees identical stdlib surface regardless of entry point.
//!
//! Historical note: this used to live in `hudhudscript-cli/src/common.rs`
//! (a bin crate) which meant external callers could not `pub use` it.
//! Moved here during the interpreter-crate retirement migration so the test
//! suite can import it without pulling in the CLI's heavy dep tree.

use crate::vm::VM;
mod dbus_ops;
mod e2e_ops;
mod mpris_ops;
mod platform;
mod torrent_ops;
mod tts_ops;

mod fs;
mod log;
mod math;
mod net;
mod security;
mod serial;
mod sys;
mod tui;
mod web;

/// Register all stdlib modules from shared-builtins on the given VM
/// (#928). Shared between `run_bytecode_with_config`, `run_file_vm_with_config`,
/// and the external test harness so that bytecode files, `hudi run`, and
/// tests all get the same module surface.
pub fn register_vm_stdlib_modules(vm: &mut VM) {
    serial::register(vm);
    net::register(vm);
    fs::register(vm);
    sys::register(vm);
    math::register(vm);
    web::register(vm);
    security::register(vm);
    platform::register_platform_modules(vm);
    log::register(vm);
    tui::register(vm);
}
