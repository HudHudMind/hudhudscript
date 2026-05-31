//! Seccomp BPF filter builder (Issue #603)
//!
//! Provides a safe abstraction over Linux seccomp-BPF filters for restricting
//! the set of system calls available to sandboxed code. Actual `prctl` /
//! `seccomp` invocations are gated behind `#[cfg(target_os = "linux")]` so
//! the module compiles and tests pass on all platforms.

use crate::Result;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Policy
// ---------------------------------------------------------------------------

/// Action to take when a syscall matches a filter rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeccompPolicy {
    /// Allow the syscall to proceed.
    Allow,
    /// Deny the syscall (returns EPERM to the caller).
    Deny,
    /// Allow the syscall but emit a log entry.
    Log,
}

// ---------------------------------------------------------------------------
// SeccompFilter
// ---------------------------------------------------------------------------

/// A low-level BPF filter that maps individual syscall numbers to policies.
#[derive(Debug, Clone)]
pub struct SeccompFilter {
    /// Per-syscall overrides.  Anything not listed falls through to `default`.
    rules: Vec<(u32, SeccompPolicy)>,
    /// Default policy for syscalls without an explicit rule.
    default: SeccompPolicy,
}

impl SeccompFilter {
    /// Create a new filter whose default policy is `Deny`.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default: SeccompPolicy::Deny,
        }
    }

    /// Explicitly allow syscall `nr`.
    pub fn allow_syscall(&mut self, nr: u32) -> &mut Self {
        self.rules.push((nr, SeccompPolicy::Allow));
        self
    }

    /// Explicitly deny syscall `nr`.
    pub fn deny_syscall(&mut self, nr: u32) -> &mut Self {
        self.rules.push((nr, SeccompPolicy::Deny));
        self
    }

    /// Log-but-allow syscall `nr`.
    pub fn log_syscall(&mut self, nr: u32) -> &mut Self {
        self.rules.push((nr, SeccompPolicy::Log));
        self
    }

    /// Set the default policy for syscalls that have no explicit rule.
    pub fn default_policy(&mut self, policy: SeccompPolicy) -> &mut Self {
        self.default = policy;
        self
    }

    /// Return an iterator over the per-syscall rules.
    pub fn rules(&self) -> &[(u32, SeccompPolicy)] {
        &self.rules
    }

    /// Return the current default policy.
    pub fn get_default_policy(&self) -> SeccompPolicy {
        self.default
    }

    /// Look up the effective policy for a given syscall number.
    pub fn effective_policy(&self, nr: u32) -> SeccompPolicy {
        // Last matching rule wins (allows overriding earlier entries).
        for &(rule_nr, policy) in self.rules.iter().rev() {
            if rule_nr == nr {
                return policy;
            }
        }
        self.default
    }

    /// Install the filter into the current thread via `prctl(PR_SET_SECCOMP)`.
    ///
    /// On Linux: builds a BPF program from the rules and installs it.
    /// On non-Linux platforms returns an error (instead of silent no-op).
    pub fn apply(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::io;
            // PR_SET_NO_NEW_PRIVS must be set before seccomp strict/filter mode
            const PR_SET_NO_NEW_PRIVS: libc::c_int = 38;
            let ret = unsafe { libc::prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
            if ret != 0 {
                return Err(crate::SandboxError::SystemCallFailed(format!(
                    "prctl(PR_SET_NO_NEW_PRIVS) failed: {}",
                    io::Error::last_os_error()
                )));
            }

            // Build BPF filter program
            // Each rule: BPF_STMT(BPF_LD | BPF_W | BPF_ABS, syscall_nr_offset)
            //            BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, nr, 0, 1)
            //            BPF_STMT(BPF_RET | BPF_K, action)
            // Default action at the end.
            let mut bpf_instrs: Vec<libc::sock_filter> = Vec::new();

            // Load syscall number: BPF_LD+BPF_W+BPF_ABS, offset 0 in seccomp_data
            bpf_instrs.push(libc::sock_filter {
                code: 0x20, // BPF_LD | BPF_W | BPF_ABS
                jt: 0,
                jf: 0,
                k: 0, // offsetof(seccomp_data, nr)
            });

            for &(nr, policy) in &self.rules {
                let action: u32 = match policy {
                    SeccompPolicy::Allow => 0x7fff_0000, // SECCOMP_RET_ALLOW
                    SeccompPolicy::Deny => 0x0005_0000,  // SECCOMP_RET_ERRNO | EPERM
                    SeccompPolicy::Log => 0x7ffc_0000,   // SECCOMP_RET_LOG
                };
                // BPF_JMP | BPF_JEQ | BPF_K: if nr == k, jt=0 (next), jf=1 (skip)
                bpf_instrs.push(libc::sock_filter {
                    code: 0x15, // BPF_JMP | BPF_JEQ | BPF_K
                    jt: 0,
                    jf: 1,
                    k: nr,
                });
                // BPF_RET | BPF_K: return action
                bpf_instrs.push(libc::sock_filter {
                    code: 0x06, // BPF_RET | BPF_K
                    jt: 0,
                    jf: 0,
                    k: action,
                });
            }

            // Default action
            let default_action: u32 = match self.default {
                SeccompPolicy::Allow => 0x7fff_0000,
                SeccompPolicy::Deny => 0x0005_0000,
                SeccompPolicy::Log => 0x7ffc_0000,
            };
            bpf_instrs.push(libc::sock_filter {
                code: 0x06,
                jt: 0,
                jf: 0,
                k: default_action,
            });

            let prog = libc::sock_fprog {
                len: bpf_instrs.len() as u16,
                filter: bpf_instrs.as_ptr() as *mut _,
            };

            // SECCOMP_SET_MODE_FILTER = 1
            const SECCOMP_SET_MODE_FILTER: libc::c_ulong = 1;
            let ret = unsafe {
                libc::syscall(
                    libc::SYS_seccomp,
                    SECCOMP_SET_MODE_FILTER,
                    0u64, // flags
                    &prog as *const _,
                )
            };
            if ret != 0 {
                return Err(crate::SandboxError::SystemCallFailed(format!(
                    "seccomp(SET_MODE_FILTER) failed: {}",
                    io::Error::last_os_error()
                )));
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(crate::SandboxError::SystemCallFailed(
                "seccomp sandbox not supported on this platform".to_string(),
            ))
        }
    }
}

impl Default for SeccompFilter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SeccompProfile — high-level presets
// ---------------------------------------------------------------------------

/// Pre-built seccomp profiles for common use-cases.
#[derive(Debug, Clone)]
pub struct SeccompProfile {
    filter: SeccompFilter,
    /// Human-readable name of this profile.
    name: String,
}

impl SeccompProfile {
    /// Very restricted profile — only the bare minimum syscalls needed for a
    /// pure-computation sandbox (read, write, exit, mmap, brk, …).
    pub fn minimal() -> Self {
        let mut filter = SeccompFilter::new();
        filter.default_policy(SeccompPolicy::Deny);

        // Minimal set for a computation-only sandbox (x86-64 numbers).
        let allowed: &[u32] = &[
            0,   // read
            1,   // write
            3,   // close
            9,   // mmap
            10,  // mprotect
            11,  // munmap
            12,  // brk
            60,  // exit
            231, // exit_group
            13,  // rt_sigaction
            14,  // rt_sigprocmask
            35,  // nanosleep
            158, // arch_prctl
            218, // set_tid_address
            302, // prlimit64
        ];
        for &nr in allowed {
            filter.allow_syscall(nr);
        }

        Self {
            filter,
            name: "minimal".into(),
        }
    }

    /// Standard profile — allows common I/O, file, and memory syscalls while
    /// blocking dangerous ones (mount, reboot, kexec, …).
    pub fn standard() -> Self {
        let mut filter = SeccompFilter::new();
        filter.default_policy(SeccompPolicy::Allow);

        // Explicitly deny dangerous syscalls.
        let denied: &[u32] = &[
            165, // mount
            166, // umount2
            169, // reboot
            246, // kexec_load
            304, // open_by_handle_at
            175, // init_module
            176, // delete_module
            180, // nfsservctl
        ];
        for &nr in denied {
            filter.deny_syscall(nr);
        }

        Self {
            filter,
            name: "standard".into(),
        }
    }

    /// Permissive profile — allows almost everything; only the most
    /// catastrophic syscalls are blocked.
    pub fn permissive() -> Self {
        let mut filter = SeccompFilter::new();
        filter.default_policy(SeccompPolicy::Allow);

        let denied: &[u32] = &[
            169, // reboot
            246, // kexec_load
        ];
        for &nr in denied {
            filter.deny_syscall(nr);
        }

        Self {
            filter,
            name: "permissive".into(),
        }
    }

    /// Return the profile name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return a reference to the inner filter.
    pub fn filter(&self) -> &SeccompFilter {
        &self.filter
    }

    /// Return a mutable reference so the caller can tweak the preset.
    pub fn filter_mut(&mut self) -> &mut SeccompFilter {
        &mut self.filter
    }

    /// Install the profile's filter.
    pub fn apply(&self) -> Result<()> {
        self.filter.apply()
    }

    /// Return the set of explicitly allowed syscall numbers.
    pub fn allowed_syscalls(&self) -> HashSet<u32> {
        self.filter
            .rules()
            .iter()
            .filter(|(_, p)| *p == SeccompPolicy::Allow)
            .map(|(nr, _)| *nr)
            .collect()
    }

    /// Return the set of explicitly denied syscall numbers.
    pub fn denied_syscalls(&self) -> HashSet<u32> {
        self.filter
            .rules()
            .iter()
            .filter(|(_, p)| *p == SeccompPolicy::Deny)
            .map(|(nr, _)| *nr)
            .collect()
    }

    /// Create a custom profile from an existing filter.
    pub fn custom(name: impl Into<String>, filter: SeccompFilter) -> Self {
        Self {
            filter,
            name: name.into(),
        }
    }
}
