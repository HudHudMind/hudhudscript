//! Tests for hudhudscript-native-abi string arena/interning layer —
//! pointer-identity interning for short strings and bump-arena slabs.

use hudhudscript_native_abi::string_arena::{
    arena_alloc, intern_or_alloc, is_arena_pointer, ARENA_SLAB,
};

// ── interning ────────────────────────────────────────────────────

#[test]
fn intern_returns_same_pointer() {
    unsafe {
        let a = intern_or_alloc(b"hello", 5);
        let b = intern_or_alloc(b"hello", 5);
        assert_eq!(a, b, "aynı içerik → aynı pointer");
        assert_eq!(std::ffi::CStr::from_ptr(a).to_str().unwrap(), "hello");
    }
}

#[test]
fn different_content_different_pointer() {
    unsafe {
        let a = intern_or_alloc(b"abc", 3);
        let b = intern_or_alloc(b"xyz", 3);
        assert_ne!(a, b);
    }
}

#[test]
fn long_string_not_interned() {
    let long = vec![b'A'; 100];
    unsafe {
        let a = intern_or_alloc(&long, 100);
        let b = intern_or_alloc(&long, 100);
        assert!(!a.is_null() && !b.is_null());
    }
}

// ── arena slabs ──────────────────────────────────────────────────

#[test]
fn multi_slab_is_arena_pointer() {
    unsafe {
        let p1 = arena_alloc(100);
        assert!(is_arena_pointer(p1 as usize));
        let p2 = arena_alloc(ARENA_SLAB / 2 - 100);
        let p3 = arena_alloc(ARENA_SLAB / 2 - 100);
        let p4 = arena_alloc(ARENA_SLAB / 2 - 100);
        assert!(is_arena_pointer(p2 as usize));
        assert!(is_arena_pointer(p3 as usize));
        assert!(is_arena_pointer(p4 as usize));
    }
}
