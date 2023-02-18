#!/bin/sh

rustup override set stable || exit -1
rustup update stable

git clone https://github.com/coffebar/waybar-module-pacman-updates.git /tmp/waybar-module-pacman-updates
pushd /tmp/waybar-module-pacman-updates && cargo build --release

mkdir -p ~/.local/bin/
cp target/release/waybar-module-pacman-updates ~/.local/bin/

popd && rm -rf /tmp/waybar-module-pacman-updates
