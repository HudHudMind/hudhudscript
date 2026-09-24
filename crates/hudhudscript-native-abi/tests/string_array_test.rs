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
