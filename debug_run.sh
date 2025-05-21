#!/bin/bash
for pid in $(pgrep -f spotify); do
    kill -9 $pid
done

LD_PRELOAD=./target/release/libspotifyadblock.so spotify --enable-features=useozoneplatform --ozone-platform=wayland
