use super::is_ad_related_url;

#[test]
fn ad_rules_cover_core_routes_and_allowlisted_license() {
    assert!(is_ad_related_url("https://spclient.wg.spotify.com/v1/ads/foo"));
    assert!(!is_ad_related_url(
        "https://spclient.wg.spotify.com/license/user"
    ));
}

#[test]
fn ad_rules_cover_sponsored_and_ida_paths() {
    assert!(is_ad_related_url(
        "https://open.spotify.com/promotion/campaign"
    ));
    assert!(is_ad_related_url(
        "https://spclient.wg.spotify.com/v1/podcast/nextAdSegment"
    ));
}

#[test]
fn ad_rules_scope_playback_restrictions_to_spotify_client_hosts() {
    assert!(is_ad_related_url(
        "https://spclient.wg.spotify.com/playback/restrictions"
    ));
    assert!(!is_ad_related_url(
        "https://example.com/playback/restrictions"
    ));
}
