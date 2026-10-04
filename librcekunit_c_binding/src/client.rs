use core::ffi::c_char;
use std::ffi::{CStr, CString};
use std::path::PathBuf;

use librcekunit_client::prelude::{
    Auth, Client, ClientBuilder, Config, CookieStore, Crud, Dashboard, Form,
};
use librcekunit_handler::InputUser;

use crate::buffer::{bytes_to_cstring_lossy, vec_into_raw};
use crate::error::{LrStatus, guard, write_error};

pub struct LrClient {
    runtime: tokio::runtime::Runtime,
    inner: Client,
}

impl core::fmt::Debug for LrClient {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LrClient").finish_non_exhaustive()
    }
}


unsafe fn cstr<'a>(ptr: *const c_char, name: &str) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err(format!("{name} is null"));
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|e| format!("{name} is not valid UTF-8: {e}"))
}

unsafe fn parse_form(ptr: *const c_char) -> Result<Form, String> {
    let json = unsafe { cstr(ptr, "form_json")? };
    let map: std::collections::HashMap<String, String> =
        serde_json::from_str(json).map_err(|e| format!("invalid form JSON: {e}"))?;
    Ok(map)
}

unsafe fn out_pointer<'a, T>(ptr: *mut *mut T, name: &str) -> Result<&'a mut *mut T, String> {
    if ptr.is_null() {
        return Err(format!("{name} is null"));
    }
    Ok(unsafe { &mut *ptr })
}

fn write_body(bytes: &[u8], out: *mut *mut c_char) {
    if out.is_null() {
        return;
    }
    let cs = bytes_to_cstring_lossy(bytes);
    unsafe { *out = cs.into_raw() };
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_new(
    base_url: *const c_char,
    cookie_file: *const c_char,
    user_agent: *const c_char,
    timeout_secs: u64,
    out_handle: *mut *mut LrClient,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            let base_url = match cstr(base_url, "base_url") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };

            if let Err(e) = out_pointer(out_handle, "out_handle") {
                write_error(out_error, &e);
                return LrStatus::InvalidArg;
            }

            let mut builder = ClientBuilder::new()
                .base_url(base_url)
                .timeout_secs(timeout_secs);

            if !user_agent.is_null() {
                if let Ok(ua) = cstr(user_agent, "user_agent") {
                    builder = builder.user_agent(ua);
                }
            }

            if !cookie_file.is_null() {
                if let Ok(path) = cstr(cookie_file, "cookie_file") {
                    builder = builder.cookie_store(CookieStore::Persistent(PathBuf::from(path)));
                }
            }

            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    write_error(out_error, &format!("tokio runtime: {e}"));
                    return LrStatus::Internal;
                }
            };

            match runtime.block_on(builder.build()) {
                Ok(inner) => {
                    let boxed = Box::new(LrClient { runtime, inner });
                    unsafe { *out_handle = Box::into_raw(boxed) };
                    LrStatus::Ok
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_free(handle: *mut LrClient) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_login(
    handle: *mut LrClient,
    email: *const c_char,
    password: *const c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let email = match cstr(email, "email") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            let password = match cstr(password, "password") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };

            match client.runtime.block_on(client.inner.login(email, password)) {
                Ok(()) => LrStatus::Ok,
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_logout(
    handle: *mut LrClient,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            match client.runtime.block_on(client.inner.logout()) {
                Ok(()) => LrStatus::Ok,
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_index(
    handle: *mut LrClient,
    out_body: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let result = client.runtime.block_on(async {
                let resp = client.inner.index().await?;
                let status = resp.status().as_u16();
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
                Ok::<_, librcekunit_handler::Error>((status, bytes))
            });

            match result {
                Ok((status, bytes)) => {
                    write_body(&bytes, out_body);
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_store(
    handle: *mut LrClient,
    form_json: *const c_char,
    out_body: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let form = match parse_form(form_json) {
                Ok(f) => f,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };

            let result = client.runtime.block_on(async {
                let resp = client.inner.store(form).await?;
                let status = resp.status().as_u16();
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
                Ok::<_, librcekunit_handler::Error>((status, bytes))
            });

            match result {
                Ok((status, bytes)) => {
                    write_body(&bytes, out_body);
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_show(
    handle: *mut LrClient,
    id: u64,
    out_body: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let result = client.runtime.block_on(async {
                let resp = client.inner.show(id).await?;
                let status = resp.status().as_u16();
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
                Ok::<_, librcekunit_handler::Error>((status, bytes))
            });
            match result {
                Ok((status, bytes)) => {
                    write_body(&bytes, out_body);
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_update(
    handle: *mut LrClient,
    id: u64,
    form_json: *const c_char,
    out_body: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let form = match parse_form(form_json) {
                Ok(f) => f,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            let result = client.runtime.block_on(async {
                let resp = client.inner.update(id, form).await?;
                let status = resp.status().as_u16();
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
                Ok::<_, librcekunit_handler::Error>((status, bytes))
            });
            match result {
                Ok((status, bytes)) => {
                    write_body(&bytes, out_body);
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_destroy(
    handle: *mut LrClient,
    id: u64,
    out_body: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let result = client.runtime.block_on(async {
                let resp = client.inner.destroy(id).await?;
                let status = resp.status().as_u16();
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
                Ok::<_, librcekunit_handler::Error>((status, bytes))
            });
            match result {
                Ok((status, bytes)) => {
                    write_body(&bytes, out_body);
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_delete_all(
    handle: *mut LrClient,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            match client.runtime.block_on(client.inner.delete_all()) {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_delete_by_category(
    handle: *mut LrClient,
    column: *const c_char,
    value: *const c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let column = match cstr(column, "column") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            let value = match cstr(value, "value") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            match client
                .runtime
                .block_on(client.inner.delete_by_category(column, value))
            {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_export(
    handle: *mut LrClient,
    format: *const c_char,
    sort: *const c_char,
    direction: *const c_char,
    out_data: *mut *mut u8,
    out_len: *mut usize,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() || out_data.is_null() || out_len.is_null() {
                write_error(out_error, "handle/out_data/out_len is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let format = match cstr(format, "format") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            let sort = match cstr(sort, "sort") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            let direction = match cstr(direction, "direction") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };

            let result = client.runtime.block_on(async {
                let resp = client.inner.export(format, sort, direction).await?;
                let status = resp.status().as_u16();
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
                Ok::<_, librcekunit_handler::Error>((status, bytes))
            });

            match result {
                Ok((status, bytes)) => {
                    let (ptr, len) = vec_into_raw(bytes.to_vec());
                    unsafe {
                        *out_data = ptr;
                        *out_len = len;
                    }
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_input_user_export(
    handle: *mut LrClient,
    params_json: *const c_char,
    out_data: *mut *mut u8,
    out_len: *mut usize,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() || out_data.is_null() || out_len.is_null() {
                write_error(out_error, "handle/out_data/out_len is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let params = match parse_form(params_json) {
                Ok(f) => f,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };

            let result = client.runtime.block_on(async {
                let resp = client.inner.input_user_export(params).await?;
                let status = resp.status().as_u16();
                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| librcekunit_handler::Error::Network(e.to_string()))?;
                Ok::<_, librcekunit_handler::Error>((status, bytes))
            });

            match result {
                Ok((status, bytes)) => {
                    let (ptr, len) = vec_into_raw(bytes.to_vec());
                    unsafe {
                        *out_data = ptr;
                        *out_len = len;
                    }
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_client_import_csv(
    handle: *mut LrClient,
    file_path: *const c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            if handle.is_null() {
                write_error(out_error, "client handle is null");
                return LrStatus::InvalidArg;
            }
            let client = &*handle;
            let path = match cstr(file_path, "file_path") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            let path = std::path::Path::new(path);
            match client.runtime.block_on(client.inner.import_csv(path)) {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if (200..300).contains(&status) {
                        LrStatus::Ok
                    } else {
                        LrStatus::Api
                    }
                }
                Err(e) => {
                    write_error(out_error, &e.to_string());
                    LrStatus::from_error(&e)
                }
            }
        })
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn lr_config_debug_string(
    base_url: *const c_char,
    out_str: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> LrStatus {
    unsafe {
        guard(out_error, || {
            let base_url = match cstr(base_url, "base_url") {
                Ok(s) => s,
                Err(e) => {
                    write_error(out_error, &e);
                    return LrStatus::InvalidArg;
                }
            };
            let config = Config::new(base_url);
            let debug = format!("{config:?}");
            match CString::new(debug) {
                Ok(cs) => {
                    if !out_str.is_null() {
                        *out_str = cs.into_raw();
                    }
                    LrStatus::Ok
                }
                Err(_) => LrStatus::Internal,
            }
        })
    }
}
