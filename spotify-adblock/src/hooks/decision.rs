//! The block/allow decision shared by every request path: native
//! `cef_urlrequest_create` calls and the UI's own (Chromium network) requests.

use crate::config::CONFIG;

use super::request_classification::classify_url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Verdict {
    /// Let the request through; the label says why.
    Allow(&'static str),
    /// Refuse the request; the label says which rule matched.
    Block(&'static str),
}

pub(super) fn decide(url: &str, method: &str) -> Verdict {
    let classification = classify_url(url, method);

    if classification.is_discord_rpc {
        return Verdict::Allow("DISCORD RPC");
    }
    if classification.is_gabo || classification.is_dealer {
        return Verdict::Allow("SERVICE");
    }
    // Gabo event POSTs can carry ad data.
    if classification.is_gabo_event_post {
        return Verdict::Block("BLOCKED GABO POST");
    }
    if classification.is_ad_related {
        return Verdict::Block("BLOCKED AD");
    }
    if CONFIG.denylist.is_match(url) {
        return Verdict::Block("BLOCKED CONFIG");
    }
    Verdict::Allow("ALLOWED")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_ui_upsell_and_ad_routes() {
        for url in [
            "https://gue1-spclient.spotify.com/quicksilver/v2/triggers?trig_type=URI&ctv_type=web-modal&",
            "https://spclient.wg.spotify.com/quicksilver/v2/messages?ctv_type=web-modal&trigger=app%3Astart",
            "https://spclient.wg.spotify.com/ads/v3/ads?slots=hpto",
            "https://spclient.wg.spotify.com/sponsoredplaylist/v1/sponsored",
        ] {
            assert!(matches!(decide(url, "GET"), Verdict::Block(_)), "{url}");
        }
    }

    #[test]
    fn allows_ordinary_ui_traffic() {
        for url in [
            "https://api-partner.spotify.com/pathfinder/v2/query",
            "https://i.scdn.co/image/ab67616d0000b273abcdef",
            "https://gue1-spclient.spotify.com/color-lyrics/v2/track/abc",
            "https://xpui.app.spotify.com/index.html",
            "https://gue1-spclient.spotify.com/playlist/v2/playlist/37i9dQZF1DX0XUsuxWHRQd",
        ] {
            assert!(matches!(decide(url, "GET"), Verdict::Allow(_)), "{url}");
        }
    }

    #[test]
    fn dealer_and_discord_traffic_is_always_allowed() {
        assert_eq!(decide("wss://gue1-dealer.g2.spotify.com/?access_token=x", "GET"), Verdict::Allow("SERVICE"));
        assert_eq!(
            decide("https://gue1-spclient.spotify.com/connect-state/v1/devices/abc", "PUT"),
            Verdict::Allow("DISCORD RPC")
        );
    }
}
