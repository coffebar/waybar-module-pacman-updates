use std::io::Error;
use std::process::Command;
use std::sync::Mutex;
use std::{thread, time::Duration};

lazy_static::lazy_static! {
    static ref DATABASE_SYNC_MUTEX: Mutex<()> = Mutex::new(());
}
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
        let (updates, stdout) = get_updates();
        if updates > 0 {
            let tooltip = stdout.trim_end().replace("\"", "\\\"").replace("\n", "\\n");
            println!("{{\"text\":\"{}\",\"tooltip\":\"{}\",\"class\":\"has-updates\",\"alt\":\"has-updates\"}}", updates, tooltip);
        } else {
            println!("{{\"text\":\"{}\",\"tooltip\":\"System updated\",\"class\": \"updated\",\"alt\":\"updated\"}}", updates);
        }
        iter += 1;
        std::thread::sleep(SLEEP_DURATION);
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

// get updates info without network operations
fn get_updates() -> (u16, String) {
    // checkupdates --nosync --nocolor
    let output = Command::new("checkupdates")
        .args(["--nosync", "--nocolor"])
        .output()
        .expect("failed to execute process");
    return match output.status.code() {
        Some(_code) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            if stdout == "" {
                return (0, "0".to_string());
            }
            return ((stdout.split(" -> ").count() as u16) - 1, stdout);
        }
        None => (0, "0".to_string()),
    };
}
