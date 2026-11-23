# spotify-adblock

Ad blocker for the Spotify desktop client on Linux. It is an `LD_PRELOAD` library that hooks
`getaddrinfo` and `cef_urlrequest_create` and blocks ad, upsell and (optionally) telemetry
requests before they leave the client.

Works with the official Spotify package (deb/AUR/Gentoo, `/opt/spotify` or `/usr/share/spotify`).
It does **not** work with the Snap package.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/coleleavitt/spotify-adblock/main/install.sh | sh
```

This installs, for your user only:

| File | Purpose |
| --- | --- |
| `~/.local/lib/spotify-adblock/libspotifyadblock.so` | the preload library |
| `~/.config/spotify-adblock/config.toml` | allowlist and denylist |
| `~/.local/bin/spotify-adblock` | launcher: runs `spotify` with the library preloaded |
| `~/.local/share/applications/spotify-adblock.desktop` | "Spotify (adblock)" in your app launcher |

Then start Spotify with `spotify-adblock` (or from the launcher). Re-run the same command to update.

Options go after `sh -s --`:

```sh
curl -fsSL .../install.sh | sh -s -- --privacy-hard    # also block telemetry (see below)
curl -fsSL .../install.sh | sh -s -- --version 2.0.0   # a specific release
curl -fsSL .../install.sh | sh -s -- --uninstall
```

`--prefix DIR`, `--keep-config` and `--force` are described in `install.sh --help`. The launcher honours
`SPOTIFY_BIN` (Spotify executable) and `SPOTIFY_ADBLOCK_LIB` (library path).

### Privacy-hard build

`--privacy-hard` installs a build that also blocks Spotify's event-service, logging, event sender,
pending events, stream reporting, remote config and common-capping traffic. Wrapped, listening
history and recommendations may suffer.

### Flatpak

```sh
curl -fsSL .../install.sh | sh
flatpak override --user --filesystem=~/.local/lib/spotify-adblock:ro --filesystem=~/.config/spotify-adblock:ro com.spotify.Client
flatpak run --command=sh com.spotify.Client -c 'eval "$(sed s#LD_PRELOAD=#LD_PRELOAD=$HOME/.local/lib/spotify-adblock/libspotifyadblock.so:#g /app/bin/spotify)"'
```

It may fail if the Flatpak runtime's glibc is older than 2.34.

## Configuration

`config.toml` is read from, in order: `./config.toml`, `$XDG_CONFIG_HOME/spotify-adblock/config.toml`
(default `~/.config/...`), `/etc/spotify-adblock/config.toml`.

* `allowlist`: regexes of hostnames `getaddrinfo` may resolve (`*.spotify.com` is always allowed).
* `denylist`: regexes of URLs `cef_urlrequest_create` refuses.

Built-in rules (`spotify-adblock/src/hooks/rules/`) block known ad, sponsorship and in-app upsell
endpoints regardless of the config.

Set `SPOTIFY_ADBLOCK_DEBUG=1` to log every request instead of blocking. Run `spotify-adblock` from a
terminal to see the `[+]` allowed and `[-]` blocked lines.

## Build from source

Building needs Rust 1.85+ and a [CEF binary distribution](https://cef-builds.spotifycdn.com/index.html)
(for the C headers):

```sh
git clone https://github.com/coleleavitt/spotify-adblock && cd spotify-adblock
curl -LO https://cef-builds.spotifycdn.com/cef_binary_<version>_linux64.tar.bz2 && tar xjf cef_binary_*.tar.bz2
export CEF_ROOT=$PWD/$(ls -d cef_binary_*/ | head -1)
cargo build --release                                              # target/release/libspotifyadblock.so
cargo build --release --features spotify-adblock/privacy-hard-blocking
```

Then run `LD_PRELOAD=$PWD/target/release/libspotifyadblock.so spotify`, or `sudo make install` to put
it in `/usr/local/lib/spotify-adblock.so` and `/etc/spotify-adblock/config.toml`.

## Releasing

The version lives only in `Cargo.toml` (`[workspace.package] version`). Either:

* bump it, commit and push to `main`, or
* run **Actions → release → Run workflow** with the new version (the workflow commits the bump).

The `release` workflow then tests and builds both variants and publishes `v<version>` with the
libraries, `config.toml`, `install.sh` and `SHA256SUMS`. `ci/verify-version.sh` fails CI if
`Cargo.lock` or a tag disagrees with that version.
