use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const TYPE_AWAIT_IN_ATOMICALLY: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(299),
        long_code: "HHS_E_TYPE_AWAIT_IN_ATOMICALLY",
        short_code: "E0299",
        title: "await is not allowed inside atomically blocks",
        short_description: "An await expression appears inside an STM atomically block, which would break transaction isolation.",
        long_description: "HudHudScript's STM transactions (`atomically { ... }`) must execute synchronously from the runtime's perspective. Suspending in the middle of a transaction would let other tasks observe inconsistent intermediate state and would make retry semantics meaningless. The type checker therefore rejects any `await` that appears, statically or dynamically, inside an atomically block.

Move the awaited operation outside the transaction: compute the future first, await it, then enter `atomically` with the resulting value. If you need the result of the await inside the transaction, pass it in via a local binding.

This rule is part of the concurrency hierarchy: STM lives outside async, never inside it. Actor sends are likewise restricted within atomically blocks for the same reason.",
        hints: &["Await the future before entering atomically, then use the resolved value", "Never call async fns from inside atomically — wrap them outside", "Pure data and TVar reads/writes are the only things STM blocks should do"],
        example_bad: Some("atomically {
  let v = await fetch_balance();
  account.balance = v;
}"),
        example_good: Some("let v = await fetch_balance();
atomically {
  account.balance = v;
}"),
        see_also: &["HHS_E_TYPE_INVALID_AWAIT"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_DUPLICATE_VARIABLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(300),
        long_code: "HHS_E_TYPE_DUPLICATE_VARIABLE",
        short_code: "E0300",
        title: "Variable declared more than once in the same scope",
        short_description: "Two `let`/`const`/`var` declarations in the same scope use the same name.",
        long_description: "The type checker found two declarations introducing the same identifier in a single scope. HudHudScript does not allow shadowing within the same block — you must either rename one of the bindings or move it into a nested scope.

Rename one of the variables, or wrap the second declaration in `{ ... }` to create a fresh scope. If you intended to update the original variable, drop the `let` and assign to it directly (provided it was declared with `let` or `var`, not `const`).

This check fires before code generation so neither the interpreter nor the VM will see the conflicting bindings.",
        hints: &["Rename one of the duplicate bindings to disambiguate", "Drop the second 'let' if you meant to reassign, not redeclare", "Wrap a redeclaration in a nested { } block to create a fresh scope"],
        example_bad: Some("let x = 1;
let x = 2;"),
        example_good: Some("let x = 1;
x = 2;"),
        see_also: &["HHS_E_TYPE_UNDEFINED_VARIABLE"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_INVALID_AWAIT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(301),
        long_code: "HHS_E_TYPE_INVALID_AWAIT",
        short_code: "E0301",
        title: "await applied to a non-promise value",
        short_description: "The expression after `await` does not have a Promise type.",
        long_description: "`await` only operates on values of type `Promise<T>`. The type checker found an `await` whose operand has a different type — for example a plain `Number`, `String`, or a function value that was never called.

Fix it by awaiting the right thing. A common mistake is forgetting the parentheses on a function call: `await fetch` awaits the function itself, while `await fetch()` awaits the promise it returns. Another common case is awaiting a value that has already been resolved — simply remove the `await`.

If you genuinely need a promise, wrap the value with the appropriate constructor or call the async function that produces it.",
        hints: &["Make sure you are calling the function: 'await f()' not 'await f'", "Drop the await if the value is already resolved", "Only async fns and promise-returning APIs produce Promise<T>"],
        example_bad: Some("let result = await 42;"),
        example_good: Some("let result = await fetch_value();"),
        see_also: &["HHS_E_TYPE_AWAIT_IN_ATOMICALLY", "HHS_E_TYPE_MISMATCH"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_INVALID_INDEX: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(302),
        long_code: "HHS_E_TYPE_INVALID_INDEX",
        short_code: "E0302",
        title: "Indexing applied to a non-indexable type",
        short_description: "The `[ ]` indexing operator was used on a value whose type does not support indexing.",
        long_description: "Index expressions like `xs[i]` are only valid for arrays, strings, maps, and other types that explicitly implement indexing. The type checker found an indexing operation on a value whose type does not.

Fix it by using the right access pattern for the type: dot notation for struct/class fields, method calls for collections without index access, or convert the value to an indexable type first.

If you expected the value to be an array or map, the actual mistake is probably upstream — trace where the value comes from and check its declared or inferred type.",
        hints: &["Use '.field' for struct/class members, not '[\"field\"]'", "Maps are indexed with their key type; arrays with Number", "Check upstream — the value may not be the collection type you expected"],
        example_bad: Some("let n: Number = 42;
let x = n[0];"),
        example_good: Some("let xs: Array<Number> = [42];
let x = xs[0];"),
        see_also: &["HHS_E_TYPE_INVALID_MEMBER", "HHS_E_TYPE_MISMATCH"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_INVALID_MEMBER: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(303),
        long_code: "HHS_E_TYPE_INVALID_MEMBER",
        short_code: "E0303",
        title: "Member access on a type without that field",
        short_description: "A `.field` access references a member that the value's type does not have.",
        long_description: "The type checker resolved the receiver's type but found that it has no field, property, or method with the requested name. This can be a typo in the member name, a missing import that hides the type's real shape, or an attempt to use a method from a different class.

Double-check the spelling and the type of the receiver. Hovering over the variable in an IDE shows its inferred type and available members. If the member exists on a subclass, you may need to downcast or test the runtime type first.

For optional values, use the `?.` operator — `obj?.field` short-circuits to `null` rather than raising a type error when the type is `T?`.",
        hints: &["Verify spelling — member lookup is case-sensitive", "Hover over the receiver in your IDE to see its real type and members", "Use ?. for optional chaining when the receiver might be null"],
        example_bad: Some("class Dog { name: String; }
let d = Dog { name: \"Rex\" };
d.bark();"),
        example_good: Some("class Dog { name: String; fn bark(self) {} }
let d = Dog { name: \"Rex\" };
d.bark();"),
        see_also: &["HHS_E_TYPE_INVALID_INDEX", "HHS_E_TYPE_UNDEFINED_FUNCTION"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_INVALID_OPERATOR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(304),
        long_code: "HHS_E_TYPE_INVALID_OPERATOR",
        short_code: "E0304",
        title: "Operator not defined for operand type",
        short_description: "A binary or unary operator was applied to a type that does not support it.",
        long_description: "The type checker found an operator whose operand types are not compatible with it — for example multiplying two strings, or applying unary minus to a boolean. HudHudScript only allows operators on the types they are defined for; there is no implicit coercion between unrelated types.

Convert the operands to a compatible type or use a different operation. For string concatenation, use `+` only between strings (or call an explicit `to_string()` first). For numeric work, ensure both sides are `Number`.

If you are working with a custom class, check whether the operator has been implemented for it; if not, define it or call a regular method instead.",
        hints: &["Convert operands explicitly with to_string() / to_number() as needed", "+ requires both sides to be Number or both to be String", "Custom classes may need an explicit operator implementation"],
        example_bad: Some("let x = \"hello\" - 1;"),
        example_good: Some("let x = \"hello\".len() - 1;"),
        see_also: &["HHS_E_TYPE_MISMATCH"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_MISMATCH: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(305),
        long_code: "HHS_E_TYPE_MISMATCH",
        short_code: "E0305",
        title: "Type mismatch between expected and actual",
        short_description: "An expression has a different type than the surrounding context requires.",
        long_description: "The type checker computed two types — the one required by the context (a parameter, a return type, an assignment target) and the one produced by the expression — and they are not compatible. The error message names both.

Fix it by either changing the expression to produce the expected type or by changing the surrounding declaration to accept what you actually have. Common fixes include adding an explicit conversion, removing an unwanted layer (e.g. forgetting to call `.unwrap()`), or correcting a function return annotation.

If the inferred type surprises you, hover over each sub-expression in your IDE to see where the mismatch is introduced.",
        hints: &["Read both 'expected' and 'found' — they tell you what to convert", "Check if you forgot to call a function (T vs fn() -> T)", "Optional T? and T are distinct — use ?? or ?. to bridge them"],
        example_bad: Some("fn add(a: Number, b: Number) -> Number { a + b }
let x: String = add(1, 2);"),
        example_good: Some("fn add(a: Number, b: Number) -> Number { a + b }
let x: Number = add(1, 2);"),
        see_also: &["HHS_E_TYPE_TYPE_MISMATCH_IN_DECL", "HHS_E_TYPE_INVALID_OPERATOR"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_TYPE_MISMATCH_IN_DECL: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(306),
        long_code: "HHS_E_TYPE_TYPE_MISMATCH_IN_DECL",
        short_code: "E0306",
        title: "Declared type does not match initializer",
        short_description: "A `let`/`const`/`var` declaration's annotated type disagrees with its initializer's type.",
        long_description: "You wrote an explicit type annotation on a binding, but the initializer expression produces a different type. The checker reports both the annotation and the inferred initializer type so you can decide which one is wrong.

Either correct the annotation to match the initializer, or change the initializer (or convert it) to match the annotation. If the binding name appears in the message, it identifies the exact declaration to fix.

A frequent cause is annotating an integer-looking value as `Number` when it is actually a `String` literal, or annotating an `Array<T>` with the wrong element type.",
        hints: &["Drop the annotation to let inference confirm the real initializer type", "Convert the initializer with to_string()/to_number() if needed", "Check element types when annotating arrays and maps"],
        example_bad: Some("let count: Number = \"five\";"),
        example_good: Some("let count: Number = 5;"),
        see_also: &["HHS_E_TYPE_MISMATCH"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_UNDEFINED_FUNCTION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(307),
        long_code: "HHS_E_TYPE_UNDEFINED_FUNCTION",
        short_code: "E0307",
        title: "Call to an undefined function",
        short_description: "The type checker could not find any function with the called name in scope.",
        long_description: "A call expression references a function that is not declared in the current scope and is not imported from any visible module. This is usually a typo, a missing import, or a call placed before the function is in scope.

Fix it by checking the spelling, importing the module that defines the function, or moving the function declaration so it is visible at the call site. If the function is a method on a value, use dot notation (`receiver.method()`) instead of a free function call.

For stdlib functions, make sure the relevant module is imported — HudHudScript does not auto-import everything.",
        hints: &["Check spelling — identifier lookup is case-sensitive", "Did you forget an 'import' for a stdlib or user module?", "If it's a method, call it as receiver.method() not method(receiver)"],
        example_bad: Some("let n = lenght(\"hi\");"),
        example_good: Some("let n = length(\"hi\");"),
        see_also: &["HHS_E_TYPE_UNDEFINED_VARIABLE", "HHS_E_TYPE_WRONG_ARGUMENT_COUNT"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_UNDEFINED_VARIABLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(308),
        long_code: "HHS_E_TYPE_UNDEFINED_VARIABLE",
        short_code: "E0308",
        title: "Reference to an undefined variable",
        short_description: "An identifier is used as a value but no binding with that name is in scope.",
        long_description: "The type checker could not find any `let`, `const`, `var`, function parameter, or imported binding matching the referenced name. The reference may be a typo, a use-before-declaration, or a leak across scopes that the checker correctly forbids.

Fix it by declaring the variable, correcting the spelling, or moving the use into the scope where the binding lives. If you intended to access a class field, prefix it with `self.`. If you intended a function call, see the undefined-function error.

For module-level imports, ensure the symbol you want is actually exported by the module you imported from.",
        hints: &["Check spelling and case", "Use 'self.field' to read a field from inside a method", "Make sure the variable is declared in a scope that contains this use"],
        example_bad: Some("fn main() { print(name); }"),
        example_good: Some("fn main() { let name = \"world\"; print(name); }"),
        see_also: &["HHS_E_TYPE_UNDEFINED_FUNCTION", "HHS_E_TYPE_DUPLICATE_VARIABLE"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_WRONG_ARGUMENT_COUNT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(309),
        long_code: "HHS_E_TYPE_WRONG_ARGUMENT_COUNT",
        short_code: "E0309",
        title: "Wrong number of arguments in function call",
        short_description: "A function was called with more or fewer arguments than its declaration accepts.",
        long_description: "The type checker compared the call's argument count against the resolved function's arity and they did not match. The message reports the expected number and the number actually supplied.

Add or remove arguments to match the signature. If the function has overloads, double-check which overload you intended to call. For methods, remember that the implicit `self` argument is not written at the call site but is part of the declared parameter list.

If you intended to pass a variable number of arguments, the function must explicitly declare a variadic or array parameter — HudHudScript does not silently accept extras.",
        hints: &["Hover the function to see its full signature, then match the call", "Methods take an implicit self — don't pass it explicitly at the call", "Pack extras into an Array<T> if the function takes a list parameter"],
        example_bad: Some("fn add(a: Number, b: Number) -> Number { a + b }
let x = add(1);"),
        example_good: Some("fn add(a: Number, b: Number) -> Number { a + b }
let x = add(1, 2);"),
        see_also: &["HHS_E_TYPE_UNDEFINED_FUNCTION", "HHS_E_TYPE_MISMATCH"],
        since_version: "0.4.0",
        category: ErrorCategory::Type,
    };

pub const TYPE_NON_EXHAUSTIVE_MATCH: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(324),
        long_code: "HHS_E_TYPE_NON_EXHAUSTIVE_MATCH",
        short_code: "E0324",
        title: "Non-exhaustive match on union type",
        short_description: "A match expression on a union type does not cover all variant types and has no wildcard arm.",
        long_description: "When matching on a value whose type is a union (e.g. `String | Number | Boolean`), every variant type should be handled by at least one match arm, or a wildcard (`_`) arm should be present to catch unmatched variants. Without full coverage, values of uncovered types will fall through without being handled, which is likely a bug.

Add match arms for the missing variant types, or add a wildcard `_ => { ... }` arm as a catch-all. The warning message lists which variant types are not covered.

This check only applies when the match subject has a statically known union type. If the subject is `Any`, no exhaustiveness warning is emitted.",
        hints: &["Add a match arm for each missing variant type", "Add a wildcard '_ => { ... }' arm to handle all remaining cases", "This warning only fires for union types — enum exhaustiveness is separate"],
        example_bad: Some("let x: String | Number = get_value();
match x {
  \"hello\" => { print(\"string\"); }
}"),
        example_good: Some("let x: String | Number = get_value();
match x {
  \"hello\" => { print(\"string\"); }
  _ => { print(\"other\"); }
}"),
        see_also: &["HHS_E_TYPE_MISMATCH"],
        since_version: "0.4.50",
        category: ErrorCategory::Type,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    TYPE_AWAIT_IN_ATOMICALLY,
    TYPE_DUPLICATE_VARIABLE,
    TYPE_INVALID_AWAIT,
    TYPE_INVALID_INDEX,
    TYPE_INVALID_MEMBER,
    TYPE_INVALID_OPERATOR,
    TYPE_MISMATCH,
    TYPE_NON_EXHAUSTIVE_MATCH,
    TYPE_TYPE_MISMATCH_IN_DECL,
    TYPE_UNDEFINED_FUNCTION,
    TYPE_UNDEFINED_VARIABLE,
    TYPE_WRONG_ARGUMENT_COUNT,
];
