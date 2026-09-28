//! SOP olay-adı kaydı (M6, v0.9.39). Pre-pass (`hir_loop`) effect adlarını
//! buraya yazar; `hir_expr_lower` çağrı callee'sini eşleşen olay için
//! `__event__X`'e çömler — VM'in `ability::/effect` çağrı denklemi.

use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static EVENT_NAMES: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

pub(crate) fn set_event_names(names: Vec<String>) {
    EVENT_NAMES.with(|s| {
        *s.borrow_mut() = names.into_iter().collect();
    });
}

/// Çağrı adı SOP olayıysa `__event__X`'e çözümler (M6).
pub(crate) fn resolve_call_callee(name: &str) -> Option<String> {
    EVENT_NAMES.with(|s| {
        if s.borrow().contains(name) {
            Some(format!("__event__{name}"))
        } else {
            None
        }
    })
}
