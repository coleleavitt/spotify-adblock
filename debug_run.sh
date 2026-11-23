#!/bin/bash
for pid in $(pgrep -f spotify-client); do
    kill -9 $pid 2>/dev/null
done
sleep 1

export LD_PRELOAD=$HOME/.local/lib/spotify-adblock/libspotifyadblock.so
export LD_LIBRARY_PATH="/usr/lib64/apulse:/usr/lib/ffmpeg7/lib64"

# Inherit session env from a running GUI process
eval "$(tr '\0' '\n' < /proc/$(pgrep -u $USER zen-browser | head -1 || pgrep -u $USER -x kitty | head -1)/environ 2>/dev/null | grep -E '^(DBUS_SESSION_BUS_ADDRESS|WAYLAND_DISPLAY|DISPLAY|XDG_RUNTIME_DIR|XDG_SESSION)=' | sed 's/^/export /')"

LOGFILE="/home/cole/.prime/agent/session-artifacts/01a1da6b-e18e-7729-9293-3b1cd74d0b75/scratchpad/spotify_blocking.log"
exec /opt/spotify/spotify-client/spotify --no-zygote --force-device-scale-factor=1 > "$LOGFILE" 2>&1
