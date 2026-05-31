use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const VCS_BRANCH_ALREADY_EXISTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(319),
        long_code: "HHS_E_VCS_BRANCH_ALREADY_EXISTS",
        short_code: "E0319",
        title: "Branch name is already in use",
        short_description: "A branch creation attempt failed because a branch with the same name already exists in the repository.",
        long_description: "Git refuses to create a branch over an existing one without an explicit force flag. This error wraps that refusal so scripts can react cleanly — for example by switching to the existing branch instead of failing, or by generating a unique suffix.

Fix it by deciding on a policy: skip if exists, force-overwrite (`-f`), or generate a fresh name (e.g. append a timestamp). Whatever the policy, make it explicit in the script so concurrent runs do not race.

This error often appears in automation that creates a feature branch per ticket; idempotent helpers should check `git rev-parse --verify <branch>` first.",
        hints: &["Decide explicit policy: skip / force / unique-name", "Check `git rev-parse --verify <branch>` for idempotency", "Append a timestamp or run id to keep names unique", "Avoid concurrent script runs racing on the same branch"],
        example_bad: Some("vcs::create_branch(\"feature/x\"); // already exists"),
        example_good: Some("if !vcs::branch_exists(\"feature/x\") { vcs::create_branch(\"feature/x\"); }"),
        see_also: &["VcsBranchNotFound", "VcsInvalidOperation", "GitCommandFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const VCS_BRANCH_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(320),
        long_code: "HHS_E_VCS_BRANCH_NOT_FOUND",
        short_code: "E0320",
        title: "Branch does not exist in repository",
        short_description: "An operation tried to reference a branch that the repository does not have locally or on the configured remote.",
        long_description: "Git operations like checkout, delete, and merge require the target branch to exist. This error fires when the lookup returns nothing — typical causes are a typo, a branch that lives only on the remote and was never fetched, or a branch that has already been deleted by another script run.

Fix it by running `git fetch --all --prune` before the operation if the branch may live on a remote, and by listing branches (`git branch -a`) to verify spelling. For automation, check existence first with `git rev-parse --verify`.

When deleting branches, make the call idempotent: a missing branch is a successful end state, not an error.",
        hints: &["Run `git fetch --all --prune` before remote-only lookups", "Verify spelling with `git branch -a`", "Use `git rev-parse --verify` for existence checks", "Make delete idempotent — missing == success"],
        example_bad: Some("vcs::checkout(\"feature/typoo\");"),
        example_good: Some("vcs::fetch(); vcs::checkout(\"feature/typo\");"),
        see_also: &["VcsBranchAlreadyExists", "VcsInvalidOperation", "GitRepositoryNotFound"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const VCS_INVALID_OPERATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(321),
        long_code: "HHS_E_VCS_INVALID_OPERATION",
        short_code: "E0321",
        title: "VCS operation is not allowed in current state",
        short_description: "The repository is in a state that forbids the requested operation, e.g. merging on a detached HEAD or committing with no staged changes.",
        long_description: "Git enforces preconditions on many operations: you cannot commit without staged changes, cannot merge during a rebase, cannot switch branches with conflicting unstaged changes, and so on. This error wraps that family of refusals.

Fix it by inspecting the wrapped reason and the repo state (`git status`) to see what precondition was violated. Resolve the blocking state — finish or abort the in-progress operation, stash or commit dirty changes, attach HEAD to a branch — before retrying.

For scripts, snapshot `git status --porcelain` at the start and refuse to proceed if the tree is unexpectedly dirty.",
        hints: &["Read the wrapped reason — it names the violated precondition", "Inspect `git status` to see the repo's actual state", "Finish or abort in-progress merges/rebases before continuing", "Snapshot `git status --porcelain` at script entry"],
        example_bad: Some("vcs::commit(\"empty\"); // nothing staged"),
        example_good: Some("vcs::add([\"file.txt\"]);
vcs::commit(\"add file\");"),
        see_also: &["VcsMergeConflict", "VcsBranchNotFound", "GitCommandFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const VCS_PARSE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(323),
        long_code: "HHS_E_VCS_PARSE_ERROR",
        short_code: "E0323",
        title: "Failed to parse VCS output",
        short_description: "The VCS layer could not parse output from an underlying command into its expected structured form.",
        long_description: "The high-level VCS layer parses porcelain or format-string output from git into structured records (branches, commits, statuses). This error means parsing failed — usually because of locale, an unrecognized git version, or an unexpected hook printing extra bytes onto stdout.

Fix it by forcing `LC_ALL=C` and `LANG=C` before VCS calls, by silencing or redirecting noisy hooks, and by reproducing the failing command manually to inspect the actual bytes. If the raw output looks well-formed, the parser may have a gap and is worth reporting.

This is the high-level cousin of `GitParseError` and shares the same remediation playbook.",
        hints: &["Force LC_ALL=C and LANG=C before VCS calls", "Silence or redirect hooks that print to stdout", "Reproduce the failing command manually to inspect bytes", "Report parser gaps with raw output and git version"],
        example_bad: Some("// noisy post-checkout hook prints banner to stdout"),
        example_good: Some("env::set(\"LC_ALL\", \"C\");
let log = vcs::log(10);"),
        see_also: &["GitParseError", "VcsInvalidOperation", "GitCommandFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };
