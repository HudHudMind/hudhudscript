use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TABLE: [ExceptionEntry; 6] = [
    ExceptionEntry {
        code: ExceptionCode(255),
        long_code: "HHS_E_SANDBOX_FILE_SYSTEM_DENIED",
        short_code: "E0255",
        title: "Filesystem Access Denied By Sandbox",
        short_description: "A filesystem operation was blocked by the active sandbox policy because the path is outside the allowed set.",
        long_description: "`hudhudscript-sandbox` mediates all filesystem access from sandboxed scripts. Each open, read, write, or stat call is checked against the policy in force. When the path is not in the allowed set — or is in the explicit deny set — this error is raised before the syscall is issued.

This is a security boundary, not a permissions error from the OS. Even if the underlying user has access, the sandbox refuses the call. The diagnostic includes the path so you can adjust the policy if appropriate.

Decide whether to widen the sandbox policy or to change the script to use an allowed path. Widening the policy should be a deliberate, reviewed decision.",
        hints: &["Check whether the path should be in the allowlist", "Prefer changing the script to use an allowed path when possible", "Treat policy widening as a reviewable change", "Distinguish sandbox denial from OS-level permission errors"],
        example_bad: None,
        example_good: None,
        see_also: &["SandboxNetworkDenied", "SandboxProcessDenied", "SandboxInvalidConfig"],
        since_version: "0.4.0",
        category: ExceptionCategory::Security,
    },

    ExceptionEntry {
        code: ExceptionCode(256),
        long_code: "HHS_E_SANDBOX_INVALID_CONFIG",
        short_code: "E0256",
        title: "Invalid Sandbox Configuration",
        short_description: "The sandbox could not start because its configuration is malformed, contradictory, or references unknown features.",
        long_description: "Sandbox configuration declares allowlists, denylists, resource limits, and capability toggles. The sandbox parses and validates this configuration on construction. Any structural problem — bad syntax, unknown key, contradictory rules, out-of-range limit — fails the construction with this error.

No sandbox is created when this fires, so any code that depends on a running sandbox will not start. The wrapped message identifies the offending field.

Fix the configuration and retry. Validate configuration as part of CI to catch problems before deployment.",
        hints: &["Read the wrapped message for the offending configuration field", "Validate sandbox configuration as part of CI", "Check for typos in capability or feature names", "Resolve contradictions between allowlist and denylist explicitly"],
        example_bad: None,
        example_good: None,
        see_also: &["SandboxFileSystemDenied", "SandboxNetworkDenied", "SandboxResourceLimitExceeded"],
        since_version: "0.4.0",
        category: ExceptionCategory::Security,
    },

    ExceptionEntry {
        code: ExceptionCode(257),
        long_code: "HHS_E_SANDBOX_NETWORK_DENIED",
        short_code: "E0257",
        title: "Network Access Denied By Sandbox",
        short_description: "A network operation was blocked because the destination host or port is not in the sandbox allowlist.",
        long_description: "Sandboxed scripts may make network calls only to destinations explicitly permitted by the policy. The sandbox inspects each connect, bind, or send call and refuses any that target a host or port outside the allowlist. This error is raised in place of the underlying network call.

The denial is enforced before any packet leaves the process, so there is no observable side effect on the network. The error message names the destination so you can decide whether to update the policy.

Either add the destination to the allowlist with proper review, or rework the script to use a permitted endpoint.",
        hints: &["Check whether the destination should be on the allowlist", "Allowlist updates should go through review", "Specify exact host/port pairs rather than wildcarding broadly", "Distinguish sandbox denial from DNS or routing failures"],
        example_bad: None,
        example_good: None,
        see_also: &["SandboxFileSystemDenied", "SandboxProcessDenied", "SandboxInvalidConfig"],
        since_version: "0.4.0",
        category: ExceptionCategory::Security,
    },

    ExceptionEntry {
        code: ExceptionCode(258),
        long_code: "HHS_E_SANDBOX_PROCESS_DENIED",
        short_code: "E0258",
        title: "Process Execution Denied By Sandbox",
        short_description: "A request to spawn a child process was rejected because process execution is not permitted by the sandbox policy.",
        long_description: "Spawning subprocesses is one of the most common ways to escape a sandbox, so the policy controls it tightly. When a script attempts to invoke an external program without the corresponding capability, this error is raised before any fork or spawn happens.

The wrapped message names the binary that was requested. Granting process execution should be considered carefully because it often broadens the effective trust surface dramatically.

Replace the subprocess call with an in-process equivalent if possible. If a subprocess is unavoidable, narrow the policy to permit only the specific binary needed.",
        hints: &["Prefer in-process implementations over spawning subprocesses", "Allowlist the specific binary, not all process execution", "Treat process capability as broadly trust-extending", "Audit any code path that triggers this error in production"],
        example_bad: None,
        example_good: None,
        see_also: &["SandboxFileSystemDenied", "SandboxNetworkDenied", "SandboxSystemCallFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Security,
    },

    ExceptionEntry {
        code: ExceptionCode(259),
        long_code: "HHS_E_SANDBOX_RESOURCE_LIMIT_EXCEEDED",
        short_code: "E0259",
        title: "Sandbox Resource Limit Exceeded",
        short_description: "A script exceeded a resource limit (memory, CPU time, file handles, etc.) configured by the active sandbox policy.",
        long_description: "Sandbox policies cap resource usage to prevent runaway scripts. When the script exceeds any of those caps — memory ceiling, wall-clock or CPU budget, open-file count, allocation rate — the sandbox terminates the offending operation and returns this error.

The specific limit that tripped is named in the wrapped message. Hitting a limit is not necessarily a bug; it can also indicate unexpectedly large input or a missing batching strategy.

Decide whether to optimize the script, raise the limit (with review), or batch the work into smaller pieces.",
        hints: &["Identify which specific limit tripped from the wrapped message", "Consider batching large work into smaller pieces", "Raise limits only after a review and a clear justification", "Profile the script to find unexpected allocation hot spots"],
        example_bad: None,
        example_good: None,
        see_also: &["SandboxInvalidConfig", "SandboxSystemCallFailed", "SandboxFileSystemDenied"],
        since_version: "0.4.0",
        category: ExceptionCategory::Security,
    },

    ExceptionEntry {
        code: ExceptionCode(260),
        long_code: "HHS_E_SANDBOX_SYSTEM_CALL_FAILED",
        short_code: "E0260",
        title: "Sandbox System Call Failed",
        short_description: "A system call permitted by the sandbox policy failed at the OS level and the sandbox surfaced the error to the script.",
        long_description: "Even when the sandbox allows a syscall, the OS may still reject it (ENOENT, EACCES, EINTR, etc.). This variant wraps such OS-level failures so callers can distinguish them from sandbox-policy denials. The wrapped message carries the original errno or message.

This variant is not a security event — the policy permitted the call. It is a normal failure that happens to be reported through the sandbox boundary.

Treat it like any OS error: inspect the cause, decide on retry or recovery, and surface a meaningful message to the user. Do not confuse it with the `*Denied` variants.",
        hints: &["Treat this as a regular OS error, not a sandbox policy denial", "Inspect the wrapped errno or message for the real cause", "Distinguish from `*Denied` variants when triaging", "Apply normal retry/recovery logic appropriate to the operation"],
        example_bad: None,
        example_good: None,
        see_also: &["SandboxFileSystemDenied", "SandboxResourceLimitExceeded", "SandboxInvalidConfig"],
        since_version: "0.4.0",
        category: ExceptionCategory::Security,
    }
];
