# spotify-adblock

Spotify adblocker for Linux (macOS untested) that works by wrapping `getaddrinfo` and `cef_urlrequest_create`. It blocks requests to domains that are not on the allowlist, as well as URLs that are on the denylist.

## Recent Updates

* Now compatible with Rust 2024 edition
* Complete codebase restructuring with modular architecture for better maintainability
* Implemented JPL safety standards with radiation hardening and fault containment
* Improved code safety with explicit unsafe blocks and triple modular redundancy
* Enhanced blocking of ad-related endpoints
* Support for cargo-make build system alongside traditional Makefile

### Notes
* This **does not** work with the snap Spotify package.
* This **might not** work with the Flatpak Spotify package, depending on your system's shared libraries' versions.
* On Debian-based distributions (e.g. Ubuntu), the Debian Spotify package can be installed by following the instructions at the bottom of [this page](https://www.spotify.com/us/download/linux/). *(recommended)*

## Build
Prerequisites:
* Git
* Make or cargo-make
* Rust 1.75+ (supports 2024 edition)
* [Cargo](https://doc.rust-lang.org/cargo/)
* CEF (Chromium Embedded Framework) binary - automatically downloaded in CI/CD

### Download CEF Binary (Local Development)
```bash
# Download CEF binary for your platform from Spotify's CDN
wget https://cef-builds.spotifycdn.com/cef_binary_150.0.1%2Bg3f36c80%2Bchromium-150.0.7871.4_linux64_beta.tar.bz2
tar -xjf cef_binary_150.0.1+g3f36c80+chromium-150.0.7871.4_linux64_beta.tar.bz2
```

### Build Options

**Using debug_run.sh (Recommended for Development):**
```bash
$ git clone https://github.com/coleleavitt/spotify-adblock.git
$ cd spotify-adblock
# Download CEF binary first (see above)
$ ./debug_run.sh
```

**Using traditional Make:**
```bash
$ export CEF_ROOT="$PWD/cef_binary_150.0.1+g3f36c80+chromium-150.0.7871.4_linux64_beta"
$ make
```

**Using cargo-make (install with: cargo install cargo-make):**
```bash
# cargo-make will auto-detect CEF in project directory
$ cargo make build
```

**Manual build with cargo:**
```bash
$ export CEF_ROOT="$PWD/cef_binary_150.0.1+g3f36c80+chromium-150.0.7871.4_linux64_beta"
$ cargo build --release --lib
```

**Privacy hard build:**
```bash
$ export CEF_ROOT="$PWD/cef_binary_150.0.1+g3f36c80+chromium-150.0.7871.4_linux64_beta"
$ cargo build --release --lib -p spotify-adblock --features privacy-hard-blocking
```

This opt-in feature also blocks broad Spotify telemetry routes found in the IDA dump, including event-service, logging, event sender, pending events, stream reporting, remote config, and common capping traffic. It may affect Wrapped, listening history, recommendations, or diagnostics.

## Install
```bash
# Using traditional Make
$ sudo make install

# Using cargo-make
$ sudo cargo make install
```

#### Flatpak
```bash
$ mkdir -p ~/.spotify-adblock && cp target/release/libspotifyadblock.so ~/.spotify-adblock/spotify-adblock.so
$ mkdir -p ~/.var/app/com.spotify.Client/config/spotify-adblock && cp config.toml ~/.var/app/com.spotify.Client/config/spotify-adblock
$ flatpak override --user --filesystem="~/.spotify-adblock/spotify-adblock.so" --filesystem="~/.config/spotify-adblock/config.toml" com.spotify.Client
```

## Usage
### Command-line
```bash
$ LD_PRELOAD=/usr/local/lib/spotify-adblock.so spotify
```

#### Debug Mode
You can enable debug mode to see all requests (blocked and allowed) by setting the `SPOTIFY_ADBLOCK_DEBUG` environment variable:

```bash
$ SPOTIFY_ADBLOCK_DEBUG=1 LD_PRELOAD=/usr/local/lib/spotify-adblock.so spotify
```

#### Flatpak
```bash
$ flatpak run --command=sh com.spotify.Client -c 'eval "$(sed s#LD_PRELOAD=#LD_PRELOAD=$HOME/.spotify-adblock/spotify-adblock.so:#g /app/bin/spotify)"'
```

### Desktop file
You can integrate it with your desktop environment by creating a `.desktop` file (e.g. `spotify-adblock.desktop`) in `~/.local/share/applications`. This lets you easily run it from an application launcher without opening a terminal.

Examples:

<details> 
  <summary>Debian Package</summary>
  <p>

```desktop
[Desktop Entry]
Type=Application
Name=Spotify (adblock)
GenericName=Music Player
Icon=spotify-client
TryExec=spotify
Exec=env LD_PRELOAD=/usr/local/lib/spotify-adblock.so spotify %U
Terminal=false
MimeType=x-scheme-handler/spotify;
Categories=Audio;Music;Player;AudioVideo;
StartupWMClass=spotify
```
  </p>
</details>

<details>
  <summary>Flatpak</summary>
  <p>

```desktop
[Desktop Entry]
Type=Application
Name=Spotify (adblock)
GenericName=Music Player
Icon=com.spotify.Client
Exec=flatpak run --file-forwarding --command=sh com.spotify.Client -c 'eval "$(sed s#LD_PRELOAD=#LD_PRELOAD=$HOME/.spotify-adblock/spotify-adblock.so:#g /app/bin/spotify)"' @@u %U @@
Terminal=false
MimeType=x-scheme-handler/spotify;
Categories=Audio;Music;Player;AudioVideo;
StartupWMClass=spotify
```
  </p>
</details>

## Uninstall
```sh
# Using traditional Make
$ sudo make uninstall

# Using cargo-make
$ sudo cargo make uninstall
```

#### Flatpak
```sh
$ rm -r ~/.spotify-adblock ~/.config/spotify-adblock
$ flatpak override --user --reset com.spotify.Client
```

## Configuration
The allowlist and denylist can be configured in a config file located at (in descending order of precedence):
* `config.toml` in the working directory
* `$XDG_CONFIG_HOME/spotify-adblock/config.toml`
* `~/.config/spotify-adblock/config.toml`
* `/etc/spotify-adblock/config.toml` *(default)*

## How It Works

The adblocker uses two main strategies to block ads:
1. **Domain filtering**: Uses the `getaddrinfo` hook to block connections to domains not on the allowlist
2. **URL filtering**: Uses the `cef_urlrequest_create` hook to block URLs on the denylist

Special categories automatically handled:
* Discord RPC connections (allowed)
* Dealer/websocket connections (allowed)
* Ad-related endpoints (blocked)

## Safety Features

This project now implements several safety features following JPL coding standards:
* Triple modular redundancy for critical functions
* Bounded execution with memory limits
* Explicit unsafe blocks with proper error handling
* Overflow checks and fault containment
