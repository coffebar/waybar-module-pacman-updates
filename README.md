# waybar module for Arch to show system updates available

Shows updates from official repositories and AUR packages.

![screenshot](/screenshot.jpg)

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
- **NEW**: Includes AUR packages updates (uses `pacman -Qm` + AUR API, no AUR helper required).

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

`--no-aur` - disable checking for AUR updates.

`--interval-seconds` - interval to run checkupdates without network usage.

`--network-interval-seconds` - interval to run checkupdates and AUR updates with network usage.

`--tooltip-align-columns` - format tooltip as a table using given monospaced font.

`--color-semver-updates` - color each package update in a color corresponding to the type of update (major, minor, patch, pre)

`--arrow-style` - change the arrow symbols that are displayed between version updates.

`--column-color-overrides` - override the color of each of the four columns corresponding to (package, previous version, arrow, new version)

`--max-version-length` - limit how many characters a version column may take. Longer versions are truncated to end in `...`. Minimum 4, requires `--tooltip-align-columns`.

Some packages carry version strings long enough to wrap the tooltip:

<img width="640" alt="tooltip without --max-version-length" src="https://github.com/user-attachments/assets/d3483acf-8ff0-4ffe-8e28-ae2326659154" />

The same updates with `--max-version-length 10`:

<img width="563" alt="tooltip with --max-version-length 10" src="https://github.com/user-attachments/assets/ea2dfb95-4ed0-4917-8e13-3279482fa320" />

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

Or use this format if you want to show icon before the number of updates:

```json
...
{
	"format": "{icon} {0}",
	...
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

