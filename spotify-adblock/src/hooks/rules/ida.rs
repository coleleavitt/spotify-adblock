use super::matchers::{contains_any, is_spotify_client_url};

pub(super) fn is_ida_ad_signal(url: &str) -> bool {
    ad_event_reporting(url)
        || podcast_ad_segment(url)
        || ad_pod_or_decision_tree(url)
        || esperanto_ad_service(url)
        || ad_tracking_attribution(url)
        || ad_stream_reporting(url)
        || legacy_ida_ad_signal(url)
}

fn ad_event_reporting(url: &str) -> bool {
    url.contains("audio_ad_event_reporter")
        || url.contains("/AdEvent")
        || url.contains("/EndAd")
        || url.contains("/AdDecision")
        || url.contains("/AdDecisionEvent")
        || url.contains("/AdRequestEvent")
        || url.contains("/AdTransparencyEvent")
        || url.contains("/AdDetectionResult")
}

fn podcast_ad_segment(url: &str) -> bool {
    url.contains("/PodcastAdSegment")
        || url.contains("/GetNextAdSegment")
        || (is_spotify_client_url(url) && url.contains("nextAdSegment"))
        || url.contains("AdSegmentsMetadataReceived")
}

fn ad_pod_or_decision_tree(url: &str) -> bool {
    url.contains("/AdPodResponse") || url.contains("/AdDecisionTree")
}

fn esperanto_ad_service(url: &str) -> bool {
    url.contains("esperanto")
        && (url.contains("/ads/")
            || url.contains("/Ads")
            || url.contains("AdOpportunity")
            || url.contains("PodcastAds")
            || url.contains("/Targeting")
            || url.contains("/ad-")
            || url.contains("_ad_"))
}

fn ad_tracking_attribution(url: &str) -> bool {
    url.contains("/branch_io")
        || url.contains("/branchIo")
        || url.contains("branch-io")
        || (is_spotify_client_url(url)
            && contains_any(
                url,
                &[
                    "/partner_user_id",
                    "/partner-user-id",
                    "partner-userid",
                    "partnerUserId",
                ],
            ))
}

fn ad_stream_reporting(url: &str) -> bool {
    (url.contains("stream_reporting") || url.contains("stream-reporting"))
        && stream_reporting_ad_marker(url)
}

fn stream_reporting_ad_marker(url: &str) -> bool {
    contains_any(
        url,
        &[
            "/ad/",
            "/ads/",
            "/ad-",
            "/ad_",
            "/ad.",
            "_ad_",
            "-ad-",
            ".ad.",
            ":ad:",
            "ad-event",
            "ad_event",
            "adEvent",
            "AdEvent",
            "AdDecision",
            "audio_ad",
            "sponsor",
            "promotion",
            "promoted",
        ],
    )
}

fn legacy_ida_ad_signal(url: &str) -> bool {
    contains_any(
        url,
        &[
            "open.spotify.com/ad/",
            "spotify:ad:",
            "open.spotify.com/interruption/",
            "spotify:interruption:",
            "open.spotify.com/promotion/",
            "contains_sponsored_content",
            "SponsoredContentListenerPayload",
            "PODCAST_SPONSORED_CONTENT",
            "slot_has_active_ad",
            "slot_fetching_turned_off",
            "PrepareSlotRequest",
            "AdPodResponse",
            "SlotRealtimeDecisions",
            "audio_ad_event",
            "viewable_impression",
            "fire_impression_on_end",
            "tracking_events",
            "trackingEvents",
            "PROMO_V1_TRAIT",
            "PROMO_V3_TRAIT",
            "AUDIOBOOK_PROMOTION",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ida_ad_misses_when_present() {
        assert!(is_ida_ad_signal(
            "https://spclient.wg.spotify.com/v1/podcast/nextAdSegment"
        ));
        assert!(is_ida_ad_signal(
            "https://spclient.wg.spotify.com/foo/partner-userid/encrypted/bar"
        ));
        assert!(is_ida_ad_signal("/AdDecision"));
    }

    #[test]
    fn leaves_spotify_client_specific_routes_host_scoped() {
        assert!(!is_ida_ad_signal("https://example.com/v1/podcast/nextAdSegment"));
        assert!(!is_ida_ad_signal("https://example.com/foo/partner-userid"));
    }

    #[test]
    fn stream_reporting_uses_explicit_ad_markers() {
        assert!(is_ida_ad_signal(
            "https://spclient.wg.spotify.com/stream_reporting/ad-event"
        ));
        assert!(is_ida_ad_signal(
            "https://spclient.wg.spotify.com/stream-reporting/sponsor"
        ));
        assert!(!is_ida_ad_signal(
            "https://spclient.wg.spotify.com/stream_reporting/metadata"
        ));
        assert!(!is_ida_ad_signal(
            "https://spclient.wg.spotify.com/stream_reporting/download"
        ));
    }
}
