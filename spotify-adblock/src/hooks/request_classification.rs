#![allow(clippy::struct_excessive_bools)]

use super::rules;

const MAX_URL_LENGTH: usize = 2048;

pub(super) struct UrlClassification {
    pub(super) is_discord_rpc: bool,
    pub(super) is_gabo: bool,
    pub(super) is_dealer: bool,
    pub(super) is_ad_related: bool,
    pub(super) is_product_state: bool,
    pub(super) is_gabo_event_post: bool,
}

pub(super) fn classify_url(url: &str, method: &str) -> UrlClassification {
    let url = bounded_url(url);

    UrlClassification {
        is_discord_rpc: is_discord_rpc(url),
        is_gabo: is_allowed_gabo_service(url),
        is_dealer: url.contains("dealer"),
        is_ad_related: rules::is_ad_related_url(url),
        is_product_state: is_product_state(url),
        is_gabo_event_post: is_gabo_event_post(url, method),
    }
}

fn bounded_url(url: &str) -> &str {
    if url.len() > MAX_URL_LENGTH {
        &url[0..MAX_URL_LENGTH]
    } else {
        url
    }
}

fn is_discord_rpc(url: &str) -> bool {
    url.contains("discord")
        || url.contains("discordapp")
        || url.contains("presence")
        || url.contains("/presence2/")
        || url.contains("connect-state")
        || url.contains("rpc")
}

fn is_allowed_gabo_service(url: &str) -> bool {
    url.contains("gabo-receiver-service")
        && !url.contains("/advertisement")
        && !url.contains("/ad-opportunity")
        && !url.contains("/adlogic")
        && !url.contains("/ads")
        && !url.contains("/v3/events/")
        && !url.contains("/public/v3/events/")
}

fn is_gabo_event_post(url: &str, method: &str) -> bool {
    url.contains("gabo-receiver-service") && url.contains("/events") && method == "POST"
}

fn is_product_state(url: &str) -> bool {
    url.contains("product_state") || url.contains("product-state")
}
