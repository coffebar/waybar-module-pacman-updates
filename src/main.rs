use std::env;
use std::io::Error;
use std::process::Command;
use std::sync::Mutex;
use std::{thread, time::Duration};

lazy_static::lazy_static! {
    static ref DATABASE_SYNC_MUTEX: Mutex<()> = Mutex::new(());
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
            iter = 0;
        }
        let (updates, mut stdout) = get_updates();
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

fn highlight_semantic_version(
    packages: String,
    colors: [&str; 5],
    padding: Option<[usize; 4]>,
) -> String {
    packages
        .lines()
        .map(|package| {
            let fragments = package.split_whitespace().collect::<Vec<_>>();

            let mut text = package.to_string();

            if let Some(padding) = padding {
                text = fragments
                    .iter()
                    .enumerate()
                    .map(|(index, word)| {
                        word.to_string() + " ".repeat(padding[index % 4] - word.len()).as_str()
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
            }

            if fragments.len() != 4 {
                // unknown format, so we skip formatting
                return text;
            }

            let (Ok(old_version), Ok(new_version)) = (
                lenient_semver::parse(fragments[1]),
                lenient_semver::parse(fragments[3]),
            ) else {
                return text;
            };

            let color = {
                if new_version.major > old_version.major {
                    colors[0]
                } else if new_version.minor > old_version.minor {
                    colors[1]
                } else if new_version.patch > old_version.patch {
                    colors[2]
                } else if new_version.pre > old_version.pre {
                    colors[3]
                } else {
                    colors[4]
                }
            };

            format!("<span color='#{}'>{}</span>", color, text)
        })
        .collect::<Vec<_>>()
        .join("\n")
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
                return (0, "0".to_string());
            }
            ((stdout.split(" -> ").count() as u16) - 1, stdout)
        }
        None => (0, "0".to_string()),
    }
}
