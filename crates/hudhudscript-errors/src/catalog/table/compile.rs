use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const COMPILE_GENERIC: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(34),
        long_code: "HHS_E_COMPILE_GENERIC",
        short_code: "E0034",
        title: "Generic compilation failure in bytecode emitter",
        short_description: "The bytecode compiler reported a generic error that does not fit a more specific category.",
        long_description: "While lowering the typed AST to bytecode, the compiler hit a failure that it could not classify under a more specific variant. This is a catch-all used by the bytecode crate when an internal invariant fails or when an error bubbles up from a helper without a precise location.

Look at the accompanying message for context. If the underlying cause is reproducible, simplify the source until the error disappears, then file a minimal repro — generic compile errors usually indicate a missing diagnostic that should be promoted to a dedicated variant.

This variant most often appears in early-stage feature work or when AST shapes from the parser do not match what the compiler expects.",
        hints: &["Read the wrapped message — it usually pinpoints the failing construct", "Try compiling smaller fragments to narrow down which expression triggers it", "If the message mentions an unsupported node, see HHS_E_COMPILE_UNSUPPORTED_FEATURE"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_COMPILE_GENERIC_AT", "HHS_E_COMPILE_UNSUPPORTED_FEATURE"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub const COMPILE_GENERIC_AT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(35),
        long_code: "HHS_E_COMPILE_GENERIC_AT",
        short_code: "E0035",
        title: "Generic compilation failure with source location",
        short_description: "The bytecode compiler reported a generic error tied to a specific source location.",
        long_description: "Same as the location-less generic compile error, except the compiler was able to attach a span (file, line, column) to the failure. The attached location points at the construct that was being lowered when the error occurred.

Use the location to inspect the offending expression or statement. If the message is unclear, comment out surrounding code and recompile to confirm which node is to blame, then report the case so a more specific diagnostic can be added.

Common causes: half-implemented language features, malformed AST produced by recovery, or unexpected operator/operand combinations.",
        hints: &["Jump to the reported file/line and inspect the highlighted expression", "Reduce the snippet to the minimum that still reproduces the error", "Check the changelog — newly added syntax may not yet be wired through the compiler"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_COMPILE_GENERIC", "HHS_E_COMPILE_INVALID_BYTECODE_AT"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub const COMPILE_INVALID_BYTECODE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(36),
        long_code: "HHS_E_COMPILE_INVALID_BYTECODE",
        short_code: "E0036",
        title: "Compiler produced invalid bytecode",
        short_description: "The bytecode emitter generated an instruction sequence that fails validation.",
        long_description: "After emitting bytecode the compiler runs a validator that checks stack balance, jump targets, register usage and constant-pool indices. The validator rejected the produced module because one of these invariants was violated.

This is almost always an internal compiler bug rather than a problem in user source. The recommended action is to capture the failing input and file an issue with the exact source plus the validator message.

If you are extending the compiler yourself, check that every code path pushes/pops the correct number of stack slots and that branch offsets are computed after all preceding instructions are emitted.",
        hints: &["This usually indicates a compiler bug — please file a minimal repro", "If you are hacking on the emitter, audit stack effects of new opcodes", "Run with HHS_LOG=debug to see the offending instruction stream"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_COMPILE_INVALID_BYTECODE_AT", "HHS_E_COMPILE_GENERIC"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub const COMPILE_INVALID_BYTECODE_AT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(37),
        long_code: "HHS_E_COMPILE_INVALID_BYTECODE_AT",
        short_code: "E0037",
        title: "Invalid bytecode at specific source location",
        short_description: "Bytecode validation failed at an instruction tied to a known source span.",
        long_description: "The bytecode validator rejected an instruction and was able to map it back to the source location whose lowering produced it. The error message describes which invariant was violated (bad stack depth, dangling jump, unknown constant index, etc.).

Report the failing source plus the validator message. If you are working on the compiler, the attached location is usually a precise indicator of which lowering rule needs to be fixed.

User-side workaround: rewrite the offending expression in a slightly different shape (e.g. extract a sub-expression to a let binding) — this often sidesteps the buggy code path while a real fix lands.",
        hints: &["Treat this as a compiler bug and file an issue with the snippet", "Try extracting sub-expressions into temporaries as a workaround", "Check if you are using a brand-new language feature with incomplete support"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_COMPILE_INVALID_BYTECODE", "HHS_E_COMPILE_GENERIC_AT"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub const COMPILE_RUNTIME_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(38),
        long_code: "HHS_E_COMPILE_RUNTIME_ERROR",
        short_code: "E0038",
        title: "Runtime error surfaced during compilation",
        short_description: "A runtime evaluation triggered from the compiler (e.g. const-eval) failed.",
        long_description: "Some compile-time work in HudHudScript needs to actually run code — for example evaluating a constant initializer, expanding a macro, or running a build-time check. That evaluation raised a runtime error which the compiler is now reporting.

Fix the underlying runtime problem in the code that the compiler tried to evaluate. The wrapped message is the original runtime error and usually contains the actionable detail (division by zero, index out of bounds, etc.).

If you did not intend a piece of code to run at compile time, check whether you accidentally placed it in a `const` context or a top-level initializer that the compiler eagerly evaluates.",
        hints: &["Read the wrapped runtime message — that is the real failure", "Constant initializers run at compile time; move side-effects out of them", "Wrap potentially-failing init code in a function called at startup instead"],
        example_bad: Some("const SIZE = 10 / 0;"),
        example_good: Some("const SIZE = 10;"),
        see_also: &["HHS_E_COMPILE_RUNTIME_ERROR_AT", "HHS_E_COMPILE_GENERIC"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub const COMPILE_RUNTIME_ERROR_AT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(39),
        long_code: "HHS_E_COMPILE_RUNTIME_ERROR_AT",
        short_code: "E0039",
        title: "Runtime error during compilation at source location",
        short_description: "Compile-time evaluation failed and the failing expression has a known location.",
        long_description: "The same as the runtime-error-during-compile variant, but the compiler captured the precise source span of the expression that failed when it was evaluated. The location points at the const/initializer/macro input, not at the consumer.

Go to the reported location and fix the runtime fault — for example replace a divisor of zero, guard an array access, or remove a panic from compile-time code paths.

This variant is your best lever for hunting down problematic constant evaluations because the span is exact.",
        hints: &["Open the highlighted line — that is what the compiler tried to evaluate", "Avoid panics, divisions by zero, and unchecked indexing in const init", "Move expensive or fallible work into a regular function called at runtime"],
        example_bad: Some("const FIRST = arr[0]; // arr is empty at compile time"),
        example_good: Some("const FIRST: Number = 0;"),
        see_also: &["HHS_E_COMPILE_RUNTIME_ERROR", "HHS_E_COMPILE_GENERIC_AT"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub const COMPILE_UNSUPPORTED_FEATURE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(40),
        long_code: "HHS_E_COMPILE_UNSUPPORTED_FEATURE",
        short_code: "E0040",
        title: "Language feature not yet supported by the compiler",
        short_description: "The compiler recognized a construct it does not yet know how to lower to bytecode.",
        long_description: "The parser and type checker accepted your code, but the bytecode emitter has not implemented the lowering rule for this construct. This typically affects features that are still being rolled out across the interpreter/VM parity boundary.

Until support lands, rewrite the snippet using an equivalent supported construct, or run the affected code through the tree-walking interpreter (which often supports features earlier than the VM does).

If the feature is documented as supported, this is a bug — please report it with a minimal repro.",
        hints: &["Check the language reference for which features the VM supports", "Try the tree-walking interpreter as a fallback for new features", "Refactor to a supported equivalent if you need to ship today"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_COMPILE_UNSUPPORTED_FEATURE_AT", "HHS_E_COMPILE_GENERIC"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub const COMPILE_UNSUPPORTED_FEATURE_AT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(41),
        long_code: "HHS_E_COMPILE_UNSUPPORTED_FEATURE_AT",
        short_code: "E0041",
        title: "Unsupported language feature at source location",
        short_description: "The compiler hit an unsupported construct and reports its precise source location.",
        long_description: "Same as the unsupported-feature error, but with an attached span pointing to the exact expression or statement the compiler does not yet handle. Use the span to find the offending construct and rewrite it.

If the highlighted feature is one you rely on, please open an issue with the location and the surrounding code so the team can prioritize the lowering rule.

Workarounds depend on the feature — typical strategies are to expand the construct manually, switch to an explicit form, or run the file under the interpreter.",
        hints: &["Inspect the reported line and rewrite the construct in a simpler form", "Some features only work in interpreted mode — switch backends if needed", "File an issue with the snippet so the lowering rule can be implemented"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_COMPILE_UNSUPPORTED_FEATURE", "HHS_E_COMPILE_GENERIC_AT"],
        since_version: "0.4.11",
        category: ErrorCategory::Compile,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    COMPILE_GENERIC,
    COMPILE_GENERIC_AT,
    COMPILE_INVALID_BYTECODE,
    COMPILE_INVALID_BYTECODE_AT,
    COMPILE_RUNTIME_ERROR,
    COMPILE_RUNTIME_ERROR_AT,
    COMPILE_UNSUPPORTED_FEATURE,
    COMPILE_UNSUPPORTED_FEATURE_AT,
];
