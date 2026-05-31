use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const APPROVAL_INVALID_TRANSITION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(1),
        long_code: "HHS_E_APPROVAL_INVALID_TRANSITION",
        short_code: "E0001",
        title: "Approval state transition is invalid",
        short_description: "An approval request cannot move from its current state to the requested state under the approval workflow rules.",
        long_description: "Approval requests follow a finite state machine: typically Pending -> Approved or Pending -> Denied, with terminal states that cannot be re-entered. This error means a caller tried an illegal transition, for example approving an already-denied request or re-opening one that was previously resolved.

Fix it by inspecting the current state of the request before issuing the transition, and by ensuring no two callers race to resolve the same request. If the workflow legitimately needs to revisit a decision, create a new approval request rather than mutating the old one.

This commonly appears in human-in-the-loop scripts where the operator clicks a button twice, or where retry logic re-issues a decision after a transient network failure.",
        hints: &["Fetch the current approval state before calling approve/deny", "Treat Approved and Denied as terminal — never transition out", "Make decision endpoints idempotent so retries are safe", "Issue a new approval request if a fresh decision is needed"],
        example_bad: Some("let req = approval::get(id);
approval::approve(id); // already Denied — invalid transition"),
        example_good: Some("let req = approval::get(id);
if req.state == \"Pending\" {
    approval::approve(id);
}"),
        see_also: &["ApprovalNotFound", "ToolExecutionFailed", "ToolValidation"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const APPROVAL_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(2),
        long_code: "HHS_E_APPROVAL_NOT_FOUND",
        short_code: "E0002",
        title: "Approval request id does not exist",
        short_description: "The approval store has no entry matching the supplied request id, so the operation cannot be applied.",
        long_description: "The approval subsystem keys requests by an opaque id. This error means that id is unknown — it was never created, has expired and been garbage collected, or belongs to a different approval store than the one being queried.

Fix it by ensuring the id is captured from the original `approval::request(...)` call and passed unchanged. Beware of stringification, JSON round-trips that lose precision, or stores that are scoped per-process and not shared with the consumer.

A common cause is calling the approver from a fresh script run without persisting the request id from the previous run, or pointing the approver and the requester at different backends.",
        hints: &["Verify the id came from a successful approval::request() call", "Check that requester and approver share the same backend store", "Watch for id truncation across JSON / database / URL boundaries", "Confirm the request has not exceeded its TTL and been purged"],
        example_bad: Some("approval::approve(\"req-123\"); // never created"),
        example_good: Some("let id = approval::request({ title: \"Deploy prod\" });
approval::approve(id);"),
        see_also: &["ApprovalInvalidTransition", "ToolExecutionFailed", "ToolInvalidArguments"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const DATABASE_CONNECTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(65),
        long_code: "HHS_E_DATABASE_CONNECTION_FAILED",
        short_code: "E0065",
        title: "Database connection could not be established",
        short_description: "The database driver failed to open a connection to the configured server, often due to network, auth, or DSN issues.",
        long_description: "This error wraps the underlying driver failure when opening a connection. Causes include unreachable host, wrong port, TLS handshake failure, expired credentials, exhausted connection pool, or a database that is not yet accepting connections during startup.

Fix it by validating the connection string with a CLI client (`psql`, `mysql`, `sqlite3`) using the same DSN, then checking firewall and DNS resolution. For pooled connections, confirm the pool size and timeout settings allow new connections under load.

In containerized environments this often appears at startup when the script races the database. Add a retry-with-backoff loop or a readiness probe before issuing the first query.",
        hints: &["Verify the DSN with a native CLI client first", "Check host, port, TLS mode, username, and password individually", "Add retry-with-backoff for startup races against the database", "Inspect server logs for refused or rate-limited connections"],
        example_bad: Some("let db = database::connect(\"postgres://user@unreachable/db\");"),
        example_good: Some("let db = retry(3, || database::connect(env(\"DATABASE_URL\")));"),
        see_also: &["DatabaseQueryFailed", "DatabaseFeatureNotEnabled", "DatabaseUnsupportedBackend"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const DATABASE_FEATURE_NOT_ENABLED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(66),
        long_code: "HHS_E_DATABASE_FEATURE_NOT_ENABLED",
        short_code: "E0066",
        title: "Database feature flag is disabled at build time",
        short_description: "The runtime was compiled without the `db` feature, so full sqlx-backed database support is unavailable.",
        long_description: "HudHudScript can be built in a slim configuration that omits database drivers. When a script calls `database::*` against such a build, this error is returned to make the missing capability explicit instead of silently failing later.

Fix it by rebuilding the runtime with `--features db` (or the equivalent meta-feature for your distribution), or by switching to a build that already enables it. CI images and minimal Docker variants are the most common offenders.

If you cannot enable the feature, restructure the script to use an external query tool over a process boundary, or move database work into a service that exposes an HTTP/JSON API.",
        hints: &["Rebuild the runtime with `cargo build --features db`", "Check `hudhud --version` for the enabled feature list", "Use a full image instead of the slim/minimal variant", "Wrap database code in `if has_feature(\"db\")` for portability"],
        example_bad: Some("// runtime built without `db` feature
let rows = database::query(\"SELECT 1\");"),
        example_good: Some("// build: cargo build --release --features db
let rows = database::query(\"SELECT 1\");"),
        see_also: &["DatabaseConnectionFailed", "DatabaseUnsupportedBackend", "ToolExecutionFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const DATABASE_INVALID_ARGUMENTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(67),
        long_code: "HHS_E_DATABASE_INVALID_ARGUMENTS",
        short_code: "E0067",
        title: "Database call received invalid arguments",
        short_description: "Arguments passed to a database function failed validation before any query was issued.",
        long_description: "This error fires before the query reaches the driver. Typical causes are missing required fields (no DSN, no SQL string), wrong types in the parameter binding list, mismatched placeholder counts, or unsupported option keys in the call options object.

Fix it by reading the function signature for the database call you are using and matching every required argument with the right type. For parameterized queries make sure the number of `?`/`$1` placeholders matches the length of the bind list.

This often shows up when refactoring from string interpolation to bind parameters, or when migrating between Postgres-style and MySQL-style placeholders.",
        hints: &["Match placeholder count to bind list length exactly", "Use the right placeholder syntax for your backend ($1 vs ?)", "Pass scalars/arrays — not objects — as bind parameters", "Re-read the database::* signature for required vs optional args"],
        example_bad: Some("database::query(\"SELECT * FROM t WHERE a=$1 AND b=$2\", [42]);"),
        example_good: Some("database::query(\"SELECT * FROM t WHERE a=$1 AND b=$2\", [42, \"x\"]);"),
        see_also: &["DatabaseQueryFailed", "ToolInvalidArguments", "ToolValidation"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const DATABASE_QUERY_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(68),
        long_code: "HHS_E_DATABASE_QUERY_FAILED",
        short_code: "E0068",
        title: "SQL query execution failed",
        short_description: "The database server rejected or aborted the query — typical causes are syntax errors, missing tables, or constraint violations.",
        long_description: "The driver successfully sent the query but the server returned an error. The wrapped message comes straight from the backend and usually identifies the offending column, constraint, or syntax position. Common cases include unknown table/column, NOT NULL violation, foreign-key violation, deadlock, or permission denied.

Fix it by reading the wrapped backend message carefully — it is far more specific than the HudHudScript wrapper. Reproduce the failing query in a SQL client with the exact same parameters to isolate whether the issue is data, schema, or permissions.

For transient errors like deadlocks or serialization failures, retry the whole transaction. For schema drift, run migrations before the script.",
        hints: &["Read the wrapped backend message — it names the column/constraint", "Reproduce the failing query in a native SQL client", "Retry deadlocks and serialization failures from the top of the txn", "Run migrations before scripts that depend on schema changes"],
        example_bad: Some("database::query(\"SELECT * FROM users WHERE eemail = $1\", [e]);"),
        example_good: Some("database::query(\"SELECT * FROM users WHERE email = $1\", [e]);"),
        see_also: &["DatabaseConnectionFailed", "DatabaseInvalidArguments", "DatabaseUnsupportedBackend"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const DATABASE_UNSUPPORTED_BACKEND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(69),
        long_code: "HHS_E_DATABASE_UNSUPPORTED_BACKEND",
        short_code: "E0069",
        title: "Database backend is not supported",
        short_description: "The DSN scheme refers to a backend that this runtime build cannot drive (for example oracle:// or mssql://).",
        long_description: "HudHudScript currently ships drivers for a known set of backends — typically Postgres, MySQL/MariaDB, and SQLite. A DSN whose scheme is outside that set produces this error so the script fails fast instead of hanging on a half-implemented driver.

Fix it by switching to a supported backend, or by routing through a compatibility proxy (FDW, ODBC bridge, or a small service) that exposes one of the supported wire protocols.

If you control the DSN, double-check the URL scheme — typos like `postgress://` or `mysql2://` also reach this branch.",
        hints: &["Use a supported scheme: postgres, mysql, sqlite", "Check for typos in the DSN scheme prefix", "Front the unsupported backend with a Postgres FDW or proxy", "Move backend-specific work into an HTTP microservice"],
        example_bad: Some("database::connect(\"oracle://...\");"),
        example_good: Some("database::connect(\"postgres://user:pass@host/db\");"),
        see_also: &["DatabaseConnectionFailed", "DatabaseFeatureNotEnabled", "DatabaseInvalidArguments"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const GIT_COMMAND_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(87),
        long_code: "HHS_E_GIT_COMMAND_FAILED",
        short_code: "E0087",
        title: "git subprocess exited non-zero",
        short_description: "The git CLI ran but exited with a non-zero status; the wrapped output explains the underlying git error.",
        long_description: "HudHudScript shells out to the system `git` binary for VCS operations. When git itself reports failure (merge conflict, dirty working tree, rejected push, missing remote, etc.) the exit code and combined stdout/stderr are bundled into this error.

Fix it by reading the wrapped git output — it is the canonical diagnosis. Reproduce the same command on the command line in the same working directory to confirm the cause and to try interactive recovery (`git status`, `git fetch`, `git pull --rebase`).

When scripting git, always check the working tree state before operations that mutate it, and prefer porcelain commands with explicit flags over relying on user config.",
        hints: &["Read the wrapped git stderr — it is the real diagnosis", "Reproduce the failing command in a shell at the same cwd", "Check `git status` before mutating commands", "Pin behavior with explicit flags instead of relying on config"],
        example_bad: Some("git::run([\"push\"]); // rejected: non-fast-forward"),
        example_good: Some("git::run([\"fetch\"]);
git::run([\"pull\", \"--rebase\"]);
git::run([\"push\"]);"),
        see_also: &["GitGitNotFound", "GitSpawnFailed", "VcsMergeConflict"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const GIT_GIT_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(88),
        long_code: "HHS_E_GIT_GIT_NOT_FOUND",
        short_code: "E0088",
        title: "git binary is not on PATH",
        short_description: "HudHudScript could not locate the `git` executable on the system PATH and cannot run any VCS operations.",
        long_description: "The git tool subsystem requires the `git` binary at runtime. This error means a `which git` (or its platform equivalent) returned nothing. Common environments where this bites are minimal Docker images, Alpine without `git` apk, Lambda layers, and CI runners with a stripped PATH.

Fix it by installing git in the execution environment (`apt-get install git`, `apk add git`, etc.) and making sure the install directory is on PATH for the user running the script.

If the runtime is running under a service manager that scrubs the environment, set PATH explicitly in the unit file or container manifest.",
        hints: &["Install git in the runtime environment", "Verify with `which git` as the same user that runs the script", "Set PATH explicitly under systemd / Docker / Lambda", "Use a base image that already includes git"],
        example_bad: Some("// runtime image without git installed
git::run([\"status\"]);"),
        example_good: Some("// Dockerfile: RUN apt-get install -y git
git::run([\"status\"]);"),
        see_also: &["GitSpawnFailed", "GitCommandFailed", "ToolExecutionFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const GIT_INVALID_ARGUMENTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(89),
        long_code: "HHS_E_GIT_INVALID_ARGUMENTS",
        short_code: "E0089",
        title: "git tool received invalid arguments",
        short_description: "Arguments passed to a git helper failed validation before the subprocess was launched.",
        long_description: "The git tool wrapper validates arguments locally — non-empty repo path, well-formed refspecs, allowed subcommand list — to catch obvious mistakes before forking. This error fires when that pre-flight check fails.

Fix it by reading the function signature of the git helper you are using and supplying every required field with the right type. Pay special attention to ref names (no spaces, no leading dashes) and to URL fields when cloning.

This is also raised when a script tries to call a subcommand that the wrapper does not expose, such as filter-branch or interactive rebase.",
        hints: &["Match the helper signature for required vs optional fields", "Sanitize ref names — no whitespace, no leading dashes", "Use only the subcommands the wrapper exposes", "Quote paths that contain spaces"],
        example_bad: Some("git::checkout(\"\"); // empty ref"),
        example_good: Some("git::checkout(\"main\");"),
        see_also: &["GitCommandFailed", "GitParseError", "ToolInvalidArguments"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const GIT_PARSE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(90),
        long_code: "HHS_E_GIT_PARSE_ERROR",
        short_code: "E0090",
        title: "Failed to parse git output",
        short_description: "The git tool could not parse the porcelain output of a git command into its expected structured form.",
        long_description: "Several git helpers parse machine-readable output (`--porcelain`, `for-each-ref --format=...`, `log --format=...`) into structured values. This error means the parser hit something unexpected — usually because of a non-English locale leaking through, an unfamiliar git version, or hooks that prepend text to the output.

Fix it by forcing a stable locale (`LC_ALL=C`, `LANG=C`) for the git invocation, and by running the same command manually to inspect the actual bytes received. If a custom hook prints to stdout, redirect its output elsewhere.

A matching parser bug can also cause this; if the raw output looks correct, file an issue with the git version and the captured output.",
        hints: &["Set LC_ALL=C and LANG=C before invoking git", "Run the same git command manually to inspect raw output", "Disable hooks that print to stdout (or redirect them)", "Report parser bugs with git version + raw output attached"],
        example_bad: Some("// hook prints \"hello\" to stdout, breaking porcelain parsing"),
        example_good: Some("env::set(\"LC_ALL\", \"C\");
let status = git::status();"),
        see_also: &["GitCommandFailed", "GitInvalidArguments", "VcsParseError"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const GIT_REPOSITORY_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(91),
        long_code: "HHS_E_GIT_REPOSITORY_NOT_FOUND",
        short_code: "E0091",
        title: "No git repository at the given path",
        short_description: "The supplied path does not contain a `.git` directory and is not inside any git working tree.",
        long_description: "Git tool helpers that require a repository walk upward from the given path looking for `.git`. This error means that walk reached the filesystem root without finding one, so the operation cannot proceed.

Fix it by confirming the path is correct and that the directory has actually been initialized (`git init`) or cloned (`git clone`). For scripts that pass relative paths, double-check the working directory at the time of the call.

A second common cause is a `.git` file (worktree pointer) whose target directory has been deleted; clean up the orphaned worktree with `git worktree prune` from the main repo.",
        hints: &["Confirm the path with an absolute, not relative, location", "Run `git init` or `git clone` if the repo is missing", "Check for orphaned git worktree pointer files", "Verify the script's cwd at call time"],
        example_bad: Some("git::status({ cwd: \"/tmp/empty\" });"),
        example_good: Some("git::clone(\"https://github.com/me/repo\", \"/tmp/repo\");
git::status({ cwd: \"/tmp/repo\" });"),
        see_also: &["GitCommandFailed", "GitInvalidArguments", "VcsBranchNotFound"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

pub const GIT_SPAWN_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(92),
        long_code: "HHS_E_GIT_SPAWN_FAILED",
        short_code: "E0092",
        title: "Failed to spawn git subprocess",
        short_description: "The OS refused to start the `git` process — usually permissions, ulimits, or a missing interpreter for git's helpers.",
        long_description: "Unlike `GitGitNotFound` (binary not on PATH) and `GitCommandFailed` (binary ran and exited non-zero), this error covers the narrow window where the OS itself refused to fork/exec the binary. Causes include exhausted process ulimit, denied execute permission on the binary, SELinux/AppArmor policy, or a chroot missing libraries that git needs.

Fix it by running `git --version` as the same user in the same environment to confirm the binary is actually executable. Check `ulimit -u`, file permissions, and any LSM denials in the audit log.

In container sandboxes, ensure the seccomp/apparmor profile allows fork/exec for the runtime user.",
        hints: &["Run `git --version` as the same user that runs the script", "Check `ulimit -u` and process count limits", "Inspect SELinux/AppArmor audit logs for denials", "Verify execute permission on the git binary"],
        example_bad: Some("// running under a seccomp profile that blocks execve"),
        example_good: Some("// loosen sandbox or pre-fork git in a sidecar"),
        see_also: &["GitGitNotFound", "GitCommandFailed", "ToolExecutionFailed"],
        since_version: "0.4.0",
        category: ErrorCategory::Tool,
    };

mod core;
pub use core::*;

pub static ENTRIES: &[ErrorEntry] = &[
    APPROVAL_INVALID_TRANSITION,
    APPROVAL_NOT_FOUND,
    DATABASE_CONNECTION_FAILED,
    DATABASE_FEATURE_NOT_ENABLED,
    DATABASE_INVALID_ARGUMENTS,
    DATABASE_QUERY_FAILED,
    DATABASE_UNSUPPORTED_BACKEND,
    GIT_COMMAND_FAILED,
    GIT_GIT_NOT_FOUND,
    GIT_INVALID_ARGUMENTS,
    GIT_PARSE_ERROR,
    GIT_REPOSITORY_NOT_FOUND,
    GIT_SPAWN_FAILED,
    HTTP_TOOL_INVALID_URL,
    HTTP_TOOL_PARSE_ERROR,
    HTTP_TOOL_REQUEST_FAILED,
    HTTP_TOOL_TIMEOUT,
    OPEN_API_PARSE_ERROR,
    OPEN_API_REGISTRY_ERROR,
    TOOL_EXECUTION_FAILED,
    TOOL_INVALID_ARGUMENTS,
    TOOL_SECURITY_VIOLATION,
    TOOL_VALIDATION,
    VCS_BRANCH_ALREADY_EXISTS,
    VCS_BRANCH_NOT_FOUND,
    VCS_INVALID_OPERATION,
    VCS_MERGE_CONFLICT,
    VCS_PARSE_ERROR,
];
