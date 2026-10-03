//! Tests for hudhudscript-localization builtin alias resolution
//! (moved from src/builtin_aliases.rs).

use hudhudscript_localization::builtin_aliases::canonical_builtin;

#[test]
fn aliases_resolve_to_canonical() {
    assert_eq!(canonical_builtin("yazdır"), Some("print"));
    assert_eq!(canonical_builtin("打印"), Some("print"));
    assert_eq!(canonical_builtin("اطبع"), Some("print"));
    assert_eq!(canonical_builtin("print"), Some("print"));
    assert_eq!(canonical_builtin("satıryaz"), Some("println"));
    assert_eq!(canonical_builtin("oku"), Some("input"));
}

#[test]
fn unknown_names_stay_unknown() {
    assert_eq!(canonical_builtin("fogsgon"), None);
    assert_eq!(canonical_builtin("main"), None);
}
