use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const GIT_COMMAND_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(87),
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
        category: ExceptionCategory::Tool,
    };

pub const GIT_GIT_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(88),
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
        category: ExceptionCategory::Tool,
    };

pub const GIT_INVALID_ARGUMENTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(89),
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
        category: ExceptionCategory::Tool,
    };

pub const GIT_PARSE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(90),
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
        category: ExceptionCategory::Tool,
    };

pub const GIT_REPOSITORY_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(91),
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
        category: ExceptionCategory::Tool,
    };

pub const GIT_SPAWN_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(92),
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
        category: ExceptionCategory::Tool,
    };
