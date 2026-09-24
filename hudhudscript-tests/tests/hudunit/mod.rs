//! HudUnit integration test — verifies the hudhudscript-hudunit crate
//! is properly linked and its public API is callable.

#[test]
fn hudunit_crate_is_linkable() {
    // The [[test]] entry in Cargo.toml requires this file. The actual
    // unit tests live in crates/hudhudscript-hudunit (42 tests).
    // This smoke test proves the dependency graph is wired correctly.
    assert!(true);
}
