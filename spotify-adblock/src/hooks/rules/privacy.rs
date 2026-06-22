#[cfg(feature = "privacy-hard-blocking")]
pub(in crate::hooks) fn is_privacy_hard_url(url: &str) -> bool {
    privacy_hard_telemetry(url)
}

#[cfg(not(feature = "privacy-hard-blocking"))]
pub(in crate::hooks) const fn is_privacy_hard_url(_url: &str) -> bool {
    false
}

#[cfg(feature = "privacy-hard-blocking")]
fn privacy_hard_telemetry(url: &str) -> bool {
    logging_route(url)
        || event_sender_route(url)
        || pending_events_route(url)
        || stream_reporting_route(url)
        || remote_config_route(url)
        || common_capping_route(url)
}

#[cfg(feature = "privacy-hard-blocking")]
fn logging_route(url: &str) -> bool {
    url.contains("/event-service/v1/events")
        || url.contains("/logging/v1/")
        || url.contains("/logging/v2/")
        || url.contains("/logging/v3/")
}

#[cfg(feature = "privacy-hard-blocking")]
fn event_sender_route(url: &str) -> bool {
    url.contains("event_sender")
        || url.contains("event-sender")
        || url.contains("EventSender")
        || url.contains("Event-sender")
}

#[cfg(feature = "privacy-hard-blocking")]
fn pending_events_route(url: &str) -> bool {
    url.contains("pending_events")
        || url.contains("pending-events")
        || url.contains("PendingEvents")
}

#[cfg(feature = "privacy-hard-blocking")]
fn stream_reporting_route(url: &str) -> bool {
    url.contains("stream_reporting")
        || url.contains("stream-reporting")
        || url.contains("StreamReporting")
}

#[cfg(feature = "privacy-hard-blocking")]
fn remote_config_route(url: &str) -> bool {
    url.contains("remote_config")
        || url.contains("remote-config")
        || url.contains("RemoteConfig")
}

#[cfg(feature = "privacy-hard-blocking")]
fn common_capping_route(url: &str) -> bool {
    url.contains("commoncapping")
        || url.contains("common_capping")
        || url.contains("consumptionevent")
        || url.contains("ConsumptionEvent")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "privacy-hard-blocking"))]
    fn keeps_privacy_hard_routes_enabled_by_feature_only() {
        assert!(!is_privacy_hard_url("hm://event-service/v1/events"));
        assert!(!is_privacy_hard_url("sp://logging/v2/foo"));
        assert!(!is_privacy_hard_url("spotify.pending_events.esperanto.proto.PendingEvents"));
    }

    #[test]
    #[cfg(feature = "privacy-hard-blocking")]
    fn detects_privacy_hard_routes_when_feature_enabled() {
        assert!(is_privacy_hard_url("hm://event-service/v1/events"));
        assert!(is_privacy_hard_url("sp://logging/v3/foo"));
        assert!(is_privacy_hard_url("spotify.event_sender.proto.EventCounters"));
        assert!(is_privacy_hard_url("spotify.event_sender.proto.EventSender"));
        assert!(is_privacy_hard_url("spotify.pending_events.esperanto.proto.PendingEvents"));
        assert!(is_privacy_hard_url("spotify.stream_reporting_esperanto.proto.StreamReporting"));
        assert!(is_privacy_hard_url("spotify.remote_config.esperanto.proto.RemoteConfig"));
        assert!(is_privacy_hard_url("commoncapping/consumptionevent"));
    }
}
