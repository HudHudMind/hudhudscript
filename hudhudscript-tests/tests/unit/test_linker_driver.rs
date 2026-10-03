//! Tests for hudhudscript-linker — host linker discovery and the C entry
//! shim generated for native executables.

use hudhudscript_linker::{entry_shim, find_linker};

// ── linker discovery ─────────────────────────────────────────────

#[test]
fn finds_linker_on_host() {
    assert!(find_linker().is_ok());
}

// ── entry shim ───────────────────────────────────────────────────

#[test]
fn entry_shim_calls_entry_symbol() {
    let shim = entry_shim(Some("hudhud__hudhud_init"), None);
    assert!(shim.contains("hudhud__hudhud_init"));
    assert!(shim.contains("int main(void)"));
}
