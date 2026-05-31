//! External tests for `hudhudscript_parser::interner` module.

use hudhudscript_parser::{intern_identifier, SharedInterner, StringInterner};
use std::sync::Arc;

#[test]
fn test_intern_deduplicates() {
    let mut interner = StringInterner::new();
    let a = interner.intern("hello");
    let b = interner.intern("hello");
    assert!(Arc::ptr_eq(&a, &b), "same string should return same Arc");
}

#[test]
fn test_intern_different_strings() {
    let mut interner = StringInterner::new();
    let a = interner.intern("hello");
    let b = interner.intern("world");
    assert!(!Arc::ptr_eq(&a, &b));
    assert_eq!(&*a, "hello");
    assert_eq!(&*b, "world");
}

#[test]
fn test_intern_len() {
    let mut interner = StringInterner::new();
    assert_eq!(interner.len(), 0);
    interner.intern("a");
    interner.intern("b");
    interner.intern("a"); // duplicate — should not grow
    assert_eq!(interner.len(), 2);
}

#[test]
fn test_shared_interner_thread_safety() {
    let interner = SharedInterner::new();
    let i1 = interner.clone();
    let i2 = interner.clone();

    let h1 = std::thread::spawn(move || {
        for _ in 0..100 {
            i1.intern("shared_string");
        }
    });
    let h2 = std::thread::spawn(move || {
        for _ in 0..100 {
            i2.intern("shared_string");
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    // Only one entry should exist
    assert_eq!(interner.len(), 1);
}

#[test]
fn test_intern_identifier_returns_string() {
    let mut interner = StringInterner::new();
    let s = intern_identifier(&mut interner, "provider");
    assert_eq!(s, "provider");
}

#[test]
fn test_empty() {
    let interner = StringInterner::new();
    assert!(interner.is_empty());
}

#[test]
fn test_not_empty_after_intern() {
    let mut interner = StringInterner::new();
    interner.intern("x");
    assert!(!interner.is_empty());
}

#[test]
fn test_reserve() {
    let mut interner = StringInterner::new();
    interner.reserve(100);
    // Should not panic and should still function
    let a = interner.intern("test");
    assert_eq!(&*a, "test");
}

#[test]
fn test_intern_empty_string() {
    let mut interner = StringInterner::new();
    let a = interner.intern("");
    let b = interner.intern("");
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(&*a, "");
    assert_eq!(interner.len(), 1);
}

#[test]
fn test_intern_unicode() {
    let mut interner = StringInterner::new();
    let a = interner.intern("متغير");
    let b = interner.intern("متغير");
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(&*a, "متغير");
}

#[test]
fn test_intern_many_distinct() {
    let mut interner = StringInterner::new();
    for i in 0..50 {
        interner.intern(&format!("var_{}", i));
    }
    assert_eq!(interner.len(), 50);
}

#[test]
fn test_intern_identifier_multiple() {
    let mut interner = StringInterner::new();
    let s1 = intern_identifier(&mut interner, "foo");
    let s2 = intern_identifier(&mut interner, "foo");
    assert_eq!(s1, s2);
    assert_eq!(s1, "foo");
    assert_eq!(interner.len(), 1);
}

#[test]
fn test_shared_interner_is_empty() {
    let interner = SharedInterner::new();
    assert!(interner.is_empty());
    interner.intern("x");
    assert!(!interner.is_empty());
}

#[test]
fn test_shared_interner_len() {
    let interner = SharedInterner::new();
    assert_eq!(interner.len(), 0);
    interner.intern("a");
    interner.intern("b");
    interner.intern("a"); // duplicate
    assert_eq!(interner.len(), 2);
}

#[test]
fn test_shared_interner_deduplicates() {
    let interner = SharedInterner::new();
    let a = interner.intern("hello");
    let b = interner.intern("hello");
    assert!(Arc::ptr_eq(&a, &b));
}
