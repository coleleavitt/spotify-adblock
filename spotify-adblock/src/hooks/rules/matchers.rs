pub(super) fn contains_any(url: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| url.contains(needle))
}

pub(super) fn is_spotify_client_url(url: &str) -> bool {
    url.contains("spclient.wg.spotify.com") || url.contains("-spclient.spotify.com")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_any_matches_any_needle() {
        assert!(contains_any("https://example.com/ads/foo", &["/ads/", "/metrics/"]));
        assert!(!contains_any("https://example.com/tracks/foo", &["/ads/", "/metrics/"]));
    }

    #[test]
    fn spotify_client_url_matches_known_client_hosts() {
        assert!(is_spotify_client_url("https://spclient.wg.spotify.com/foo"));
        assert!(is_spotify_client_url("https://gae2-spclient.spotify.com/foo"));
        assert!(!is_spotify_client_url("https://example.com/foo"));
    }
}
