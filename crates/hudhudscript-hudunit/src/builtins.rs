//! The assertion prelude injected into every test run.
//!
//! Assertions are implemented in plain HudHudScript and fail via `throw`,
//! which surfaces as a runtime error from `VM::call_public` — the runner
//! turns that into a `Failed` outcome carrying the message. This keeps the
//! framework entirely outside the lexer/parser/VM: no builtin names are
//! registered, so the normal `hudhud` runtime is unaffected.
//!
//! Coverage marks use the same trick: `__hudunit_mark(id)` appends the id to
//! the global `__hudunit_seen` string, which the runner drains after each
//! test. Ids are passed as STRINGS — the VM currently rejects string+=number
//! concatenation inside functions (E0310), so string+string is the safe form.

/// Script source prepended (as a separate AST — spans of the user's file stay
/// untouched) to every test file.
///
/// Every assertion accepts a trailing optional `msg` argument
/// (`assert_eq(1, 2, "toplama kontrolü")`); missing arguments arrive as
/// `null` (verified VM behavior), which the guard skips.
pub const PRELUDE: &str = "\
let __hudunit_seen = \"\";
fn __hudunit_mark(id) {
    __hudunit_seen = ((__hudunit_seen + id) + \",\");
}
fn assert(cond, msg) {
    if (!cond) {
        if (msg != null) {
            throw (msg + \": \") + \"assert failed: value is not truthy\";
        }
        throw \"assert failed: value is not truthy\";
    }
}
fn assert_true(v, msg) {
    if (v != true) {
        if (msg != null) {
            throw (msg + \": \") + \"assert_true failed\";
        }
        throw \"assert_true failed\";
    }
}
fn assert_false(v, msg) {
    if (v != false) {
        if (msg != null) {
            throw (msg + \": \") + \"assert_false failed\";
        }
        throw \"assert_false failed\";
    }
}
fn assert_eq(got, want, msg) {
    if (got != want) {
        let base = (\"assert_eq failed: expected \" + want) + (\", got \" + got);
        if (msg != null) {
            throw (msg + \": \") + base;
        }
        throw base;
    }
}
fn assert_ne(a, b, msg) {
    if (a == b) {
        let base = \"assert_ne failed: both values are \" + a;
        if (msg != null) {
            throw (msg + \": \") + base;
        }
        throw base;
    }
}
fn assert_null(v, msg) {
    if (v != null) {
        if (msg != null) {
            throw (msg + \": \") + \"assert_null failed: value is not null\";
        }
        throw \"assert_null failed: value is not null\";
    }
}
fn assert_contains(haystack, needle, msg) {
    if (!haystack.contains(needle)) {
        let base = (\"assert_contains failed: \" + haystack) + (\" does not contain \" + needle);
        if (msg != null) {
            throw (msg + \": \") + base;
        }
        throw base;
    }
}
fn assert_length(v, n, msg) {
    if (len(v) != n) {
        let base = (\"assert_length failed: expected \" + n) + (\", got \" + len(v));
        if (msg != null) {
            throw (msg + \": \") + base;
        }
        throw base;
    }
}
fn assert_approx(got, want, msg) {
    let d = got - want;
    if (d < 0) {
        d = -d;
    }
    if (!(d <= 0.000000001)) {
        let base = (\"assert_approx failed: expected ~\" + want) + (\", got \" + got);
        if (msg != null) {
            throw (msg + \": \") + base;
        }
        throw base;
    }
}
fn assert_throws(f, msg) {
    try {
        f();
    } catch (err) {
        return;
    }
    if (msg != null) {
        throw (msg + \": \") + \"assert_throws failed: expected an exception, none was thrown\";
    }
    throw \"assert_throws failed: expected an exception, none was thrown\";
}
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_parses_as_valid_hudhudscript() {
        assert!(hudhudscript_parser::parse(PRELUDE).is_ok(), "prelude must parse");
    }
}
