pub(super) fn contains_any(url: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| url.contains(needle))
}
