//! SSL/TLS hooks for intercepting native HTTPS traffic
//!
//! Spotify's native code (Rust/C++) makes HTTPS requests through `OpenSSL`
//! that bypass CEF entirely. This module hooks `SSL_write` to intercept
//! all outgoing HTTPS traffic including cosmos/hermes protocol, leavebehind
//! ads, and spclient API calls.

use std::ffi::c_void;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

use crate::config::DEBUG_MODE;
use crate::hook;
use crate::utils::logging;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SSL {
    _private: [u8; 0],
}

static SSL_VERBOSE: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

const MAX_INSPECT_LEN: usize = 4096;

fn should_block_ssl_request(data: &[u8]) -> Option<String> {
    let header_len = data
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map_or(data.len(), |header_end| header_end + 4);
    let data_str = std::str::from_utf8(&data[..header_len]).ok()?;

    if !data_str.starts_with("GET ")
        && !data_str.starts_with("POST ")
        && !data_str.starts_with("PUT ")
        && !data_str.starts_with("DELETE ")
        && !data_str.starts_with("PATCH ")
        && !data_str.starts_with("HEAD ")
        && !data_str.starts_with("OPTIONS ")
    {
        return None;
    }

    let mut request_parts = data_str.lines().next()?.split_whitespace();
    let method = request_parts.next()?;
    let path = request_parts.next()?;

    let host = data_str
        .lines()
        .find(|line| line.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("host:")))
        .map_or("unknown", |line| line[5..].trim());

    let url = format!("https://{host}{path}");

    // Ad blocking patterns - endpoints that bypass CEF
    let is_ad_related =
        // Leavebehind ads
        path.contains("leavebehind") ||
        path.contains("leave-behind") ||
        path.contains("podcast-ap4p/leavebehind") ||
        path.contains("/ap4p/") ||
        // Sponsored content
        path.contains("sponsoredplaylist") ||
        path.contains("/sponsored") ||
        path.contains("/sponsor") ||
        // Ad tracking
        host.contains("aet.spotify.com") ||
        path.contains("/ads/") ||
        path.contains("/ad-logic") ||
        path.contains("/adlogic") ||
        // Gabo ad events
        (path.contains("gabo-receiver-service") && (
            path.contains("/advertisement") ||
            path.contains("/ad-opportunity") ||
            path.contains("/ads") ||
            path.contains("/v3/events/") ||
            path.contains("/public/v3/events/")
        )) ||
        // Ad creative delivery
        path.contains("ad-creative") ||
        path.contains("ad_creative") ||
        path.contains("/promotion/") ||
        // Partner/attribution tracking
        host.contains("branch.io") ||
        host.contains("adjust.com") ||
        path.contains("/partner_user_id") ||
        // Podcast ad networks
        host.contains("megaphone.fm") ||
        host.contains("art19.com") ||
        host.contains("chartable.com") ||
        host.contains("podsights.com") ||
        // Display ad segments
        (path.contains("display-segments") && path.contains("sponsor")) ||
        // Ad event reporting
        path.contains("/AdEvent") ||
        path.contains("/EndAd") ||
        path.contains("/AdDecision") ||
        // Skip limits
        path.contains("skip-limit") ||
        path.contains("skip_limit") ||
        path.contains("/playback/restrictions");

    if is_ad_related {
        Some(format!("{method} {url}"))
    } else {
        if *DEBUG_MODE || SSL_VERBOSE.load(Ordering::Relaxed) {
            logging::log_debug(&format!("[SSL] {method} {host} {path}"));
        }
        None
    }
}

hook! {
    SSL_write(ssl: *mut SSL, buf: *const c_void, num: c_int) -> c_int => REAL_SSL_WRITE {
        if ssl.is_null() || buf.is_null() || num <= 0 {
            return REAL_SSL_WRITE(ssl, buf, num);
        }

        let Ok(len) = usize::try_from(num) else {
            return REAL_SSL_WRITE(ssl, buf, num);
        };
        let inspect_len = len.min(MAX_INSPECT_LEN);
        // SAFETY: Category 10 - out-of-bounds. `SSL_write` receives a non-null
        // buffer and positive byte count; `inspect_len` is capped to that count.
        let data = unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), inspect_len) };

        if let Some(blocked_url) = should_block_ssl_request(data) {
            logging::log_blocked("BLOCKED SSL", "HTTPS", &blocked_url);
            // Return -1 to signal SSL_ERROR_SYSCALL, forcing proper error handling
            return -1;
        }

        REAL_SSL_WRITE(ssl, buf, num)
    }
}

#[allow(dead_code)]
pub fn enable_verbose_logging() {
    SSL_VERBOSE.store(true, Ordering::Relaxed);
}

#[allow(dead_code)]
pub fn disable_verbose_logging() {
    SSL_VERBOSE.store(false, Ordering::Relaxed);
}
