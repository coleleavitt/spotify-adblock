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
    let is_gabo_event_post = is_gabo_event_post(url, method);

    UrlClassification {
        is_discord_rpc: is_discord_rpc(url),
        is_gabo: is_allowed_gabo_service(url) && !is_gabo_event_post,
        is_dealer: url.contains("dealer"),
        is_ad_related: rules::is_ad_related_url(url),
        is_product_state: is_product_state(url),
        is_gabo_event_post,
    }
}

fn bounded_url(url: &str) -> &str {
    if url.len() <= MAX_URL_LENGTH {
        return url;
    }

    let mut cutoff = MAX_URL_LENGTH;
    while !url.is_char_boundary(cutoff) {
        cutoff -= 1;
    }
    &url[..cutoff]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_url_truncates_at_utf8_boundary() {
        let url = format!("{}é", "a".repeat(MAX_URL_LENGTH - 1));

        assert_eq!(bounded_url(&url), "a".repeat(MAX_URL_LENGTH - 1));
    }

    #[test]
    fn gabo_event_post_does_not_get_service_allowance() {
        let classification = classify_url("https://gabo-receiver-service.spotify.com/events", "POST");

        assert!(classification.is_gabo_event_post);
        assert!(!classification.is_gabo);
    }
}
