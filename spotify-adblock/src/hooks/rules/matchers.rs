pub(super) fn contains_any(url: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| url.contains(needle))
}

pub(super) fn is_spotify_client_url(url: &str) -> bool {
    url_host(url).is_some_and(is_spotify_client_host)
}

fn url_host(url: &str) -> Option<&str> {
    let (_, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host_port)| host_port);
    let host = host_port.split_once(':').map_or(host_port, |(host, _)| host);
    Some(host.trim_end_matches('.'))
}

fn is_spotify_client_host(host: &str) -> bool {
    const SPCLIENT_SUFFIX: &str = "-spclient.spotify.com";

    host.eq_ignore_ascii_case("spclient.wg.spotify.com")
        || host
            .get(host.len().saturating_sub(SPCLIENT_SUFFIX.len())..)
            .is_some_and(|tail| tail.eq_ignore_ascii_case(SPCLIENT_SUFFIX))
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
        assert!(is_spotify_client_url("https://spclient.wg.spotify.com:443/foo"));
        assert!(is_spotify_client_url("https://SPCLIENT.WG.SPOTIFY.COM/foo"));
        assert!(is_spotify_client_url("https://gae2-spclient.spotify.com/foo"));
        assert!(!is_spotify_client_url("https://example.com/foo"));
        assert!(!is_spotify_client_url(
            "https://spclient.wg.spotify.com.evil.tld/foo"
        ));
        assert!(!is_spotify_client_url(
            "https://example.com/?next=https://spclient.wg.spotify.com/foo"
        ));
    }
}
