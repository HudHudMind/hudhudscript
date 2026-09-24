//! Exception handling ABI and thread-local state.

thread_local! {
    static CURRENT_EXCEPTION: std::cell::Cell<Option<i64>> = const { std::cell::Cell::new(None) };
}

/// Set active exception payload
#[no_mangle]
pub unsafe extern "C" fn hudhud_throw(val: i64) {
    CURRENT_EXCEPTION.with(|c| c.set(Some(val)));
}

/// Check if an exception is currently in flight (1 if true, 0 if false)
#[no_mangle]
pub unsafe extern "C" fn hudhud_has_exception() -> i64 {
    CURRENT_EXCEPTION.with(|c| if c.get().is_some() { 1 } else { 0 })
}

/// Retrieve and clear the current exception payload
#[no_mangle]
pub unsafe extern "C" fn hudhud_catch() -> i64 {
    CURRENT_EXCEPTION.with(|c| c.replace(None).unwrap_or(0))
}

pub fn take_exception() -> Option<i64> {
    CURRENT_EXCEPTION.with(|c| c.replace(None))
}

pub fn has_active_exception() -> bool {
    CURRENT_EXCEPTION.with(|c| c.get().is_some())
}
