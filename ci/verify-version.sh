#!/bin/sh
# Single source of truth for the release version is [workspace.package] version
# in Cargo.toml (like rust-lang/rust's src/version). This fails when
#   - Cargo.lock is out of date with Cargo.toml, or
#   - the ref being built is a tag that does not match that version.
# Prints the version on success.
set -eu

version=$(cargo metadata --locked --format-version 1 |
    python3 -c 'import json, sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "spotify-adblock"))')

case "${GITHUB_REF:-}" in
    refs/tags/*)
        tag=${GITHUB_REF#refs/tags/}
        if [ "$tag" != "v$version" ]; then
            echo "error: tag $tag does not match Cargo.toml version $version" >&2
            exit 1
        fi
        ;;
esac

echo "$version"
