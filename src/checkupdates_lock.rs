use fs2::FileExt;
use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn checkupdates_db_path() -> PathBuf {
    if let Ok(db_path) = env::var("CHECKUPDATES_DB") {
        if !db_path.trim().is_empty() {
            return PathBuf::from(db_path);
        }
    }

    let uid = env::var("UID").unwrap_or_else(|_| "unknown".to_string());
    env::temp_dir().join(format!("checkup-db-{}", uid))
}

pub fn checkupdates_lock_path() -> PathBuf {
    checkupdates_db_path().with_extension("lock")
}

fn with_exclusive_lock_at<T, F>(lock_path: &Path, f: F) -> T
where
    F: FnOnce() -> T,
{
    let lock_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(lock_path)
        .expect("failed to open checkupdates lock file");

    lock_file
        .lock_exclusive()
        .expect("failed to lock checkupdates mutex file");

    f()
}

pub fn run_checkupdates_with_lock(args: &[&str]) -> Output {
    let lock_path = checkupdates_lock_path();
    with_exclusive_lock_at(&lock_path, || {
        Command::new("checkupdates")
            .args(args)
            .output()
            .expect("failed to execute checkupdates")
    })
}

#[cfg(test)]
mod tests {
    use super::{checkupdates_db_path, checkupdates_lock_path, with_exclusive_lock_at};
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::sync::Mutex;
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_checkupdates_paths_respect_env_override() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let old_db = env::var("CHECKUPDATES_DB").ok();
        let old_uid = env::var("UID").ok();

        env::set_var("CHECKUPDATES_DB", "/tmp/custom-checkup-db");
        env::set_var("UID", "9999");

        assert_eq!(
            checkupdates_db_path(),
            PathBuf::from("/tmp/custom-checkup-db")
        );
        assert_eq!(
            checkupdates_lock_path(),
            PathBuf::from("/tmp/custom-checkup-db.lock")
        );

        match old_db {
            Some(value) => env::set_var("CHECKUPDATES_DB", value),
            None => env::remove_var("CHECKUPDATES_DB"),
        }
        match old_uid {
            Some(value) => env::set_var("UID", value),
            None => env::remove_var("UID"),
        }
    }

    #[test]
    fn test_checkupdates_paths_use_default_db_location() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let old_db = env::var("CHECKUPDATES_DB").ok();
        let old_uid = env::var("UID").ok();

        env::remove_var("CHECKUPDATES_DB");
        env::set_var("UID", "4242");

        let expected_db = env::temp_dir().join("checkup-db-4242");
        let expected_lock = env::temp_dir().join("checkup-db-4242.lock");

        assert_eq!(checkupdates_db_path(), expected_db);
        assert_eq!(checkupdates_lock_path(), expected_lock);

        match old_db {
            Some(value) => env::set_var("CHECKUPDATES_DB", value),
            None => env::remove_var("CHECKUPDATES_DB"),
        }
        match old_uid {
            Some(value) => env::set_var("UID", value),
            None => env::remove_var("UID"),
        }
    }

    #[test]
    fn test_exclusive_lock_serializes_concurrent_access() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let lock_path = env::temp_dir().join(format!("checkupdates-lock-test-{}.lock", unique));
        let lock_path_for_thread = lock_path.clone();

        let (acquired_tx, acquired_rx) = mpsc::channel();
        let t1 = thread::spawn(move || {
            with_exclusive_lock_at(&lock_path_for_thread, || {
                acquired_tx.send(()).unwrap();
                thread::sleep(Duration::from_millis(250));
            });
        });

        acquired_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("first thread did not acquire lock");

        let start = Instant::now();
        with_exclusive_lock_at(&lock_path, || {});
        let waited = start.elapsed();

        t1.join().expect("thread join failed");

        assert!(
            waited >= Duration::from_millis(200),
            "expected second lock acquisition to block, waited only {:?}",
            waited
        );

        let _ = fs::remove_file(lock_path);
    }
}
