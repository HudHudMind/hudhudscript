//! Tests for hudhudscript-hudunit assertion prelude — parseability of the
//! injected script source.

use hudhudscript_hudunit::builtins::PRELUDE;

#[test]
fn prelude_parses_as_valid_hudhudscript() {
    assert!(hudhudscript_parser::parse(PRELUDE).is_ok(), "prelude must parse");
}
