use std::{ptr::null_mut, slice::from_raw_parts};

use cef_sys::{
    _cef_request_context_t, _cef_request_t, _cef_urlrequest_client_t, cef_string_userfree_utf16_t, cef_urlrequest_t,
};

use crate::config::{CONFIG, DEBUG_MODE};
use crate::hook;
use crate::hooks::memory::cef_string_userfree_utf16_free;
use crate::utils::logging;

use super::request_classification::classify_url;

fn cef_userfree_utf16_to_string(value: cef_string_userfree_utf16_t) -> Option<String> {
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

hook! {
    cef_urlrequest_create(request: *mut _cef_request_t, client: *mut _cef_urlrequest_client_t, request_context: *mut _cef_request_context_t) -> *mut cef_urlrequest_t => REAL_CEF_URLREQUEST_CREATE {
        // Validate input pointers
        if request.is_null() {
            logging::log_error("Null request pointer in cef_urlrequest_create");
            return null_mut();
        }

        // Extract URL with safety checks
        // SAFETY: Category 8 - FFI boundary. `request` is non-null and CEF owns
        // the callback table for the duration of this hook call.
        let url_cef = unsafe {
            if let Some(get_url) = (*request).get_url { get_url(request) } else {
                logging::log_error("Missing get_url function in request");
                return REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            }
        };

        if url_cef.is_null() {
            return REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
        }

        let Some(url) = cef_userfree_utf16_to_string(url_cef) else {
            cef_string_userfree_utf16_free(url_cef);
            return REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
        };

        // SAFETY: Category 8 - FFI boundary. `request` is non-null and CEF owns
        // the callback table for the duration of this hook call.
        let method_cef = unsafe {
            if let Some(get_method) = (*request).get_method { get_method(request) } else {
                logging::log_error("Missing get_method function in request");
                cef_string_userfree_utf16_free(url_cef);
                return REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            }
        };

        let Some(method) = cef_userfree_utf16_to_string(method_cef) else {
            cef_string_userfree_utf16_free(url_cef);
            cef_string_userfree_utf16_free(method_cef);
            return REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
        };
        cef_string_userfree_utf16_free(method_cef);

        // Classify URL using fault-contained function
        let classification = classify_url(&url, &method);

        // Debug mode handling
        if *DEBUG_MODE {
            logging::log_debug(&format!("{method} {url}"));
            let result = REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            cef_string_userfree_utf16_free(url_cef);
            return result;
        }

        // Decision logic with proper cleanup in all paths

        // Monitor product state checks (informational)
        if classification.is_product_state {
            logging::log_debug(&format!("PRODUCT STATE CHECK: {method} {url}"));
        }

        if classification.is_discord_rpc {
            logging::log_allowed("DISCORD RPC", &method, &url);
            let result = REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            cef_string_userfree_utf16_free(url_cef);
            return result;
        } else if classification.is_gabo || classification.is_dealer {
            logging::log_allowed("SERVICE", &method, &url);
            let result = REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            cef_string_userfree_utf16_free(url_cef);
            return result;
        }

        // Block aggressive Gabo POST events (payload might contain ad data)
        if classification.is_gabo_event_post {
            logging::log_blocked("BLOCKED GABO POST", &method, &url);
            cef_string_userfree_utf16_free(url_cef);
            return null_mut();
        }

        if classification.is_ad_related {
            logging::log_blocked("BLOCKED AD", &method, &url);
            // No response capturing for now to avoid segfaults
            cef_string_userfree_utf16_free(url_cef);
            return null_mut();
        }

        let result = if CONFIG.denylist.is_match(&url) {
            logging::log_blocked("BLOCKED CONFIG", &method, &url);
            null_mut()
        } else {
            logging::log_allowed("ALLOWED", &method, &url);
            REAL_CEF_URLREQUEST_CREATE(request, client, request_context)
        };

        cef_string_userfree_utf16_free(url_cef);
        result
    }
}
