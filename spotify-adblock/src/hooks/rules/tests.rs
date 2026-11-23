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
fn ad_rules_cover_in_app_message_upsells() {
    assert!(is_ad_related_url(
        "https://spclient.wg.spotify.com/quicksilver/v2/triggers?trig_type=URI&ctv_type=web-modal&"
    ));
    assert!(is_ad_related_url(
        "https://gae2-spclient.spotify.com/quicksilver/v2/messages?ctv_type=web-modal&trigger=app%3Astart&locale=en&trig_type=CLIENT_EVENT"
    ));
    assert!(is_ad_related_url(
        "https://spclient.wg.spotify.com/pendragon/com.spotify.pendragon.v1.ClientMessageService/FetchMessageList"
    ));
    assert!(!is_ad_related_url(
        "https://spclient.wg.spotify.com/metadata/4/track/abc"
    ));
}

#[test]
fn ad_rules_cover_ad_metadata_sponsored_content_and_gabo_events() {
    assert!(is_ad_related_url(
        "https://spclient.wg.spotify.com/ad-dsa-metadata-lookup/v1/lookup"
    ));
    assert!(is_ad_related_url(
        "https://api-partner.spotify.com/pathfinder/v2/query?operationName=episodeSponsoredContent"
    ));
    assert!(is_ad_related_url(
        "https://spclient.wg.spotify.com/gabo-receiver-service/v3/events"
    ));
    assert!(is_ad_related_url(
        "https://spclient.wg.spotify.com/gabo-receiver-service/public/v3/events/"
    ));
    assert!(!is_ad_related_url(
        "https://spclient.wg.spotify.com/gabo-receiver-service/v3/other"
    ));
    assert!(!is_ad_related_url(
        "https://api-partner.spotify.com/pathfinder/v2/query?operationName=home"
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
