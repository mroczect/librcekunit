use core::ffi::{c_char, c_void};
use std::ffi::CString;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LrStatus {
        Ok = 0,
        InvalidArg = 1,
        Config = 2,
        Auth = 3,
        Network = 4,
        Api = 5,
        Io = 6,
        Internal = 7,
        Panic = 8,
}

impl LrStatus {
        #[must_use]
    pub fn from_error(err: &librcekunit_handler::Error) -> Self {
        use librcekunit_handler::Error;
        match err {
            Error::Config(_) => Self::Config,
            Error::Auth(_) | Error::NotLoggedIn | Error::CsrfNotFound => Self::Auth,
            Error::Network(_) => Self::Network,
            Error::Api(_, _) => Self::Api,
            Error::Io(_) | Error::CookieStore(_) => Self::Io,
            Error::Json(_) => Self::Internal,
            _ => Self::Internal,
        }
    }
}

pub unsafe fn write_error(out_error: *mut *mut c_char, msg: &str) {
    if out_error.is_null() {
        return;
    }
    let sanitized = msg.replace('\0', " ");
    match CString::new(sanitized) {
        Ok(cs) => unsafe { *out_error = cs.into_raw() },
        Err(_) => {
            let fallback = CString::new("error message contains NUL")
                .unwrap_or_else(|_| CString::new("error").unwrap());
            unsafe { *out_error = fallback.into_raw() };
        }
    }
}

pub unsafe fn write_panic(out_error: *mut *mut c_char, payload: &(dyn core::any::Any + Send)) {
    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        String::from("rust panic")
    };
    unsafe { write_error(out_error, &msg) };
}

pub fn guard<F>(out_error: *mut *mut c_char, f: F) -> LrStatus
where
    F: FnOnce() -> LrStatus,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(status) => status,
        Err(payload) => {
            unsafe { write_panic(out_error, payload.as_ref()) };
            LrStatus::Panic
        }
    }
}

#[allow(dead_code)]
type _VoidAlias = c_void;
