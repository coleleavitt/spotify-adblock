//! Blocks the UI's own requests.
//!
//! xpui's `fetch()` calls (quicksilver upsell popups, pathfinder, webgate) go
//! through Chromium's network stack and never reach `cef_urlrequest_create`.
//! CEF consults a request context's scheme handler factories for every
//! request, so this registers one `https` factory, for every domain, on the
//! global context (after `cef_initialize`) and on each context Spotify
//! creates. Domain-specific factories (such as Spotify's own
//! `xpui.app.spotify.com` handler) still take precedence. For an allowed URL
//! the factory returns NULL and CEF continues with normal network handling.
//! For a blocked URL it returns a handler that cancels the request.
//!
//! `SPOTIFY_ADBLOCK_NO_UI_HOOK=1` disables this module.
//!
//! Reference counting follows CEF's C API: struct arguments passed to a
//! callback carry a reference the callee must release; the factory and handler
//! here are process-lifetime statics, so their own counts are no-ops.

use std::ffi::{c_int, c_void};
use std::mem::size_of;
use std::ptr::{self, null_mut};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use cef_sys::{
    _cef_base_ref_counted_t, _cef_browser_t, _cef_callback_t, _cef_frame_t, _cef_request_context_t, _cef_request_t,
    _cef_resource_handler_t, _cef_resource_read_callback_t, _cef_resource_skip_callback_t, _cef_response_t,
    _cef_scheme_handler_factory_t, cef_base_ref_counted_t, cef_resource_handler_t, cef_scheme_handler_factory_t,
    cef_string_t,
};

use crate::config::DEBUG_MODE;
use crate::hook;
use crate::hooks::memory::cef_string_userfree_utf16_free;
use crate::utils::logging;

use super::cef_string::cef_userfree_utf16_to_string;
use super::decision::{Verdict, decide};

static DISABLED: LazyLock<bool> = LazyLock::new(|| std::env::var_os("SPOTIFY_ADBLOCK_NO_UI_HOOK").is_some());
static GLOBAL_REGISTERED: AtomicBool = AtomicBool::new(false);

const HTTPS: [u16; 5] = [b'h' as u16, b't' as u16, b't' as u16, b'p' as u16, b's' as u16];

// ---- static, never-freed CEF objects -------------------------------------------------

const extern "C" fn static_add_ref(_self: *mut _cef_base_ref_counted_t) {}
const extern "C" fn static_release(_self: *mut _cef_base_ref_counted_t) -> c_int {
    0
}
const extern "C" fn static_has_one_ref(_self: *mut _cef_base_ref_counted_t) -> c_int {
    1
}
const extern "C" fn static_has_at_least_one_ref(_self: *mut _cef_base_ref_counted_t) -> c_int {
    1
}

const fn static_base(size: usize) -> cef_base_ref_counted_t {
    cef_base_ref_counted_t {
        size,
        add_ref: Some(static_add_ref),
        release: Some(static_release),
        has_one_ref: Some(static_has_one_ref),
        has_at_least_one_ref: Some(static_has_at_least_one_ref),
    }
}

static FACTORY: LazyLock<cef_scheme_handler_factory_t> = LazyLock::new(|| cef_scheme_handler_factory_t {
    base: static_base(size_of::<cef_scheme_handler_factory_t>()),
    create: Some(factory_create),
});

static BLOCK_HANDLER: LazyLock<cef_resource_handler_t> = LazyLock::new(|| cef_resource_handler_t {
    base: static_base(size_of::<cef_resource_handler_t>()),
    open: Some(block_open),
    process_request: Some(block_process_request),
    get_response_headers: Some(block_get_response_headers),
    skip: Some(block_skip),
    read: Some(block_read),
    read_response: Some(block_read_response),
    cancel: Some(block_cancel),
});

/// Releases one reference on a CEF ref-counted struct (its first member is the base).
fn release<T>(object: *mut T) {
    if object.is_null() {
        return;
    }
    let base = object.cast::<_cef_base_ref_counted_t>();
    // SAFETY: Category 8 - FFI boundary. Every CEF ref-counted struct starts
    // with `cef_base_ref_counted_t`, and CEF handed us one reference to it.
    unsafe {
        if let Some(release) = (*base).release {
            release(base);
        }
    }
}

// ---- factory ---------------------------------------------------------------------------

extern "C" fn factory_create(
    _self: *mut _cef_scheme_handler_factory_t,
    browser: *mut _cef_browser_t,
    frame: *mut _cef_frame_t,
    _scheme_name: *const cef_string_t,
    request: *mut _cef_request_t,
) -> *mut _cef_resource_handler_t {
    release(browser);
    release(frame);
    let verdict = classify_request(request);
    release(request);

    match verdict {
        Some(true) => ptr::from_ref::<cef_resource_handler_t>(&BLOCK_HANDLER).cast_mut(),
        _ => null_mut(),
    }
}

/// `Some(true)` to block, `Some(false)` to allow, `None` when the request is unreadable.
fn classify_request(request: *mut _cef_request_t) -> Option<bool> {
    if request.is_null() {
        return None;
    }
    // SAFETY: Category 8 - FFI boundary. `request` is a live CEF request for the
    // duration of this callback; its function table is owned by CEF.
    let (url_cef, method_cef) = unsafe { ((*request).get_url?(request), (*request).get_method?(request)) };
    let url = cef_userfree_utf16_to_string(url_cef);
    let method = cef_userfree_utf16_to_string(method_cef);
    cef_string_userfree_utf16_free(url_cef);
    cef_string_userfree_utf16_free(method_cef);
    let (url, method) = (url?, method.unwrap_or_default());

    if *DEBUG_MODE {
        logging::log_debug(&format!("UI {method} {url}"));
        return Some(false);
    }
    match decide(&url, &method) {
        Verdict::Block(reason) => {
            logging::log_blocked(&format!("UI {reason}"), &method, &url);
            Some(true)
        }
        Verdict::Allow(_) => Some(false),
    }
}

// ---- handler that cancels the request ------------------------------------------------

extern "C" fn block_open(
    _self: *mut _cef_resource_handler_t,
    request: *mut _cef_request_t,
    handle_request: *mut c_int,
    callback: *mut _cef_callback_t,
) -> c_int {
    release(request);
    release(callback);
    if !handle_request.is_null() {
        // SAFETY: Category 8 - FFI boundary. CEF passes a valid out-parameter.
        // `handle_request = 1` with a false return cancels the request now.
        unsafe { *handle_request = 1 };
    }
    0
}

extern "C" fn block_process_request(
    _self: *mut _cef_resource_handler_t,
    request: *mut _cef_request_t,
    callback: *mut _cef_callback_t,
) -> c_int {
    release(request);
    release(callback);
    0
}

extern "C" fn block_get_response_headers(
    _self: *mut _cef_resource_handler_t,
    response: *mut _cef_response_t,
    response_length: *mut i64,
    _redirect_url: *mut cef_string_t,
) {
    release(response);
    if !response_length.is_null() {
        // SAFETY: Category 8 - FFI boundary. CEF passes a valid out-parameter.
        unsafe { *response_length = 0 };
    }
}

extern "C" fn block_skip(
    _self: *mut _cef_resource_handler_t,
    _bytes_to_skip: i64,
    bytes_skipped: *mut i64,
    callback: *mut _cef_resource_skip_callback_t,
) -> c_int {
    release(callback);
    if !bytes_skipped.is_null() {
        // SAFETY: Category 8 - FFI boundary. CEF passes a valid out-parameter.
        unsafe { *bytes_skipped = -2 }; // ERR_FAILED
    }
    0
}

extern "C" fn block_read(
    _self: *mut _cef_resource_handler_t,
    _data_out: *mut c_void,
    _bytes_to_read: c_int,
    bytes_read: *mut c_int,
    callback: *mut _cef_resource_read_callback_t,
) -> c_int {
    release(callback);
    if !bytes_read.is_null() {
        // SAFETY: Category 8 - FFI boundary. CEF passes a valid out-parameter.
        unsafe { *bytes_read = 0 };
    }
    0
}

extern "C" fn block_read_response(
    _self: *mut _cef_resource_handler_t,
    _data_out: *mut c_void,
    _bytes_to_read: c_int,
    bytes_read: *mut c_int,
    callback: *mut _cef_callback_t,
) -> c_int {
    release(callback);
    if !bytes_read.is_null() {
        // SAFETY: Category 8 - FFI boundary. CEF passes a valid out-parameter.
        unsafe { *bytes_read = 0 };
    }
    0
}

const extern "C" fn block_cancel(_self: *mut _cef_resource_handler_t) {}

// ---- registration --------------------------------------------------------------------

fn https_and_any_domain() -> (cef_string_t, cef_string_t) {
    let scheme = cef_string_t { str_: HTTPS.as_ptr().cast_mut(), length: HTTPS.len(), dtor: None };
    let any_domain = cef_string_t { str_: null_mut(), length: 0, dtor: None };
    (scheme, any_domain)
}

fn factory_ptr() -> *mut _cef_scheme_handler_factory_t {
    ptr::from_ref::<cef_scheme_handler_factory_t>(&FACTORY).cast_mut()
}

type RegisterGlobal =
    unsafe extern "C" fn(*const cef_string_t, *const cef_string_t, *mut _cef_scheme_handler_factory_t) -> c_int;

fn register_on_global_context() {
    if *DISABLED || GLOBAL_REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }
    let name = c"cef_register_scheme_handler_factory";
    // SAFETY: Category 8 - FFI boundary. Looked up at runtime so the preload
    // library has no link-time dependency on libcef.
    let symbol = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr()) };
    if symbol.is_null() {
        logging::log_error("cef_register_scheme_handler_factory not found; UI requests are not filtered");
        return;
    }
    // SAFETY: Category 8 - FFI boundary. The symbol is libcef's exported
    // `cef_register_scheme_handler_factory`, whose C signature matches.
    let register: RegisterGlobal = unsafe { std::mem::transmute::<*mut c_void, RegisterGlobal>(symbol) };
    let (scheme, domain) = https_and_any_domain();
    // SAFETY: Category 8 - FFI boundary. Both strings outlive the call (CEF
    // copies them); the factory is a process-lifetime static.
    let ok = unsafe { register(&raw const scheme, &raw const domain, factory_ptr()) };
    report_registration("global", ok);
}

fn register_on_context(context: *mut _cef_request_context_t) {
    if *DISABLED || context.is_null() {
        return;
    }
    let (scheme, domain) = https_and_any_domain();
    // SAFETY: Category 8 - FFI boundary. `context` was just returned by CEF and
    // is alive; the strings outlive the call; the factory is a static.
    let ok = unsafe {
        (*context)
            .register_scheme_handler_factory
            .map_or(0, |register| register(context, &raw const scheme, &raw const domain, factory_ptr()))
    };
    report_registration("request context", ok);
}

fn report_registration(target: &str, ok: c_int) {
    if ok == 0 {
        logging::log_error(&format!("could not register the UI request filter on the {target}"));
    } else {
        logging::log_info(&format!("UI request filter registered on the {target}"));
    }
}

hook! {
    cef_initialize(args: *const c_void, settings: *const c_void, application: *mut c_void, windows_sandbox_info: *mut c_void) -> c_int => REAL_CEF_INITIALIZE {
        let ok = REAL_CEF_INITIALIZE(args, settings, application, windows_sandbox_info);
        if ok != 0 {
            register_on_global_context();
        }
        ok
    }
}

hook! {
    cef_request_context_create_context(settings: *const c_void, handler: *mut c_void) -> *mut _cef_request_context_t => REAL_CEF_REQUEST_CONTEXT_CREATE_CONTEXT {
        let context = REAL_CEF_REQUEST_CONTEXT_CREATE_CONTEXT(settings, handler);
        register_on_context(context);
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_objects_report_their_own_struct_size() {
        assert_eq!(FACTORY.base.size, size_of::<cef_scheme_handler_factory_t>());
        assert_eq!(BLOCK_HANDLER.base.size, size_of::<cef_resource_handler_t>());
    }

    #[test]
    fn block_handler_cancels_immediately() {
        let mut handle = 0;
        let handler = ptr::from_ref::<cef_resource_handler_t>(&BLOCK_HANDLER).cast_mut();
        let opened = block_open(handler, null_mut(), &raw mut handle, null_mut());
        assert_eq!((opened, handle), (0, 1));
    }

    #[test]
    fn factory_allows_unreadable_requests() {
        assert!(factory_create(factory_ptr(), null_mut(), null_mut(), ptr::null(), null_mut()).is_null());
    }

    #[test]
    fn https_scheme_string_is_utf16() {
        let (scheme, domain) = https_and_any_domain();
        // SAFETY: `scheme` points at the static HTTPS buffer of `length` units.
        let units = unsafe { std::slice::from_raw_parts(scheme.str_, scheme.length) };
        assert_eq!(String::from_utf16_lossy(units), "https");
        assert_eq!(domain.length, 0);
    }
}
