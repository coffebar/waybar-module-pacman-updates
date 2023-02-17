# waybar module for Arch to show system updates available

![screenshot](https://raw.githubusercontent.com/coffebar/waybar-module-pacman-updates/main/screenshot.png)

## Reason

#### Why not just exec `checkupdates` in custom waybar module? 

- This module will provide relevant local information constantly and periodically update data from the network in backgroud. Direct "checkupdates" will only give you one of two things: updating the information with a long delay or having the module constantly active on the network.
- This module has 2 states which gives you the ability to display different icons depending on status.
- Waybar expects JSON in an infinite loop from modules. So we have this.

This small program will give you fast updates with less network usage. After you have installed all the updates, the module will immediately go into the Updated state. You don't need to send signals to waybar to update this module state.


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
 - install nerd font to see icons or change icons as you like

### Install from cargo crates

```bash
cargo install waybar-module-pacman-updates
```

Make sure you have `$HOME/.cargo/bin` in your *PATH* env variable.

### Install from source

 ```bash
rustup override set stable
rustup update stable

git clone https://github.com/coffebar/waybar-module-pacman-updates.git /tmp/waybar-module-pacman-updates
pushd /tmp/waybar-module-pacman-updates && cargo build --release

mkdir -p ~/.local/bin/
cp target/release/waybar-module-pacman-updates ~/.local/bin/

popd && rm -rf /tmp/waybar-module-pacman-updates
```

Make sure you have `$HOME/.local/bin` in your *PATH* env variable.

