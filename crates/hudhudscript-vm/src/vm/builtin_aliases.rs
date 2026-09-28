//! VM yerleşik ad eş anlamlıları — tek doğruluk kaynağı artık
//! `hudhudscript-localization::builtin_aliases` (v0.9.34, M1 taşınması).
//! Bu modül yalnızca yeniden ihraç eder: VM içi kullanım noktaları
//! (dispatch_core.rs, util.rs) aynı yol/alanlarla çalışmaya devam eder.

pub(crate) use hudhudscript_localization::builtin_aliases::{
    is_eprint_alias, is_eprintln_alias, is_input_alias, is_print_alias, is_println_alias,
    is_putf_alias, is_put_alias, canonical_builtin, EPRINT_ALIASES, EPRINTLN_ALIASES,
    INPUT_ALIASES, PRINT_ALIASES, PRINTLN_ALIASES, PUTF_ALIASES, PUT_ALIASES,
};
