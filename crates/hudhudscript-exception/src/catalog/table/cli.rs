use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TABLE: [ExceptionEntry; 5] = [
    ExceptionEntry {
        code: ExceptionCode(3),
        long_code: "HHS_E_ARG_INVALID_VALUE",
        short_code: "E0003",
        title: "CLI Argument Has Invalid Value",
        short_description: "A command-line argument was supplied with a value that does not satisfy its declared type or accepted set.",
        long_description: "The CLI parser successfully matched the flag or positional you provided, but the value attached to it failed conversion or validation. This commonly happens when a numeric flag receives a non-numeric token, an enum-style flag receives a name outside the allowed list, or a path flag receives a string the OS rejects.

Value parsing in `hudhudscript-cli` runs after argument matching, so the diagnostic includes both the flag name and the offending token. The command does not execute when this error fires; the process exits before any subcommand handler runs.

Fix the value to match the type expected by the flag. If you intended a different flag, double-check spelling — an unknown flag would surface a different diagnostic instead.",
        hints: &["Check the expected type with `hhs <command> --help`", "Quote values containing spaces or shell metacharacters", "For enum flags, copy the exact spelling from --help output", "Verify numeric ranges — some flags accept only positive integers"],
        example_bad: Some("hhs run --threads abc"),
        example_good: Some("hhs run --threads 8"),
        see_also: &["ArgUnknownArgument", "ArgMissingRequired", "ArgOther"],
        since_version: "0.1.0",
        category: ExceptionCategory::Cli,
    },

    ExceptionEntry {
        code: ExceptionCode(4),
        long_code: "HHS_E_ARG_MISSING_REQUIRED",
        short_code: "E0004",
        title: "Required CLI Argument Not Provided",
        short_description: "A command-line argument marked as required by the active subcommand was omitted from the invocation.",
        long_description: "Each HudHudScript subcommand declares which flags and positionals are mandatory. When you invoke a subcommand without one of these required inputs, the parser stops before dispatching and reports the missing name.

This check runs after subcommand selection, so the diagnostic is scoped to the specific subcommand you typed. Different subcommands have different required sets — `hhs deploy` requires a project directory while `hhs run` does not.

Provide the missing argument on the command line or, where supported, set it via the corresponding environment variable or config file. The `--help` output for the subcommand lists every required input near the top.",
        hints: &["Run `hhs <subcommand> --help` to see all required arguments", "Check whether the value can be supplied via env var instead", "Verify you are invoking the intended subcommand", "Use shell history expansion only after confirming the previous call was correct"],
        example_bad: Some("hhs deploy"),
        example_good: Some("hhs deploy --project-dir ./my-app"),
        see_also: &["ArgMissingSubcommand", "ArgInvalidValue", "ArgUnknownArgument"],
        since_version: "0.1.0",
        category: ExceptionCategory::Cli,
    },

    ExceptionEntry {
        code: ExceptionCode(5),
        long_code: "HHS_E_ARG_MISSING_SUBCOMMAND",
        short_code: "E0005",
        title: "No Subcommand Specified",
        short_description: "The CLI was invoked without selecting a subcommand and no default action is configured for the bare binary.",
        long_description: "`hhs` is a multi-tool: most useful work happens through subcommands such as `run`, `build`, `deploy`, `lsp`, or `repl`. Calling the binary with no subcommand (and no global flag like `--version`) leaves the parser without an action to dispatch.

This is distinct from a missing argument inside a subcommand — here the parser never gets past the top level. The error message lists the available subcommands so you can pick one without consulting the manual.

Provide a subcommand or one of the recognized top-level flags. If you wanted a help summary, use `--help`.",
        hints: &["Run `hhs --help` to list all available subcommands", "Common subcommands: run, build, deploy, lsp, repl, fmt", "Use `hhs --version` if you only wanted the version string", "Shell aliases that strip arguments are a common cause"],
        example_bad: Some("hhs"),
        example_good: Some("hhs run script.hhs"),
        see_also: &["ArgMissingRequired", "ArgUnknownArgument", "ArgOther"],
        since_version: "0.1.0",
        category: ExceptionCategory::Cli,
    },

    ExceptionEntry {
        code: ExceptionCode(6),
        long_code: "HHS_E_ARG_OTHER",
        short_code: "E0006",
        title: "Generic CLI Parsing Failure",
        short_description: "The argument parser failed for a reason that does not fit the more specific CLI error categories.",
        long_description: "This is a catch-all variant emitted when `hudhudscript-cli` encounters a parsing problem outside the well-known categories of unknown argument, missing required, missing subcommand, or invalid value. Examples include conflicting flag combinations, malformed UTF-8 in argv, or internal `clap` errors that the wrapper did not classify.

Because the cause is open-ended, the wrapped message from the underlying parser is the most important piece of information. Always read the embedded text — it usually pinpoints the offending token or rule.

If the message mentions a conflict, remove one of the flags. If it mentions UTF-8, check your shell's locale settings. If you cannot interpret it, file an issue with the exact command line you used.",
        hints: &["Read the wrapped message — it carries the actual cause", "Look for conflicting flag pairs (e.g. --quiet and --verbose)", "Check LANG/LC_ALL if non-ASCII characters are involved", "Reproduce with `--help` after removing one suspect flag at a time"],
        example_bad: Some("hhs run --quiet --verbose"),
        example_good: Some("hhs run --verbose"),
        see_also: &["ArgInvalidValue", "ArgUnknownArgument", "ArgMissingRequired"],
        since_version: "0.1.0",
        category: ExceptionCategory::Cli,
    },

    ExceptionEntry {
        code: ExceptionCode(7),
        long_code: "HHS_E_ARG_UNKNOWN_ARGUMENT",
        short_code: "E0007",
        title: "Unrecognized CLI Argument",
        short_description: "The CLI parser encountered a flag or positional that the active subcommand does not declare.",
        long_description: "Every subcommand has a fixed set of accepted flags and positionals. When you pass something outside that set — whether through a typo, an outdated tutorial, or a flag that belongs to a different subcommand — the parser refuses to guess and reports the offending token.

Unknown arguments are diagnosed before any required-argument checks, so seeing this error means parsing failed at the unrecognized token specifically. The remainder of the command line is not validated.

Check the spelling against `--help`. Be aware that some flags only exist on a specific subcommand; passing a `run`-only flag to `build` will trip this error even though the spelling looks fine.",
        hints: &["Run `hhs <subcommand> --help` to see the supported flags", "Check for typos — `--verbosee` vs `--verbose`", "Make sure the flag belongs to the subcommand you typed", "Long flags use `--name`, short flags use `-n`"],
        example_bad: Some("hhs run --verbsoe script.hhs"),
        example_good: Some("hhs run --verbose script.hhs"),
        see_also: &["ArgInvalidValue", "ArgMissingRequired", "ArgOther"],
        since_version: "0.1.0",
        category: ExceptionCategory::Cli,
    }
];
