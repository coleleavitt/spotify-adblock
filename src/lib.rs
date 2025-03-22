mod cef;

use cef::{
    _cef_request_context_t, _cef_request_t, _cef_urlrequest_client_t, cef_string_userfree_utf16_t, cef_urlrequest_t,
};
use lazy_static::lazy_static;
use libc::{EAI_FAIL, RTLD_NEXT, addrinfo, c_char, dlsym};
use regex::RegexSet;
use serde::Deserialize;
use std::{env, ffi::CStr, fs::read_to_string, mem, path::PathBuf, ptr::null, slice::from_raw_parts, string::String};

// Add debug mode option that can be enabled with environment variable
lazy_static! {
    static ref DEBUG_MODE: bool = env::var("SPOTIFY_ADBLOCK_DEBUG").is_ok();
}

macro_rules! hook {
    ($function_name:ident($($parameter_name:ident: $parameter_type:ty),*) -> $return_type:ty => $new_function_name:ident $body:block) => {
        lazy_static! {
            static ref $new_function_name: fn($($parameter_type),*) -> $return_type = unsafe {
                let function_name = CStr::from_bytes_with_nul(concat!(stringify!($function_name), "\0").as_bytes()).unwrap();
                let function_pointer = dlsym(RTLD_NEXT, function_name.as_ptr());
                if function_pointer.is_null() {
                    panic!("[*] Error: Unable to find function \"{}\"", stringify!($function_name));
                }
                mem::transmute(function_pointer)
            };
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn $function_name($($parameter_name: $parameter_type),*) -> $return_type {
            $body
        }
    }
}

#[derive(Deserialize)]
struct Config {
    #[serde(with = "serde_regex")]
    allowlist: RegexSet,
    #[serde(with = "serde_regex")]
    denylist: RegexSet,
}

lazy_static! {
    static ref CONFIG: Config = {
        let config_paths = vec![
            PathBuf::from("config.toml"),
            match env::var("XDG_CONFIG_HOME") {
                Ok(xdg_config_home) => PathBuf::from(xdg_config_home),
                #[allow(deprecated)] // std::env::home_dir() is only broken on Windows
                Err(_) => PathBuf::from(env::home_dir().unwrap()).join(".config")
            }.join("spotify-adblock/config.toml"),
            PathBuf::from("/etc/spotify-adblock/config.toml"),
        ];

        if let Some(path) = config_paths.into_iter().find(|path| path.exists()) {
            println!("[*] Config file: {}", path.to_str().unwrap());
            match read_to_string(path) {
                Ok(config_string) => match toml::from_str(&config_string) {
                    Ok(config) => {
                        return config;
                    }
                    Err(error) => {
                        println!("[*] Error: Parse config file ({})", error);
                    }
                },
                Err(error) => {
                    println!("[*] Error: Read config file ({})", error);
                }
            }
        } else {
            println!("[*] Error: No config file");
        };
        Config {
            allowlist: RegexSet::empty(),
            denylist: RegexSet::empty(),
        }
    };
}

hook! {
    getaddrinfo(node: *const c_char, service: *const c_char, hints: *const addrinfo, res: *const *const addrinfo) -> i32 => REAL_GETADDRINFO {
        let domain = unsafe { CStr::from_ptr(node) }.to_str().unwrap_or("");

        // Always allow dealer domains which are needed for websockets
        if domain.contains("dealer") || domain.contains("spotify.com") {
            println!("[+] getaddrinfo:\t\t {}", domain);
            return REAL_GETADDRINFO(node, service, hints, res);
        }

        if CONFIG.allowlist.is_match(&domain) {
            println!("[+] getaddrinfo:\t\t {}", domain);
             REAL_GETADDRINFO(node, service, hints, res)
        } else {
            println!("[-] getaddrinfo:\t\t {}", domain);
            EAI_FAIL
        }
    }
}

hook! {
    cef_urlrequest_create(request: *mut _cef_request_t, client: *const _cef_urlrequest_client_t, request_context: *const _cef_request_context_t) -> *const cef_urlrequest_t => REAL_CEF_URLREQUEST_CREATE {
        let url_cef = unsafe { (*request).get_url.unwrap()(request) };
        if url_cef.is_null() {
            REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
        }

        let url_utf16 = unsafe { from_raw_parts((*url_cef).str_, (*url_cef).length as usize) };
        let url = String::from_utf16(url_utf16).unwrap_or_else(|_| String::new());

        // Get request method for additional context
        let method_cef = unsafe { (*request).get_method.unwrap()(request) };
        let method_utf16 = unsafe { from_raw_parts((*method_cef).str_, (*method_cef).length as usize) };
        let method = String::from_utf16(method_utf16).unwrap_or_else(|_| String::from("UNKNOWN"));
         cef_string_userfree_utf16_free(method_cef);

        // Rest of your logic stays the same, just add unsafe blocks around unsafe operations

        // Improved Discord RPC detection
        let is_discord_rpc = url.contains("discord") ||
                           url.contains("discordapp") ||
                           url.contains("presence") ||
                           url.contains("/presence2/") ||
                           url.contains("connect-state") ||
                           url.contains("rpc");

        // GABO receiver handles both ads and events including Discord communication
        let is_gabo = url.contains("gabo-receiver-service");
        let is_dealer = url.contains("dealer");
        let is_ad_related = url.contains("/ads/") ||
                          url.contains("ad-logic") ||
                          url.contains("doubleclick") ||
                          url.contains("googleads") ||
                          url.contains("adswizz") ||
                          url.contains("analytics") ||
                          (url.contains("track") && url.contains("event")) ||
                          (url.contains("ads") && !url.contains("gabo"));

        if *DEBUG_MODE {
            println!("[DEBUG] {} {}", method, url);
            let result = REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            cef_string_userfree_utf16_free(url_cef);
            return result;
        }

        if is_discord_rpc {
            println!("[+] DISCORD RPC: {} {}", method, url);
            let result = REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            cef_string_userfree_utf16_free(url_cef);
            return result;
        } else if is_gabo || is_dealer {
            println!("[+] SERVICE: {} {}", method, url);
            let result = REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
            cef_string_userfree_utf16_free(url_cef);
            return result;
        }

        if is_ad_related {
            println!("[-] BLOCKED AD: {} {}", method, url);
            cef_string_userfree_utf16_free(url_cef);
            return null();
        }

        let result = if CONFIG.denylist.is_match(&url) {
            println!("[-] BLOCKED CONFIG: {} {}", method, url);
            null()
        } else {
            println!("[+] ALLOWED: {} {}", method, url);
            return REAL_CEF_URLREQUEST_CREATE(request, client, request_context);
        };

        cef_string_userfree_utf16_free(url_cef);
        result
    }
}

hook! {
    cef_string_userfree_utf16_free(_str: cef_string_userfree_utf16_t) -> () => REAL_CEF_STRING_USERFREE_UTF16_FREE {
        if !_str.is_null() {
          REAL_CEF_STRING_USERFREE_UTF16_FREE(_str);
        }
    }
}
