use super::ida;
use super::matchers::contains_any;
use super::privacy;

pub(in crate::hooks) fn is_ad_related_url(url: &str) -> bool {
    !is_critical_allowlisted(url)
        && (core_ad_endpoint(url)
            || audio_ad_content(url)
            || spotify_ad_domain(url)
            || third_party_ad_network(url)
            || podcast_ad_or_tracking(url)
            || ad_specific_analytics(url)
            || sponsored_or_promoted_content(url)
            || display_video_or_creative_ad(url)
            || skip_limit_or_restriction(url)
            || display_segment_ad(url)
            || metadata_queue_or_playlist_ad(url)
            || entitlement_ad_check(url)
            || gabo_ad_event(url)
            || ida::is_ida_ad_signal(url)
            || concert_location_tracking(url)
            || leavebehind_ad(url)
            || misc_ad_related(url)
            || privacy::is_privacy_hard_url(url))
}

fn is_critical_allowlisted(url: &str) -> bool {
    contains_any(
        url,
        &[
            "/license/user",
            "/product_state/get",
            "/subscription/status",
            "/user/product",
            "/subscription/validate",
        ],
    )
}

fn core_ad_endpoint(url: &str) -> bool {
    contains_any(
        url,
        &[
            "/ads/",
            "sp://ads/v1/ads/",
            "/v1/ads/",
            "ad-logic",
            "adlogic",
            "adsegments",
            "/adrequest",
            "/ad-request",
            "spotify.ads.esperanto.proto.",
            "spotify.ads.proto.",
            "VND.Spotify.Ads-Payload",
            "injected-ad",
        ],
    )
}

fn audio_ad_content(url: &str) -> bool {
    (url.contains("audio-fa.scdn.co")
        && contains_any(url, &["/ad/", "/ads/", "_ad_", "/sponsored/"]))
        || (url.contains("audio-ak.spotify.com.edgesuite.net") && url.contains("/ad"))
        || (url.contains("audio-ak-spotify-com") && url.contains("/ad"))
        || contains_any(url, &["/ad_audio/", "/sponsored_audio/"])
}

fn spotify_ad_domain(url: &str) -> bool {
    contains_any(
        url,
        &[
            "ads.spotify.com",
            "adstudio.spotify.com",
            "audio-ads.spotify.com",
            "creativeservice-production",
        ],
    )
}

fn third_party_ad_network(url: &str) -> bool {
    contains_any(
        url,
        &["pubads.google.com/ad", "doubleclick", "googleads", "adswizz"],
    )
}

fn podcast_ad_or_tracking(url: &str) -> bool {
    url.contains("megaphone.fm")
        || url.contains("art19.com")
        || (url.contains("simplecast.com") && url.contains("episodes"))
        || contains_any(url, &["chartable.com", "podsights.com", "podscribe.com"])
}

fn ad_specific_analytics(url: &str) -> bool {
    (url.contains("analytics") && contains_any(url, &["ad", "sponsor", "promotion"]))
        || contains_any(url, &["branch.io", "app.link", "adjust.com", "kochava.com"])
        || (url.contains("clientsettings")
            && url.contains("api")
            && contains_any(url, &["ad", "sponsor", "promotion"]))
        || (url.contains("track") && url.contains("event") && url.contains("ad"))
}

fn sponsored_or_promoted_content(url: &str) -> bool {
    contains_any(
        url,
        &[
            "sponsor",
            "/promotion/",
            "spotify:promotion:",
            "/partner/",
            "spotify:partner:",
            "partnership",
            "promoted",
        ],
    )
}

fn display_video_or_creative_ad(url: &str) -> bool {
    contains_any(
        url,
        &[
            "companion-ad",
            "companion_content",
            "companion-content",
            "canvas_ad",
            "canvas-ad",
            "/figs/",
            "video-ad",
            "videoad",
            "/ad.mp4",
            "/ads.mp4",
            "video-fa.scdn.co",
            "canvasVideo",
            "ad-creative",
            "ad_creative",
        ],
    ) || (url.contains("/canvas/") && url.contains("ad"))
}

fn skip_limit_or_restriction(url: &str) -> bool {
    contains_any(
        url,
        &[
            "RemainingSkipsRequest",
            "RemainingSkipsResponse",
            "skip-limit",
            "skip_limit",
            "/v1/me/player/skip-limits",
            "/skip-counter",
            "/playback/restrictions",
        ],
    )
}

fn display_segment_ad(url: &str) -> bool {
    contains_any(
        url,
        &["display-segments", "display_segments", "DisplaySegments"],
    ) && contains_any(url, &["sponsor", "promoted", "ad"])
}

fn metadata_queue_or_playlist_ad(url: &str) -> bool {
    (url.contains("/track-metadata") && url.contains("ad"))
        || (url.contains("/resolve") && url.contains("spotify:ad:"))
        || (url.contains("/metadata") && url.contains("injected"))
        || (url.contains("/queue/add") && url.contains("ad"))
        || (url.contains("/playlist/modify")
            && contains_any(url, &["ad", "injected", "sponsor"]))
        || contains_any(
            url,
            &[
                "/playlist/decoration",
                "/playlist/branding",
                "/playlist/sponsor-info",
            ],
        )
        || (url.contains("/v1/views/") && url.contains("sponsored"))
        || (url.contains("i.scdn.co") && url.contains("sponsor"))
        || (url.contains("mosaic.scdn.co") && url.contains("promo"))
}

fn entitlement_ad_check(url: &str) -> bool {
    (url.contains("/license/") && url.contains("ad"))
        || (url.contains("/entitlement/") && contains_any(url, &["ad", "sponsor"]))
}

fn gabo_ad_event(url: &str) -> bool {
    url.contains("gabo-receiver-service")
        && contains_any(
            url,
            &[
                "/advertisement",
                "/ad-opportunity",
                "/adlogic",
                "/ads",
                "/v3/events/",
                "/public/v3/events/",
            ],
        )
}

fn concert_location_tracking(url: &str) -> bool {
    contains_any(
        url,
        &["concert_location", "concert-location", "concertLocation"],
    )
}

fn leavebehind_ad(url: &str) -> bool {
    contains_any(
        url,
        &[
            "leavebehind",
            "leave-behind",
            "leave_behind",
            "podcast-ap4p/leavebehind",
            "podcast-ap4p/leavebehinds",
            "podcast-ap4p",
            "/ap4p/",
            "sponsoredplaylist",
            "aet.spotify.com",
            "USE_GET_LEAVEBEHIND_ADS",
            "leavebehindAds",
            "leavebehinds-wrapper",
            "leavebehinds-list",
        ],
    ) || (url.contains("graphql")
        && contains_any(url, &["leavebehind", "getLeavebehind", "GetLeavebehind"]))
}

fn misc_ad_related(url: &str) -> bool {
    ((url.contains("brand") || url.contains("branding"))
        && contains_any(
            url,
            &["/sponsor", "/promotion", "/partner", "/playlist", "/ad"],
        ))
        || contains_any(url, &["whatsapp", "hpto", "takeover"])
}
