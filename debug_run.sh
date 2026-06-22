#!/bin/bash
for pid in $(pgrep -f spotify); do
    kill -9 $pid
done

# If you got CEF for testing
export CEF_ROOT="${CEF_ROOT:-$PWD/cef_binary_150.0.1+g3f36c80+chromium-150.0.7871.4_linux64_beta}"
cargo check
cargo build --release --lib

LD_PRELOAD=./target/release/libspotifyadblock.so spotify --enable-features=useozoneplatform --ozone-platform=wayland --remote-debugging-port=9222
