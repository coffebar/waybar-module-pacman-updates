use std::io::Error;
use std::process::Command;
use std::{thread, time::Duration};

const SLEEP_DURATION: Duration = Duration::from_secs(1);
const REFRESH_TIMEOUT: u16 = 300;

fn main() -> Result<(), Error> {
    thread::spawn(move || {
        sync_database();
    });

    let mut sec: u16 = 0;
    loop {
        if sec >= REFRESH_TIMEOUT {
            sync_database();
            sec = 0;
        }
        let updates = get_updates_count();
        if updates > 0 {
            println!("{{\"text\": \"{}\", \"class\": \"has-updates\", \"alt\": \"has-updates\"}}", updates);
        } else {
            println!("{{\"text\": \"\", \"class\": \"updated\", \"alt\": \"updated\"}}",);
        }

        sec += 1;
        std::thread::sleep(SLEEP_DURATION);
    }
}

// check updates from network
fn sync_database() {
    // checkupdates --nocolor
    Command::new("checkupdates")
        .args(["--nocolor"])
        .output()
        .expect("failed to execute process");
}

// get updates count without network operations
fn get_updates_count() -> u16 {
    // checkupdates --nosync --nocolor
    let output = Command::new("checkupdates")
        .args(["--nosync", "--nocolor"])
        .output()
        .expect("failed to execute process");
    return match output.status.code() {
        Some(_code) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            if stdout == "" {
                return 0;
            }
            (stdout.split(" -> ").count() as u16) - 1
        }
        None => 0,
    };
}
