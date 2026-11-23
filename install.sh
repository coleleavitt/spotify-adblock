#!/bin/sh
# spotify-adblock installer.
#
#   curl -fsSL https://raw.githubusercontent.com/coleleavitt/spotify-adblock/main/install.sh | sh
#   curl -fsSL .../install.sh | sh -s -- --privacy-hard
#   curl -fsSL .../install.sh | sh -s -- --uninstall
#
# Installs, per user (no root):
#   ~/.local/lib/spotify-adblock/libspotifyadblock.so   the LD_PRELOAD library
#   ~/.config/spotify-adblock/config.toml              allowlist/denylist
#   ~/.local/bin/spotify-adblock                       launcher (spotify with the preload)
#   ~/.local/share/applications/spotify-adblock.desktop
set -eu

REPO=coleleavitt/spotify-adblock
MARKER="# managed by spotify-adblock install.sh"

version=latest
variant=
prefix=${HOME}/.local
config_dir=${XDG_CONFIG_HOME:-${HOME}/.config}/spotify-adblock
apps_dir=${XDG_DATA_HOME:-${HOME}/.local/share}/applications
action=install
keep_config=0
force=0

usage() {
    cat <<EOF
usage: install.sh [options]

  --version X.Y.Z   install that release instead of the latest
  --privacy-hard    install the build that also blocks Spotify telemetry
  --prefix DIR      install lib/ and bin/ under DIR (default: ~/.local)
  --keep-config     keep an existing config.toml instead of updating it
  --force           overwrite a launcher or desktop file this script did not create
  --uninstall       remove what this script installed (config is kept)
  -h, --help        show this help
EOF
}

say() { printf '%s\n' "spotify-adblock: $*"; }
die() { printf '%s\n' "spotify-adblock: error: $*" >&2; exit 1; }

while [ $# -gt 0 ]; do
    case $1 in
        --version) [ $# -ge 2 ] || die "--version needs a value"; version=${2#v}; shift ;;
        --privacy-hard) variant=-privacy-hard ;;
        --prefix) [ $# -ge 2 ] || die "--prefix needs a value"; prefix=$2; shift ;;
        --keep-config) keep_config=1 ;;
        --force) force=1 ;;
        --uninstall) action=uninstall ;;
        -h | --help) usage; exit 0 ;;
        *) usage >&2; die "unknown option: $1" ;;
    esac
    shift
done

lib_dir=$prefix/lib/spotify-adblock
lib=$lib_dir/libspotifyadblock.so
launcher=$prefix/bin/spotify-adblock
desktop=$apps_dir/spotify-adblock.desktop

# Files we generate carry MARKER; leave anyone else's alone unless --force.
is_ours() { [ ! -e "$1" ] || [ "$force" = 1 ] || grep -qF "$MARKER" "$1"; }

if [ "$action" = uninstall ]; then
    rm -f "$lib" && rmdir "$lib_dir" 2>/dev/null || true
    for f in "$launcher" "$desktop"; do
        if [ -e "$f" ] && grep -qF "$MARKER" "$f"; then rm -f "$f"; fi
    done
    say "uninstalled (config left in $config_dir)"
    exit 0
fi

[ "$(uname -s)" = Linux ] || die "only Linux is supported"
[ "$(uname -m)" = x86_64 ] || die "only x86_64 builds are published; build from source on $(uname -m)"

if [ -n "${SPOTIFY_ADBLOCK_BASE_URL:-}" ]; then
    base=$SPOTIFY_ADBLOCK_BASE_URL
elif [ "$version" = latest ]; then
    base=https://github.com/$REPO/releases/latest/download
else
    base=https://github.com/$REPO/releases/download/v$version
fi

if command -v curl >/dev/null 2>&1; then
    fetch() { curl -fsSL --proto '=https,file' --retry 3 -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
    fetch() { wget -q -O "$2" "$1"; }
else
    die "curl or wget is required"
fi
command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

asset=spotify-adblock$variant-x86_64-linux.so
say "downloading $asset ($version)"
for f in "$asset" config.toml SHA256SUMS; do
    fetch "$base/$f" "$tmp/$f" || die "failed to download $base/$f"
done
(cd "$tmp" && grep -E " (\*)?($asset|config\.toml)$" SHA256SUMS | sha256sum -c --quiet -) ||
    die "checksum mismatch"

mkdir -p "$lib_dir" "$prefix/bin" "$config_dir" "$apps_dir"

# Install next to the old file and rename over it, so a running Spotify keeps
# its mapped copy instead of crashing on a truncated library.
install -m 755 "$tmp/$asset" "$lib.new" && mv -f "$lib.new" "$lib"
say "installed $lib"

config=$config_dir/config.toml
if [ ! -e "$config" ]; then
    install -m 644 "$tmp/config.toml" "$config"
    say "installed $config"
elif cmp -s "$tmp/config.toml" "$config"; then
    :
elif [ "$keep_config" = 1 ]; then
    install -m 644 "$tmp/config.toml" "$config.new"
    say "kept $config; the release config is at $config.new"
else
    backup=$config.bak.$(date +%Y%m%d%H%M%S)
    cp "$config" "$backup"
    install -m 644 "$tmp/config.toml" "$config"
    say "updated $config (previous one saved as $backup)"
fi

if is_ours "$launcher"; then
    cat >"$launcher" <<EOF
#!/bin/sh
$MARKER
lib="\${SPOTIFY_ADBLOCK_LIB:-$lib}"
[ -r "\$lib" ] || { echo "spotify-adblock: missing \$lib" >&2; exit 127; }
export LD_PRELOAD="\$lib\${LD_PRELOAD:+:\$LD_PRELOAD}"
exec "\${SPOTIFY_BIN:-spotify}" "\$@"
EOF
    chmod 755 "$launcher"
    say "installed $launcher"
else
    say "left your own $launcher in place (use --force to replace it)"
fi

if is_ours "$desktop"; then
    cat >"$desktop" <<EOF
$MARKER
[Desktop Entry]
Type=Application
Name=Spotify (adblock)
GenericName=Music Player
Icon=spotify-client
TryExec=$launcher
Exec=$launcher %U
Terminal=false
MimeType=x-scheme-handler/spotify;
Categories=Audio;Music;Player;AudioVideo;
StartupWMClass=spotify
EOF
    say "installed $desktop"
else
    say "left your own $desktop in place (use --force to replace it)"
fi

case ":$PATH:" in
    *":$prefix/bin:"*) ;;
    *) say "note: $prefix/bin is not on your PATH" ;;
esac
say "done; start Spotify with: spotify-adblock (restart it if it is running)"
