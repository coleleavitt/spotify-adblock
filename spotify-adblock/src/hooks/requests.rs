use std::ptr::null_mut;

use cef_sys::{
    _cef_request_context_t, _cef_request_t, _cef_urlrequest_client_t, cef_urlrequest_t,
};

use crate::config::DEBUG_MODE;
use crate::hook;
use crate::hooks::memory::cef_string_userfree_utf16_free;
use crate::utils::logging;

use super::cef_string::cef_userfree_utf16_to_string;
use super::decision::{Verdict, decide};

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

        // Debug mode logs everything and blocks nothing.
        if *DEBUG_MODE {
            logging::log_debug(&format!("{method} {url}"));
            let result = REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            cef_string_userfree_utf16_free(url_cef);
            return result;
        }

        let result = match decide(&url, &method) {
            Verdict::Block(reason) => {
                logging::log_blocked(reason, &method, &url);
                null_mut()
            }
            Verdict::Allow(reason) => {
                logging::log_allowed(reason, &method, &url);
                REAL_CEF_URLREQUEST_CREATE(request, client, request_context)
            }
        };

        cef_string_userfree_utf16_free(url_cef);
        result
    }
}
