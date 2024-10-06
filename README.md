# waybar module for Arch to show system updates available

![screenshot](/screenshot.png)

![](https://img.shields.io/aur/version/waybar-module-pacman-updates-git)
![](https://img.shields.io/crates/v/waybar-module-pacman-updates)
![](https://img.shields.io/aur/license/waybar-module-pacman-updates-git)
![](https://img.shields.io/crates/d/waybar-module-pacman-updates)
![](https://img.shields.io/github/issues-raw/coffebar/waybar-module-pacman-updates)

## Reason

#### Why not just exec `checkupdates` in custom waybar module? 

- This module will provide relevant local information constantly and periodically update data from the network in backgroud. Direct "checkupdates" will only give you one of two things: updating the information with a long delay or having the module constantly active on the network.
- This module has 2 states which gives you the ability to display different icons depending on status.
- Waybar expects JSON in an infinite loop from modules. So we have this.
- See updates list in tooltip.

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
    "exec": "waybar-module-pacman-updates --interval-seconds 5 --network-interval-seconds 300"
}
```
 - add `"custom/updates"` to one of `modules-left`, `modules-center` or `modules-right`
 - install nerd font to see icons or change icons as you like and restart waybar

## Options

`--no-zero-output` - don't print "0" if there are no updates available.

`--interval-seconds` - interval to run checkupdates without network usage.

`--network-interval-seconds` - interval to run checkupdates with network usage.

`--tooltip-align-columns` - format tooltip as a table using given monospaced font.

### How to hide the module when there are no updates available

##### waybar config

```json
"custom/updates": {
    "format": "{} {icon}",
    "return-type": "json",
    "format-icons": {
        "has-updates": "󱍷",
        "updated": ""
    },
    "exec-if": "which waybar-module-pacman-updates",
    "exec": "waybar-module-pacman-updates --no-zero-output"
}
```

##### style.css

```css
#custom-updates {
	background-color: transparent;
}
```

## installation options

### Install from cargo crates

```bash
cargo install waybar-module-pacman-updates
```

Make sure you have `$HOME/.cargo/bin` in your *PATH* env variable.

### Install from [AUR](https://aur.archlinux.org/packages/waybar-module-pacman-updates-git)

```bash
yay -S waybar-module-pacman-updates-git
```

### Install from source

 ```bash
sh -c "$(wget -O- https://raw.githubusercontent.com/coffebar/waybar-module-pacman-updates/master/installer.sh)"
```

Make sure you have `$HOME/.local/bin` in your *PATH* env variable.

