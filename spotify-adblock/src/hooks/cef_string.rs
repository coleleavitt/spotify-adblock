//! CEF UTF-16 string helpers shared by the request hooks.

use std::slice::from_raw_parts;

use cef_sys::cef_string_userfree_utf16_t;

pub(super) fn cef_userfree_utf16_to_string(value: cef_string_userfree_utf16_t) -> Option<String> {
    if value.is_null() {
        return None;
    }

    // SAFETY: Category 8 - FFI boundary. `value` is a non-null CEF userfree
    // string pointer returned by the request API for the duration of this call.
    let cef_string = unsafe { &*value };
    if cef_string.length == 0 {
        return Some(String::new());
    }

    if cef_string.str_.is_null() {
        return None;
    }

    // SAFETY: Category 10 - out-of-bounds. CEF reports `length` UTF-16 code
    // units for the non-null `str_` pointer in this userfree string.
    let utf16 = unsafe { from_raw_parts(cef_string.str_, cef_string.length) };
    Some(String::from_utf16_lossy(utf16))
}
