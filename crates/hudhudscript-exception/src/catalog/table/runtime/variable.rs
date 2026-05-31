use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RUNTIME_IMMUTABLE_VARIABLE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(233),
        long_code: "HHS_E_RUNTIME_IMMUTABLE_VARIABLE",
        short_code: "E0233",
        title: "Assignment to immutable variable",
        short_description: "An assignment targeted a variable declared as immutable (via `let` or `const`) rather than mutable (`var` or `let mut`).",
        long_description: "HudHudScript distinguishes mutable and immutable bindings at declaration. `const` and plain `let` create immutable bindings; only `var` (or `let mut`, depending on dialect) allows reassignment. Attempting to assign to an immutable binding is a runtime error when it escapes static analysis, or reported here when discovered dynamically.

To fix, either declare the variable as mutable from the start, or restructure the code to avoid reassignment (e.g., introduce a new binding with a different name). Preferring immutability is idiomatic and helps the optimizer.

Note that immutability of the binding is separate from mutability of the referent: an immutable binding to a list still allows mutating the list's contents via methods, because only the binding is frozen.",
        hints: &["Declare the variable with `var` (or `let mut`) if you need reassignment", "Prefer creating a new binding with `let` over mutating an old one", "Immutable binding still allows mutating fields of the referent", "Check for accidental shadowing when you intended to reassign"],
        example_bad: Some("let x = 1;
x = 2;"),
        example_good: Some("var x = 1;
x = 2;"),
        see_also: &["HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED", "HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_UNINITIALIZED_VARIABLE"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_INDEX_OUT_OF_BOUNDS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(234),
        long_code: "HHS_E_RUNTIME_INDEX_OUT_OF_BOUNDS",
        short_code: "E0234",
        title: "Index out of bounds",
        short_description: "An index expression addressed a position outside the valid range `[0, length)` of the collection.",
        long_description: "HudHudScript collections are bounds-checked at every access. Reading or writing `coll[i]` with `i < 0` or `i >= coll.len()` raises this error. The message carries both the offending index and the collection's length for debugging.

Fix by validating the index before use, by using a safe accessor like `coll.get(i)` that returns `null` for out-of-range indices, or by iterating with a constructs that cannot go out of range (e.g., `for x in coll`).

Off-by-one bugs at the edges (`coll[coll.len()]`) and reliance on signed arithmetic that can produce negative indices are the most common sources.",
        hints: &["Use `coll.get(i)` for safe access that returns `null` on OOB", "Prefer iteration over manual indexing when possible", "Check for off-by-one: the last valid index is `len - 1`", "Negative indices are not wrapped — they raise this error"],
        example_bad: Some("let xs = [1, 2, 3];
print(xs[3]);"),
        example_good: Some("let xs = [1, 2, 3];
if let Some(v) = xs.get(3) { print(v); }"),
        see_also: &["HHS_E_RUNTIME_TYPE_ERROR", "HHS_E_RUNTIME_PROPERTY_NOT_FOUND", "HHS_E_RUNTIME_INVALID_OPERATION"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_UNDEFINED_VARIABLE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(251),
        long_code: "HHS_E_RUNTIME_UNDEFINED_VARIABLE",
        short_code: "E0251",
        title: "Reference to undefined variable",
        short_description: "The interpreter tried to read a variable that has not been declared in any visible scope.",
        long_description: "At runtime, an expression referenced a name for which no binding exists. The variable might be misspelled, declared in a different scope, used before its `let`/`const`/`var` declaration is reached, or removed by a refactor. The interpreter reports the offending name and source position.

To fix this, declare the variable before use, correct the spelling, or import it from the module that defines it. Unlike a parse error, this is detected only when execution reaches the offending expression — branches that are never taken will not raise it.

For module-qualified references, verify that the module is actually imported and that the symbol is exported from it.",
        hints: &["Check spelling and capitalization of the identifier", "Verify the variable is declared in an enclosing scope, not a sibling block", "If declared with `let`, the binding only exists after its declaration", "For module symbols, verify the `import` and that it is exported"],
        example_bad: Some("fn main() {
    println(usrname);
}"),
        example_good: Some("fn main() {
    let username = \"alice\";
    println(username);
}"),
        see_also: &["HHS_E_RUNTIME_UNINITIALIZED_VARIABLE", "HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED", "HHS_E_RUNTIME_PROPERTY_NOT_FOUND"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_UNINITIALIZED_VARIABLE: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(252),
        long_code: "HHS_E_RUNTIME_UNINITIALIZED_VARIABLE",
        short_code: "E0252",
        title: "Access before initialization (temporal dead zone)",
        short_description: "A variable was read before its declaration in the same scope executed — the binding exists but has no value yet.",
        long_description: "HudHudScript implements a temporal dead zone (TDZ) for `let`/`const` bindings: the binding is considered to exist from the start of its enclosing scope, but any read before its declaration statement runs is an error. This catches code that accidentally uses a name that will be declared later as if it were a global or a hoisted `var`.

Fix by moving the read below the declaration, or by declaring the variable earlier in the scope with an initial value. Do not rely on hoisting — `let`/`const` bindings are not initialized to a default.

This error is distinct from `UndefinedVariable`: the binding does exist here, it just has not been assigned yet.",
        hints: &["Move the use below the declaration", "Initialize the variable when declaring it, not later", "Do not rely on hoisting — `let`/`const` are not hoisted like `var`", "Check nested blocks: TDZ applies per scope"],
        example_bad: Some("fn main() {
    println(x);
    let x = 1;
}"),
        example_good: Some("fn main() {
    let x = 1;
    println(x);
}"),
        see_also: &["HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_IMMUTABLE_VARIABLE", "HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_VARIABLE_ALREADY_DEFINED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(253),
        long_code: "HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED",
        short_code: "E0253",
        title: "Variable already defined in scope",
        short_description: "A declaration collided with an existing binding of the same name in the same lexical scope.",
        long_description: "HudHudScript forbids re-declaring a name within the same scope. The second `let`/`const`/`var` hits this error because silent shadowing in the same block is a bug magnet. Shadowing in a nested inner scope is still allowed — only collisions in the same block are rejected.

Fix by choosing a different name, by moving the second declaration into an inner block where shadowing is permitted, or by converting the first declaration to `var` and reassigning rather than redeclaring.

If you are converting code from a language that allows free re-declaration, the fix is almost always to rename the second binding.",
        hints: &["Pick a different name for the second declaration", "Move the second declaration into a nested block if shadowing is intended", "Use reassignment with `var` instead of re-declaring", "Check for accidental copies/pastes that duplicated the declaration"],
        example_bad: Some("let x = 1;
let x = 2;"),
        example_good: Some("let x = 1;
{ let x = 2; /* inner shadow */ }"),
        see_also: &["HHS_E_RUNTIME_IMMUTABLE_VARIABLE", "HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_UNINITIALIZED_VARIABLE"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };
