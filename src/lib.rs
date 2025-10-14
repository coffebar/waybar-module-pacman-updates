use std::{
    process::{Command, Stdio},
    sync::LazyLock,
};

use regex::Regex;

#[derive(Debug)]
pub enum UpdateType {
    Major,
    Minor,
    Patch,
    Pre,
    Other,
}
#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub update_type: UpdateType,
}

impl Package {
    fn determine_update_type(old_ver: &str, new_ver: &str) -> UpdateType {
        let old_parsed = lenient_semver::parse(old_ver);
        let new_parsed = lenient_semver::parse(new_ver);
        match (old_parsed, new_parsed) {
            (Ok(old), Ok(new)) => {
                if new.major > old.major {
                    UpdateType::Major
                } else if new.minor > old.minor {
                    UpdateType::Minor
                } else if new.patch > old.patch {
                    UpdateType::Patch
                } else if new.pre > old.pre {
                    UpdateType::Pre
                } else {
                    UpdateType::Other
                }
            }
            _ => UpdateType::Other,
        }
    }
}

// Compiled only once
static PACKAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\S+)\s+(\S+)\s+->\s+(\S+)$").expect("Failed to compile package regex")
});

impl TryFrom<String> for Package {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let caps = PACKAGE_REGEX.captures(&s).ok_or_else(|| {
            format!(
                "Invalid format: expected 'name old_version -> new_version', got '{}'",
                s
            )
        })?;

        let name = caps[1].to_string();
        let old_version = caps[2].to_string();
        let new_version = caps[3].to_string();

        let update_type = Package::determine_update_type(&old_version, &new_version);

        Ok(Package {
            name,
            old_version,
            new_version,
            update_type,
        })
    }
}

pub trait IsPackageRepo {
    fn local_updates(&mut self);
    fn sync_updates(&mut self);
    fn packages(&self) -> impl Iterator<Item = &Package>;
}

#[derive(Debug, Default)]
pub struct OfficialRepo {
    packages: Vec<Package>,
}

impl OfficialRepo {
    fn common_updates(&mut self, sync: bool) {
        let mut args = vec!["--nocolor"];
        if !sync {
            args.push("--nosync");
        }
        let output = Command::new("checkupdates").args(&args).output();
        match output {
            Ok(out) if out.status.success() => {
                self.packages = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter_map(|line| Package::try_from(line.to_string()).ok())
                    .collect();
            }
            Ok(_) => {}
            Err(e) => eprintln!("Failed to check Official updates: {}", e),
        }
    }
}

impl IsPackageRepo for OfficialRepo {
    fn local_updates(&mut self) {
        self.common_updates(false);
    }
    fn sync_updates(&mut self) {
        self.common_updates(true);
    }
    fn packages(&self) -> impl Iterator<Item = &Package> {
        self.packages.iter()
    }
}

#[derive(Debug, Default)]
pub struct AURepo {
    packages: Vec<Package>,
}

impl IsPackageRepo for AURepo {
    fn local_updates(&mut self) {}
    fn sync_updates(&mut self) {
        let pacman = match Command::new("pacman")
            .arg("-Qm")
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to run pacman: {}", e);
                return;
            }
        };

        let output = Command::new("aur")
            .arg("vercmp")
            .stdin(pacman.stdout.unwrap())
            .stdout(Stdio::piped())
            .output();

        match output {
            Ok(out) if out.status.success() => {
                self.packages = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter_map(|line| Package::try_from(line.to_string()).ok())
                    .collect();
            }
            Ok(_) => {
                eprintln!("aur vercmp exited with a non-zero status");
            }
            Err(e) => {
                eprintln!(
                    "Failed to check AUR updates (probably aurutils is not installed): {}",
                    e
                );
            }
        }
    }
    fn packages(&self) -> impl Iterator<Item = &Package> {
        self.packages.iter()
    }
}
