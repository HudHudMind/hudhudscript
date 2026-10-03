//! Tests for hudhudscript-cli startup thread-stack resolution (G09):
//! env-over-TOML precedence, bounds, byte conversion, and thread naming.

use hudhudscript_cli::common::startup::{resolve_thread_stack_mb, run_with_stack, stack_bytes};

// ── precedence ────────────────────────────────────────────────────

#[test]
fn stack_env_overrides_toml() {
    assert_eq!(resolve_thread_stack_mb(Some("128"), 32).unwrap(), Some(128));
}

#[test]
fn stack_toml_used_when_env_absent() {
    assert_eq!(resolve_thread_stack_mb(None, 32).unwrap(), Some(32));
}

#[test]
fn stack_default_is_64_when_both_absent() {
    assert_eq!(resolve_thread_stack_mb(None, 64).unwrap(), Some(64));
}

#[test]
fn stack_zero_disables_child_thread() {
    assert_eq!(resolve_thread_stack_mb(Some("0"), 64).unwrap(), None);
    assert_eq!(resolve_thread_stack_mb(None, 0).unwrap(), None);
}

#[test]
fn stack_invalid_env_preserves_64mb_compatibility() {
    assert_eq!(
        resolve_thread_stack_mb(Some("not-a-number"), 64).unwrap(),
        Some(64)
    );
    assert_eq!(resolve_thread_stack_mb(Some("  "), 64).unwrap(), Some(64));
}

// ── bounds and conversion ─────────────────────────────────────────

#[test]
fn stack_upper_bound_is_rejected() {
    assert!(resolve_thread_stack_mb(Some("2048"), 64).is_err());
    assert!(resolve_thread_stack_mb(None, 1025).is_err());
    assert!(resolve_thread_stack_mb(Some("1024"), 64).is_ok());
}

#[test]
fn stack_byte_conversion_is_checked() {
    assert_eq!(stack_bytes(64).unwrap(), 64 * 1024 * 1024);
    assert_eq!(stack_bytes(1024).unwrap(), 1024 * 1024 * 1024);
    // checked_mul only overflows on 32-bit targets; on 64-bit the
    // full u32 MB range converts exactly.
    if std::mem::size_of::<usize>() < 8 {
        assert_eq!(
            stack_bytes(u32::MAX).unwrap_err(),
            "thread stack byte conversion overflowed"
        );
    } else {
        assert_eq!(
            stack_bytes(u32::MAX).unwrap(),
            u32::MAX as usize * 1024 * 1024
        );
    }
}

// ── spawned thread ────────────────────────────────────────────────

#[test]
fn spawned_thread_is_named_hudhud_main() {
    let name = std::sync::Arc::new(std::sync::Mutex::new(None));
    let observed = std::sync::Arc::clone(&name);
    run_with_stack(8, move || {
        *observed.lock().unwrap() = Some(std::thread::current().name().map(str::to_string));
    })
    .unwrap();
    assert_eq!(
        name.lock()
            .unwrap()
            .as_ref()
            .and_then(|inner| inner.as_deref()),
        Some("hudhud-main")
    );
}
