mod ad;
mod ida;
mod matchers;
mod privacy;

#[cfg(test)]
mod tests;

pub(super) use ad::is_ad_related_url;

/// Uploads that the client retries until they succeed (event batches, metrics,
/// error reports). Blocking them is better answered with an empty success than
/// a network error, so the client drops the batch instead of retrying forever.
pub(super) fn is_telemetry_upload(url: &str) -> bool {
    matchers::contains_any(
        url,
        &[
            "/gabo-receiver-service/",
            "/melody/v1/msg/",
            ".ingest.sentry.io/",
            "spotify.com/api/growth-events/",
        ],
    )
}
