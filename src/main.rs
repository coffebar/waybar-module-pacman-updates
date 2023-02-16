use std::io::Error;
use std::process::Command;
use std::{thread, time::Duration};

const SLEEP_SECONDS: u16 = 5;
const SLEEP_DURATION: Duration = Duration::from_secs(SLEEP_SECONDS as u64);

fn main() -> Result<(), Error> {
    thread::spawn(move || {
        sync_database();
    });
    let mut iter: u16 = 0;
    let update_on_iter = 300 / SLEEP_SECONDS;
    loop {
        if iter >= update_on_iter {
            sync_database();
            iter = 0;
        }
        let updates = get_updates_count();
        if updates > 0 {
            println!("{{\"text\": \"{}\", \"class\": \"has-updates\", \"alt\": \"has-updates\"}}", updates);
        } else {
            println!("{{\"text\": \"\", \"class\": \"updated\", \"alt\": \"updated\"}}",);
        }
        iter += 1;
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
