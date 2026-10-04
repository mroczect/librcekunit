use core::ffi::c_char;
use std::ffi::CString;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_bytes_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let slice = core::ptr::slice_from_raw_parts_mut(ptr, len);
    unsafe {
        drop(Box::from_raw(slice));
    }
}

pub fn vec_into_raw(mut v: Vec<u8>) -> (*mut u8, usize) {
    let len = v.len();
    let ptr = v.as_mut_ptr();
    core::mem::forget(v);
    (ptr, len)
}

pub fn bytes_to_cstring_lossy(bytes: &[u8]) -> CString {
    let s = String::from_utf8_lossy(bytes).into_owned();
    let sanitized = s.replace('\0', " ");
    CString::new(sanitized).unwrap_or_else(|_| CString::new("").unwrap())
}
