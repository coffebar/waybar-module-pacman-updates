# waybar module for Arch to show system updates available

## How to use

 - install binary `waybar-module-pacman-updates` to your *PATH*
 - add to ~/.config/waybar/config 

 ```json
    "custom/updates": {
        "format": "{} {icon}",
        "return-type": "json",
        "format-icons": {
            "has-updates": "󱍷",
            "updated": "󰂪"
        },
        "exec-if": "which waybar-module-pacman-updates",
        "exec": "waybar-module-pacman-updates"
    }
 ```

 ### Building binary from source

 ```bash
rustup override set stable
rustup update stable

git clone https://github.com/coffebar/waybar-module-pacman-updates.git /tmp/waybar-module-pacman-updates
pushd /tmp/waybar-module-pacman-updates && cargo build --release

mkdir -p ~/.local/bin/
cp target/release/waybar-module-pacman-updates ~/.local/bin/

popd && rm -rf /tmp/waybar-module-pacman-updates
```
