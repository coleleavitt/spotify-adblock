use super::matchers::contains_any;

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
        || url.contains("/AdDecisionEvent")
        || url.contains("/AdRequestEvent")
        || url.contains("/AdTransparencyEvent")
        || url.contains("/AdDetectionResult")
}

fn podcast_ad_segment(url: &str) -> bool {
    url.contains("/PodcastAdSegment")
        || url.contains("/GetNextAdSegment")
        || url.contains("nextAdSegment")
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
        || url.contains("/partner_user_id")
        || url.contains("/partner-user-id")
        || url.contains("partner-userid")
        || url.contains("partnerUserId")
}

fn ad_stream_reporting(url: &str) -> bool {
    (url.contains("stream_reporting")
        && (url.contains("ad") || url.contains("sponsor") || url.contains("promotion")))
        || (url.contains("stream-reporting") && (url.contains("ad") || url.contains("sponsor")))
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
        assert!(is_ida_ad_signal("/v1/podcast/nextAdSegment"));
        assert!(is_ida_ad_signal(
            "https://spclient.wg.spotify.com/foo/partner-userid/encrypted/bar"
        ));
    }
}
