use std::ffi::{CStr, CString};
use hudhudscript_native_abi::*;

fn to_c(s: &str) -> *mut std::ffi::c_char {
    let c = CString::new(s).unwrap();
    let ptr = c.into_raw();
    hudhudscript_native_abi::register_string(ptr as usize);
    ptr
}

unsafe fn from_c(p: *const std::ffi::c_char) -> String {
    CStr::from_ptr(p).to_str().unwrap().to_string()
}

#[test]
fn test_string_trim_case_slice() {
    unsafe {
        let s = to_c("  Hello World!  ");
        let trimmed = hudhud_string_trim(s);
        assert_eq!(from_c(trimmed), "Hello World!");

        let lower = hudhud_string_to_lower(trimmed);
        assert_eq!(from_c(lower), "hello world!");

        let upper = hudhud_string_to_upper(trimmed);
        assert_eq!(from_c(upper), "HELLO WORLD!");

        assert_eq!(hudhud_string_starts_with(trimmed, to_c("Hello")), 1);
        assert_eq!(hudhud_string_starts_with(trimmed, to_c("World")), 0);
        assert_eq!(hudhud_string_ends_with(trimmed, to_c("World!")), 1);
        assert_eq!(hudhud_string_contains(trimmed, to_c("lo Wo")), 1);
        assert_eq!(hudhud_string_contains(trimmed, to_c("xyz")), 0);

        let replaced = hudhud_string_replace(trimmed, to_c("World"), to_c("HudHud"));
        assert_eq!(from_c(replaced), "Hello HudHud!");

        assert_eq!(hudhud_string_char_code_at(trimmed, 0), 72); // 'H'
        assert_eq!(hudhud_string_char_code_at(trimmed, 1), 101); // 'e'

        hudhud_string_free(s);
        hudhud_string_free(trimmed);
        hudhud_string_free(lower);
        hudhud_string_free(upper);
        hudhud_string_free(replaced);
    }
}

#[test]
fn test_array_operations() {
    unsafe {
        let arr = hudhud_array_new(0);
        hudhud_array_push(arr, 30);
        hudhud_array_push(arr, 10);
        hudhud_array_push(arr, 20);

        assert_eq!(hudhud_array_len(arr), 3);
        assert_eq!(hudhud_array_index_of(arr, 10), 1);
        assert_eq!(hudhud_array_includes(arr, 20), 1);
        assert_eq!(hudhud_array_includes(arr, 99), 0);

        // Sort
        hudhud_array_sort(arr);
        assert_eq!(hudhud_array_get(arr, 0), 10);
        assert_eq!(hudhud_array_get(arr, 1), 20);
        assert_eq!(hudhud_array_get(arr, 2), 30);

        // Reverse
        hudhud_array_reverse(arr);
        assert_eq!(hudhud_array_get(arr, 0), 30);
        assert_eq!(hudhud_array_get(arr, 1), 20);
        assert_eq!(hudhud_array_get(arr, 2), 10);

        // Slice
        let sliced = hudhud_array_slice(arr, 1, 3);
        assert_eq!(hudhud_array_len(sliced), 2);
        assert_eq!(hudhud_array_get(sliced, 0), 20);
        assert_eq!(hudhud_array_get(sliced, 1), 10);

        // Shift and Unshift
        let first = hudhud_array_shift(arr);
        assert_eq!(first, 30);
        assert_eq!(hudhud_array_len(arr), 2);

        hudhud_array_unshift(arr, 40);
        assert_eq!(hudhud_array_len(arr), 3);
        assert_eq!(hudhud_array_get(arr, 0), 40);

        // Concat
        let arr2 = hudhud_array_new(0);
        hudhud_array_push(arr2, 50);
        let joined = hudhud_array_concat(arr, arr2);
        assert_eq!(hudhud_array_len(joined), 4);
        assert_eq!(hudhud_array_get(joined, 3), 50);

        hudhud_array_free(arr);
        hudhud_array_free(arr2);
        hudhud_array_free(sliced);
        hudhud_array_free(joined);
    }
}

#[test]
fn test_string_concat_interning_and_ascii_chars() {
    unsafe {
        let a = to_c("abc");
        let b = to_c("def");
        let c1 = hudhud_string_concat(a, b);
        let c2 = hudhud_string_concat(a, b);
        assert_eq!(from_c(c1), "abcdef");
        assert_eq!(from_c(c2), "abcdef");
        // Short strings <= 32 bytes are interned in string arena, returning same pointer
        assert_eq!(c1, c2, "short strings should be interned to the same pointer");

        // String equality fast path on same pointer and different pointers
        assert_eq!(hudhud_string_eq(c1, c2), 1);
        assert_eq!(hudhud_string_eq(c1, a), 0);

        // ASCII chars table verification
        let ch_a = hudhud_string_char_at(c1, 0);
        let ch_b = hudhud_string_char_at(c1, 1);
        assert_eq!(from_c(ch_a), "a");
        assert_eq!(from_c(ch_b), "b");
        assert_eq!(ch_a as *const u8, hudhud_ascii_chars[b'a' as usize].as_ptr());
        assert_eq!(ch_b as *const u8, hudhud_ascii_chars[b'b' as usize].as_ptr());

        // Out of bounds / zero index
        let empty_ch = hudhud_string_char_at(c1, 999);
        assert_eq!(from_c(empty_ch), "");
        assert_eq!(empty_ch as *const u8, hudhud_ascii_chars[0].as_ptr());

        hudhud_string_free(a);
        hudhud_string_free(b);
    }
}
