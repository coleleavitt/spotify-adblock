//! Spotify adblock library conforming to JPL safety standards
//!
//! This module intercepts network requests to block advertisements
//! in the Spotify client while ensuring radiation-hardened operation.
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::unreadable_literal)]
#![allow(unsafe_code)]
#![allow(unused_unsafe)]
#![allow(unused_doc_comments)]

pub mod config;
pub mod hooks;
pub mod utils;

// Define the hook macro for intercepting functions
#[macro_export]
macro_rules! hook {
    ($function_name:ident($($parameter_name:ident: $parameter_type:ty),*) -> $return_type:ty => $new_function_name:ident $body:block) => {
        lazy_static::lazy_static! {
            static ref $new_function_name: extern "C" fn($($parameter_type),*) -> $return_type = {
                let Ok(function_name) = std::ffi::CStr::from_bytes_with_nul(concat!(stringify!($function_name), "\0").as_bytes()) else {
                    unreachable!("hook symbol names are compile-time C strings");
                };
                // SAFETY: `dlsym` is called with `RTLD_NEXT` and a valid
                // NUL-terminated symbol name. The pointer is checked for null
                // before conversion.
                let function_pointer = unsafe { libc::dlsym(libc::RTLD_NEXT, function_name.as_ptr()) };
                if function_pointer.is_null() {
                    panic!("[*] Error: Unable to find function \"{}\"", stringify!($function_name));
                }
                // SAFETY: Category 8 - FFI boundary. The macro is only used
                // for interposed symbols whose parameter and return types
                // match the corresponding C ABI function exactly.
                unsafe { std::mem::transmute(function_pointer) }
            };
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn $function_name($($parameter_name: $parameter_type),*) -> $return_type {
            $body
        }
    }
}

pub use hooks::memory::cef_string_userfree_utf16_free;
pub use hooks::network::getaddrinfo;
pub use hooks::requests::cef_urlrequest_create;
pub use hooks::ssl::SSL_write;
