//! Linux capability management (Issue #603)
//!
//! Provides types for declaring and manipulating POSIX / Linux capabilities
//! (e.g. `CAP_NET_ADMIN`, `CAP_SYS_ADMIN`). Actual capability manipulation
//! (via `prctl` / `capset`) is gated behind `#[cfg(target_os = "linux")]`.

use crate::Result;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Capability enum
// ---------------------------------------------------------------------------

/// A subset of the Linux capability constants.
///
/// Only the capabilities most relevant to sandboxing are listed; the full
/// set can be extended as needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// CAP_CHOWN — change file ownership.
    Chown,
    /// CAP_DAC_OVERRIDE — bypass file read/write/execute permission checks.
    DacOverride,
    /// CAP_DAC_READ_SEARCH — bypass file read and directory search checks.
    DacReadSearch,
    /// CAP_FOWNER — bypass permission checks on operations that require the
    /// file owner.
    Fowner,
    /// CAP_KILL — send signals to arbitrary processes.
    Kill,
    /// CAP_NET_ADMIN — various network-related operations.
    NetAdmin,
    /// CAP_NET_BIND_SERVICE — bind to ports < 1024.
    NetBindService,
    /// CAP_NET_RAW — use raw / packet sockets.
    NetRaw,
    /// CAP_SETUID — make arbitrary manipulations of process UIDs.
    Setuid,
    /// CAP_SETGID — make arbitrary manipulations of process GIDs.
    Setgid,
    /// CAP_SYS_ADMIN — a catch-all capability for various admin ops.
    SysAdmin,
    /// CAP_SYS_BOOT — use reboot(2).
    SysBoot,
    /// CAP_SYS_CHROOT — use chroot(2).
    SysChroot,
    /// CAP_SYS_MODULE — load / unload kernel modules.
    SysModule,
    /// CAP_SYS_PTRACE — trace arbitrary processes.
    SysPtrace,
    /// CAP_SYS_RAWIO — perform I/O port operations.
    SysRawio,
    /// CAP_SYS_RESOURCE — override various resource limits.
    SysResource,
    /// CAP_SYS_TIME — set system clock.
    SysTime,
    /// CAP_MKNOD — create special files.
    Mknod,
    /// CAP_AUDIT_WRITE — write to the kernel audit log.
    AuditWrite,
}

impl Capability {
    /// Return all known capabilities.
    pub fn all() -> &'static [Capability] {
        use Capability::*;
        &[
            Chown,
            DacOverride,
            DacReadSearch,
            Fowner,
            Kill,
            NetAdmin,
            NetBindService,
            NetRaw,
            Setuid,
            Setgid,
            SysAdmin,
            SysBoot,
            SysChroot,
            SysModule,
            SysPtrace,
            SysRawio,
            SysResource,
            SysTime,
            Mknod,
            AuditWrite,
        ]
    }

    /// Convert a Linux capability number to a Capability enum value.
    pub fn from_number(nr: u32) -> Option<Capability> {
        use Capability::*;
        match nr {
            0 => Some(Chown),
            1 => Some(DacOverride),
            2 => Some(DacReadSearch),
            3 => Some(Fowner),
            5 => Some(Kill),
            12 => Some(NetAdmin),
            10 => Some(NetBindService),
            13 => Some(NetRaw),
            7 => Some(Setuid),
            6 => Some(Setgid),
            21 => Some(SysAdmin),
            22 => Some(SysBoot),
            18 => Some(SysChroot),
            16 => Some(SysModule),
            19 => Some(SysPtrace),
            17 => Some(SysRawio),
            24 => Some(SysResource),
            25 => Some(SysTime),
            27 => Some(Mknod),
            29 => Some(AuditWrite),
            _ => None,
        }
    }

    /// Return a human-readable name matching the `CAP_*` constant.
    pub fn name(&self) -> &'static str {
        use Capability::*;
        match self {
            Chown => "CAP_CHOWN",
            DacOverride => "CAP_DAC_OVERRIDE",
            DacReadSearch => "CAP_DAC_READ_SEARCH",
            Fowner => "CAP_FOWNER",
            Kill => "CAP_KILL",
            NetAdmin => "CAP_NET_ADMIN",
            NetBindService => "CAP_NET_BIND_SERVICE",
            NetRaw => "CAP_NET_RAW",
            Setuid => "CAP_SETUID",
            Setgid => "CAP_SETGID",
            SysAdmin => "CAP_SYS_ADMIN",
            SysBoot => "CAP_SYS_BOOT",
            SysChroot => "CAP_SYS_CHROOT",
            SysModule => "CAP_SYS_MODULE",
            SysPtrace => "CAP_SYS_PTRACE",
            SysRawio => "CAP_SYS_RAWIO",
            SysResource => "CAP_SYS_RESOURCE",
            SysTime => "CAP_SYS_TIME",
            Mknod => "CAP_MKNOD",
            AuditWrite => "CAP_AUDIT_WRITE",
        }
    }
}

// ---------------------------------------------------------------------------
// CapabilitySet
// ---------------------------------------------------------------------------

/// A mutable set of capabilities that can be built up or pared down.
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    caps: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a set containing every known capability.
    pub fn full() -> Self {
        Self {
            caps: Capability::all().iter().copied().collect(),
        }
    }

    /// Add a capability to the set.
    pub fn add(&mut self, cap: Capability) -> &mut Self {
        self.caps.insert(cap);
        self
    }

    /// Remove a capability from the set.
    pub fn remove(&mut self, cap: Capability) -> &mut Self {
        self.caps.remove(&cap);
        self
    }

    /// Check whether the set contains `cap`.
    pub fn contains(&self, cap: Capability) -> bool {
        self.caps.contains(&cap)
    }

    /// Drop all capabilities (empty the set).
    pub fn drop_all(&mut self) -> &mut Self {
        self.caps.clear();
        self
    }

    /// Retain only the capabilities in `keep`, dropping everything else.
    pub fn retain_only(&mut self, keep: &HashSet<Capability>) -> &mut Self {
        self.caps.retain(|c| keep.contains(c));
        self
    }

    /// Return the number of capabilities in the set.
    pub fn len(&self) -> usize {
        self.caps.len()
    }

    /// Return whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.caps.is_empty()
    }

    /// Return the capabilities as a sorted vector (sorted by name for
    /// deterministic display).
    pub fn to_sorted_vec(&self) -> Vec<Capability> {
        let mut v: Vec<Capability> = self.caps.iter().copied().collect();
        v.sort_by_key(|c| c.name());
        v
    }

    /// Return the set of names for display / logging.
    pub fn names(&self) -> Vec<&'static str> {
        self.to_sorted_vec().iter().map(|c| c.name()).collect()
    }

    /// Apply this set as the effective capabilities of the current process.
    ///
    /// On Linux: uses `prctl(PR_CAP_AMBIENT_RAISE/LOWER)` to set ambient
    /// capabilities. On non-Linux platforms this is a no-op.
    pub fn apply(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::io;
            const PR_CAP_AMBIENT: libc::c_int = 47;
            const PR_CAP_AMBIENT_RAISE: libc::c_ulong = 2;
            const PR_CAP_AMBIENT_LOWER: libc::c_ulong = 3;
            const CAP_LAST_CAP: u32 = 40; // Linux 6.x cap count

            for cap_nr in 0..=CAP_LAST_CAP {
                let cap_name = Capability::from_number(cap_nr);
                let in_set = cap_name.map(|c| self.caps.contains(&c)).unwrap_or(false);
                let op = if in_set {
                    PR_CAP_AMBIENT_RAISE
                } else {
                    PR_CAP_AMBIENT_LOWER
                };
                let ret = unsafe { libc::prctl(PR_CAP_AMBIENT, op, cap_nr as libc::c_ulong, 0, 0) };
                if ret != 0 && in_set {
                    // Raising a capability we don't have is expected to fail —
                    // only treat it as an error if we explicitly wanted it
                    let err = io::Error::last_os_error();
                    if err.raw_os_error() != Some(libc::EPERM) {
                        return Err(crate::SandboxError::SystemCallFailed(format!(
                            "prctl(PR_CAP_AMBIENT) failed for cap {}: {}",
                            cap_nr, err
                        )));
                    }
                }
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(crate::SandboxError::SystemCallFailed(
                "Capability manipulation only supported on Linux".to_string(),
            ))
        }
    }
}

/// Return the effective capabilities of the current process.
///
/// On Linux: reads from `/proc/self/status` CapEff line and returns the
/// parsed capability set.
///
/// On non-Linux: returns an explicit error — capability detection is
/// not available on this platform. Callers must decide their own policy.
///
/// **SECURITY**: Previously this function returned `CapabilitySet::full()`
/// on failure, which silently granted all permissions. That was a critical
/// security bug — a sandbox MUST fail-closed, not fail-open. Fixed in
/// v0.4.47.9.
pub fn effective_capabilities() -> std::result::Result<CapabilitySet, crate::SandboxError> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").map_err(|e| {
            crate::SandboxError::SystemCallFailed(format!(
                "Cannot read /proc/self/status for capability detection: {}. \
                 Sandbox cannot determine current capabilities; failing closed.",
                e
            ))
        })?;

        for line in status.lines() {
            if let Some(hex_str) = line.strip_prefix("CapEff:\t") {
                let bitmask = u64::from_str_radix(hex_str.trim(), 16).map_err(|e| {
                    crate::SandboxError::SystemCallFailed(format!(
                        "Invalid CapEff bitmask in /proc/self/status: {}",
                        e
                    ))
                })?;
                let mut caps = std::collections::HashSet::new();
                for bit in 0..40u32 {
                    if bitmask & (1u64 << bit) != 0 {
                        if let Some(cap) = Capability::from_number(bit) {
                            caps.insert(cap);
                        }
                    }
                }
                return Ok(CapabilitySet { caps });
            }
        }

        Err(crate::SandboxError::SystemCallFailed(
            "No CapEff line found in /proc/self/status".to_string(),
        ))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(crate::SandboxError::SystemCallFailed(
            "Capability detection only supported on Linux. \
             Sandbox cannot determine current capabilities on this platform; \
             configure manually."
                .to_string(),
        ))
    }
}
