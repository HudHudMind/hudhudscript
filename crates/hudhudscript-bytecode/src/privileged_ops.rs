//! Opt-in gate for operations that escalate privileges (`sudo …`).
//!
//! `hudhud-firewall` (`sudo ufw …`) and `hudhud-apt` (`sudo apt-get …`) shell out
//! to `sudo` directly. Their `dispatch(method, args)` entry points take no VM and
//! no policy, so the VM's `HostAccessPolicy` could not protect them — and unit
//! tests call those entry points directly, which meant `cargo test` really did
//! run `sudo ufw reset --force` and `sudo apt-get install -y curl` on the
//! developer's machine, prompting for a password (or, with passwordless sudo,
//! silently reconfiguring the firewall and installing packages).
//!
//! The gate is **deny by default** and process-global, because that is the only
//! context available at those call sites. The runtime opts in once, after its own
//! policy check; nothing else may. This is not a fallback path (Kural 7c): when
//! the capability is absent the operation returns an error, it does not quietly
//! degrade to something else.
//!
//! Lives here rather than in each ops crate so the decision has one definition
//! (Kural 7) — `hudhud-firewall` and `hudhud-apt` both already depend on this
//! crate, and so does the VM that flips the switch.

use hudhudscript_errors::HudHudResult;
use std::sync::atomic::{AtomicBool, Ordering};

static PRIVILEGED_OPS_ALLOWED: AtomicBool = AtomicBool::new(false);

/// Grant privilege-escalating operations for this process.
///
/// Called by the runtime once its `HostAccessPolicy` has allowed the module.
/// Never call this from a library, a test helper, or a leaf op.
pub fn allow_privileged_ops() {
    PRIVILEGED_OPS_ALLOWED.store(true, Ordering::Relaxed);
}

/// Revoke the grant. Exists so an embedder can drop the capability again.
pub fn deny_privileged_ops() {
    PRIVILEGED_OPS_ALLOWED.store(false, Ordering::Relaxed);
}

pub fn privileged_ops_allowed() -> bool {
    PRIVILEGED_OPS_ALLOWED.load(Ordering::Relaxed)
}

/// Guard for every call site that is about to run `sudo`.
///
/// `op` names the operation in the error, e.g. `"firewall.enable"`.
pub fn ensure_privileged_ops_allowed(op: &str) -> HudHudResult<()> {
    if privileged_ops_allowed() {
        return Ok(());
    }
    Err(hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::RuntimeCustom,
        format!(
            "{op} needs privilege escalation (sudo) and it is not granted. \
             Enable it in the runtime's host-access policy; it is off by default \
             so that library calls and tests cannot change system state."
        ),
    ))
}
