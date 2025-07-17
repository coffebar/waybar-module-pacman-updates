use serde::Deserialize;
use std::env;
use std::io::Error;
use std::process::Command;
use std::sync::Mutex;
use std::{thread, time::Duration, time::SystemTime};
use waybar_module_pacman_updates::{highlight_semantic_version, is_version_newer};

#[derive(Deserialize)]
struct AurResponse {
    results: Vec<AurPackage>,
}

#[derive(Deserialize)]
struct AurPackage {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    version: String,
}

lazy_static::lazy_static! {
    static ref DATABASE_SYNC_MUTEX: Mutex<()> = Mutex::new(());
    // AUR cache: (last_update_time, update_count, formatted_output)
    static ref AUR_CACHE: Mutex<(Option<SystemTime>, u16, String)> = Mutex::new((None, 0, String::new()));
}

fn display_help() {
    println!("Usage: {} [options]", env::current_exe().unwrap().display());
    println!();
    println!("Options:");
    println!("  --interval-seconds <seconds>   Set the interval between updates (default: 5)");
    println!("  --network-interval-seconds <seconds>  Set the interval between network updates (default: 300)");
    println!(
        "  --no-zero-output               Don't print '0' when there are no updates available"
    );
    println!("  --hide-aur                      Don't print available AUR updates");
    println!("  --tooltip-align-columns <font> Format tooltip as a table using given font (default: monospace)");
    println!("  --color-semver-updates <colors> Check the difference of semantic versions and color them using the given colors.");
    println!("                                  The order of pango markup hex colors for colored updates is Major, Minor, Patch, Pre, Other.");
    println!("                                  (default: ff0000,00ff00,0000ff,ff00ff,ffffff)");
    println!();
}

fn main() -> Result<(), Error> {
    thread::spawn(move || {
        sync_database();
    });
    let mut iter: u32 = 0;
    let args: Vec<String> = env::args().collect();
    let mut interval_seconds: u32 = 5;
    let mut network_interval_seconds: u32 = 300;
    let mut clean_output = false;
    let mut hide_aur = false;
    let mut tooltip_align = false;
    let mut tooltip_font = "monospace";
    let mut color_semver_updates = false;
    let mut semver_updates_colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    if args.len() > 1 {
        for (i, arg) in args.iter().enumerate() {
            if arg == "--help" {
                display_help();
                return Ok(());
            } else if arg == "--interval-seconds" && i + 1 < args.len() {
                interval_seconds = args[i + 1]
                    .parse()
                    .unwrap_or_else(|_| panic!("--interval-seconds must be greater than 0!"));
            } else if arg == "--network-interval-seconds" && i + 1 < args.len() {
                network_interval_seconds = args[i + 1].parse().unwrap_or_else(|_| {
                    panic!("--network-interval-seconds must be greater than 0!")
                });
            } else if arg == "--no-zero-output" {
                clean_output = true;
            } else if arg == "--hide-aur" {
                hide_aur = true;
            } else if arg == "--tooltip-align-columns" {
                tooltip_align = true;
                if i + 1 < args.len() && args[i + 1][..1] != *"-" {
                    tooltip_font = args[i + 1].as_str();
                }
            } else if arg == "--color-semver-updates" {
                color_semver_updates = true;
                if i + 1 < args.len() && args[i + 1][..1] != *"-" {
                    let colors = args[i + 1].as_str();

                    colors
                        .split(',')
                        .enumerate()
                        .take(semver_updates_colors.len())
                        .for_each(|(index, color)| semver_updates_colors[index] = color);
                }
            }
        }
    }
    let sleep_duration: Duration = Duration::from_secs(interval_seconds as u64);
    if (interval_seconds == 0) || (network_interval_seconds == 0) {
        panic!("interval-seconds and network-interval-seconds must be greater than 0");
    }
    let update_on_iter = network_interval_seconds / interval_seconds;
    loop {
        if iter >= update_on_iter {
            sync_database();
            sync_aur_database(network_interval_seconds);
            iter = 0;
        }
        let (pacman_updates, pacman_stdout) = get_updates();
        let (aur_updates, aur_stdout) = get_aur_updates();
        let updates = pacman_updates + if !hide_aur { aur_updates } else { 0 };
        let mut stdout = if !hide_aur && !aur_stdout.is_empty() && !pacman_stdout.is_empty() {
            format!("{}\n{}", pacman_stdout, aur_stdout)
        } else if !hide_aur && !aur_stdout.is_empty() {
            aur_stdout
        } else {
            pacman_stdout
        };
        if updates > 0 {
            if tooltip_align {
                let mut padding = [0; 4];
                stdout
                    .split_whitespace()
                    .enumerate()
                    .for_each(|(index, word)| {
                        padding[index % 4] = padding[index % 4].max(word.len())
                    });

                if color_semver_updates {
                    stdout =
                        highlight_semantic_version(stdout, semver_updates_colors, Some(padding));
                } else {
                    stdout = stdout
                        .split_whitespace()
                        .enumerate()
                        .map(|(index, word)| {
                            word.to_string() + " ".repeat(padding[index % 4] - word.len()).as_str()
                        })
                        .collect::<Vec<String>>()
                        .chunks(4)
                        .map(|line| line.join(" "))
                        .collect::<Vec<String>>()
                        .join("\n")
                }

                stdout = format!("<span font-family='{}'>{}</span>", tooltip_font, stdout);
            } else if color_semver_updates {
                stdout = highlight_semantic_version(stdout, semver_updates_colors, None);
            }
            let tooltip = stdout.trim_end().replace("\"", "\\\"").replace("\n", "\\n");
            println!("{{\"text\":\"{}\",\"tooltip\":\"{}\",\"class\":\"has-updates\",\"alt\":\"has-updates\"}}", updates, tooltip);
        } else {
            println!("{{\"text\":{},\"tooltip\":\"System updated\",\"class\": \"updated\",\"alt\":\"updated\"}}", if clean_output {"\"\""} else {"\"0\""});
        }
        iter += 1;
        thread::sleep(sleep_duration);
    }
}

// check updates from network
fn sync_database() {
    let _lock = DATABASE_SYNC_MUTEX.lock().unwrap();
    // checkupdates --nocolor
    Command::new("checkupdates")
        .args(["--nocolor"])
        .output()
        .expect("failed to execute process");
}

// check AUR updates from network
fn sync_aur_database(network_interval_seconds: u32) {
    // Lock AUR cache to read/update: (last_update_time, update_count, formatted_output)
    let mut cache = AUR_CACHE.lock().unwrap();
    let now = SystemTime::now();

    if let Some(last_update) = cache.0 {
        if let Ok(elapsed) = now.duration_since(last_update) {
            if elapsed.as_secs() < network_interval_seconds as u64 {
                return;
            }
        }
    }

    // Get locally installed AUR packages
    let output = Command::new("pacman").args(["-Qm"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let local_packages: Vec<(String, String)> = stdout
            .lines()
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next()?.to_string();
                let version = parts.next()?.to_string();
                Some((name, version))
            })
            .collect();

        if local_packages.is_empty() {
            // No AUR packages installed, reset cache
            cache.0 = Some(now); // Update cache timestamp
            cache.1 = 0; // No updates available
            cache.2 = String::new(); // Clear output
            return;
        }

        // Query AUR API for updates
        let package_names: Vec<&str> = local_packages
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();

        match query_aur_api(&package_names) {
            Ok(aur_packages) => {
                let mut updates = Vec::new();

                for (local_name, local_version) in &local_packages {
                    if let Some(aur_pkg) = aur_packages.iter().find(|p| p.name == *local_name) {
                        // Only show update if AUR version is actually newer
                        if is_version_newer(&aur_pkg.version, local_version) {
                            updates.push(format!(
                                "{} {} -> {}",
                                local_name, local_version, aur_pkg.version
                            ));
                        }
                    }
                }

                let count = updates.len() as u16;
                let stdout = updates.join("\n");

                cache.0 = Some(now);
                cache.1 = count;
                cache.2 = stdout;
            }
            Err(_) => {
                // AUR API failed (offline/error) - keep existing cache data but update timestamp
                // to prevent repeated failed requests during this interval
                cache.0 = Some(now);
            }
        }
    }
}

fn query_aur_api(package_names: &[&str]) -> Result<Vec<AurPackage>, Box<dyn std::error::Error>> {
    if package_names.is_empty() {
        return Ok(Vec::new());
    }

    let mut url = "https://aur.archlinux.org/rpc/?v=5&type=info".to_string();
    for name in package_names {
        url.push_str(&format!("&arg[]={}", name));
    }

    let response: AurResponse = ureq::get(&url).call()?.into_json()?;
    Ok(response.results)
}

// get AUR updates from cache
fn get_aur_updates() -> (u16, String) {
    // Lock AUR cache to read: (last_update_time, update_count, formatted_output)
    let cache = AUR_CACHE.lock().unwrap();
    (cache.1, cache.2.clone())
}

// get updates info without network operations
fn get_updates() -> (u16, String) {
    // checkupdates --nosync --nocolor
    let output = Command::new("checkupdates")
        .args(["--nosync", "--nocolor"])
        .output()
        .expect("failed to execute process");
    match output.status.code() {
        Some(_code) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            if stdout.is_empty() {
                return (0, String::new());
            }
            ((stdout.split(" -> ").count() as u16) - 1, stdout)
        }
        None => (0, String::new()),
    }
}
